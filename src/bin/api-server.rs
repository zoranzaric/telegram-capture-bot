#[macro_use]
extern crate rocket;

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Capture {
    id: Option<u32>,
    content: String,
    created_at: String,
    processed_at: Option<String>,
}

#[rocket_sync_db_pools::database("main_db")]
struct DbConn(rocket_sync_db_pools::rusqlite::Connection);

#[get("/")]
async fn index(db: DbConn) -> Json<Vec<Capture>> {
    Json(load_captures(&db).await)
}

async fn load_captures(db: &DbConn) -> Vec<Capture> {
    db.run(|conn| {
      let mut stmt = conn
          .prepare(
              "SELECT id, content, created_at, processed_at FROM capture WHERE processed_at IS NULL ORDER BY created_at ASC",
          )
          .unwrap();
      stmt.query_map([], |row| Ok(Capture {
          id: row.get(0).unwrap(),
          content: row.get(1).unwrap(),
          created_at: row.get(2).unwrap(),
          processed_at: row.get(3).unwrap(),
      }))
      .unwrap()
      .map(|r| r.unwrap())
      .collect::<Vec<_>>()
    }).await
}

#[put("/processed/<id>")]
async fn mark_capture_processed(db: DbConn, id: u32) {
    db.run(move |conn| {
        let mut stmt = conn
            .prepare("UPDATE capture SET processed_at = CURRENT_TIMESTAMP WHERE id = ?")
            .unwrap();

        stmt.execute(&[&id]).unwrap();
    })
    .await;
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(DbConn::fairing())
        .mount("/", routes![index, mark_capture_processed])
}
