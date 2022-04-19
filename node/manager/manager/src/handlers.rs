use std::sync::Arc;

use http_api_problem::StatusCode;
use shared_models::node::{RouteAction, RouteStack, RoutingStackError};
use tokio::sync::RwLock;
use warp::{http::Response, Rejection};

use crate::live_store::{BidDataBase, ProvisionedDataBase};
use crate::models::{BidRecord, ProvisionedRecord};
use crate::routing::{self, NodeSituation};
use node_logic::bidding::bid;
use openfaas::models::{FunctionDefinition, Limits};
use openfaas::{DefaultApi, DefaultApiClient};
use shared_models::sla::Sla;
use shared_models::{auction::Bid, ids::Reserved, BidId};

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
    routing_table: Arc<RwLock<routing::RoutingTable>>,
    node_situation: Arc<NodeSituation>,
    mut stack: RouteStack,
) -> Result<impl warp::Reply, Rejection> {
    trace!("put routing {:?}", function_id);

    if stack.routes.is_empty() {
        return Err(warp::reject::custom(crate::Error::from(
            RoutingStackError::Empty(function_id),
        )));
    }

    let cursor = stack.routes.pop().unwrap();

    match cursor {
        RouteAction::Assign { node, next } => {
            if node == node_situation.my_id {
                routing_table
                    .write()
                    .await
                    .update_route(function_id.to_owned(), next)
                    .await;

                return update_next(function_id, stack, node_situation).await;
            }
            Err(warp::reject::custom(crate::Error::from(
                RoutingStackError::CurrentIdIsNotMine {
                    current: node,
                    stack,
                },
            )))
        }
        RouteAction::Skip { node } => {
            if node == node_situation.my_id {
                return update_next(function_id, stack, node_situation).await;
            }
            Err(warp::reject::custom(crate::Error::from(
                RoutingStackError::CurrentIdIsNotMine {
                    current: node,
                    stack,
                },
            )))
        }
        RouteAction::Divide {
            node,
            next_from_side,
            next_to_side,
        } => {
            if node == node_situation.my_id {
                let next_node = next_to_side.routes.get(0).ok_or_else(|| {
                    warp::reject::custom(crate::Error::from(RoutingStackError::Empty(
                        function_id.to_owned(),
                    )))
                })?;
                let next_node = match next_node {
                    RouteAction::Assign { node, .. } | RouteAction::Divide { node, .. } => node,
                    RouteAction::Skip { node } => {
                        return Err(warp::reject::custom(crate::Error::from(
                            RoutingStackError::SkipMispositioned(node.to_owned()),
                        )))
                    }
                };
                routing_table
                    .write()
                    .await
                    .update_route(function_id.to_owned(), next_node.to_owned())
                    .await;

                update_next(
                    function_id.to_owned(),
                    *next_to_side,
                    node_situation.to_owned(),
                )
                .await?;
                return update_next(function_id, *next_from_side, node_situation).await;
            }
            Err(warp::reject::custom(crate::Error::from(
                RoutingStackError::CurrentIdIsNotMine {
                    current: node,
                    stack,
                },
            )))
        }
    }
}

async fn update_next(
    function_id: BidId,
    stack: RouteStack,
    node_situation: Arc<NodeSituation>,
) -> Result<impl warp::Reply, Rejection> {
    let next_node = stack.routes.get(0).ok_or_else(|| {
        warp::reject::custom(crate::Error::from(RoutingStackError::Empty(
            function_id.to_owned(),
        )))
    })?;

    let next_node = match next_node {
        RouteAction::Assign { node, .. }
        | RouteAction::Skip { node }
        | RouteAction::Divide { node, .. } => node,
    };

    let url = &node_situation
        .get(next_node)
        .ok_or_else(|| {
            warp::reject::custom(crate::Error::from(
                RoutingStackError::NextNodeIsNotAChildOfMine(stack.clone()),
            ))
        })?
        .uri;

    let client = reqwest::Client::new();
    match client
        .put(&format!("http://{}/routing/{}", url, function_id))
        .body(serde_json::to_string(&stack).unwrap())
        .send()
        .await
    {
        Ok(response) => Ok(Response::builder()
            .status(response.status())
            .body(response.bytes().await.map_err(crate::Error::from)?)),
        Err(e) => {
            error!("{:#?}", e);
            Err(warp::reject::custom(crate::Error::from(e)))
        }
    }
}

#[derive(Debug)]
enum Routing {
    Outside(String),  // url
    OpenFaaS(String), // function_name
    Market(BidId),
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
        routing_action = routing_table.read().await.route(function_id.clone()).await;
    }

    trace!("routing action {:?}", routing_action);

    let routing_choice = match routing_action {
        routing::Forward::Outside(function_id, node_id) => match node_situation.get(&node_id) {
            Some(node) => {
                Routing::Outside(format!("http://{}/api/routing/{}", node.uri, function_id))
            }
            None => Routing::Market(function_id),
        },
        routing::Forward::Inside(function_id) => {
            match provisioned_db.read().await.get(&function_id) {
                Some(provisioned) => Routing::OpenFaaS(provisioned.function_name.to_owned()),
                None => Routing::Market(function_id),
            }
        }
        routing::Forward::ToMarket(provisioned) => {
            if node_situation.is_market {
                match provisioned_db.read().await.get(&function_id) {
                    Some(provisioned) => Routing::OpenFaaS(provisioned.function_name.to_owned()),
                    None => {
                        trace!("Could not find the function name so redirected to market");
                        Routing::Market(function_id)
                    }
                }
            } else {
                Routing::Market(provisioned)
            }
        }
    };

    trace!("routing choice is: {:?}", routing_choice);

    let client = reqwest::Client::new();
    match routing_choice {
        Routing::Outside(url) => match client.post(url).body(raw_body).send().await {
            Ok(response) => Ok(Response::builder()
                .status(response.status())
                .body(response.bytes().await.map_err(crate::Error::from)?)),
            Err(e) => {
                error!("{:#?}", e);
                Err(warp::reject::custom(crate::Error::from(e)))
            }
        },
        Routing::OpenFaaS(function_name) => {
            openfaas_client
                .async_function_name_post(function_name.as_str(), raw_body)
                .await
                .map_err(|e| {
                    trace!("{:#?}", e);
                    warp::reject::custom(crate::Error::from(e))
                })?;

            Ok(Response::builder()
                .status(StatusCode::ACCEPTED)
                .body("".to_string().into()))
        }
        Routing::Market(function_id) => {
            if let Some(node_to_market) = &node_situation.to_market {
                return match client
                    .post(format!(
                        "http://{}/api/routing/{}",
                        node_to_market.uri, function_id
                    ))
                    .body(raw_body)
                    .send()
                    .await
                {
                    Ok(response) => Ok(Response::builder()
                        .status(response.status())
                        .body(response.bytes().await.map_err(crate::Error::from)?)),
                    Err(e) => {
                        error!("{:#?}", e);
                        Err(warp::reject::custom(crate::Error::from(e)))
                    }
                };
            } else if let Some(reserved_id) = function_id.into() {
                match reserved_id {
                    Reserved::MarketPing => {
                        if let Some(market_url) = &node_situation.market_url {
                            return match client
                                .post(format!("http://{}/api/node", market_url))
                                .body(raw_body)
                                .send()
                                .await
                            {
                                Ok(response) => Ok(Response::builder()
                                    .status(response.status())
                                    .body(response.bytes().await.map_err(crate::Error::from)?)),
                                Err(e) => {
                                    error!("{:#?}", e);
                                    Err(warp::reject::custom(crate::Error::TransmittingRequest(
                                        None,
                                    )))
                                }
                            };
                        }
                    }
                }
            }

            // default case
            Err(warp::reject::custom(crate::Error::TransmittingRequest(
                None,
            )))
        }
    }
}
