use redis::Commands;
use serde::{Deserialize, Serialize};
use std::{error::Error, convert::Infallible};
use warp::{filters::BoxedFilter, http::StatusCode, Filter, Reply};

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    light: bool,
    blink: bool,
    condition: u8,
}

fn query_db() -> Result<Response, Box<dyn Error>> {
    let client = redis::Client::open("redis://redis-server/")?;
    let mut con = client.get_connection()?;
    
    let light = con.get("light")?;
    let blink = con.get("blink")?;
    let condition = con.get("condition")?;

    Ok(Response { light, blink, condition })
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
