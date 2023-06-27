use crate::{INFLUX_ADDRESS, INFLUX_BUCKET, INFLUX_ORG, INFLUX_TOKEN};

use anyhow::{Context, Result};
use helper::env_load;
use helper::monitoring::{
    InfluxAddress, InfluxBucket, InfluxOrg, InfluxToken,
};
use model::dto::function::{FunctionRecord, Proposed, Provisioned};
use model::BidId;
use openfaas::models::delete_function_request::DeleteFunctionRequest;
use openfaas::models::{FunctionDefinition, Limits, Requests};
use openfaas::DefaultApiClient;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

const ENV_VAR_SLA: &str = "SLA";

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

        let mut env_vars = HashMap::new();
        env_vars.insert(
            ENV_VAR_SLA.to_string(),
            serde_json::to_string(&bid.0.sla).with_context(|| {
                format!("Failed to serialize the sla of function {}", id)
            })?,
        );

        let bucket = env_load!(InfluxBucket, INFLUX_BUCKET);
        let address = env_load!(InfluxAddress, INFLUX_ADDRESS);
        let org = env_load!(InfluxOrg, INFLUX_ORG);
        let token = env_load!(InfluxToken, INFLUX_TOKEN);

        env_vars.insert(INFLUX_BUCKET.to_string(), bucket.into_inner());
        env_vars.insert(INFLUX_ADDRESS.to_string(), address.into_inner());
        env_vars.insert(INFLUX_ORG.to_string(), org.into_inner());
        env_vars.insert(INFLUX_TOKEN.to_string(), token.into_inner());

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
