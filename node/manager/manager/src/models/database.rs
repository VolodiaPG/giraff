use serde::{Deserialize, Serialize};
use shared_models::sla::Sla;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BidRecord {
    pub bid: f64,
    pub sla: Sla,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProvisionedRecord {
    pub bid: BidRecord,
    pub function_name: String,
}
