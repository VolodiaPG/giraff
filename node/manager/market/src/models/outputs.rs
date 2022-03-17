use serde::{Deserialize, Serialize};
use sla::Sla;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Satisfiable {
    #[serde(rename = "isSatisfiable")]
    pub is_satisfiable: bool,
    pub sla: Sla,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bid {
    pub bid: f64,
    pub sla: Sla,
    pub id: Uuid,
}
