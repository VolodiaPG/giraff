use super::NodeId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize, Clone, Validate)]
// pub struct RegisterNode {
//     pub ip: String,
// }

/// changes information of a node
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostNode {
    #[serde(rename = "createdAt")]
    #[serde_as(as = "Option<super::DateTimeHelper>")]
    pub created_at: Option<DateTime<Utc>>,

    pub from: NodeId,
}

/// The answer to the post node request
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostNodeResponse {
    #[serde(rename = "answeredAt")]
    #[serde_as(as = "super::DateTimeHelper")]
    pub answered_at: DateTime<Utc>,
}
