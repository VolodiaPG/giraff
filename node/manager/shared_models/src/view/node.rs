use crate::NodeId;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Update information in the node node
/// - Must indicate the time the request was createdAt in order to update the rolling average
/// - Subsequent requests will also include the last_answered_at time, returned in [PostNodeResponse]
/// - Same for last_answered_at
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PostNode {
    #[schemars(schema_with = "crate::chrono_helper::schema_function")]
    #[serde_as(as = "crate::chrono_helper::DateTimeHelper")]
    pub created_at: DateTime<Utc>,

    #[schemars(schema_with = "crate::chrono_helper::schema_function")]
    #[serde_as(as = "Option<crate::chrono_helper::DateTimeHelper>")]
    #[serde(default)]
    pub last_answered_at: Option<DateTime<Utc>>,

    #[schemars(schema_with = "crate::chrono_helper::schema_function")]
    #[serde_as(as = "Option<crate::chrono_helper::DateTimeHelper>")]
    #[serde(default)]
    pub last_answer_received_at: Option<DateTime<Utc>>,

    pub from: NodeId,
}

/// The answer to [PostNode]
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PostNodeResponse {
    #[schemars(schema_with = "crate::chrono_helper::schema_function")]
    #[serde_as(as = "crate::chrono_helper::DateTimeHelper")]
    pub answered_at: DateTime<Utc>,
}
