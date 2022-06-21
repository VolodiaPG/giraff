use crate::model::dto::auction::BidRecord;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProvisionedRecord {
    pub bid: BidRecord,
    pub function_name: String,
}
