use async_trait::async_trait;

use crate::{models, openfaas::models::function_definition::FunctionDefinition};

use super::{configuration, Error};

type Result<T> = std::result::Result<T, Error<serde_json::Value>>;

#[derive(Clone)]
pub struct DefaultApiClient {
    configuration: configuration::Configuration,
}

impl DefaultApiClient {
    pub fn new(configuration: configuration::Configuration) -> DefaultApiClient {
        DefaultApiClient {
            configuration: configuration,
        }
    }
}

#[async_trait]
pub trait DefaultApi {
    async fn system_functions_get(&self) -> Result<Vec<models::FunctionListEntry>>;
    async fn system_functions_post(&self, body: FunctionDefinition) -> Result<()>;
}

#[async_trait]
impl DefaultApi for DefaultApiClient {
    async fn system_functions_get(&self) -> Result<Vec<models::FunctionListEntry>> {
        let uri_str = format!("{}/system/functions", self.configuration.base_path);

        let mut builder = self.configuration.client.get(&uri_str);

        if let Some((username, password)) = &self.configuration.basic_auth {
            builder = builder.basic_auth(username, password.as_ref());
        }

        Ok(builder.send().await?.json().await?)
    }

    async fn system_functions_post(&self, body: FunctionDefinition) -> Result<()> {
        let uri_str = format!("{}/system/functions", self.configuration.base_path);

        let mut builder = self
            .configuration
            .client
            .post(&uri_str)
            .body(serde_json::to_string(&body)?);

        if let Some((username, password)) = &self.configuration.basic_auth {
            builder = builder.basic_auth(username, password.as_ref());
        }

        let response = builder.send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::from((response.status(), response.json().await)))
        }
    }
}
