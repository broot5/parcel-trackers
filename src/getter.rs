use serde_json::Value;
use std::error::Error;

use super::*;

pub async fn get(company: &str, tracking_number: &String) -> Option<Parcel> {
    match company {
        "CJ대한통운" => Some(get_cj_logistics(tracking_number).await.unwrap()),
        _ => None,
    }
}

async fn get_cj_logistics(tracking_number: &String) -> Result<Parcel, Box<dyn Error>> {
    let params = [("wblNo", tracking_number)];

    let client = reqwest::Client::new();
    let parcel = client
        .post("https://trace.cjlogistics.com/next/rest/selectTrackingWaybil.do")
        .form(&params)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let tracking = client
        .post("https://trace.cjlogistics.com/next/rest/selectTrackingDetailList.do")
        .form(&params)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let tracking_number = parcel["data"]["wblNo"].as_str().unwrap().parse().unwrap();
    let sender = parcel["data"]["sndrNm"].as_str().unwrap().to_string();
    let receiver = parcel["data"]["rcvrNm"].as_str().unwrap().to_string();
    let item = parcel["data"]["repGoodsNm"].as_str().unwrap().to_string();

    let mut tracking_status: Vec<TrackingStatus> = Vec::new();
    for i in tracking["data"]["svcOutList"].as_array().unwrap() {
        //UTC+9
        let time = i["workDt"].as_str().unwrap().to_owned()
            + " "
            + i["workHms"].as_str().unwrap()
            + " "
            + "+0900";

        let time = DateTime::parse_from_str(&time, "%Y-%m-%d %H:%M:%S %z").unwrap();

        let status = i["crgStDnm"].as_str().unwrap().to_string();
        let location = i["branNm"].as_str().unwrap().to_string();
        let detail = i["crgStDcdVal"].as_str().unwrap().to_string();

        tracking_status.push(TrackingStatus {
            time,
            status,
            location,
            detail,
        })
    }

    let delivery_status = tracking_status
        .last()
        .map(
            |last_tracking_status| match last_tracking_status.status.as_str() {
                "배송완료" => DeliveryStatus::Completed,
                _ => DeliveryStatus::InProgress,
            },
        )
        .unwrap_or(DeliveryStatus::Unknown);

    let last_updated_time = tracking_status
        .last()
        .map(|last_tracking_status| last_tracking_status.time)
        .unwrap_or_default();

    Ok(Parcel {
        tracking_number,
        sender,
        receiver,
        item,
        delivery_status,
        tracking_status,
        last_updated_time,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn cj_logistics() {
        let parcel = getter::get_cj_logistics(&std::env::var("TEST_CJ_LOGISTICS").unwrap())
            .await
            .unwrap();

        println!("{:#?}", parcel)
    }
}
