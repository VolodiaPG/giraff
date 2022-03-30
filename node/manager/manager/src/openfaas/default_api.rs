use async_trait::async_trait;

use super::{configuration, Error, models::FunctionDefinition};
use super::models::FunctionListEntry;

type Result<T> = std::result::Result<T, Error<String>>;

#[derive(Clone)]
pub struct DefaultApiClient {
    configuration: configuration::Configuration,
}

impl DefaultApiClient {
    pub fn new(configuration: configuration::Configuration) -> DefaultApiClient {
        DefaultApiClient {
            configuration,
        }
    }
}

#[async_trait]
pub trait DefaultApi {
    async fn system_functions_get(&self) -> Result<Vec<FunctionListEntry>>;
    async fn system_functions_post(&self, body: FunctionDefinition) -> Result<()>;
    async fn async_function_name_post(&self, function_name: &str, input: warp::hyper::body::Bytes) -> Result<()>;
}

#[async_trait]
impl DefaultApi for DefaultApiClient {
    async fn system_functions_get(&self) -> Result<Vec<FunctionListEntry>> {
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
        trace!("response: {:#?}", response);

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::from((response.status(), response.text().await)))
        }
    }

    async fn async_function_name_post(&self, function_name: &str, input: warp::hyper::body::Bytes) -> Result<()> {
        // TODO redo in async
        let uri_str = format!("{}/async-function/{}", self.configuration.base_path, function_name);

        let mut builder = self
            .configuration
            .client
            .post(&uri_str)
            .body(input);

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
