use serde::{Deserialize, Serialize};
use std::error::Error;
use std::result::Result;
use validator::Validate;

// {
//     "temperature_celsius": 25.4,
//     "humidity_percent": 70.0,
//     "wind_kph": 100.0,
//     "rain": false
//     }

#[derive(Deserialize, Validate)]
struct IncomingPayload {
    #[validate(range(min = -273.15, max = 100.0))]
    temperature_celsius: f32,
    #[validate(range(min = 0.0, max = 100.0))]
    humidity_percent: f32,
    #[validate(range(min = 0.0, max = 150.0))]
    wind_kph: f32,
    rain: bool,
}

#[derive(Serialize)]
struct OutgoingPayload {
    road_condition: u8,
}

fn calculate_road_condition(payload: IncomingPayload) -> u8 {
    let mut condition = 0;
    if payload.temperature_celsius < 4.0 {
        condition += 1;
    }

    if payload.humidity_percent > 75.0 {
        condition += 1;
    }

    // beaufort 7
    if payload.wind_kph > 15.0 {
        condition += 1;
    }

    // beaufort 10
    if payload.wind_kph > 25.0 {
        condition += 1;
    }
    if payload.rain {
        condition += 1;
    }

    condition
}

pub async fn handle(req: String) -> Result<String, Box<dyn Error>> {
    let des: IncomingPayload = serde_json::from_str(req.as_str())?;
    des.validate()?;
    let client = reqwest::Client::new();
    let conditions = OutgoingPayload {
        road_condition: calculate_road_condition(des),
    };

    let ret = serde_json::to_string(&conditions)?;
    client
        .post("http://gateway.openfaas:8080/function/lightcalculation")
        .body(ret)
        .send().await?;

    Ok("".to_string())
}
