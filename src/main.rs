use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dotenv::dotenv;
use max_api_kernel::{MaxClient, MaxError};
use rust_tdlib::client::{
    AuthStateHandlerProxy, Client, ClientState, ConsoleClientStateHandler, Worker,
};
use rust_tdlib::tdjson;
use rust_tdlib::types::{
    FormattedText, GetMe, InputMessageContent, InputMessageText, MessageContent,
    SendMessage, TdlibParameters, Update,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BridgeMessage {
    v: u8, // version
    t: String, // type, e.g. "msg"
    dir: String, // direction: "in" or "out" (in - TG→MAX, out - MAX→TG)
    cid: i64, // chat_id
    mid: i64, // message_id
    from: String, // sender name or ID
    body: String, // message text
    ts: u64, // timestamp
}

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

    let _me = tg_client.get_me(GetMe::builder().build()).await.unwrap();
    println!("[TG] Авторизован.");

    let max_client = Arc::new(MaxClient::new(&max_phone).expect("Не удалось создать Max клиент"));

    let (max_to_tg_tx, mut max_to_tg_rx) = tokio::sync::mpsc::channel::<String>(100);

    let max_sent: Arc<Mutex<HashSet<i64>>> = Arc::new(Mutex::new(HashSet::new()));
    let tg_sent: Arc<Mutex<HashSet<i64>>> = Arc::new(Mutex::new(HashSet::new()));

    {
        let tx = max_to_tg_tx.clone();
        max_client.on_message(move |msg| {
            let tx = tx.clone();
            async move {
                if msg.chat_id != Some(0) || msg.text.is_empty() {
                    return;
                }
                tx.send(msg.text.clone()).await.ok();
            }
        });
    }

    {
        let max = Arc::clone(&max_client);
        tokio::spawn(async move {
            if let Err(e) = max.start().await {
                eprintln!("[MAX] Ошибка соединения: {e}");
            }
        });
    }

    println!("[MAX] Подключение к Max...");
    println!("Мост запущен. Ctrl+C для выхода.");

    loop {
        tokio::select! {
            // Max → Telegram
            msg = max_to_tg_rx.recv() => {
                let Some(text) = msg else { break; };

                // Пробуем распарсить входящий JSON от Max
                if let Ok(bridge_msg) = serde_json::from_str::<BridgeMessage>(&text) {
                    if bridge_msg.dir == "in" {
                        continue; // Игнорируем эхо своих же сообщений
                    }

                    println!("[MAX→TG] Пересылка в чат {}: {}", bridge_msg.cid, bridge_msg.body);

                    let send = SendMessage::builder()
                        .chat_id(bridge_msg.cid)
                        .input_message_content(InputMessageContent::InputMessageText(
                            InputMessageText::builder()
                                .text(FormattedText::builder().text(bridge_msg.body).build())
                                .build(),
                        ))
                        .build();

                    if let Err(e) = tg_client.send_message(send).await {
                        eprintln!("[TG] Ошибка отправки: {e:?}");
                    }
                } else {
                    println!("[MAX] Неизвестный формат сообщения: {}", text);
                }
            }

            // Telegram → Max
            update = tg_updates_rx.recv() => {
                let Some(update) = update else { break; };
                if let Update::NewMessage(new_msg) = update.as_ref() {
                    let message = new_msg.message();
                    let chat_id = message.chat_id();
                    let message_id = message.id();

                    if let MessageContent::MessageText(t) = message.content() {
                        let text = t.text().text().to_string();
                        if !text.is_empty() {
                            let ts = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();

                            // В реальном приложении можно делать запрос к TG для получения имени
                            // Здесь заполняем заглушкой или ID
                            let from_name = if message.is_outgoing() { "Me".to_string() } else { "TG User".to_string() };
                            let dir = if message.is_outgoing() { "out".to_string() } else { "in".to_string() };

                            let bridge_msg = BridgeMessage {
                                v: 1,
                                t: "msg".to_string(),
                                dir,
                                cid: chat_id,
                                mid: message_id,
                                from: from_name,
                                body: text.clone(),
                                ts,
                            };

                            if let Ok(json_str) = serde_json::to_string(&bridge_msg) {
                                println!("[TG→MAX] {}", json_str);

                                let max = Arc::clone(&max_client);
                                tokio::spawn(async move {
                                    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
                                    loop {
                                        while !max.is_connected() {
                                            if tokio::time::Instant::now() >= deadline {
                                                eprintln!("[MAX] Нет соединения 30с");
                                                return;
                                            }
                                            tokio::time::sleep(Duration::from_millis(500)).await;
                                        }
                                        match max.send_message(0, &json_str, true, None, None).await {
                                            Ok(_) => return,
                                            Err(_) => {
                                                if tokio::time::Instant::now() >= deadline {
                                                    eprintln!("[MAX] Тайм-аут отправки");
                                                    return;
                                                }
                                                tokio::time::sleep(Duration::from_millis(500)).await;
                                            }
                                        }
                                    }
                                });
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
