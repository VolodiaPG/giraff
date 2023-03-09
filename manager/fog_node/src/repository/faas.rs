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

#[derive(Debug)]
pub struct FaaSBackend {
    client: Arc<DefaultApiClient>,
}

impl FaaSBackend {
    pub fn new(client: Arc<DefaultApiClient>) -> Self { Self { client } }

    pub async fn provision_function(
        &self,
        id: BidId,
        bid: FunctionRecord<Proposed>,
    ) -> Result<FunctionRecord<Provisioned>> {
        let function_name = format!("fogfn-{}", id); // Respect DNS-1035 formatting (letter as first char of name)

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
            env_vars: Some(HashMap::from([(
                "SLA".to_string(),
                serde_json::to_string(&bid.0.sla).with_context(|| {
                    format!("Failed to serialize the sla of function {}", id)
                })?,
            )])),
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
