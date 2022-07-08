use std::error::Error;

use warp::{
    filters::BoxedFilter,
    http::{Response, StatusCode},
    Filter, Reply,
};

async fn handle() -> Result<Box<dyn Reply>, Box<dyn Error>> {
    Ok(Box::new(StatusCode::OK))
}

async fn reply() -> Result<Box<dyn Reply>, warp::Rejection> {
    match handle().await {
        Ok(reply) => Ok(reply),
        Err(e) => Ok(Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(e.to_string()),
        )),
    }
}


pub fn main() -> BoxedFilter<(Box<dyn Reply>,)> {
    loop {
        const LIMIT: usize = 1_000_000_000_000;

        let count = primal::StreamingSieve::prime_pi(LIMIT);
        println!("there are {} primes below 1 million", count);
    }

    warp::post().and_then(reply).boxed()
}
