use std::sync::Arc;

use anyhow;
use bytes::Bytes;

use manager::model::domain::routing::{FunctionRoutingStack, Packet};

use crate::service::routing::Router;

pub async fn register_route(
    router: &Arc<dyn Router>,
    stack: FunctionRoutingStack,
) -> anyhow::Result<()> {
    trace!("put routing {:?}", stack.function);
    router
        .register_function_route(stack)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn post_forward_function_routing(
    packet: &Packet<'_>,
    router: &Arc<dyn Router>,
) -> anyhow::Result<Bytes> {
    trace!("post forward routing from packet {:?}", packet);
    Ok(router
        .forward(packet)
        .await
        .map_err(|e| anyhow::anyhow!(e))?)
}
