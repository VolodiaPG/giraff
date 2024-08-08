use super::{FaaSBackend, RemovableFunctionRecord};
use crate::{
    INFLUX_ADDRESS, INFLUX_BUCKET, INFLUX_ORG, INFLUX_TOKEN,
    OTEL_EXPORTER_OTLP_ENDPOINT_FUNCTION,
};
use anyhow::{Context, Result};
use helper::env_load;
use helper::monitoring::{
    InfluxAddress, InfluxBucket, InfluxOrg, InfluxToken,
};
use model::dto::function::{Paid, Provisioned};
use model::SlaId;
use openfaas::models::delete_function_request::DeleteFunctionRequest;
use openfaas::models::{FunctionDefinition, Limits, Requests};
use openfaas::DefaultApiClient;
use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::{info, instrument};

const ENV_VAR_SLA: &str = "SLA";
#[derive(Debug)]
pub struct FaaSBackendImpl {
    client: Arc<DefaultApiClient>,
}

impl FaaSBackendImpl {
    pub fn new(client: Arc<DefaultApiClient>) -> Self {
        info!("Using online faas backend");
        Self { client }
    }
}
#[async_trait::async_trait]
impl FaaSBackend for FaaSBackendImpl {
    #[instrument(level = "trace", skip(self))]
    async fn provision_function(
        &self,
        id: SlaId,
        bid: Paid,
    ) -> Result<Provisioned> {
        let function_name = format!("fogfn-{}", id); // Respect DNS-1035 formatting (letter as first char of name)

        let mut env_vars = HashMap::new();
        env_vars.insert(
            ENV_VAR_SLA.to_string(),
            serde_json::to_string(&bid.sla).with_context(|| {
                format!("Failed to serialize the sla of function {}", id)
            })?,
        );

        env_vars.extend(bid.sla.env_vars.iter().cloned());

        let bucket = env_load!(InfluxBucket, INFLUX_BUCKET);
        let address = env_load!(InfluxAddress, INFLUX_ADDRESS);
        let org = env_load!(InfluxOrg, INFLUX_ORG);
        let token = env_load!(InfluxToken, INFLUX_TOKEN);

        env_vars.insert(INFLUX_BUCKET.to_string(), bucket.into_inner());
        env_vars.insert(INFLUX_ADDRESS.to_string(), address.into_inner());
        env_vars.insert(INFLUX_ORG.to_string(), org.into_inner());
        env_vars.insert(INFLUX_TOKEN.to_string(), token.into_inner());
        env_vars.insert("ID".to_string(), id.to_string());
        env_vars.insert(
            "NAME".to_string(),
            bid.sla.function_live_name.to_string(),
        );

        let otel_endpoint_function =
            env::var(OTEL_EXPORTER_OTLP_ENDPOINT_FUNCTION.to_string());
        if let Ok(otel_endpoint_function) = otel_endpoint_function {
            env_vars.insert(
                OTEL_EXPORTER_OTLP_ENDPOINT_FUNCTION.to_string(),
                otel_endpoint_function,
            );
        }

        let definition = FunctionDefinition {
            image: bid.sla.function_image.to_owned(),
            service: function_name.to_owned(),
            limits: Some(Limits {
                memory: bid.sla.memory,
                cpu:    bid.sla.cpu,
            }),
            requests: Some(Requests {
                memory: bid.sla.memory,
                cpu:    bid.sla.cpu,
            }),
            env_vars: Some(env_vars),
            labels: Some(HashMap::from([(
                "com.openfaas.scale.max".to_string(),
                bid.sla.max_replica.to_string(),
            )])),
            ..Default::default()
        };

        self.client.system_functions_post(definition).await?;

        let bid = bid.to_provisioned(function_name);
        Ok(bid)
    }

    async fn check_is_live(&self, function: &Provisioned) -> Result<()> {
        self.client.check_is_live(function.function_name.clone()).await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    async fn remove_function(
        &self,
        function: RemovableFunctionRecord,
    ) -> Result<()> {
        self.client
            .system_functions_delete(DeleteFunctionRequest {
                function_name: function.function_name.clone(),
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to delete function named '{}' from the cluster",
                    function.function_name
                )
            })?;
        Ok(())
    }
}
