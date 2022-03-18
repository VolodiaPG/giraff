use anyhow::Result;
use if_chain::if_chain;
use sla::Sla;
use std::{collections::BinaryHeap, sync::Arc, convert::Infallible};

use crate::{
    live_store::{BidDataBase, NodesDataBase},
    models::{Bid, BidProposal, BidRecord, ClientId, NodeId, NodeRecord},
};

pub async fn call_for_bids(
    sla: Sla,
    bid_db: Arc<tokio::sync::Mutex<BidDataBase>>,
    node_db: Arc<tokio::sync::Mutex<NodesDataBase>>,
) -> Result<ClientId, Infallible> {
    let nodes;
    {
        nodes = node_db.lock().await.get_bid_candidates(&sla);
    }

    let mut handles = Vec::new();
    for (node_id, node_record) in nodes.iter() {
        trace!("node @{:?}: {:#?}", node_id, node_record);

        let node_id = node_id.to_owned();
        let node_record = node_record.to_owned();

        let job = tokio::spawn(async move { request_bids(node_id, node_record) });

        handles.push(job);
    }

    let mut bid_record = BidRecord {
        sla: sla,
        bids: BinaryHeap::new(),
    };

    for handle in handles {
        if_chain! {
            if let Some(join) = handle.await.ok();
            if let Some(proposal) = join.await.ok();
            then
            {
                trace!("got bid proposal: {:?}", proposal);
                bid_record.bids.push(proposal);
            }
        }
    }

    let id: ClientId;

    {
        id = bid_db.lock().await.insert(bid_record);
    }

    Ok(id)
}
async fn request_bids(
    node_id: NodeId,
    node_record: NodeRecord,
) -> Result<BidProposal, reqwest::Error> {
    let bid = reqwest::get(format!("http://{}/api/bid", node_record.ip))
        .await?
        .json::<Bid>()
        .await?;

    Ok(BidProposal {
        node_id: node_id.clone(),
        id: bid.id,
        bid: bid.bid,
    })
}
