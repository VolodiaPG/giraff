use serde::Serialize;
use serde_json::Value;
use uom::si::f64::{Information, Ratio};

#[derive(Debug, Serialize, Default)]
pub struct FunctionDefinition {
    /// Name of deployed function
    #[serde(rename = "service")]
    pub service: String,
    /// Network, usually func_functions for Swarm (deprecated)
    #[serde(rename = "network")]
    pub network: Option<String>,
    /// Docker image in accessible registry
    #[serde(rename = "image")]
    pub image: String,
    /// Process for watchdog to fork
    #[serde(rename = "envProcess")]
    pub env_process: String,
    /// Overrides to environmental variables
    #[serde(rename = "envVars")]
    pub env_vars: Option<::std::collections::HashMap<String, String>>,
    #[serde(rename = "constraints")]
    pub constraints: Option<Vec<String>>,
    /// A map of labels for making scheduling or routing decisions
    #[serde(rename = "labels")]
    pub labels: Option<::std::collections::HashMap<String, String>>,
    /// A map of annotations for management, orchestration, events and build tasks
    #[serde(rename = "annotations")]
    pub annotations: Option<::std::collections::HashMap<String, String>>,
    #[serde(rename = "secrets")]
    pub secrets: Option<Vec<String>>,
    /// Private registry base64-encoded basic auth (as present in ~/.docker/config.json)
    #[serde(rename = "registryAuth")]
    pub registry_auth: Option<String>,
    #[serde(rename = "limits")]
    pub limits: Option<Limits>,
    #[serde(rename = "requests")]
    pub requests: Option<Value>,
    /// Make the root filesystem of the function read-only
    #[serde(rename = "readOnlyRootFilesystem")]
    pub read_only_root_filesystem: Option<bool>,
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Default)]
pub struct Limits {
    #[serde_as(as = "super::RatioHelper")]
    pub cpu: Ratio,
    #[serde_as(as = "super::InformationHelper")]
    pub memory: Information,
}
