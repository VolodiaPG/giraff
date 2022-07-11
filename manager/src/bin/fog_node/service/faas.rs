use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;

use manager::{
    model::{
        dto::{auction::BidRecord, faas::ProvisionedRecord},
        BidId,
    },
    openfaas::{
        models::{FunctionDefinition, Limits},
        DefaultApi, DefaultApiClient,
    },
};

use crate::repository::provisioned::Provisioned as ProvisionedRepository;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    OpenFaaS(#[from] manager::openfaas::Error<String>),
}

#[async_trait]
pub trait FaaSBackend: Debug + Sync + Send {
    /// Provision the function from the bid description
    /// Return the function's name
    async fn provision_function(&self, id: BidId, bid: BidRecord) -> Result<String, Error>;
    async fn get_provisioned_function(&self, id: &BidId) -> Option<ProvisionedRecord>;
}

#[derive(Debug)]
pub struct OpenFaaSBackend {
    client:                Arc<DefaultApiClient>,
    provisioned_functions: Arc<dyn ProvisionedRepository>,
}

impl OpenFaaSBackend {
    pub fn new(
        client: Arc<DefaultApiClient>,
        provisioned_functions: Arc<dyn ProvisionedRepository>,
    ) -> Self {
        Self { client, provisioned_functions }
    }
}

#[async_trait]
impl FaaSBackend for OpenFaaSBackend {
    async fn provision_function(&self, id: BidId, bid: BidRecord) -> Result<String, Error> {
        let function_name = bid.sla.function_live_name.to_owned().unwrap_or_else(|| "".to_string())
            + "-"
            + id.to_string().as_str();

        let definition = FunctionDefinition {
            image: bid.sla.function_image.to_owned(),
            service: function_name.to_owned(),
            limits: Some(Limits { memory: bid.sla.memory, cpu: bid.sla.cpu }),
            ..Default::default()
        };

        self.client.system_functions_post(definition).await?;

        self.provisioned_functions
            .insert(id, ProvisionedRecord { bid, function_name: function_name.to_owned() })
            .await;

        Ok(function_name)
    }

    async fn get_provisioned_function(&self, id: &BidId) -> Option<ProvisionedRecord> {
        self.provisioned_functions.get(id).await
    }
}
