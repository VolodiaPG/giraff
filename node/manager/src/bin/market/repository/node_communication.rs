use std::sync::Arc;

use async_trait::async_trait;

use manager::model::domain::sla::Sla;
use manager::model::dto::node::NodeRecord;
use manager::model::view::auction::{Bid, BidProposal};
use manager::model::NodeId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
    #[error("The client failed to take the bid offer")]
    FailedToTakeOffer(NodeId),
}
#[async_trait]
pub trait NodeCommunication: Sync + Send {
    async fn request_bid_from_node(
        &self,
        node: NodeId,
        record: &NodeRecord,
        sla: &Sla,
    ) -> Result<BidProposal, Error>;
    async fn take_offer(&self, node_record: &NodeRecord, bid: &BidProposal) -> Result<(), Error>;
}

pub struct NodeCommunicationImpl {}

impl NodeCommunicationImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl NodeCommunication for NodeCommunicationImpl {
    async fn request_bid_from_node(
        &self,
        node: NodeId,
        record: &NodeRecord,
        sla: &Sla,
    ) -> Result<BidProposal, Error> {
        let client = reqwest::Client::new();
        let bid: Bid = client
            .post(format!("http://{}/api/bid", record.ip))
            .body(serde_json::to_string(sla)?)
            .send()
            .await?
            .json()
            .await?;

        Ok(BidProposal {
            node_id: node,
            id: bid.id,
            bid: bid.bid,
        })
    }

    async fn take_offer(&self, node_record: &NodeRecord, bid: &BidProposal) -> Result<(), Error> {
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
            Err(Error::FailedToTakeOffer(bid.node_id.to_owned()))
        }
    }
}
