use chrono::{DateTime, Utc};
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::SystemTime;
use std::{convert::Infallible, error::Error};
use warp::{filters::BoxedFilter, http::StatusCode, Filter, Reply};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
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
struct Object {
    object_type: ObjectType,
    position_x: u16,
    position_y: u16,
    bound_x: u16,
    bound_y: u16,
}

#[derive(Serialize)]
struct Stat {
    at: String,
    count: u32,
}

async fn handle(objects: Vec<Object>) -> std::result::Result<Box<dyn Reply>, Box<dyn Error>> {
    let client = redis::Client::open("redis://redis-server/")?;
    let mut con = client.get_connection()?;

    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    let now = now.to_rfc3339();

    let mut map: HashMap<ObjectType, u32> = HashMap::new();

    objects.iter().for_each(|object| {
        *map.entry(object.object_type.clone()).or_insert(0) += 1;
    });

    for (object_type, count) in map.iter() {
        let mut hasher = DefaultHasher::new();
        object_type.hash(&mut hasher);
        let hash_value = hasher.finish();
        con.append(
            format!("trafficstatistics:{}", hash_value),
            serde_json::to_string(&Stat {
                at: now.clone(),
                count: *count,
            })
            .unwrap() + ",",
        )?;
    }

    Ok(Box::new(StatusCode::OK))
}

async fn reply(
    // result: Result<Box<dyn Reply>, Box<dyn Error>>,
    body: Vec<Object>,
) -> Result<Box<dyn Reply>, Infallible> {
    match handle(body).await {
        Ok(reply) => Ok(reply),
        _ => Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR)),
    }
}

fn json_body() -> impl Filter<Extract = (Vec<Object>,), Error = warp::Rejection> + Clone {
    // warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    warp::body::json()
}

pub fn main() -> BoxedFilter<(impl Reply,)> {
    warp::post().and(json_body()).and_then(reply).boxed()
}
