use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteFunctionRequest {
    /// Name of deployed function
    #[serde(rename = "functionName")]
    pub function_name: String,
}
