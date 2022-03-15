use warp::{http::Response, Rejection};

use node_logic::{bidding::bid, satisfiability::is_satisfiable};
use sla::Sla;

use crate::openfaas::{
    models::{Bid, Satisfiable},
    DefaultApi, DefaultApiClient,
};

pub async fn list_functions(client: DefaultApiClient) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("list_functions");
    let functions = client.get_functions().await.map_err(|e| {
        error!("{}", e);
        warp::reject::reject()
    })?;
    let body = serde_json::to_string(&functions).unwrap();
    Ok(Response::builder().body(body))
}

pub async fn post_sla(_client: DefaultApiClient, sla: Sla) -> Result<impl warp::Reply, Rejection> {
    trace!("post sla: {:?}", sla);

    match is_satisfiable(&sla).await {
        Ok(res) => Ok(Response::builder().body(
            serde_json::to_string(&Satisfiable {
                is_satisfiable: res,
                sla: Some(sla),
            })
            .unwrap(),
        )),
        Err(e) => {
            error!("{}", e);
            Err(warp::reject::custom(crate::Error::NodeLogicError(e)))
        }
    }
}

pub async fn post_bid(_client: DefaultApiClient, sla: Sla) -> Result<impl warp::Reply, Rejection> {
    trace!("post bid with sla: {:?}", sla);

    match bid(&sla).await {
        Ok(bid) => Ok(Response::builder().body(
            serde_json::to_string(&Bid {
                bid: bid,
                sla: Some(sla),
            })
            .unwrap(),
        )),
        Err(e) => {
            error!("{}", e);
            Err(warp::reject::custom(crate::Error::NodeLogicError(e)))
        }
    }
}
