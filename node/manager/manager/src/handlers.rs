use std::sync::Arc;

use tokio::sync::Mutex;
use uuid::Uuid;
use warp::{http::Response, Rejection};

use crate::live_store::BidDataBase;
use crate::models::{Bid, BidRecord, Satisfiable};
use crate::openfaas::{DefaultApi, DefaultApiClient};
use node_logic::{bidding::bid, satisfiability::is_satisfiable};
use sla::Sla;

/// Lists the functions available on the OpenFaaS gateway.
pub async fn list_functions(client: DefaultApiClient) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("list_functions");
    let functions = client.get_functions().await.map_err(|e| {
        error!("{}", e);
        warp::reject::reject()
    })?;
    let body = serde_json::to_string(&functions).unwrap();
    Ok(Response::builder().body(body))
}

/// Returns if the SLA can be satisfied.
pub async fn post_sla(_client: DefaultApiClient, sla: Sla) -> Result<impl warp::Reply, Rejection> {
    trace!("post sla: {:?}", sla);

    match is_satisfiable(&sla).await {
        Ok(res) => Ok(Response::builder().body(
            serde_json::to_string(&Satisfiable {
                is_satisfiable: res,
                sla: sla,
            })
            .unwrap(),
        )),
        Err(e) => {
            error!("{}", e);
            Err(warp::reject::custom(crate::Error::NodeLogicError(e)))
        }
    }
}

/// Returns a bid for the SLA.
pub async fn post_bid(
    _client: DefaultApiClient,
    bid_db: Arc<Mutex<BidDataBase>>,
    sla: Sla,
) -> Result<impl warp::Reply, Rejection> {
    trace!("post bid with sla: {:?}", sla);

    let bid = bid(&sla).await.map_err(|e| {
        error!("{}", e);
        warp::reject::custom(crate::Error::NodeLogicError(e))
    })?;

    let bid = BidRecord { bid: bid, sla: sla };

    let id;

    {
        id = bid_db.lock().await.insert(bid.clone());
    }

    Ok(Response::builder().body(
        serde_json::to_string(&Bid {
            bid: bid.bid,
            sla: bid.sla,
            id: id,
        })
        .map_err(|e| {
            error!("{}", e);
            warp::reject::custom(crate::Error::SerializationError(e))
        })?,
    ))
}

/// Returns a bid for the SLA.
pub async fn post_bid_accept(
    id: String,
    _client: DefaultApiClient,
    bid_db: Arc<Mutex<BidDataBase>>,
    sla: Sla,
) -> Result<impl warp::Reply, Rejection> {
    trace!("post accept bid {:?} with sla: {:?}", id, sla);

    let id = Uuid::parse_str(&id).map_err(|e| {
        error!("{}", e);
        warp::reject::custom(crate::Error::BidIdUnvalid(id, Some(e)))
    })?;

    let bid: BidRecord;
    {
        bid = bid_db.lock().await.get(&id).ok_or_else(||{
            warp::reject::custom(crate::Error::BidIdUnvalid(
                id.to_string(),
                None,
            ))
        })?.clone();
    }

    Ok(Response::builder().body(
        serde_json::to_string(&Bid {
            bid: bid.bid,
            sla: bid.sla,
            id: id,
        })
        .map_err(|e| {
            error!("{}", e);
            warp::reject::custom(crate::Error::SerializationError(e))
        })?,
    ))
}
