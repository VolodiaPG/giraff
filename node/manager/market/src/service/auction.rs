use anyhow::Result;
use if_chain::if_chain;
use shared_models::{
    domain::sla::Sla,
    view::auction::{Bid, BidProposal},
    BidId, NodeId,
};
use std::{convert::Infallible, sync::Arc};

use crate::{
    service::live_store::{BidDataBase, NodesDataBase},
    model::{AuctionStatus, BidRecord, NodeRecord},
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
