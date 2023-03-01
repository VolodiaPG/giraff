use bytes::Bytes;
use model::dto::function::{FunctionRecord, Proposed, Provisioned};
use model::BidId;
use openfaas::models::{FunctionDefinition, Limits, Requests};
use openfaas::DefaultApiClient;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    OpenFaaS(#[from] openfaas::Error<String>),
    #[error("BidId {0} not found")]
    IdNotFound(BidId),
}

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
    ) -> Result<FunctionRecord<Provisioned>, Error> {
        let function_name = format!("{}-{}", bid.0.sla.function_live_name, id);

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
                serde_json::to_string(&bid.0.sla).unwrap(),
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
    ) -> Result<Option<Bytes>, Error> {
        Ok(self.client.functions_get_metrics(&record.0.function_name).await?)
    }
}
