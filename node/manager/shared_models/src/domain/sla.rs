use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uom::si::f64::{Information, Ratio, Time};

/// Describe the SLA of a function submitted to be provisioned
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Sla {
    #[schemars(schema_with = "uom_helpers::information::schema_function")]
    #[serde_as(as = "uom_helpers::information::Helper")]
    pub storage: Information,

    #[schemars(schema_with = "uom_helpers::information::schema_function")]
    #[serde_as(as = "uom_helpers::information::Helper")]
    pub memory: Information,

    #[schemars(schema_with = "uom_helpers::ratio::schema_function")]
    #[serde_as(as = "uom_helpers::ratio::Helper")]
    pub cpu: Ratio,

    #[schemars(schema_with = "uom_helpers::time::schema_function")]
    #[serde_as(as = "uom_helpers::time::Helper")]
    pub latency_max: Time,

    #[schemars(schema_with = "uom_helpers::information::schema_function")]
    #[serde_as(as = "uom_helpers::information::Helper")]
    pub data_input_max_size: Information,

    #[schemars(schema_with = "uom_helpers::information::schema_function")]
    #[serde_as(as = "uom_helpers::information::Helper")]
    pub data_output_max_size: Information,

    #[schemars(schema_with = "uom_helpers::time::schema_function")]
    #[serde_as(as = "uom_helpers::time::Helper")]
    pub max_time_before_hot: Time,

    #[schemars(schema_with = "uom_helpers::time::schema_function")]
    #[serde_as(as = "uom_helpers::time::Helper")]
    pub reevaluation_period: Time,

    pub function_image: String,

    pub function_live_name: Option<String>,
}
