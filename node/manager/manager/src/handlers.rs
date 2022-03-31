use std::sync::Arc;

use http_api_problem::StatusCode;
use tokio::sync::RwLock;
use warp::{http::Response, Rejection};

use crate::live_store::{BidDataBase, ProvisionedDataBase};
use crate::models::{
    Bid, BidId, BidRecord, NodeId, ProvisionedRecord, Satisfiable,
};
use crate::openfaas::models::{FunctionDefinition, Limits};
use crate::openfaas::{DefaultApi, DefaultApiClient};
use crate::routing::{self, NodeSituation};
use node_logic::{bidding::bid, satisfiability::is_satisfiable};
use sla::Sla;

/// Lists the functions available on the OpenFaaS gateway.
pub async fn list_functions(client: DefaultApiClient) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("list_functions");
    let functions = client.system_functions_get().await.map_err(|e| {
        error!("{:#?}", e);
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
                sla,
            })
            .unwrap(),
        )),
        Err(e) => {
            error!("{:#?}", e);
            Err(warp::reject::custom(crate::Error::NodeLogic(e)))
        }
    }
}

/// Returns a bid for the SLA.
pub async fn post_bid(
    _client: DefaultApiClient,
    bid_db: Arc<RwLock<BidDataBase>>,
    sla: Sla,
) -> Result<impl warp::Reply, Rejection> {
    trace!("post bid with sla: {:?}", sla);

    let bid = bid(&sla).await.map_err(|e| {
        error!("{:#?}", e);
        warp::reject::custom(crate::Error::NodeLogic(e))
    })?;

    let bid = BidRecord { bid, sla };

    let id;

    {
        id = bid_db.write().await.insert(bid.clone());
    }

    Ok(Response::builder().body(
        serde_json::to_string(&Bid {
            bid: bid.bid,
            sla: bid.sla,
            id,
        })
        .map_err(|e| {
            error!("{:#?}", e);
            warp::reject::custom(crate::Error::Serialization(e))
        })?,
    ))
}

/// Returns a bid for the SLA.
/// Creates the function on OpenFaaS and use the SLA to enable the limits
pub async fn post_bid_accept(
    id: BidId,
    client: DefaultApiClient,
    bid_db: Arc<RwLock<BidDataBase>>,
    provisioned_db: Arc<RwLock<ProvisionedDataBase>>,
) -> Result<impl warp::Reply, Rejection> {
    trace!("post accept bid {:?}", id);

    let bid: BidRecord;
    {
        bid = bid_db
            .read()
            .await
            .get(&id)
            .ok_or_else(|| warp::reject::custom(crate::Error::BidIdUnvalid(id.to_string(), None)))?
            .clone();
    }

    let function_name = bid
        .sla
        .function_live_name
        .to_owned()
        .unwrap_or_else(|| "".to_string())
        + "-"
        + id.to_string().as_str();

    let definition = FunctionDefinition {
        image: bid.sla.function_image.to_owned(),
        service: function_name.to_owned(),
        limits: Some(Limits {
            memory: bid.sla.memory,
            cpu: bid.sla.cpu,
        }),
        ..Default::default()
    };

    trace!("post accept bid {:#?}", definition);

    client
        .system_functions_post(definition)
        .await
        .map_err(|e| {
            error!("{:#?}", e);
            warp::reject::custom(crate::Error::from(e))
        })?;

    {
        bid_db.write().await.remove(&id);
    }
    {
        provisioned_db
            .write()
            .await
            .insert(id, ProvisionedRecord { bid, function_name });
    }

    Ok(Response::builder().body(""))
}

pub async fn put_routing(
    function_id: BidId,
    node_id: NodeId,
    routing_table: Arc<RwLock<routing::RoutingTable>>,
) -> Result<impl warp::Reply, Rejection> {
    trace!("put routing {:?}", function_id);

    {
        routing_table
            .write()
            .await
            .update_route(function_id, node_id)
            .await;
    }

    Ok(Response::builder().body(""))
}

#[derive(Debug)]
enum Routing {
    Outside(String),  // url
    OpenFaaS(String), // function_name
    Market(BidId)
}

pub async fn post_forward_routing(
    function_id: BidId,
    node_situation: Arc<NodeSituation>,
    routing_table: Arc<RwLock<routing::RoutingTable>>,
    provisioned_db: Arc<RwLock<ProvisionedDataBase>>,
    openfaas_client: DefaultApiClient,
    raw_body: warp::hyper::body::Bytes,
) -> Result<impl warp::Reply, Rejection> {
    trace!("post forward routing {:?}", function_id);
    let routing_action;
    {
        routing_action = routing_table.read().await.route(function_id).await;
    }

    trace!("routing action {:?}", routing_action);

    let routing_choice = match routing_action {
        routing::Forward::Outside(function_id, node_id) => {
            match node_situation.get(&node_id) {
                Some(node) => {
                    Routing::Outside(format!("http://{}/api/routing/{}", node.uri, function_id))
                }
                None => Routing::Market(function_id),
            }
        }
        routing::Forward::Inside(function_id) => {
            match provisioned_db.read().await.get(&function_id) {
                Some(provisioned) => Routing::OpenFaaS(provisioned.function_name.to_owned()),
                None => Routing::Market(function_id),
            }
        }
        routing::Forward::ToMarket(provisioned) => {
            if node_situation.is_market{
                if let Some(provisioned) = provisioned_db.read().await.get(&function_id){
                    Routing::OpenFaaS(provisioned.function_name.to_owned())
                }
                else{
                    trace!("Could not find the function name so redirected to market");
                    Routing::Market(provisioned)
                }
            }else{
                Routing::Market(provisioned)
            }
        },
    };

    trace!("routing choice is: {:?}", routing_choice);

    let client = reqwest::Client::new();
    match routing_choice {
        Routing::Outside(url) => {
            match client.post(url).body(raw_body).send().await{
                Ok(_) => (),
                Err(e) => {
                    error!("{:#?}", e);
                }
            }
        }
        Routing::OpenFaaS(function_name) => {
            openfaas_client.async_function_name_post(function_name.as_str(), raw_body).await.map_err(|e| {
                trace!("{:#?}", e);
                warp::reject::custom(crate::Error::from(e))
            })?;
        },
        Routing::Market(function_id) => {
            if let Some(node_to_market) = &node_situation.to_market {
                match client
                .post(format!(
                    "http://{}/api/routing/{}",
                    node_to_market.uri, function_id
                ))
                .body(raw_body)
                .send()
                .await{
                    Ok(_) => (),
                    Err(e) => {
                        error!("{:#?}", e);
                    }
                }
            } else {
                trace!("no node to market");
            }
        }
    }
    Ok(Response::builder().status(StatusCode::ACCEPTED).body(""))
}
