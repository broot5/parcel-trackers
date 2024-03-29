use crate::*;

pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;

    Ok(())
}

pub async fn list(bot: Bot, msg: Message) -> HandlerResult {
    let trackers = db::list_tracker(msg.chat.id.0).await;

    for tracker in &trackers {
        let parcel = getter::get(&tracker.company, &tracker.tracking_number)
            .await
            .unwrap();

        bot.send_message(
            msg.chat.id,
            format!(
                "Index: {}\nCompany: {}\nTracking number: {}\nItem: {}\nAdded time: {}",
                tracker.id,
                tracker.company,
                tracker.tracking_number,
                parcel.item,
                DateTime::<Utc>::from_timestamp(tracker.added_timestamp, 0).unwrap()
            ),
        )
        .await?;
    }

    if trackers.is_empty() {
        bot.send_message(
            msg.chat.id,
            "Your tracker list is empty. Start adding trackers by typing /add <company> <tracking_number>",
        )
        .await?;
    }

    Ok(())
}

pub async fn add(
    bot: Bot,
    msg: Message,
    company: String,
    tracking_number: String,
) -> HandlerResult {
    match getter::get(&company, &tracking_number).await {
        Some(parcel) => {
            db::add_tracker(
                msg.chat.id.0,
                &company,
                &tracking_number,
                parcel.last_updated_time.timestamp(),
            )
            .await;

            bot.send_message(
                msg.chat.id,
                format!(
                    "Added Tracker\nCompany: {}\nTracking number: {}",
                    parcel.company, parcel.tracking_number
                ),
            )
            .await?;

            bot.send_message(
                msg.chat.id,
                format!(
                    "Sender: {}\nReceiver: {}\nItem: {}\nDelivery Satus: {:?}",
                    parcel.sender, parcel.receiver, parcel.item, parcel.delivery_status
                ),
            )
            .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Invalid company name")
                .await?;
        }
    }
    Ok(())
}

pub async fn delete(bot: Bot, msg: Message, id: i64) -> HandlerResult {
    db::delete_tracker(id, msg.chat.id.0).await;
    bot.send_message(msg.chat.id, format!("Deleted tracker {}", id))
        .await?;
    Ok(())
}
