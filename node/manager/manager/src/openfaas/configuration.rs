#[derive(Clone)]
pub struct Configuration {
  pub base_path: String,
  pub client: reqwest::Client,
  pub basic_auth: Option<BasicAuth>,
}

pub type BasicAuth = (String, Option<String>);