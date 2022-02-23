use hyper::body;
use hyper::{Body, Request, Response, StatusCode};
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

#[derive(Serialize, Deserialize, Validate)]
struct IncomingPayload {
    #[validate(range(min = -273.15, max = 100.0))]
    temperature_celsius: f32,
    #[validate(range(min = 0.0, max = 100.0))]
    humidity_percent: f32,
    #[validate(range(min = 0.0, max = 150.0))]
    wind_kph: f32,
    rain: bool,
}

pub async fn handle(req: Request<Body>) -> Result<Response<Body>, Box<dyn Error>> {
    let bytes = body::to_bytes(req.into_body()).await?;
    let body: IncomingPayload = serde_json::from_slice(&bytes)?;
    body.validate()?;

    let client = reqwest::Client::new();
    client
        .post("http://gateway.openfaas:8080/function/roadcondition")
        .body(bytes)
        .send()
        .await?;

    let res = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(vec![]))?;
    Ok(res)
}
