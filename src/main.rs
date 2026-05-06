use std::collections::HashSet;
use std::env;
use std::sync::Arc;

use dotenv::dotenv;
use max_api_kernel::MaxClient;
use rust_tdlib::client::{
    AuthStateHandlerProxy, Client, ClientState, ConsoleClientStateHandler, Worker,
};
use rust_tdlib::tdjson;
use rust_tdlib::types::{
    CreatePrivateChat, FormattedText, GetMe, InputMessageContent, InputMessageText, MessageContent,
    SendMessage, TdlibParameters, Update,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tdjson::set_log_verbosity_level(
        env::var("TDLIB_LOG_VERBOSITY")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap(),
    );
    env_logger::init();

    let api_id: i32 = env::var("API_ID")
        .expect("API_ID must be set")
        .parse()
        .expect("API_ID must be a number");
    let api_hash = env::var("API_HASH").expect("API_HASH must be set");
    let max_phone = env::var("MAX_PHONE").expect("MAX_PHONE must be set");

    // === Telegram ===
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

    let (tg_updates_tx, mut tg_updates_rx) = tokio::sync::mpsc::channel::<Box<Update>>(1000);

    let tg_client = Client::builder()
        .with_tdlib_parameters(tdlib_parameters)
        .with_updates_sender(tg_updates_tx)
        .with_client_auth_state_handler(ConsoleClientStateHandler)
        .build()
        .unwrap();

    let mut worker = Worker::builder()
        .with_auth_state_handler(AuthStateHandlerProxy::default())
        .build()
        .unwrap();
    worker.start();
    let tg_client = worker.bind_client(tg_client).await.unwrap();

    loop {
        if worker.wait_client_state(&tg_client).await.unwrap() == ClientState::Opened {
            break;
        }
    }

    let me = tg_client.get_me(GetMe::builder().build()).await.unwrap();
    let tg_saved_chat = tg_client
        .create_private_chat(CreatePrivateChat::builder().user_id(me.id()).build())
        .await
        .unwrap();
    let tg_saved_chat_id = tg_saved_chat.id();
    println!("[TG] Авторизован. Избранное chat_id = {tg_saved_chat_id}");

    // === Max ===
    let max_client = Arc::new(MaxClient::new(&max_phone).expect("Не удалось создать Max клиент"));

    // Канал: Max → Telegram
    let (max_to_tg_tx, mut max_to_tg_rx) = tokio::sync::mpsc::channel::<String>(100);

    // Защита от эха: тексты, отправленные мостом в Max и в Telegram
    let max_sent: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let tg_sent: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    // Обработчик входящих из Max
    {
        let tx = max_to_tg_tx.clone();
        let max_sent_ref = Arc::clone(&max_sent);
        max_client.on_message(move |msg| {
            let tx = tx.clone();
            let max_sent = Arc::clone(&max_sent_ref);
            async move {
                if msg.chat_id != Some(0) || msg.text.is_empty() {
                    return;
                }
                let text = msg.text.clone();
                let mut guard = max_sent.lock().await;
                if guard.remove(&text) {
                    return; // эхо от нашей отправки — пропускаем
                }
                drop(guard);
                tx.send(text).await.ok();
            }
        });
    }

    // Запускаем Max в фоне (start() блокирует)
    {
        let max = Arc::clone(&max_client);
        tokio::spawn(async move {
            if let Err(e) = max.start().await {
                eprintln!("[MAX] Ошибка соединения: {e}");
            }
        });
    }

    println!("[MAX] Подключение к Max...");
    println!("Мост запущен: Telegram Избранное ↔ Max Избранное. Ctrl+C для выхода.");

    // === Главный цикл моста ===
    loop {
        tokio::select! {
            // Max → Telegram
            msg = max_to_tg_rx.recv() => {
                let Some(text) = msg else { break; };
                println!("[MAX→TG] {text}");
                tg_sent.lock().await.insert(text.clone());
                let send = SendMessage::builder()
                    .chat_id(tg_saved_chat_id)
                    .input_message_content(InputMessageContent::InputMessageText(
                        InputMessageText::builder()
                            .text(FormattedText::builder().text(text).build())
                            .build(),
                    ))
                    .build();
                if let Err(e) = tg_client.send_message(send).await {
                    eprintln!("[TG] Ошибка отправки: {e:?}");
                }
            }

            // Telegram → Max
            update = tg_updates_rx.recv() => {
                let Some(update) = update else { break; };
                if let Update::NewMessage(new_msg) = update.as_ref() {
                    let message = new_msg.message();
                    if message.chat_id() == tg_saved_chat_id {
                        if let MessageContent::MessageText(t) = message.content() {
                            let text = t.text().text().to_string();
                            if !text.is_empty() {
                                let mut guard = tg_sent.lock().await;
                                if !guard.remove(&text) {
                                    drop(guard);
                                    println!("[TG→MAX] {text}");
                                    max_sent.lock().await.insert(text.clone());
                                    let max = Arc::clone(&max_client);
                                    tokio::spawn(async move {
                                        if let Err(e) = max.send_message(0, &text, true, None, None).await {
                                            eprintln!("[MAX] Ошибка отправки: {e:?}");
                                        }
                                    });
                                }
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

    max_client.close().await;
    tg_client.stop().await.unwrap();
    loop {
        if worker.wait_client_state(&tg_client).await.unwrap() == ClientState::Closed {
            break;
        }
    }
    worker.stop();
}
