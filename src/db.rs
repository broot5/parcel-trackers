use crate::*;
use sqlx::{migrate::MigrateDatabase, FromRow, Pool, Sqlite, SqlitePool};

const DB_URL: &str = "sqlite://sqlite.db";

#[derive(Clone, FromRow, Debug)]
pub struct Tracker {
    pub id: i64,
    pub chat_id: i64,
    pub company: String,
    pub tracking_number: String,
    pub added_timestamp: i64,
    pub last_updated_timestamp: i64,
    pub keep: u8,
}

async fn connect_db() -> Pool<Sqlite> {
    SqlitePool::connect(DB_URL).await.unwrap()
}

pub async fn create_db() {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        log::info!("Creating DB {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => log::info!("Successfully created DB"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        log::info!("DB already exists");
    }
}

pub async fn create_trackers_table() {
    let db = connect_db().await;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS trackers (
        id INTEGER PRIMARY KEY NOT NULL,
        chat_id INTEGER NOT NULL,
        company TEXT NOT NULL,
        tracking_number TEXT NOT NULL,
        added_timestamp INTEGER NOT NULL,
        last_updated_timestamp INTEGER NOT NULL,
        keep INTEGER NOT NULL);",
    )
    .execute(&db)
    .await
    .unwrap();

    log::info!("Successfully created trackers table");
}

pub async fn list_tracker(chat_id: i64) -> Vec<Tracker> {
    let db = connect_db().await;

    sqlx::query_as::<_, Tracker>(
        "SELECT *
        FROM trackers
        WHERE chat_id = $1;",
    )
    .bind(chat_id)
    .fetch_all(&db)
    .await
    .unwrap()
}

pub async fn list_all_tracker() -> Vec<Tracker> {
    let db = connect_db().await;
    sqlx::query_as::<_, Tracker>(
        "SELECT *
        FROM trackers;",
    )
    .fetch_all(&db)
    .await
    .unwrap()
}

pub async fn add_tracker(
    chat_id: i64,
    company: &String,
    tracking_number: &String,
    last_updated_timestamp: i64,
) {
    let db = connect_db().await;
    sqlx::query(
        "INSERT INTO trackers
        (chat_id, company, tracking_number, added_timestamp, last_updated_timestamp, keep)
        VALUES
        ($1, $2, $3, $4, $5, $6);",
    )
    .bind(chat_id)
    .bind(company)
    .bind(tracking_number)
    .bind(Utc::now().timestamp())
    .bind(last_updated_timestamp)
    .bind(0)
    .execute(&db)
    .await
    .unwrap();

    log::info!("Successfully added tracker to DB");
}

pub async fn delete_tracker(id: i64, chat_id: i64) {
    let db = connect_db().await;
    sqlx::query(
        "DELETE
        FROM trackers
        WHERE id = $1 AND chat_id = $2;",
    )
    .bind(id)
    .bind(chat_id)
    .execute(&db)
    .await
    .unwrap();

    log::info!("Successfully deleted tracker from DB");
}

pub async fn update_last_updated_timestamp(id: i64) {
    let db = connect_db().await;
    sqlx::query(
        "UPDATE trackers
        SET last_updated_timestamp = $1
        WHERE id = $2;",
    )
    .bind(Utc::now().timestamp())
    .bind(id)
    .execute(&db)
    .await
    .unwrap();

    log::info!("Successfully Updated tracker's last_updated_timestamp");
}

pub async fn update_keep(id: i64, keep: u8) {
    let db = connect_db().await;
    sqlx::query(
        "UPDATE trackers
        SET keep = $1
        WHERE id = $2;",
    )
    .bind(keep)
    .bind(id)
    .execute(&db)
    .await
    .unwrap();

    log::info!("Successfully Updated tracker's keep");
}
