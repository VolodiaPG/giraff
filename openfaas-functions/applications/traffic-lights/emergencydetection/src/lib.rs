use std::error::Error;

use serde::{Deserialize, Serialize};
use warp::{
    filters::BoxedFilter,
    http::{Response, StatusCode},
    Filter, Reply,
};

#[derive(Serialize)]
enum EmergencyType {
    // NONE,
    LUNATIC,
}

#[derive(Serialize)]
struct Emergency {
    active: bool,
    emergency_type: EmergencyType,
}

#[derive(Serialize)]
struct Outgoing {
    emergency: Emergency,
}

#[derive(Deserialize, PartialEq)]
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


#[derive(Deserialize)]
struct Object {
    object_type: ObjectType,
    // position_x: u16,
    // position_y: u16,
    // bound_x: u16,
    // bound_y: u16,
}

async fn handle(road_objects: Vec<Object>) -> Result<Box<dyn Reply>, Box<dyn Error>> {
    let emergencies: Vec<ObjectType> = vec![ObjectType::AMBULANCE, ObjectType::TRAIN];

    let emergency = road_objects
        .iter()
        .find(|object| emergencies.contains(&object.object_type))
        .map(|_| Emergency {
            active: true,
            emergency_type: EmergencyType::LUNATIC,
        });

    if let Some(emergency) = emergency {
        let client = reqwest::Client::new();

        let response = serde_json::to_string(&Outgoing { emergency })?;
        client
            .post("http://gateway.openfaas:8080/async-function/setlightphasecalculation")
            .body(response)
            .send()
            .await?;
    }

    Ok(Box::new(StatusCode::OK))
}

async fn reply(body: Vec<Object>) -> Result<Box<dyn Reply>, warp::Rejection> {
    match handle(body).await {
        Ok(reply) => Ok(reply),
        Err(e) => Ok(Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(e.to_string()),
        )),
    }
}

fn json_body() -> impl Filter<Extract = (Vec<Object>,), Error = warp::Rejection> + Clone {
    let ret = warp::body::json();
    ret
}

pub fn main() -> BoxedFilter<(Box<dyn Reply>,)> {
    warp::post().and(json_body()).and_then(reply).boxed()
}
