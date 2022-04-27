#![feature(future_join)]
#[macro_use]
extern crate log;
extern crate rocket;

use std::{env, sync::Arc};

use rocket::{launch, routes};
use rocket_okapi::{openapi_get_routes, swagger_ui::*};
use tokio::sync::Mutex;

use manager::helper::from_disk::FromDisk;

use crate::handler::*;
use crate::repository::fog_node::FogNodeImpl;

mod controller;
mod handler;
mod repository;
mod service;

#[launch]
async fn rocket() -> _ {
    std::env::set_var("RUST_LOG", "info, market=trace");
    env_logger::init();

    let predefined_clients_path =
        env::var("PREDEFINED_NODES_PATH").unwrap_or_else(|_| "predefined_nodes.ron".to_string());

    // Repositories
    let fog_node = match FogNodeImpl::from_disk(predefined_clients_path.as_ref()).await {
        Ok(db) => {
            info!("Loading nodes from disk, path: {}", predefined_clients_path);
            db
        }
        Err(e) => {
            error!("{}", e.to_string());
            std::process::exit(1);
        }
    };
    let fog_node = Arc::new(fog_node);
    let fog_node_communication =
        Arc::new(crate::repository::node_communication::NodeCommunicationImpl::new());
    let auction_process = Arc::new(crate::repository::auction::SecondPriceAuction::new());

    // Services
    let auction_service = Arc::new(service::auction::AuctionImpl::new(
        fog_node,
        auction_process,
        fog_node_communication,
    ));

    rocket::build()
        .manage(auction_service as Arc<dyn crate::service::auction::Auction>)
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/api/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount("/api/", openapi_get_routes![put_function])
}
