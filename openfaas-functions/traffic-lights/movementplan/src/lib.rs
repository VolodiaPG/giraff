use std::error::Error;

use redis::Commands;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;
use validator::ValidationError;
use warp::{
    filters::BoxedFilter,
    http::{Response, StatusCode},
    Filter, Reply,
};

#[derive(Serialize, Deserialize, Clone)]
enum Direction {
    NORTH,
    SOUTH,
    EAST,
    WEST,
}

#[derive(Serialize, Deserialize, Validate, Clone)]
struct Vehicule {
    #[validate(length(min = 1), custom = "validate_plate")]
    plate: String,
    direction: Direction,
    #[validate(range(min = -250, max = 250))]
    speed_kph: i16,
}

#[derive(Serialize)]
struct Outgoing {
    plans: Vec<Vehicule>,
}

impl PartialEq for Vehicule {
    fn eq(&self, other: &Self) -> bool {
        self.plate == other.plate
    }
}

fn validate_plate(plate: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^[A-Z]{2} [A-Z]{2} \d{1,7}$").unwrap();

    match re.is_match(plate) {
        true => Ok(()),
        _ => Err(ValidationError::new("Invalid plate format")),
    }
}

async fn handle(cars: Vec<Vehicule>) -> Result<Box<dyn Reply>, Box<dyn Error>> {
    let client = redis::Client::open("redis://redis-server/")?;
    let mut con = client.get_connection()?;

    let cars_db: Option<String> = con.get("trafficstatistics:cars")?;
    let cars_db = match cars_db {
        Some(s) => s,
        None => "[]".to_string(),
    };
    let mut cars_db: Vec<Vehicule> = serde_json::from_str(&cars_db)?;

    for car in cars {
        car.validate()?;
        if cars_db.contains(&car) {
            let index = cars_db.iter().position(|x| x == &car).unwrap();
            cars_db.remove(index);
        }
        cars_db.push(car);
        
    }

    let cars_db = cars_db
        .iter()
        .map(|car| car.clone())
        .rev()
        .take(12)
        .collect();

    let cars: String = serde_json::to_string(&cars_db)?;
    con.set("trafficstatistics:cars", cars)?;
    let client = reqwest::Client::new();

    let response = serde_json::to_string(&Outgoing { plans: cars_db })?;
    client
        .post("http://gateway.openfaas:8080/async-function/setlightphasecalculation")
        .body(response)
        .send()
        .await?;

    Ok(Box::new(StatusCode::OK))
}

async fn reply(body: Vec<Vehicule>) -> Result<Box<dyn Reply>, warp::Rejection> {
    match handle(body).await {
        Ok(reply) => Ok(reply),
        Err(e) => Ok(Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(e.to_string()),
        )),
    }
}

fn json_body() -> impl Filter<Extract = (Vec<Vehicule>,), Error = warp::Rejection> + Clone {
    let ret = warp::body::json();
    ret
}

pub fn main() -> BoxedFilter<(Box<dyn Reply>,)> {
    warp::post().and(json_body()).and_then(reply).boxed()
}
