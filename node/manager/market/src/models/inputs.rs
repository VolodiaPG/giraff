use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct AcceptBid {
    #[serde(rename = "functionImage")]
    pub function_image: String,
    pub service: String,
}
