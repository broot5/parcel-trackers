mod command;
mod db;
mod getter;

use chrono::prelude::*;
use command::*;
use dotenvy::dotenv;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::time::{sleep, Duration};

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
struct Parcel {
    company: String,
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

    dotenv().expect(".env file not found");

    let bot = Bot::new(std::env::var("TELOXIDE_TOKEN").unwrap());

    tokio::spawn(poll(bot.clone()));

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(Update::filter_callback_query().endpoint(callback_handler));

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
    #[command(description = "/add <company> <tracking_number>", parse_with = "split")]
    Add {
        company: String,
        tracking_number: String,
    },
    #[command(description = "/delete <index>")]
    Delete { id: i64 },
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
    }
}

async fn callback_handler(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(answer) = q.data {
        let mut text = String::new();

        let callback_data = answer.parse::<i64>().unwrap();

        match callback_data {
            1..=i64::MAX => {
                db::delete_tracker(callback_data, q.from.id.0.try_into().unwrap()).await;
                text = String::from("Tracker deleted!");
            }
            0 => {}
            i64::MIN..=-1 => {
                db::update_keep(callback_data.abs(), 1).await;
                text = String::from("Tracker kept!")
            }
        }

        bot.answer_callback_query(q.id).await?;

        if let Some(Message { id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, id, text).await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, text).await?;
        }
    }

    Ok(())
}

async fn poll(bot: Bot) {
    loop {
        let trackers = db::list_all_tracker().await;

        for tracker in trackers {
            let parcel = getter::get(&tracker.company, &tracker.tracking_number)
                .await
                .unwrap();

            if parcel.last_updated_time.timestamp() > tracker.last_updated_timestamp {
                db::update_last_updated_timestamp(tracker.id).await;

                let last_tracking_status = parcel.tracking_status.last().unwrap();

                let text = format!(
                    "New update for your package!\nTime: {}\nStatus: {}\nLocation: {}{}",
                    last_tracking_status.time,
                    last_tracking_status.status,
                    last_tracking_status.location,
                    if !last_tracking_status.detail.is_empty() {
                        format!("\nDetail: {}", last_tracking_status.detail)
                    } else {
                        String::new()
                    },
                );

                bot.send_message(tracker.chat_id.to_string(), text)
                    .await
                    .unwrap();
            }

            match parcel.delivery_status {
                DeliveryStatus::InProgress => {}
                DeliveryStatus::Completed => {
                    if tracker.keep == 0 {
                        let keyboard = confirm_delete(tracker.id);
                        bot.send_message(tracker.chat_id.to_string(), "One of your trackers has been marked as completed. Do you want to remove it?")
                            .reply_markup(keyboard)
                            .await
                            .unwrap();
                    }
                }
                DeliveryStatus::Unknown => {}
            }
        }
        sleep(Duration::from_secs(60)).await;
    }
}

fn confirm_delete(id: i64) -> InlineKeyboardMarkup {
    let keyboard = vec![vec![
        InlineKeyboardButton::callback("Yes", id.to_string()), // {Yes, id} = id
        InlineKeyboardButton::callback("No", (-id).to_string()), // {No, id} = -id
    ]];

    InlineKeyboardMarkup::new(keyboard)
}
