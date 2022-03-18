use std::sync::Arc;

use tokio::sync::Mutex;
use warp::{http::Response, Rejection};

use crate::live_store::BidDataBase;
use crate::models::BidRecord;
use sla::Sla;

/// Register a SLA and starts the auctionning process
pub async fn put_sla(
    bid_db: Arc<Mutex<BidDataBase>>,
    sla: Sla,
) -> Result<impl warp::Reply, Rejection> {
    trace!("put sla sla: {:?}", sla);

    let bid = BidRecord {
        sla: sla,
        bids: vec![],
    };

    let id;

    {
        id = bid_db.lock().await.insert(bid.clone());
    }

    Ok(Response::builder().body(id.to_string()))
}
