use super::node_situation::NodeSituation;
use anyhow::{Context, Result};
use bytes::Bytes;
use model::dto::function::{FunctionRecord, Proposed, Provisioned};
use model::BidId;
use openfaas::models::delete_function_request::DeleteFunctionRequest;
use openfaas::models::{FunctionDefinition, Limits, Requests};
use openfaas::DefaultApiClient;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use uom::si::time::second;

const ENV_VAR_SLA: &str = "SLA";
const ENV_VAR_PUSH_PERIOD: &str = "PUSH_PERIOD";
const ENV_VAR_PROMEHTEUS_ADDRESS: &str = "PROMETHEUS_ADDRESS";
const ENV_VAR_INSTANCE_ADDRESS: &str = "INSTANCE_ADDRESS";

#[derive(Debug)]
pub struct FaaSBackend {
    client:         Arc<DefaultApiClient>,
    node_situation: Arc<NodeSituation>,
}

impl FaaSBackend {
    pub fn new(
        client: Arc<DefaultApiClient>,
        node_situation: Arc<NodeSituation>,
    ) -> Self {
        Self { client, node_situation }
    }

    pub async fn provision_function(
        &self,
        id: BidId,
        bid: FunctionRecord<Proposed>,
    ) -> Result<FunctionRecord<Provisioned>> {
        let function_name = format!("fogfn-{}", id); // Respect DNS-1035 formatting (letter as first char of name)

        let mut env_vars = HashMap::new();
        env_vars.insert(
            ENV_VAR_SLA.to_string(),
            serde_json::to_string(&bid.0.sla).with_context(|| {
                format!("Failed to serialize the sla of function {}", id)
            })?,
        );
        env_vars.insert(
            ENV_VAR_INSTANCE_ADDRESS.to_string(),
            format!(
                "{}:{}",
                self.node_situation.get_my_public_ip(),
                self.node_situation.get_my_public_port_http()
            ),
        );
        env_vars.insert(
            ENV_VAR_PROMEHTEUS_ADDRESS.to_string(),
            self.node_situation.get_prometheus_address().clone().into_inner(),
        );
        env_vars.insert(
            ENV_VAR_PUSH_PERIOD.to_string(),
            self.node_situation
                .get_prometheus_push_period()
                .get::<second>()
                .to_string(),
        );

        let definition = FunctionDefinition {
            image: bid.0.sla.function_image.to_owned(),
            service: function_name.to_owned(),
            limits: Some(Limits {
                memory: bid.0.sla.memory,
                cpu:    bid.0.sla.cpu,
            }),
            requests: Some(Requests {
                memory: bid.0.sla.memory,
                cpu:    bid.0.sla.cpu,
            }),
            env_vars: Some(env_vars),
            labels: Some(HashMap::from([(
                "com.openfaas.scale.max".to_string(),
                bid.0.sla.max_replica.to_string(),
            )])),
            ..Default::default()
        };

        self.client.system_functions_post(definition).await?;

        let bid = bid.to_provisioned(function_name);
        Ok(bid)
    }

    pub async fn get_metrics(
        &self,
        record: &FunctionRecord<Provisioned>,
    ) -> Result<Option<Bytes>> {
        self.client
            .functions_get_metrics(&record.0.function_name)
            .await
            .with_context(|| {
                format!(
                    "Failed to get metrics for provisioned function named {}",
                    record.0.function_name
                )
            })
    }

    pub async fn remove_function(
        &self,
        function: &FunctionRecord<Provisioned>,
    ) -> Result<()> {
        self.client
            .system_functions_delete(DeleteFunctionRequest {
                function_name: function.0.function_name.clone(),
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to delete function named '{}' from the cluster",
                    function.0.function_name
                )
            })?;
        Ok(())
    }
}
