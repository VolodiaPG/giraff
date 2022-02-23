use hyper::{Body, Request, Response, StatusCode};
use redis::Commands;
use std::error::Error;
use std::result::Result;

pub async fn handle(_req: Request<Body>) -> Result<Response<Body>, Box<dyn Error>> {
    let client = redis::Client::open("redis://redis-server/")?;
    let mut con = client.get_connection()?;
   
    let light: String = con.get("light")?;
    let blink: String = con.get("blink")?;

    let body = format!("{{\"light\":{},\"blink\": {}}}", light, blink);
    let res = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(body))?;
    Ok(res)
}
