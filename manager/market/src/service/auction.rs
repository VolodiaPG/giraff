use super::faas::FogNodeFaaS;
use super::fog_node_network::FogNodeNetwork;
use crate::monitoring::FunctionDeploymentDuration;
use crate::repository::auction::Auction as AuctionRepository;
use crate::repository::bid_tracking::BidTracking;
use crate::repository::node_communication::NodeCommunication;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use helper::monitoring::MetricsExporter;
use model::domain::auction::AuctionResult;
use model::domain::sla::Sla;
use model::dto::function::ChosenBid;
use model::dto::node::NodeRecord;
use model::view::auction::{AcceptedBid, BidProposals, InstanciatedBid};
use model::{NodeId, SlaId};
use std::sync::Arc;
use tokio::time::Instant;
use tracing::trace;

pub struct Auction {
    auction_process:    Arc<AuctionRepository>,
    node_communication: Arc<NodeCommunication>,
    fog_node_network:   Arc<FogNodeNetwork>,
    faas:               Arc<FogNodeFaaS>,
    metrics:            Arc<MetricsExporter>,
    tracking:           Arc<BidTracking>,
}

impl Auction {
    pub fn new(
        auction_process: Arc<AuctionRepository>,
        node_communication: Arc<NodeCommunication>,
        fog_node_network: Arc<FogNodeNetwork>,
        faas: Arc<FogNodeFaaS>,
        metrics: Arc<MetricsExporter>,
        tracking: Arc<BidTracking>,
    ) -> Self {
        Self {
            auction_process,
            node_communication,
            fog_node_network,
            faas,
            metrics,
            tracking,
        }
    }

    async fn call_for_bids(
        &self,
        to: NodeId,
        sla: &'_ Sla,
    ) -> Result<BidProposals> {
        trace!("call for bids: {:?}", sla);

        self.node_communication
            .request_bids_from_node(to.clone(), sla)
            .await
            .with_context(|| format!("Failed to get bids from {}", to))
    }

    async fn do_auction(
        &self,
        proposals: &BidProposals,
    ) -> Result<AuctionResult> {
        trace!("do auction: {:?}", proposals);
        let auction_result =
            self.auction_process.auction(&proposals.bids).ok_or_else(
                || anyhow!("Auction failed, no winners were selected"),
            )?;
        Ok(AuctionResult { chosen_bid: auction_result })
    }

    pub async fn provision(&self, id: SlaId) -> Result<()> {
        let node = self.tracking.get(&id).context(format!(
            "Failed to retrieve the data (node_id) correlated to the sla id \
             {}",
            id
        ))?;

        self.faas
            .provision_paid_function(id, node)
            .await
            .context("Failed to provision function")?;

        Ok(())
    }

    async fn process_paying_details(
        &self,
        proposals: BidProposals,
        chosen_bid: ChosenBid,
        sla: Sla,
    ) -> Result<AcceptedBid> {
        let sla_id = sla.id.clone();
        let node_id = chosen_bid.bid.node_id.clone();
        let NodeRecord { ip, port_faas, .. } = self
            .fog_node_network
            .get_node(&chosen_bid.bid.node_id)
            .await
            .ok_or_else(|| {
                anyhow!(
                    "Node record of {} is not present in my database",
                    chosen_bid.bid.node_id
                )
            })?;
        let accepted = AcceptedBid {
            chosen: InstanciatedBid {
                bid: chosen_bid.bid,
                price: chosen_bid.price,
                ip,
                port: port_faas,
            },
            proposals,
            sla,
        };

        self.faas
            .pay_for_function(accepted.clone())
            .await
            .context("Failed to provision function")?;

        self.tracking.save(sla_id, node_id);

        Ok(accepted)
    }

    pub async fn start_auction(
        &self,
        target_node: NodeId,
        sla: Sla,
    ) -> Result<AcceptedBid> {
        let started = Instant::now();

        let proposals = self
            .call_for_bids(target_node.clone(), &sla)
            .await
            .with_context(|| {
                format!(
                    "Failed to call the network for bids with node {} as the \
                     starting point",
                    target_node,
                )
            })?;

        let AuctionResult { chosen_bid } =
            self.do_auction(&proposals).await.context("Auction failed")?;

        let res = self
            .process_paying_details(proposals, chosen_bid.clone(), sla.clone())
            .await;

        let accepted = res
            .context("Failed to provision function after several retries.")?;

        let finished = Instant::now();

        let duration = chrono::Duration::from_std(finished - started)
            .context("Failed to convert std duration to chrono duration")?;

        self.metrics
            .observe(FunctionDeploymentDuration {
                value:         duration.num_milliseconds(),
                function_name: sla.function_live_name,
                bid_id:        chosen_bid.bid.id.to_string(),
                sla_id:        sla.id.to_string(),
                timestamp:     Utc::now(),
            })
            .await
            .context("Failed to save metrics")?;

        Ok(accepted)
    }
}
