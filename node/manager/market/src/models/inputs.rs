use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct RegisterNode {
    pub ip: String,
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct PatchNode {
    #[serde(rename = "createdAt")]
    #[serde_as(as = "Option<super::DateTimeHelper>")]
    pub created_at: Option<DateTime<Utc>>,
}
