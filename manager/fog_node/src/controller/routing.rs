use std::sync::Arc;
use std::time::Instant;

use model::domain::routing::Packet;
use serde_json::Value;
#[cfg(feature = "async_routes")]
use tokio::task;

use crate::service::routing::{Error, Router};

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

#[cfg(feature = "async_routes")]
#[instrument(level = "trace", skip(packet, router))]
pub async fn post_async_forward_function_routing(
    packet: Packet,
    router: Arc<dyn Router>,
) -> () {
    task::spawn(async {
        post_sync_forward_function_routing(packet, router).await
    });
}
