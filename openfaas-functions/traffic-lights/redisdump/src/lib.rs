use redis::Commands;
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    convert::Infallible,
    error::Error,
    hash::{Hash, Hasher},
};
use warp::{filters::BoxedFilter, http::StatusCode, Filter, Reply};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash)]
enum ObjectType {
    AMBULANCE,
    CAR,
    BICYCLE,
    PEDESTRIAN,
    TRAIN,
    TRUCK,
    VAN,
    MOTORCYCLE,
    ANIMAL,
}

#[derive(Deserialize, Serialize)]
struct Stat {
    at: String,
    count: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    light: Option<bool>,
    blink: Option<bool>,
    condition: Option<u8>,
    cars: Option<String>,
}

fn query_db() -> Result<Response, Box<dyn Error>> {
    let client = redis::Client::open("redis://redis-server/")?;
    let mut con = client.get_connection()?;

    let light = con.get("light").ok();
    let blink = con.get("blink").ok();
    let condition = con.get("condition").ok();

    let mut hasher = DefaultHasher::new();
    ObjectType::CAR.hash(&mut hasher);
    let hash_value = hasher.finish();
    let cars = con.get(format!("trafficstatistics:{}", hash_value)).ok();
    Ok(Response {
        light,
        blink,
        condition,
        cars,
    })
}

impl warp::Reply for Response {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::json(&self).into_response()
    }
}

async fn handle() -> Result<Box<dyn Reply>, Infallible> {
    match query_db() {
        Ok(response) => Ok(Box::new(response)),
        _ => Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR)),
    }
}

pub fn main() -> BoxedFilter<(Box<dyn Reply>,)> {
    warp::get().and_then(handle).boxed()
}
