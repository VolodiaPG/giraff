use std::collections::BinaryHeap;
use std::sync::Arc;

use tokio::sync::Mutex;
use warp::{http::Response, Rejection};

use crate::live_store::{BidDataBase, NodesDataBase};
use crate::{tasks, Error};
use sla::Sla;

/// Register a SLA and starts the auctionning process
pub async fn put_sla(
    bid_db: Arc<Mutex<BidDataBase>>,
    nodes_db: Arc<Mutex<NodesDataBase>>,
    sla: Sla,
) -> Result<impl warp::Reply, Rejection> {
    trace!("put sla: {:?}", sla);

    let id = tasks::call_for_bids(sla, bid_db.clone(), nodes_db).await?;

    let res;
    {
        res = bid_db.lock().await.get(&id).unwrap().clone();
    }

    Ok(
        Response::builder().body(serde_json::to_string(&res).map_err(|e| {
            error!("{}", e);
            warp::reject::custom(Error::SerializationError(e))
        })?),
    )
}
