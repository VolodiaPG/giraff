#[macro_use]
extern crate log;
extern crate rocket;

use rocket::post;
use rocket::serde::json::Json;
use rocket::{launch, routes};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Payload(pub String);

#[post("/print", data = "<payload>")]
pub async fn print(payload: String) {
    info!("{:?}", payload);
}

#[launch]
async fn rocket() -> _ {
    std::env::set_var("RUST_LOG", "info, echo=trace");
    env_logger::init();

    rocket::build().mount("/api", routes![print])
}
