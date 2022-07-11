use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FunctionListEntry {
    /// The name of the function
    name:               String,
    /// The fully qualified docker image name of the function
    image:              String,
    /// The amount of invocations for the specified function
    invocation_count:   Option<f32>,
    /// The current minimal ammount of replicas
    replicas:           f32,
    /// The current available amount of replicas
    available_replicas: f32,
    /// Process for watchdog to fork
    env_process:        String,
    /// A map of labels for making scheduling or routing decisions
    labels:             ::std::collections::HashMap<String, String>,
    /// A map of annotations for management, orchestration, events and build tasks
    annotations:        Option<::std::collections::HashMap<String, String>>,
}
