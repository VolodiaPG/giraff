use anyhow::Result;
use if_chain::if_chain;
use sla::Sla;
use std::{convert::Infallible, sync::Arc};

use crate::{
    live_store::{BidDataBase, NodesDataBase},
    models::{Bid, BidProposal, BidRecord, ClientId, NodeId, NodeRecord},
};

pub async fn call_for_bids(
    sla: Sla,
    bid_db: Arc<tokio::sync::Mutex<BidDataBase>>,
    node_db: Arc<tokio::sync::Mutex<NodesDataBase>>,
    leaf_node: NodeId
) -> Result<ClientId, Infallible> {
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

    let mut bid_record = BidRecord {
        sla,
        bids: Vec::new(),
    };

    for handle in handles {
        if_chain! {
            if let Ok(join) = handle.await;
            if let Ok(proposal) = join;
            then
            {
                trace!("got bid proposal: {:?}", proposal);
                bid_record.bids.push(proposal);
            }
        }
    }

    let id: ClientId;
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
        error!("failed to take offer");
        Err(anyhow::anyhow!("failed to take offer"))
    }
}
