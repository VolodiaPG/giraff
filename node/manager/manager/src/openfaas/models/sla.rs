use serde::{Deserialize, Serialize};
use sla::Sla;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Satisfiable {
    pub is_satisfiable: bool,
    pub sla: Option<Sla>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Bid {
    pub bid: f64,
    pub sla: Option<Sla>,
}