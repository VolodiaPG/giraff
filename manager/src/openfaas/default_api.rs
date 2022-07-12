use async_trait::async_trait;
use log::trace;
use std::fmt::Debug;

use super::models::{FunctionDefinition, FunctionListEntry};
use super::{configuration, Error};

#[derive(Clone, Debug)]
pub struct DefaultApiClient {
    configuration: configuration::Configuration,
}

impl DefaultApiClient {
    pub fn new(configuration: configuration::Configuration) -> DefaultApiClient {
        DefaultApiClient { configuration }
    }
}

#[async_trait]
pub trait DefaultApi: Debug + Sync + Send {
    async fn system_functions_get(&self) -> Result<Vec<FunctionListEntry>, Error<String>>;
    async fn system_functions_post(&self, body: FunctionDefinition) -> Result<(), Error<String>>;
    async fn async_function_name_post(&self,
                                      function_name: &str,
                                      input: String)
                                      -> Result<(), Error<String>>;
}

#[async_trait]
impl DefaultApi for DefaultApiClient {
    async fn system_functions_get(&self) -> Result<Vec<FunctionListEntry>, Error<String>> {
        let uri_str = format!("{}/system/functions", self.configuration.base_path);
        trace!("Requesting {}", uri_str);

        let mut builder = self.configuration.client.get(&uri_str);

        if let Some((username, password)) = &self.configuration.basic_auth {
            builder = builder.basic_auth(username, password.as_ref());
        }

        Ok(builder.send().await?.json().await?)
    }

    async fn system_functions_post(&self, body: FunctionDefinition) -> Result<(), Error<String>> {
        let uri_str = format!("{}/system/functions", self.configuration.base_path);
        trace!("Requesting {}", uri_str);

        let mut builder =
            self.configuration.client.post(&uri_str).body(serde_json::to_string(&body)?);

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

    async fn async_function_name_post(&self,
                                      function_name: &str,
                                      input: String)
                                      -> Result<(), Error<String>> {
        let uri_str = format!("{}/async-function/{}", self.configuration.base_path, function_name);
        trace!("Requesting {}", uri_str);

        let mut builder = self.configuration.client.post(&uri_str).body(input);

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
