use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize, Clone, Validate)]
// pub struct RegisterNode {
//     pub ip: String,
// }

/// Patches a node
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatchNode {
    #[serde(rename = "createdAt")]
    #[serde_as(as = "Option<super::DateTimeHelper>")]
    pub created_at: Option<DateTime<Utc>>,
}
