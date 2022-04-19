use anyhow::{anyhow, Result};
use futures::future::{BoxFuture, FutureExt};
use if_chain::if_chain;
use shared_models::{
    auction::{Bid, BidProposal},
    node::{RouteAction, RouteStack},
    sla::{PutSla, Sla},
    BidId, NodeId,
};
use std::{collections::VecDeque, convert::Infallible, default, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    live_store::{BidDataBase, NodesDataBase},
    models::{AuctionStatus, BidRecord, NodeRecord},
};

pub async fn call_for_bids(
    sla: Sla,
    bid_db: Arc<tokio::sync::Mutex<BidDataBase>>,
    node_db: Arc<tokio::sync::Mutex<NodesDataBase>>,
    leaf_node: NodeId,
) -> Result<BidId, Infallible> {
    trace!("call for bids: {:?}", sla);
    let nodes;
    {
        nodes = node_db.lock().await.get_bid_candidates(&sla, leaf_node);
    }

    let mut handles = Vec::new();
    let sla_safe = Arc::new(sla.clone());
    for (node_id, node_record) in nodes.iter() {
        trace!("node @{:?}: {:#?}", node_id, node_record);

        let node_id = node_id.to_owned();
        let node_record = node_record.to_owned();
        let sla_safe = sla_safe.clone();

        let job = tokio::spawn(async move { request_bids(node_id, node_record, sla_safe).await });

        handles.push(job);
    }

    let mut bids = Vec::new();

    for handle in handles {
        if_chain! {
            if let Ok(join) = handle.await;
            if let Ok(proposal) = join;
            then
            {
                trace!("got bid proposal: {:?}", proposal);
                bids.push(proposal);
            }
        }
    }

    let bid_record = BidRecord {
        sla,
        auction: AuctionStatus::Active(bids),
    };

    let id: BidId;
    {
        id = bid_db.lock().await.insert(bid_record.clone());
    }

    Ok(id)
}

async fn request_bids(
    node_id: NodeId,
    node_record: NodeRecord,
    sla: Arc<Sla>,
) -> Result<BidProposal> {
    trace!("requesting bids from node @{:?}", node_id);
    let client = reqwest::Client::new();
    let bid: Bid = client
        .post(format!("http://{}/api/bid", node_record.ip))
        .body(serde_json::to_string(&sla)?)
        .send()
        .await?
        .json()
        .await?;

    Ok(BidProposal {
        node_id,
        id: bid.id,
        bid: bid.bid,
    })
}

pub async fn take_offer(node_record: &NodeRecord, bid: &BidProposal) -> Result<()> {
    let client = reqwest::Client::new();
    if client
        .post(format!("http://{}/api/bid/{}", node_record.ip, bid.id))
        .send()
        .await?
        .status()
        .is_success()
    {
        Ok(())
    } else {
        error!(
            "The client {}@{} failed to take offer",
            bid.node_id, node_record.ip
        );
        Err(anyhow::anyhow!("failed to take offer"))
    }
}

async fn do_routes(
    couples: Vec<(NodeId, NodeId)>,
    function: BidId,
    nodes_db: Arc<Mutex<NodesDataBase>>,
    market_attached_node_uri: String,
) {
    trace!("do routes: {:?}", couples);
    let market_attached_node_uri = Arc::new(market_attached_node_uri);
    let function = Arc::new(function);

    let mut handles = Vec::new();
    for (from, to) in couples {
        let market_attached_node_uri = market_attached_node_uri.clone();
        let function = function.clone();
        let nodes_db = nodes_db.clone();
        let job = tokio::spawn(async move {
            request_route(market_attached_node_uri, function, from, to, nodes_db).await
        });

        handles.push(job);
    }

    for handle in handles {
        handle.await;
    }
}

async fn request_route(
    uri_of_market_node: Arc<String>,
    function: Arc<BidId>,
    from: NodeId,
    to: NodeId,
    nodes_db: Arc<Mutex<NodesDataBase>>,
) {
    if let Ok(stack) = do_route(from, to, nodes_db).await {
        trace!("computed route: {:?}", stack);
        let client = reqwest::Client::new();
        client
            .put(format!(
                "http://{}/api/routing/{}",
                uri_of_market_node, function
            ))
            .body(serde_json::to_string(&stack).unwrap())
            .send()
            .await;
    }
}

/// Generates the [RouteStack] follinwg the path returned from the [NodesDataBase]
fn do_route(
    from: NodeId,
    to: NodeId,
    nodes_db: Arc<Mutex<NodesDataBase>>,
) -> BoxFuture<'static, Result<RouteStack>> {
    async move {
        let route_stack = nodes_db.lock().await.get_path(&from, &to).ok_or(anyhow!("No path found"))?;
        let mut route_stack = route_stack.iter();
        route_stack.next(); // ignore the destination node, as it is useless to register a route in it

        let mut ret = RouteStack {
            ..Default::default()
        };
        let mut next = route_stack.next();
        while let Some(node_id) = next {
            next = route_stack.next();
            if_chain! {
                if let Some(node) = nodes_db.lock().await.get(node_id);
                if let Some(parent) = node.parent.to_owned();
                if let Some(next_id) = next;
                then
                {
                    if &parent != next_id {
                        ret.routes.insert(0, RouteAction::Assign{node: node_id.to_owned(),next: next_id.to_owned()});
                    }else{
                        // we are on a "^" in the tree, meaning the parent is not in the path
                        ret = RouteStack {
                            routes: vec![RouteAction::Divide{node: node_id.to_owned(),next_from_side: Box::new(do_route(from, node_id.to_owned(), nodes_db.to_owned()).await?),next_to_side: Box::new(ret)}],
                            ..Default::default()
                        };
                        break;
                    }

                }
                else
                {
                    return Err(anyhow!("Cannot keep goign because the topology of the tree doesn't correspond to  the found path"));
                }
            }
        }

        while let Some(node_id) = next {
            if let Some(_node) = nodes_db.lock().await.get(node_id) {
                ret.routes.insert(
                    0,
                    RouteAction::Skip {
                        node: node_id.to_owned(),
                    },
                );
            }
        }

        Ok(ret)
    }.boxed()
}
