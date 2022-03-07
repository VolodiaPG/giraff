use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct SLA {
    #[serde(rename = "StorageBytes")]
    pub storage_bytes: u64,
    
    #[serde(rename = "RAMBytes")]
    pub ram_bytes: u64,
    
    #[validate(range(min=0, max=1))]
    #[serde(rename = "CPUFractionPerSeconds")]
    pub cpu_fraction_per_seconds: f32,

    #[serde(rename = "LatencyMaxMilliseconds")]
    pub latency_max_ms: u64,

    #[serde(rename = "DataInputMaxBytes")]
    pub data_input_max_bytes: u64,

    #[serde(rename = "DataOutputMaxBytes")]
    pub data_output_max_bytes: u64,

    #[serde(rename = "MaxMillisecondsBeforeHot")]
    pub max_milliseconds_before_hot: u64,

    #[serde(rename = "ReevaluationPeriodSeconds")]
    pub reevaluation_period_seconds: Option<u64>,
}
