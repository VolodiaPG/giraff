use std::sync::Arc;
use std::time::Instant;

use bytes::Bytes;

use model::domain::routing::Packet;
use model::view::routing::{Route, RouteLinking};

use crate::service::routing::Router;

#[instrument(level = "trace")]
pub async fn register_route(
    router: &Arc<dyn Router>,
    route: Route,
) -> anyhow::Result<()> {
    trace!("Registering function route {:?}", route.function);
    router.register_function_route(route).await.map_err(|e| anyhow::anyhow!(e))
}

#[instrument(level = "trace")]
pub async fn route_linking(
    router: &Arc<dyn Router>,
    linking: RouteLinking,
) -> anyhow::Result<()> {
    trace!("Linking route {:?}", linking.function);
    router.route_linking(linking, false).await.map_err(|e| anyhow::anyhow!(e))
}

#[instrument(level = "trace")]
pub async fn post_forward_function_routing(
    packet: &Packet<'_>,
    router: &Arc<dyn Router>,
) -> anyhow::Result<Bytes> {
    trace!("post forward routing from packet {:?}", packet);
    let start = Instant::now();
    let res = router.forward(packet).await.map_err(|e| anyhow::anyhow!(e));
    let elapsed = start.elapsed();
    debug!("Elapsed: {:?}", elapsed);
    res
}
