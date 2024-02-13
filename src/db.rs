use super::*;
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
}

async fn connect_db() -> Pool<Sqlite> {
    SqlitePool::connect(DB_URL).await.unwrap()
}

pub async fn create_db() {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
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
        added_timestamp INTEGER,
        last_updated_timestamp INTEGER);",
    )
    .execute(&db)
    .await
    .unwrap();
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
                (chat_id, company, tracking_number, added_timestamp, last_updated_timestamp)
                VALUES
                ($1, $2, $3, $4, $5);",
    )
    .bind(chat_id)
    .bind(company)
    .bind(tracking_number)
    .bind(Utc::now().timestamp())
    .bind(last_updated_timestamp)
    .execute(&db)
    .await
    .unwrap();
}

pub async fn delete_tracker(id: i64) {
    let db = connect_db().await;
    sqlx::query(
        "DELETE
                FROM trackers
                WHERE id = $1;",
    )
    .bind(id)
    .execute(&db)
    .await
    .unwrap();
}

pub async fn update_last_updated_timestamp(id: i64) {
    let db = connect_db().await;
    sqlx::query(
        "UPDATE trackers
        SET last_updated_timestamp = $1
        WHERE id = $2",
    )
    .bind(Utc::now().timestamp())
    .bind(id)
    .execute(&db)
    .await
    .unwrap();
}
