use std::error::Error;
use hyper::{Body, Request, Response};

const PHRASE: &str = "Hello, World!";

pub async fn handle(_req: Request<Body>) -> Result<Response<Body>, Box<dyn Error>> {
    Ok(Response::new(Body::from(PHRASE)))
}
