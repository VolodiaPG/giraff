use std::sync::Arc;

use anyhow;

use manager::model::domain::routing::RoutingStack;
use manager::model::BidId;

use crate::service::routing::Router;

pub async fn register_route(router: &Arc<dyn Router>, stack: RoutingStack) -> anyhow::Result<()> {
    trace!("put routing {:?}", stack.function);
    router
        .register_route(stack)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn post_forward_function_routing(
    function_id: BidId,
    router: &Arc<dyn Router>,
    payload: Vec<u8>,
) -> anyhow::Result<()> {
    trace!("post forward routing {:?}", function_id);
    router
        .forward(
            &function_id,
            format!("/function/{}", function_id.to_string()),
            payload,
        )
        .await
        .map_err(|e| anyhow::anyhow!(e))
}
