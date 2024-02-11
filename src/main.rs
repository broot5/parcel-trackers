use chrono::prelude::*;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

mod getter;

#[derive(Debug)]
struct Parcel {
    tracking_number: usize,
    sender: String,
    receiver: String,
    item: String,
    delivery_status: DeliveryStatus,
    tracking_status: Vec<Tracking>,
}

#[derive(Debug)]
struct Tracking {
    time: DateTime<FixedOffset>,
    status: String,
    location: String,
    detail: String,
}

#[derive(Debug)]
enum DeliveryStatus {
    InProgress,
    Complete,
    Unknown,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();

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
        tracking_number: usize,
    },
    #[command(description = "list added trackers")]
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
        } => {
            bot.send_message(
                msg.chat.id,
                format!("Company: {company}\nTracking number: {tracking_number}"),
            )
            .await?
        }
        Command::List => bot.send_message(msg.chat.id, format!("List")).await?,
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cj_logistics() {
        let parcel =
            getter::get_cj_logistics(std::env::var("CJ_LOGISTICS").unwrap().parse().unwrap())
                .await
                .unwrap();

        println!("{:#?}", parcel)
    }
}
