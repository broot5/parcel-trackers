use serde_json::Value;

use super::*;

pub async fn get_cj_logistics(tracking_number: usize) -> Result<Parcel, serde_json::Error> {
    let params = [("wblNo", tracking_number)];

    let client = reqwest::Client::new();
    let parcel = client
        .post("https://trace.cjlogistics.com/next/rest/selectTrackingWaybil.do")
        .form(&params)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let tracking = client
        .post("https://trace.cjlogistics.com/next/rest/selectTrackingDetailList.do")
        .form(&params)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let tracking_number = parcel["data"]["wblNo"].as_str().unwrap().parse().unwrap();
    let sender = parcel["data"]["sndrNm"].as_str().unwrap().to_string();
    let receiver = parcel["data"]["rcvrNm"].as_str().unwrap().to_string();
    let item = parcel["data"]["repGoodsNm"].as_str().unwrap().to_string();
    let delivery_status = DeliveryStatus::Unknown;

    let mut tracking_status: Vec<Tracking> = Vec::new();
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

        tracking_status.push(Tracking {
            time,
            status,
            location,
            detail,
        })
    }

    Ok(Parcel {
        tracking_number,
        sender,
        receiver,
        item,
        delivery_status,
        tracking_status,
    })
}
