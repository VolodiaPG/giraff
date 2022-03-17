use std::sync::Arc;

use tokio::sync::Mutex;
use uuid::Uuid;
use warp::{http::Response, Rejection};

use crate::live_store::BidDataBase;
use crate::models::{AcceptBid, Bid, BidRecord, ProvisionedRecord, Satisfiable};
use sla::Sla;

/// Register a SLA and starts the auctionning process
pub async fn put_sla(
    sla: Sla,
) -> Result<impl warp::Reply, Rejection> {
    trace!("put sla sla: {:?}", sla);

    // let bid = bid(&sla).await.map_err(|e| {
    //     error!("{:#?}", e);
    //     warp::reject::custom(crate::Error::NodeLogicError(e))
    // })?;

    // let bid = BidRecord { bid: bid, sla: sla };

    // let id;

    // {
    //     id = bid_db.lock().await.insert(bid.clone());
    // }

    Ok(Response::builder().body("Hello there!"))
}
