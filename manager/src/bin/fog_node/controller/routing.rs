use std::sync::Arc;

use bytes::Bytes;

use manager::model::domain::routing::Packet;
use manager::model::view::routing::{Route, RouteLinking};

use crate::service::routing::Router;

pub async fn register_route(
    router: &Arc<dyn Router>,
    route: Route,
) -> anyhow::Result<()> {
    trace!("Registering route {:?}", route.function);
    router.register_function_route(route).await.map_err(|e| anyhow::anyhow!(e))
}

pub async fn route_linking(
    router: &Arc<dyn Router>,
    linking: RouteLinking,
) -> anyhow::Result<()> {
    trace!("Linking route {:?}", linking.function);
    router.route_linking(linking, false).await.map_err(|e| anyhow::anyhow!(e))
}

pub async fn post_forward_function_routing(
    packet: &Packet<'_>,
    router: &Arc<dyn Router>,
) -> anyhow::Result<Bytes> {
    trace!("post forward routing from packet {:?}", packet);
    router.forward(packet).await.map_err(|e| anyhow::anyhow!(e))
}
