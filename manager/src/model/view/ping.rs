use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde_with::serde_as]
#[serde(rename_all = "camelCase")]
pub struct Ping {
    #[serde_as(as = "chrono_helper::DateTimeHelper")]
    #[schemars(schema_with = "crate::helper::chrono::schema_function")]
    pub sent_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde_with::serde_as]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
    #[serde_as(as = "chrono_helper::DateTimeHelper")]
    #[schemars(schema_with = "crate::helper::chrono::schema_function")]
    pub received_at: DateTime<Utc>,
}
