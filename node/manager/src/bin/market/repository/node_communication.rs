use async_trait::async_trait;

use manager::model::domain::routing::Packet;
use manager::model::domain::sla::Sla;
use manager::model::dto::node::NodeRecord;
use manager::model::view::auction::BidProposal;
use manager::model::NodeId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
    #[error("The status code errored out {0}.")]
    ErrorStatus(reqwest::StatusCode),
}
#[async_trait]
pub trait NodeCommunication: Sync + Send {
    async fn request_bid_from_node(
        &self,
        to_uri: String,
        route_to_stack: Vec<NodeId>,
        sla: &Sla,
    ) -> Result<(), Error>;
    async fn take_offer(&self, node_record: &NodeRecord, bid: &BidProposal) -> Result<(), Error>;
}

pub struct NodeCommunicationThroughRoutingImpl {}

impl NodeCommunicationThroughRoutingImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl NodeCommunication for NodeCommunicationThroughRoutingImpl {
    async fn request_bid_from_node(
        &self,
        to_uri: String,
        route_to_stack: Vec<NodeId>,
        sla: &Sla,
    ) -> Result<(), Error> {
        let data = Packet::FogNode {
            route_to_stack,
            resource_uri: "bid".to_string(),
            data: &*serde_json::value::to_raw_value(sla)?,
        };

        let client = reqwest::Client::new();
        let url = format!("http://{}/api/routing", to_uri);
        trace!("Posting to {}", &url);
        let status = client.post(url).json(&data).send().await?.status();

        if status.is_success() {
            Ok(())
        } else {
            Err(Error::ErrorStatus(status))
        }
    }

    async fn take_offer(&self, node_record: &NodeRecord, bid: &BidProposal) -> Result<(), Error> {
        let client = reqwest::Client::new();
        let url = format!("http://{}/api/bid/{}", node_record.ip, bid.id);
        trace!("Posting to {}", &url);
        let status = client.post(url).send().await?.status();

        if status.is_success() {
            Ok(())
        } else {
            Err(Error::ErrorStatus(status))
        }
    }
}
