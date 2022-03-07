use async_trait::async_trait;

use super::{configuration, models};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
    async fn get_functions(&self) -> Result<Vec<models::FunctionListEntry>>;
}

#[async_trait]
impl DefaultApi for DefaultApiClient {
    async fn get_functions(&self) -> Result<Vec<models::FunctionListEntry>> {
        let uri_str = format!("{}/system/functions", self.configuration.base_path);

        let mut builder = self.configuration.client.get(&uri_str);

        if let Some((username, password)) = &self.configuration.basic_auth {
            builder = builder.basic_auth(username, password.as_ref());
        }

        Ok(builder.send().await?.json().await?)
    }
}
