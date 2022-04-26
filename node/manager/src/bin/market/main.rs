#[macro_use]
extern crate log;
extern crate rocket;

use std::{env, sync::Arc};

use rocket::{launch, routes};
use rocket_okapi::{openapi_get_routes, swagger_ui::*};
use tokio::sync::Mutex;

use service::live_store::{BidDataBase, NodesDataBase};

use crate::handler::*;

mod auction;
mod controller;
mod handler;
mod local_model;
mod service;


#[launch]
async fn rocket() -> _ {
    std::env::set_var("RUST_LOG", "info, market=trace");
    env_logger::init();

    let predefined_clients_path = env::var("PREDEFINED_NODES_PATH").unwrap_or_else(|_| "predefined_nodes.ron".to_string());

    let db_bid = Arc::new(Mutex::new(BidDataBase::new()));
    let db_nodes = match NodesDataBase::new(predefined_clients_path) {
        Ok(db) => db,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };
    let db_nodes = Arc::new(Mutex::new(db_nodes));

    rocket::build()
        .manage(db_bid)
        .manage(db_nodes)
        // openapi_get_routes![...] will host the openapi document at `openapi.json`
        .mount("/swagger", openapi_get_routes![put_function, post_nodes])
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/swagger/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount("/api/", routes![put_function, post_nodes])
}
