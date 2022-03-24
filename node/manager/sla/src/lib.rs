#[macro_use]
extern crate uom;

use serde::{Deserialize, Serialize};
use uom::si::f64::{Information, Ratio, Time};
use validator::Validate;

pub mod cpu_ratio;
pub mod uom_information;
pub mod uom_ratio;
pub mod uom_time;
mod utils;

#[serde_with::serde_as]
#[derive(Validate, Serialize, Deserialize, Debug, Clone)]
pub struct Sla {
    #[serde(rename = "storage")]
    #[serde_as(as = "crate::uom_information::Helper")]
    pub storage: Information,

    #[serde(rename = "memory")]
    #[serde_as(as = "crate::uom_information::Helper")]
    pub memory: Information,

    #[serde(rename = "cpu")]
    #[serde_as(as = "crate::uom_ratio::Helper")]
    pub cpu: Ratio,

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

    #[serde(rename = "functionImage")]
    pub function_image: String,

    #[serde(rename = "functionLiveName")]
    pub function_live_name: Option<String>,
}
