use async_trait::async_trait;
use futures::future::try_join_all;
use model::domain::routing::FogSegment;
use model::domain::sla::{Sla, SlaFogPoint};
use model::dto::node::NodeRecord;
use model::view::auction::AcceptedBid;
use model::view::routing::Route;
use model::{BidId, NodeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::repository::fog_node::FogNode;
use crate::repository::node_communication::NodeCommunication;
use crate::service::fog_node_network::FogNodeNetwork;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    NodeCommunication(#[from] crate::repository::node_communication::Error),
    #[error(
        "No trace of the node {0} has been found. It should have been \
         registered as a record though."
    )]
    NodeNotFound(NodeId),
    #[error(transparent)]
    FogNetwork(#[from] crate::service::fog_node_network::Error),
    #[error(
        "The function name {0} cannot be found in the provisioned functions \
         database"
    )]
    FunctionNameNotFound(String),
}

#[async_trait]
pub trait FogNodeFaaS: Debug + Sync + Send {
    /// Provision the function on the given node and establish the routes using
    /// [`establish_route`](FogNodeFaaS::establish_route)
    async fn provision_function(&self, bid: AcceptedBid) -> Result<(), Error>;

    async fn get_functions(&self) -> HashMap<NodeId, Vec<AcceptedBid>>;

    /// Establish a route between two nodes on the Fog network and saves it
    async fn establish_route(
        &self,
        function: BidId,
        segment: FogSegment,
    ) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct FogNodeFaaSImpl {
    fog_node:           Arc<dyn FogNode>,
    fog_node_network:   Arc<dyn FogNodeNetwork>,
    node_communication: Arc<dyn NodeCommunication>,
}

impl FogNodeFaaSImpl {
    pub fn new(
        fog_node: Arc<dyn FogNode>,
        fog_node_network: Arc<dyn FogNodeNetwork>,
        node_communication: Arc<dyn NodeCommunication>,
    ) -> Self {
        Self { fog_node, fog_node_network, node_communication }
    }

    async fn fog_point_to_node_id(
        &self,
        winning_node: &NodeId,
        fog_point: &SlaFogPoint,
    ) -> Result<NodeId, Error> {
        Ok(match fog_point {
            SlaFogPoint::ThisFunction => winning_node.clone(),
            SlaFogPoint::DataSource(id) => id.clone(),
            SlaFogPoint::FunctionSink(function_name) => self
                .fog_node
                .get_node_from_function(function_name)
                .await
                .ok_or_else(|| {
                    Error::FunctionNameNotFound(function_name.to_string())
                })?,
        })
    }

    /// Convert [`Sla::dataflow`](Sla) to a FogSegment vector
    async fn to_fog_segment(
        &self,
        winning_node: &NodeId,
        sla: &Sla,
    ) -> Result<Vec<FogSegment>, Error> {
        let mut res = vec![];
        for flow in &sla.data_flow {
            let from =
                self.fog_point_to_node_id(winning_node, &flow.from).await?;
            let to = self.fog_point_to_node_id(winning_node, &flow.to).await?;

            res.push(FogSegment { from, to })
        }

        Ok(res)
    }
}

#[async_trait]
impl FogNodeFaaS for FogNodeFaaSImpl {
    async fn provision_function(&self, bid: AcceptedBid) -> Result<(), Error> {
        let node = bid.chosen.bid.node_id.clone();
        let res = self
            .node_communication
            .take_offer(node.clone(), &bid.chosen.bid)
            .await?;

        let mut record: NodeRecord = self
            .fog_node
            .get(&node)
            .await
            .map(|node| node.data)
            .ok_or_else(|| Error::NodeNotFound(node.clone()))?;

        let id = bid.chosen.bid.id.clone();
        let sla = bid.sla.clone();
        record.accepted_bids.insert(id.clone(), bid);
        self.fog_node.update(&node, record).await;

        trace!("Function has been placed");

        let promises = self
            .to_fog_segment(&node, &sla)
            .await?
            .into_iter()
            .map(|segment| self.establish_route(id.clone(), segment));
        try_join_all(promises).await?;

        trace!("Routes to function have been established");

        Ok(res)
    }

    async fn get_functions(&self) -> HashMap<NodeId, Vec<AcceptedBid>> {
        self.fog_node.get_records().await
    }

    async fn establish_route(
        &self,
        function: BidId,
        segment: FogSegment,
    ) -> Result<(), Error> {
        trace!("Finding a route in the network...");
        let solution = self.fog_node_network.get_route(segment).await?;
        let route = Route {
            stack_asc: solution.stack_asc,
            stack_rev: solution.stack_rev,
            function,
        };
        trace!("Establishing that route on every Fog node...");
        Ok(self
            .node_communication
            .establish_route(solution.least_common_ancestor, route)
            .await?)
    }
}
