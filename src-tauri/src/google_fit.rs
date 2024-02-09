use std::str::FromStr;

use reqwest::Url;
use serde_json::json;

use crate::http::REQWEST_CLIENT;

pub async fn create_fit_data_source(access_token: &str) -> String {
    let client = REQWEST_CLIENT;
    let request_url =
        Url::from_str("https://www.googleapis.com/fitness/v1/users/me/dataSources").unwrap();
    let json_body = json!({
        "dataStreamName": "HydrationSource",
        "type": "raw",
        "application": {
            "detailsUrl": "https://github.com/angeloanan/hydrate-reminder",
            "name": "Hydrate Reminder",
            "version": "0.0.1"
        },
        "dataType": {
            "name": "com.google.hydration",
            "field": [
                {
                    "name": "volume",
                    "format": "floatPoint",
                    "optional": false
                }
            ]
        }
    });

    let fetch_response = client
        .post(request_url)
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&json_body)
        .send()
        .await
        .expect("Unable to create Fit data source!");

    let response_json = fetch_response
        .json::<serde_json::Value>()
        .await
        .expect("Unable to parse JSON response!");

    response_json["dataStreamId"].to_string()
}

pub async fn write_water_intake_data(
    amount_ml: f64,
    utc_sec: i64,
    data_stream_id: String,
    access_token: &str,
) {
    let client = REQWEST_CLIENT;
    let request_url = Url::from_str(&format!(
        "https://www.googleapis.com/fitness/v1/users/me/dataSources/{data_stream_id}/datasets/"
    ))
    .unwrap();
    let ns = utc_sec * 1_000_000_000;
    let amount_litre = amount_ml / 1000.0;

    let json_body = json!({
        "dataSourceId": data_stream_id,
        "maxEndTimeNs": ns,
        "minStartTimeNs": ns,
        "point": [
            {
                "dataTypeName": "com.google.hydration",
                "endTimeNanos": ns,
                "startTimeNanos": ns,
                "value": [
                    { "fpVal": amount_litre }
                ],
            }
        ]
    });

    client
        .post(request_url)
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&json_body)
        .send()
        .await
        .expect("Unable to write water intake data!");
}
