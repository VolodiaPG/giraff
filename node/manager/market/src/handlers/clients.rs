use std::sync::Arc;

use tokio::sync::Mutex;
use warp::{http::Response, Rejection};

use crate::live_store::{BidDataBase, NodesDataBase};
use crate::models::MarketBidProposal;
use crate::{auction, tasks, Error};
use sla::Sla;

/// Register a SLA and starts the auctionning process
pub async fn put_sla(
    bid_db: Arc<Mutex<BidDataBase>>,
    nodes_db: Arc<Mutex<NodesDataBase>>,
    sla: Sla,
) -> Result<impl warp::Reply, Rejection> {
    trace!("put sla: {:?}", sla);

    let id = tasks::call_for_bids(sla.clone(), bid_db.clone(), nodes_db).await?;

    let res;
    {
        res = bid_db.lock().await.get(&id).unwrap().clone();
    }

    let auctions_result = auction::second_price(&sla, &res.bids);
    let res = match auctions_result {
        Some((bid, price)) => MarketBidProposal {
            bids: res.bids,
            chosen_bid: Some(bid),
            price: Some(price),
        },
        None => MarketBidProposal {
            bids: res.bids,
            chosen_bid: None,
            price: None,
        },
    };

    Ok(
        Response::builder().body(serde_json::to_string(&res).map_err(|e| {
            error!("{}", e);
            warp::reject::custom(Error::Serialization(e))
        })?),
    )
}
