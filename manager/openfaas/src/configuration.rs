#[derive(Clone, Debug)]
pub struct Configuration {
    pub base_path:  String,
    pub basic_auth: Option<BasicAuth>,
}

pub type BasicAuth = (String, Option<String>);
