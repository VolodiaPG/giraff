use serde::{Serialize, Deserialize};
use sla::Sla;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BidRecord {
    pub bid: f64,
    pub sla: Sla,
}