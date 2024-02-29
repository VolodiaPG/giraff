use super::{configuration, Error};
use crate::models::delete_function_request::DeleteFunctionRequest;
use crate::models::{FunctionDefinition, FunctionListEntry};
use log::trace;
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::instrument;

type HttpClient = reqwest_middleware::ClientWithMiddleware;

#[derive(Clone, Debug)]
pub struct DefaultApiClient {
    configuration: configuration::Configuration,
    client:        Arc<HttpClient>,
}

impl DefaultApiClient {
    pub fn new(
        configuration: configuration::Configuration,
        client: Arc<HttpClient>,
    ) -> DefaultApiClient {
        DefaultApiClient { configuration, client }
    }

    #[instrument(level = "trace", skip(self))]
    pub async fn system_functions_get(
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
    pub async fn system_functions_post(
        &self,
        body: FunctionDefinition,
    ) -> Result<(), Error<String>> {
        let uri_str =
            format!("{}/system/functions", self.configuration.base_path);
        trace!("Requesting {}", uri_str);

        // Adding a fast timeout since it is a health route and should not take
        // long to respond
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
    pub async fn check_is_live(
        &self,
        function_name: String,
    ) -> Result<(), Error<String>> {
        let uri_str = format!(
            "{}/function/{}/health",
            self.configuration.base_path, function_name
        );
        trace!("Checking liveness of {}", uri_str);

        let mut builder = self
            .client
            .get(&uri_str)
            .timeout(std::time::Duration::from_secs(1));

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
    pub async fn async_function_name_post(
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

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::from((response.status(), response.text().await)))
        }
    }

    #[instrument(level = "trace", skip(self))]
    pub async fn functions_get_metrics(
        &self,
        function_name: &str,
    ) -> Result<Option<bytes::Bytes>, Error<String>> {
        let uri_str = format!(
            "{}/function/{}/metrics",
            self.configuration.base_path, function_name
        );
        trace!("Requesting {}", uri_str);

        let builder = self.client.get(&uri_str);

        trace!("Sending on {}", uri_str);
        let response = builder.send().await?;

        if response.status().is_success() {
            Ok(response.bytes().await.ok())
        } else {
            Err(Error::from((response.status(), response.text().await)))
        }
    }

    #[instrument(level = "trace", skip(self))]
    pub async fn system_functions_delete(
        &self,
        body: DeleteFunctionRequest,
    ) -> Result<(), Error<String>> {
        let uri_str =
            format!("{}/system/functions", self.configuration.base_path);
        trace!("Deleting {}", uri_str);

        let mut builder =
            self.client.delete(&uri_str).body(serde_json::to_string(&body)?);

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
}
