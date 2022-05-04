#![feature(future_join)]
#[macro_use]
extern crate log;
extern crate rocket;

use std::{env, sync::Arc};

use rocket::launch;
use rocket_okapi::{openapi_get_routes, swagger_ui::*};

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

    let fog_node = Arc::new(FogNodeImpl::new());
    let fog_node_communication =
        Arc::new(crate::repository::node_communication::NodeCommunicationThroughRoutingImpl::new());
    let auction_process = Arc::new(crate::repository::auction::SecondPriceAuction::new());

    // Services
    let auction_service = Arc::new(service::auction::AuctionImpl::new(
        fog_node.clone(),
        auction_process,
        fog_node_communication,
    ));
    let fog_node_network_service = Arc::new(
        service::fog_node_network::FogNodeNetworkHashTreeImpl::new(fog_node.clone()),
    );

    rocket::build()
        .manage(auction_service as Arc<dyn crate::service::auction::Auction>)
        .manage(
            fog_node_network_service as Arc<dyn crate::service::fog_node_network::FogNodeNetwork>,
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/api/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/api/",
            openapi_get_routes![put_function, post_register_node],
        )
}
