use drpc::codec::{BinCodec, JsonCodec};
use drpc::server::Server;
use model::domain::routing::Packet;
use model::FogNodeRPCPort;
use serde_json::value::RawValue;
use std::sync::Arc;

use crate::controller;
use crate::service::routing::Router;

/// Routes the request to the correct URL and node.
async fn routing(
    packet: Packet,
    router: Arc<dyn Router>,
) -> drpc::Result<Box<RawValue>> {
    controller::routing::post_forward_function_routing(packet, &router)
        .await
        .map_err(|e| drpc::Error::from(e.to_string()))
}

pub async fn serve_rpc(port: FogNodeRPCPort, router: Arc<dyn Router>) {
    let mut s = Server::<JsonCodec>::new();

    s.register_fn("routing", move |packet| {
        let router = router.clone();
        async { routing(packet, router).await }
    });

    s.serve(format!("0.0.0.0:{}", port)).await;
}
