use crate::*;
use scraper::{Html, Selector};
use serde_json::Value;
use std::error::Error;

pub async fn get(company: &str, tracking_number: &str) -> Option<Parcel> {
    match company {
        "CJ대한통운" => Some(get_cj_logistics(tracking_number).await.unwrap()),
        "우체국" => Some(get_epost(tracking_number).await.unwrap()),
        _ => None,
    }
}

async fn get_cj_logistics(tracking_number: &str) -> Result<Parcel, Box<dyn Error>> {
    let params = [("wblNo", tracking_number)];

    let client = reqwest::Client::new();
    let parcel_response = client
        .post("https://trace.cjlogistics.com/next/rest/selectTrackingWaybil.do")
        .form(&params)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let tracking_response = client
        .post("https://trace.cjlogistics.com/next/rest/selectTrackingDetailList.do")
        .form(&params)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let tracking_status = tracking_response["data"]["svcOutList"]
        .as_array()
        .ok_or("Failed to parse tracking status")?
        .iter()
        .map(|i| {
            // UTC+9
            let time = format!(
                "{} {} +0900",
                i["workDt"].as_str().ok_or("Missing workDt")?,
                i["workHms"].as_str().ok_or("Missing workHms")?
            );

            let time = DateTime::parse_from_str(&time, "%Y-%m-%d %H:%M:%S %z").unwrap();

            Ok(TrackingStatus {
                time,
                status: i["crgStDnm"].as_str().unwrap_or_default().to_string(),
                location: i["branNm"].as_str().unwrap_or_default().to_string(),
                detail: i["crgStDcdVal"].as_str().unwrap_or_default().to_string(),
            })
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    let delivery_status = tracking_status
        .last()
        .map(
            |last_tracking_status| match last_tracking_status.status.as_str() {
                "집화처리" => DeliveryStatus::InProgress,
                "간선상차" => DeliveryStatus::InProgress,
                "간선하차" => DeliveryStatus::InProgress,
                "행낭포장" => DeliveryStatus::InProgress,
                "배송출발" => DeliveryStatus::InProgress,
                "배송완료" => DeliveryStatus::Completed,
                _ => DeliveryStatus::Unknown,
            },
        )
        .unwrap_or(DeliveryStatus::Unknown);

    let last_updated_time = tracking_status
        .last()
        .map(|last_tracking_status| last_tracking_status.time)
        .unwrap_or_default();

    Ok(Parcel {
        company: String::from("CJ대한통운"),
        tracking_number: parcel_response["data"]["wblNo"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        sender: parcel_response["data"]["sndrNm"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        receiver: parcel_response["data"]["rcvrNm"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        item: parcel_response["data"]["repGoodsNm"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        delivery_status,
        tracking_status,
        last_updated_time,
    })
}

async fn get_epost(tracking_number: &str) -> Result<Parcel, Box<dyn Error>> {
    let response = reqwest::get(format!(
        "https://service.epost.go.kr/trace.RetrieveDomRigiTraceList.comm?sid1={}&displayHeader=N",
        tracking_number
    ))
    .await?;

    let text = response.text().await?;

    let document = Html::parse_document(&text);

    let delivery_status = match document.select(&Selector::parse(r#"table.table_col:nth-child(3) > tbody:nth-child(4) > tr:nth-child(1) > td:nth-child(6)"#).unwrap()).next().unwrap().text().next().unwrap().trim() {
        "접수" => DeliveryStatus::InProgress,
        "발송" => DeliveryStatus::InProgress,
        "배달준비" => DeliveryStatus::InProgress,
        "배달완료" => DeliveryStatus::Completed,
        _ => DeliveryStatus::Unknown,
    };

    let mut tracking_status: Vec<TrackingStatus> = Vec::new();

    for i in document.select(&Selector::parse(r#"#processTable > tbody > tr"#).unwrap()) {
        let time = format!(
            "{} {} +0900",
            i.select(&Selector::parse("td").unwrap())
                .nth(0)
                .unwrap()
                .inner_html(),
            i.select(&Selector::parse("td").unwrap())
                .nth(1)
                .unwrap()
                .inner_html()
        );
        let time = DateTime::parse_from_str(&time, "%Y.%m.%d %H:%M %z").unwrap();

        let status = i
            .select(&Selector::parse("td:nth-child(4) > span:nth-child(1)").unwrap())
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .to_string();

        let location = i
            .select(&Selector::parse(r#"td:nth-child(3) > a"#).unwrap())
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .to_string();

        let detail = i
            .select(&Selector::parse("td:nth-child(4) > span:nth-child(2)").unwrap())
            .next()
            .map(|e| e.text().next())
            .unwrap_or_default()
            .unwrap_or_default()
            .trim()
            .to_string();

        tracking_status.push(TrackingStatus {
            time,
            status,
            location,
            detail,
        });
    }

    let last_updated_time = tracking_status
        .last()
        .map(|last_tracking_status| last_tracking_status.time)
        .unwrap_or_default();

    Ok(Parcel {
        company: String::from("우체국"),
        tracking_number: document
            .select(&Selector::parse(r#"table.table_col > tbody > tr > th:nth-child(1)"#).unwrap())
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .into(),
        sender: document
            .select(&Selector::parse(r#"table.table_col > tbody > tr > td:nth-child(2)"#).unwrap())
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap_or_default()
            .into(),
        receiver: document
            .select(&Selector::parse(r#"table.table_col > tbody > tr > td:nth-child(3)"#).unwrap())
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap_or_default()
            .into(),
        item: document
            .select(&Selector::parse(r#"table.table_col > tbody > tr > td:nth-child(5)"#).unwrap())
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap_or_default()
            .into(),
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
        dotenv().expect(".env file not found");

        let parcel = get_cj_logistics(&std::env::var("TEST_CJ_LOGISTICS").unwrap())
            .await
            .unwrap();

        println!("{:#?}", parcel)
    }

    #[tokio::test]
    async fn epost() {
        dotenv().expect(".env file not found");

        let parcel = get_epost(&std::env::var("TEST_EPOST").unwrap())
            .await
            .unwrap();

        println!("{:#?}", parcel)
    }
}
