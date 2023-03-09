use crate::repository::fog_node::FogNode;
use crate::repository::node_communication::NodeCommunication;
use anyhow::{anyhow, Result};
use model::dto::node::NodeRecord;
use model::view::auction::AcceptedBid;
use model::NodeId;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug)]
pub struct FogNodeFaaS {
    fog_node:           Arc<FogNode>,
    node_communication: Arc<NodeCommunication>,
}

impl FogNodeFaaS {
    pub fn new(
        fog_node: Arc<FogNode>,
        node_communication: Arc<NodeCommunication>,
    ) -> Self {
        Self { fog_node, node_communication }
    }

    pub async fn provision_function(&self, bid: AcceptedBid) -> Result<()> {
        trace!("Provisioning function...");

        let node = bid.chosen.bid.node_id.clone();
        self.node_communication
            .take_offer(node.clone(), &bid.chosen.bid)
            .await?;

        let mut record: NodeRecord =
            self.fog_node.get(&node).await.map(|node| node.data).ok_or_else(
                || {
                    anyhow!(
                        "Node {} not found among my database of registered \
                         fog nodes",
                        node
                    )
                },
            )?;

        let id = bid.chosen.bid.id.clone();
        record.accepted_bids.insert(id.clone(), bid);
        self.fog_node.update(&node, record).await;

        Ok(())
    }

    pub async fn get_functions(&self) -> HashMap<NodeId, Vec<AcceptedBid>> {
        self.fog_node.get_records().await
    }
}
