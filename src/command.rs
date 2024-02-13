use super::*;

pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;

    Ok(())
}

pub async fn list(bot: Bot, msg: Message) -> HandlerResult {
    let trackers = db::list_tracker(msg.chat.id.0).await;

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
                format!("Added Tracker\nCompany: {company}\nTracking number: {tracking_number}"),
            )
            .await?;

            bot.send_message(
                msg.chat.id,
                format!(
                    "Item: {}\nDelivery Satus: {:?}",
                    parcel.item, parcel.delivery_status
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
    db::delete_tracker(id).await;
    bot.send_message(msg.chat.id, format!("Deleted tracker {}", id))
        .await?;
    Ok(())
}
