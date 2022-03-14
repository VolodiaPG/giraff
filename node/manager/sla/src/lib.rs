use serde::{Deserialize, Serialize};
use uom::si::f64::{Information, Time};
use validator::Validate;

mod uom_time;
mod utils;
mod uom_information;

#[serde_with::serde_as]
#[derive(Validate, Serialize, Deserialize, Debug)]
pub struct Sla {
    #[serde(rename = "storage")]
    #[serde_as(as = "crate::uom_information::Helper")]
    pub storage: Information,
    
    #[serde(rename = "memory")]
    #[serde_as(as = "crate::uom_information::Helper")]

    pub memory: Information,

    #[serde(rename = "cpu")]
    pub cpu: u16,

    #[serde(rename = "latencyMax")]
    #[serde_as(as = "crate::uom_time::Helper")]
    pub latency_max: Time,

    #[serde(rename = "dataInputMaxSize")]
    #[serde_as(as = "crate::uom_information::Helper")]
    pub data_input_max_size: Information,

    #[serde(rename = "dataOutputMaxSize")]
    #[serde_as(as = "crate::uom_information::Helper")]
    pub data_output_max_size: Information,

    #[serde(rename = "maxTimeBeforeHot")]
    #[serde_as(as = "crate::uom_time::Helper")]
    pub max_time_before_hot: Time,

    #[serde(rename = "reevaluationPeriod")]
    #[serde_as(as = "Option<crate::uom_time::Helper>")]
    pub reevaluation_period: Option<Time>,
}
