use rust_tdlib::client::{AuthStateHandlerProxy, Client, ClientState, ConsoleClientStateHandler, Worker};
use rust_tdlib::tdjson;
use rust_tdlib::types::{
    CreatePrivateChat, FormattedText, GetMe, InputMessageContent, InputMessageText,
    MessageContent, SendMessage, TdlibParameters, Update,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use std::env;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    tdjson::set_log_verbosity_level(
        std::env::var("TDLIB_LOG_VERBOSITY")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap(),
    );
    env_logger::init();

    dotenv().ok();

    let api_id: i32 = env::var("API_ID")
        .expect("API_ID must be set")
        .parse()
        .expect("API_ID must be a valid number");

    let api_hash: String = env::var("API_HASH")
        .expect("API_HASH must be set");

    let tdlib_parameters = TdlibParameters::builder()
        .database_directory("tdlib-data")
        .use_test_dc(false)
        .api_id(api_id)
        .api_hash(api_hash)
        .system_language_code("ru")
        .device_model("Terminal")
        .system_version("Unknown")
        .application_version(env!("CARGO_PKG_VERSION"))
        .enable_storage_optimizer(true)
        .build();

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<Box<Update>>(1000);

    let client = Client::builder()
        .with_tdlib_parameters(tdlib_parameters)
        .with_updates_sender(sender)
        .with_client_auth_state_handler(ConsoleClientStateHandler)
        .build()
        .unwrap();

    let mut worker = Worker::builder()
        .with_auth_state_handler(AuthStateHandlerProxy::default())
        .build()
        .unwrap();
    worker.start();

    let client = worker.bind_client(client).await.unwrap();

    loop {
        if worker.wait_client_state(&client).await.unwrap() == ClientState::Opened {
            break;
        }
    }

    let me = client.get_me(GetMe::builder().build()).await.unwrap();
    let saved_chat = client
        .create_private_chat(CreatePrivateChat::builder().user_id(me.id()).build())
        .await
        .unwrap();
    let saved_chat_id = saved_chat.id();
    println!("Авторизован. Избранное: chat_id = {saved_chat_id}");
    println!("Введите сообщение + Enter — отправится в Избранное. Ctrl+C для выхода.");

    let mut lines = BufReader::new(tokio::io::stdin()).lines();

    loop {
        tokio::select! {
            line = lines.next_line() => {
                match line {
                    Ok(Some(text)) if !text.is_empty() => {
                        let msg = SendMessage::builder()
                            .chat_id(saved_chat_id)
                            .input_message_content(InputMessageContent::InputMessageText(
                                InputMessageText::builder()
                                    .text(FormattedText::builder().text(text).build())
                                    .build(),
                            ))
                            .build();
                        if let Err(e) = client.send_message(msg).await {
                            eprintln!("Ошибка отправки: {e:?}");
                        }
                    }
                    Ok(None) | Err(_) => break,
                    _ => {}
                }
            }
            update = receiver.recv() => {
                let Some(update) = update else { break; };
                if let Update::NewMessage(new_message) = update.as_ref() {
                    let message = new_message.message();
                    if message.chat_id() == saved_chat_id {
                        match message.content() {
                            MessageContent::MessageText(text) => {
                                println!("[Избранное] {}", text.text().text());
                            }
                            other => {
                                println!("[Избранное] {:?}", other);
                            }
                        }
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Останавливаюсь...");
                break;
            }
        }
    }

    client.stop().await.unwrap();
    loop {
        if worker.wait_client_state(&client).await.unwrap() == ClientState::Closed {
            break;
        }
    }
    worker.stop();
}
