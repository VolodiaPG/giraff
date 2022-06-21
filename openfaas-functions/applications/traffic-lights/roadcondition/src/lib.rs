use serde::{Deserialize, Serialize};
use validator::Validate;
use warp::{filters::BoxedFilter, http::StatusCode, Filter, Reply};

type Result<T> = std::result::Result<T, warp::Rejection>;

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
struct Payload {
    road_condition: u8,
}

#[derive(Serialize)]
struct Response {
    condition: Payload,
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

async fn handle(body: IncomingPayload) -> Result<impl Reply> {
    if body.validate().is_err() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let conditions = Response {
        condition: Payload {
            road_condition: calculate_road_condition(body),
        },
    };

    let client = reqwest::Client::new();

    if let Ok(ret) = serde_json::to_string(&conditions) {
        if let Ok(_) = client
            .post("http://gateway.openfaas:8080/async-function/setlightphasecalculation")
            .body(ret.clone())
            .send()
            .await
        {
            return Ok(StatusCode::OK);
        }

        println!("{}", ret);
    }

    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

fn json_body() -> impl Filter<Extract = (IncomingPayload,), Error = warp::Rejection> + Clone {
    // warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    warp::body::json()
}

pub fn main() -> BoxedFilter<(impl Reply,)> {
    warp::post().and(json_body()).and_then(handle).boxed()
}
