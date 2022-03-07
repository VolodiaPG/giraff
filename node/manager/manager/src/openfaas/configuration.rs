#[derive(Clone)]
pub struct Configuration {
  pub base_path: String,
  pub client: reqwest::Client,
  pub basic_auth: Option<BasicAuth>,
}

pub type BasicAuth = (String, Option<String>);

pub struct ApiKey {
  pub prefix: Option<String>,
  pub key: String,
}

impl Configuration {
  pub fn new(client: reqwest::Client) -> Configuration {
    Configuration {
      base_path: "http://localhost:8080".to_owned(),
      client: client,
      basic_auth: None,
    }
  }
}
