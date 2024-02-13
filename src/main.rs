mod getter;

use chrono::prelude::*;
use getter::get;
use sqlx::{migrate::MigrateDatabase, FromRow, Sqlite, SqlitePool};
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct Parcel {
    tracking_number: String,
    sender: String,
    receiver: String,
    item: String,
    delivery_status: DeliveryStatus,
    tracking_status: Vec<TrackingStatus>,
    last_updated_time: DateTime<FixedOffset>,
}

#[derive(Debug)]
struct TrackingStatus {
    time: DateTime<FixedOffset>,
    status: String,
    location: String,
    detail: String,
}

#[derive(Debug)]
enum DeliveryStatus {
    InProgress,
    Completed,
    Unknown,
}

#[derive(Clone, FromRow, Debug)]
struct Tracker {
    id: i64,
    chat_id: i64,
    company: String,
    tracking_number: String,
    added_timestamp: i64,
    last_updated_timestamp: i64,
}

const DB_URL: &str = "sqlite://sqlite.db";

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    let db = SqlitePool::connect(DB_URL).await.unwrap();

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

    let bot = Bot::from_env();

    tokio::spawn(poll(bot.clone()));

    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text")]
    Help,
    #[command(description = "add tracker", parse_with = "split")]
    Add {
        company: String,
        tracking_number: String,
    },
    #[command(description = "delete tracker")]
    Delete { id: i64 },
    #[command(description = "list trackers")]
    List,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Add {
            company,
            tracking_number,
        } => match get(&company, &tracking_number).await {
            Some(parcel) => {
                let db = SqlitePool::connect(DB_URL).await.unwrap();

                sqlx::query(
                    "INSERT INTO trackers
                (chat_id, company, tracking_number, added_timestamp, last_updated_timestamp)
                VALUES
                ($1, $2, $3, $4, $5);",
                )
                .bind(msg.chat.id.0)
                .bind(&company)
                .bind(&tracking_number)
                .bind(Utc::now().timestamp())
                .bind(parcel.last_updated_time.timestamp())
                .execute(&db)
                .await
                .unwrap();

                bot.send_message(
                    msg.chat.id,
                    format!(
                        "Added Tracker\nCompany: {company}\nTracking number: {tracking_number}"
                    ),
                )
                .await?;

                bot.send_message(
                    msg.chat.id,
                    format!(
                        "Item: {}\nDelivery Satus: {:?}",
                        parcel.item, parcel.delivery_status
                    ),
                )
                .await?
            }
            None => {
                bot.send_message(msg.chat.id, "Invalid company name")
                    .await?
            }
        },
        Command::Delete { id } => {
            let db = SqlitePool::connect(DB_URL).await.unwrap();

            sqlx::query(
                "DELETE
                FROM trackers
                WHERE id = $1;",
            )
            .bind(id)
            .execute(&db)
            .await
            .unwrap();

            bot.send_message(msg.chat.id, format!("Deleted tracker"))
                .await?
        }
        Command::List => {
            let db = SqlitePool::connect(DB_URL).await.unwrap();

            let trackers = sqlx::query_as::<_, Tracker>(
                "SELECT *
                FROM trackers
                WHERE chat_id = $1;",
            )
            .bind(msg.chat.id.0)
            .fetch_all(&db)
            .await
            .unwrap();

            for tracker in trackers {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "{}\nCompany: {}\nTracking number: {}\nAdded time: {}",
                        tracker.id,
                        tracker.company,
                        tracker.tracking_number,
                        DateTime::<Utc>::from_timestamp(tracker.added_timestamp, 0).unwrap()
                    ),
                )
                .await?;
            }

            bot.send_message(msg.chat.id, format!("List")).await?
        }
    };

    Ok(())
}

async fn poll(bot: Bot) {
    loop {
        let db = SqlitePool::connect(DB_URL).await.unwrap();

        let trackers = sqlx::query_as::<_, Tracker>(
            "SELECT *
            FROM trackers;",
        )
        .fetch_all(&db)
        .await
        .unwrap();

        for tracker in trackers {
            let parcel = get(&tracker.company, &tracker.tracking_number)
                .await
                .unwrap();

            if parcel.last_updated_time.timestamp() > tracker.last_updated_timestamp {
                bot.send_message(
                    teloxide::types::Recipient::from(tracker.chat_id.to_string()),
                    format!("{:#?}", parcel.tracking_status),
                )
                .await
                .unwrap();
            }

            match parcel.last_updated_time.timestamp() > tracker.last_updated_timestamp {
                true => {
                    sqlx::query(
                        "UPDATE trackers
                        SET last_updated_timestamp = $1
                        WHERE track_id = $2",
                    )
                    .bind(Utc::now().timestamp())
                    .bind(tracker.id)
                    .execute(&db)
                    .await
                    .unwrap();

                    bot.send_message(
                        teloxide::types::Recipient::from(tracker.chat_id.to_string()),
                        format!("{:#?}", parcel.tracking_status.last()),
                    )
                    .await
                    .unwrap();
                }
                false => {}
            }
        }
        sleep(Duration::from_secs(30)).await;
    }
}
