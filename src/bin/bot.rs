use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::time::Duration;

use frankenstein::Api;
use frankenstein::ChatIdEnum;
use frankenstein::GetFileParams;
use frankenstein::GetUpdatesParams;
use frankenstein::SendMessageParams;
use frankenstein::TelegramApi;

use rusqlite::{Connection, Result};

use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;

fn main() {
    tracing_subscriber::fmt::init();

    let listen_address = std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
        9000,
    );
    let builder = PrometheusBuilder::new();
    builder
        .listen_address(listen_address)
        .idle_timeout(
            MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
            Some(Duration::from_secs(10)),
        )
        .install()
        .expect("failed to install Prometheus recorder");
    println!("Prometheus exporter listening on {}", listen_address);

    metrics::register_counter!("messages_received", "The number of messages received.");

    let conn = Connection::open(&"./db.sqlite").expect("Could not open DB");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS capture (
                  id              INTEGER PRIMARY KEY,
                  content            TEXT NOT NULL,
                  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                  processed_at TIMESTAMP
                  )",
        &[],
    )
    .expect("Could not create capture table");

    let token = std::env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");

    let api = Api::new(token.to_string());

    let mut update_params = GetUpdatesParams::new();
    update_params.set_allowed_updates(Some(vec!["message".to_string()]));

    loop {
        let result = api.get_updates(&update_params);

        println!("result: {:#?}", result);

        match result {
            Ok(response) => {
                for update in response.result {
                    if let Some(message) = update.message() {
                        if let Some(text) = message.text.clone() {
                            metrics::increment_counter!("messages_received", "type" => "text");
                            conn.execute("INSERT INTO capture (content) VALUES (?1)", &[&text])
                                .expect("Could not insert capture");

                            println!("Text: {}", text);
                        } else if let Some(voice) = message.voice.clone() {
                            metrics::increment_counter!("messages_received", "type" => "voice");

                            let result = api.get_file(&GetFileParams {
                                file_id: voice.file_id.clone(),
                            });
                            if let Ok(response) = result {
                                println!("get_file response: {:#?}", response);

                                if response.ok {
                                    if let Some(file_path) = response.result.file_path() {
                                        let url = format!(
                                            "https://api.telegram.org/file/bot{}/{}",
                                            token, file_path
                                        );

                                        match ureq::get(&url).call() {
                                            Ok(response) => {
                                                let file_id = voice.file_id;
                                                let mut reader =
                                                    BufReader::new(response.into_reader());
                                                let f = OpenOptions::new()
                                                    .write(true)
                                                    .create(true)
                                                    .open(file_id)
                                                    .unwrap();
                                                let mut writer = BufWriter::new(f);

                                                let mut length = 1;

                                                while length > 0 {
                                                    let buffer = reader.fill_buf().unwrap();

                                                    writer.write(buffer).unwrap();

                                                    length = buffer.len();
                                                    reader.consume(length);
                                                }
                                            }
                                            Err(e) => eprintln!("Error: {:#?}", e),
                                        }
                                    }
                                }
                            }
                        } else {
                            metrics::increment_counter!("messages_received", "type" => "UNHANDLED");
                        }

                        let mut send_message_params = SendMessageParams::new(
                            ChatIdEnum::IsizeVariant(message.chat().id()),
                            "hello".to_string(),
                        );
                        send_message_params.set_reply_to_message_id(Some(message.message_id()));

                        let _ = api.send_message(&send_message_params);

                        update_params.set_offset(Some(update.update_id() + 1))
                    }
                }
            }
            Err(error) => {
                println!("Failed to get updates: {:?}", error);
            }
        }
    }
}
