use async_trait::async_trait;
use log::trace;
use reqwest_middleware::ClientWithMiddleware;
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::instrument;

use super::{configuration, Error};
use crate::models::{FunctionDefinition, FunctionListEntry};

#[derive(Clone, Debug)]
pub struct DefaultApiClient {
    configuration: configuration::Configuration,
    client:        Arc<ClientWithMiddleware>,
}

impl DefaultApiClient {
    pub fn new(
        configuration: configuration::Configuration,
        client: Arc<ClientWithMiddleware>,
    ) -> DefaultApiClient {
        DefaultApiClient { configuration, client }
    }
}

#[async_trait]
pub trait DefaultApi: Debug + Sync + Send {
    async fn system_functions_get(
        &self,
    ) -> Result<Vec<FunctionListEntry>, Error<String>>;
    async fn system_functions_post(
        &self,
        body: FunctionDefinition,
    ) -> Result<(), Error<String>>;
    async fn async_function_name_post(
        &self,
        function_name: &str,
        input: &Value,
    ) -> Result<(), Error<String>>;
}

#[async_trait]
impl DefaultApi for DefaultApiClient {
    #[instrument(level = "trace", skip(self))]
    async fn system_functions_get(
        &self,
    ) -> Result<Vec<FunctionListEntry>, Error<String>> {
        let uri_str =
            format!("{}/system/functions", self.configuration.base_path);
        trace!("Requesting {}", uri_str);

        let mut builder = self.client.get(&uri_str);

        if let Some((username, password)) = &self.configuration.basic_auth {
            builder = builder.basic_auth(username, password.as_ref());
        }

        Ok(builder.send().await?.json().await?)
    }

    #[instrument(level = "trace", skip(self))]
    async fn system_functions_post(
        &self,
        body: FunctionDefinition,
    ) -> Result<(), Error<String>> {
        let uri_str =
            format!("{}/system/functions", self.configuration.base_path);
        trace!("Requesting {}", uri_str);

        let mut builder =
            self.client.post(&uri_str).body(serde_json::to_string(&body)?);

        if let Some((username, password)) = &self.configuration.basic_auth {
            builder = builder.basic_auth(username, password.as_ref());
        }

        let response = builder.send().await?;
        trace!("response: {:#?}", response);

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::from((response.status(), response.text().await)))
        }
    }

    #[instrument(level = "trace", skip(self))]
    async fn async_function_name_post(
        &self,
        function_name: &str,
        input: &Value,
    ) -> Result<(), Error<String>> {
        // TODO back to async
        let uri_str = format!(
            "{}/function/{}",
            self.configuration.base_path, function_name
        );
        trace!("Requesting {}", uri_str);

        let mut builder = self.client.post(&uri_str).json(input);

        if let Some((username, password)) = &self.configuration.basic_auth {
            builder = builder.basic_auth(username, password.as_ref());
        }
        trace!("Sending on {}", uri_str);
        let response = builder.send().await?;
        trace!("response: {:#?}", response);

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::from((response.status(), response.text().await)))
        }
    }
}
