use chrono::serde::ts_microseconds;
use chrono::{DateTime, Utc};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use warp::{filters::BoxedFilter, http::StatusCode, Filter, Reply};

type Result<T> = std::result::Result<T, warp::Rejection>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncomingPayload {
    address_to_call: String,
    data: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OutgoingPayload {
    #[serde(with = "ts_microseconds")]
    timestamp: DateTime<Utc>,
    data: Value,
}

async fn handle(payload: IncomingPayload) -> Result<impl Reply> {
    if reqwest::Client::new()
        .post(payload.address_to_call)
        .json(&OutgoingPayload {
            timestamp: chrono::offset::Utc::now(),
            data: payload.data,
        })
        .send()
        .await
        .is_ok()
    {
        return Ok(Box::new(StatusCode::OK));
    };
    Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
}

fn json_body() -> impl Filter<Extract = (IncomingPayload,), Error = warp::Rejection> + Clone {
    // warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    warp::body::json()
}

pub fn main() -> BoxedFilter<(impl Reply,)> {
    warp::post().and(json_body()).and_then(handle).boxed()
}
