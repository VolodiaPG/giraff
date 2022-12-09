use std::sync::Arc;
use std::time::Instant;

use model::domain::routing::Packet;
use model::view::routing::{Route, RouteLinking};
use serde_json::Value;
use tokio::task;

use crate::service::routing::{Error, Router};

#[instrument(level = "trace", skip(router, route))]
pub async fn register_route(
    router: &Arc<dyn Router>,
    route: Route,
) -> Result<(), Error> {
    trace!("Registering function route {:?}", route.function);
    router.register_function_route(route).await
}

#[instrument(level = "trace", skip(router, linking))]
pub async fn route_linking(
    router: &Arc<dyn Router>,
    linking: RouteLinking,
) -> Result<(), Error> {
    trace!("Linking route {:?}", linking.function);
    router.route_linking(linking, false).await
}

#[instrument(level = "trace", skip(packet, router))]
pub async fn post_sync_forward_function_routing(
    packet: Packet,
    router: Arc<dyn Router>,
) -> Result<Option<Value>, Error> {
    trace!("post forward routing from packet {:?}", packet);
    let start = Instant::now();
    let res = router.forward(packet).await;
    let elapsed = start.elapsed();
    debug!("Elapsed: {:?}", elapsed);
    trace!("{:?}", res);
    res
}

#[instrument(level = "trace", skip(packet, router))]
pub async fn post_async_forward_function_routing(
    packet: Packet,
    router: Arc<dyn Router>,
) -> () {
    task::spawn(async {
        post_sync_forward_function_routing(packet, router).await
    });
}
