#![feature(proc_macro_hygiene, decl_macro)]

use std::time::Duration;

use rocket_contrib::json::Json;

use rusqlite::{Connection, Result};

use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Capture {
    id: Option<u32>,
    content: String,
    created_at: String,
    processed_at: Option<String>,
}

#[rocket_contrib::database("main_db")]
struct DbConn(rocket_contrib::databases::rusqlite::Connection);

fn foo() {
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
}

#[macro_use]
extern crate rocket;

#[get("/")]
fn index(db: DbConn) -> Json<Vec<Capture>> {
    Json(load_captures(&*db))
}

fn load_captures(conn: &rocket_contrib::databases::rusqlite::Connection) -> Vec<Capture> {
    let mut stmt = conn
        .prepare(
            "SELECT id, content, created_at, processed_at FROM capture WHERE processed_at IS NULL ORDER BY created_at ASC",
        )
        .unwrap();
    stmt.query_map(&[], |row| Capture {
        id: row.get(0),
        content: row.get(1),
        created_at: row.get(2),
        processed_at: row.get(3),
    })
    .unwrap()
    .map(|r| r.unwrap())
    .collect::<Vec<_>>()
}

#[put("/processed/<id>")]
fn mark_capture_processed(db: DbConn, id: u32) {
    let mut stmt = db
        .prepare("UPDATE capture SET processed_at = CURRENT_TIMESTAMP WHERE id = ?")
        .unwrap();

    stmt.execute(&[&id]).unwrap();
}

fn main() {
    rocket::ignite()
        .attach(DbConn::fairing())
        .mount("/", routes![index, mark_capture_processed])
        .launch();
}
