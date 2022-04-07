use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionListEntry {
    /// The name of the function
    #[serde(rename = "name")]
    name: String,
    /// The fully qualified docker image name of the function
    #[serde(rename = "image")]
    image: String,
    /// The amount of invocations for the specified function
    #[serde(rename = "invocationCount")]
    invocation_count: Option<f32>,
    /// The current minimal ammount of replicas
    #[serde(rename = "replicas")]
    replicas: f32,
    /// The current available amount of replicas
    #[serde(rename = "availableReplicas")]
    available_replicas: f32,
    /// Process for watchdog to fork
    #[serde(rename = "envProcess")]
    env_process: String,
    /// A map of labels for making scheduling or routing decisions
    #[serde(rename = "labels")]
    labels: ::std::collections::HashMap<String, String>,
    /// A map of annotations for management, orchestration, events and build tasks
    #[serde(rename = "annotations")]
    annotations: Option<::std::collections::HashMap<String, String>>,
}
