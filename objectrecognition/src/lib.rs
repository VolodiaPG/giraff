use bytes::Buf;
use futures::TryStreamExt;
use jpeg_decoder::Decoder;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::Serialize;
use std::error::Error;
use std::fmt;
use warp::multipart::{FormData, Part};
use warp::{filters::BoxedFilter, http::StatusCode, Filter, Reply};
type Result<T> = std::result::Result<T, warp::Rejection>;

#[derive(Serialize)]
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

#[derive(Serialize)]
struct Object {
    object_type: ObjectType,
    position_x: u16,
    position_y: u16,
    bound_x: u16,
    bound_y: u16,
}

impl Distribution<ObjectType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ObjectType {
        match rng.gen_range(0..8) {
            _ => ObjectType::CAR,
            // 0 => ObjectType::AMBULANCE,
            // 1 => ObjectType::CAR,
            // 2 => ObjectType::BICYCLE,
            // 3 => ObjectType::PEDESTRIAN,
            // 4 => ObjectType::TRAIN,
            // 5 => ObjectType::TRUCK,
            // 6 => ObjectType::VAN,
            // 7 => ObjectType::MOTORCYCLE,
            // 8 => ObjectType::ANIMAL,
        }
    }
}

fn get_random_objects() -> Vec<Object> {
    // randomly generate between 0 and 10 Objects
    let mut rng = rand::thread_rng();
    let mut objects: Vec<Object> = Vec::new();
    for _ in 1..rng.gen_range(1..10) {
        let object = Object {
            object_type: rand::random(),
            position_x: rng.gen_range(0..1000),
            position_y: rng.gen_range(0..1000),
            bound_x: rng.gen_range(0..400),
            bound_y: rng.gen_range(0..400),
        };
        objects.push(object);
    }

    objects
}

pub async fn handle(form: FormData) -> Result<Box<dyn Reply>> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        eprintln!("form error: {}", e);
        warp::reject::reject()
    })?;

    println!("{} parts", parts.len());

    for mut p in parts {
        if p.name() == "file" {
            if Some("image/jpeg") != p.content_type() {
                return Err(warp::reject::reject());
            }

            if let Some(Ok(val)) = p.data().await {
                let mut decoder = Decoder::new(val.reader());
                // check if the image is decodable
                if decoder.decode().is_err() {
                    return Err(warp::reject::reject());
                }

                if let Some(metadata) = decoder.info() {
                    println!("{}x{}", metadata.width, metadata.height);
                }

                let objects = get_random_objects();
                if objects.len() > 0 {
                    if let Ok(ret) = serde_json::to_string(&objects) {
                        let client = reqwest::Client::new();
                        client
                            .post("http://gateway.openfaas:8080/async-function/trafficstatistics")
                            .body(ret.clone())
                            .send()
                            .await
                            .map_err(|_| warp::reject::reject())?;
                        client
                            .post("http://gateway.openfaas:8080/async-function/movementplan")
                            .body(ret.clone())
                            .send()
                            .await
                            .map_err(|_| warp::reject::reject())?;
                        client
                            .post("http://gateway.openfaas:8080/async-function/emergencydetection")
                            .body(ret)
                            .send()
                            .await
                            .map_err(|_| warp::reject::reject())?;
                        // return Ok(Box::new(ret));
                    }
                }
            }
        }
    }

    Ok(Box::new(StatusCode::OK))
}

pub fn main() -> BoxedFilter<(impl Reply,)> {
    warp::post()
        // .and(warp::multipart::form().max_length(5_000_000))
        .and(warp::multipart::form())
        .and_then(handle)
        .boxed()
}
