mod command;
mod db;
mod getter;

use chrono::prelude::*;
use command::*;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::time::{sleep, Duration};

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

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

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    db::create_db().await;
    db::create_trackers_table().await;

    let bot = Bot::from_env();

    tokio::spawn(poll(bot.clone()));

    let handler = dptree::entry().branch(
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(command_handler),
    );

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text")]
    Help,
    #[command(description = "list trackers")]
    List,
    #[command(description = "add tracker", parse_with = "split")]
    Add {
        company: String,
        tracking_number: String,
    },
    #[command(description = "delete tracker")]
    Delete { id: i64 },
    #[command(description = "get current info of tracker")]
    Info { id: i64 },
}

async fn command_handler(bot: Bot, msg: Message, cmd: Command) -> HandlerResult {
    match cmd {
        Command::Help => help(bot, msg).await,
        Command::List => list(bot, msg).await,
        Command::Add {
            company,
            tracking_number,
        } => add(bot, msg, company, tracking_number).await,
        Command::Delete { id } => delete(bot, msg, id).await,
        Command::Info { id } => Ok(()),
    }
}

async fn poll(bot: Bot) {
    loop {
        let trackers = db::list_all_tracker().await;

        for tracker in trackers {
            let parcel = getter::get(&tracker.company, &tracker.tracking_number)
                .await
                .unwrap();

            match parcel.last_updated_time.timestamp() > tracker.last_updated_timestamp {
                true => {
                    db::update_last_updated_timestamp(tracker.id).await;

                    bot.send_message(
                        teloxide::types::Recipient::from(tracker.chat_id.to_string()),
                        format!(
                            "{:#?}",
                            parcel.tracking_status.last().unwrap_or(&TrackingStatus {
                                time: Utc::now().into(),
                                status: "Not found".to_string(),
                                location: "".to_string(),
                                detail: "".to_string()
                            })
                        ),
                    )
                    .await
                    .unwrap();
                }
                false => {}
            }

            // Test
            // bot.send_message(
            //     teloxide::types::Recipient::from(tracker.chat_id.to_string()),
            //     format!(
            //         "{:#?}",
            //         parcel.tracking_status.last().unwrap_or(&TrackingStatus {
            //             time: Utc::now().into(),
            //             status: "Not found".to_string(),
            //             location: "".to_string(),
            //             detail: "".to_string()
            //         })
            //     ),
            // )
            // .await
            // .unwrap();
        }
        sleep(Duration::from_secs(60)).await;
    }
}
