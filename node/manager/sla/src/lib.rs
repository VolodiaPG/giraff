#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct SLA {
    #[serde(rename = "StorageBytes")]
    pub storage_bytes: Option<u64>,
    
    #[serde(rename = "RAMBytes")]
    pub ram_bytes: Option<u64>,
    
    #[validate(min=0, max=1)]
    #[serde(rename = "CPUFractionPerSeconds")]
    pub cpu_fraction_per_seconds: Option<f32>,

    #[serde(rename = "LatencyMaxMilliseconds")]
    pub latency_max_ms: Option<u64>,

    #[serde(rename = "DataInputMaxBytes")]
    pub data_input_max_bytes: Option<u64>,

    #[serde(rename = "DataOutputMaxBytes")]
    pub data_output_max_bytes: Option<u64>,

    #[serde(rename = "MaxMillisecondsBeforeHot")]
    pub max_milliseconds_before_hot: Option<u64>,

    #[serde(rename = "ReevaluationPeriodSeconds")]
    pub reevaluation_period_seconds: Option<u64>,
}
