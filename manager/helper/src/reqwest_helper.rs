use anyhow::{Context, Result};
use reqwest::Response;
use serde::de::DeserializeOwned;

pub async fn deserialize_response<T>(response: Response) -> Result<T>
where
    T: DeserializeOwned,
{
    let full = response.bytes().await.with_context(|| {
        "Failed to get bytes from the body of the message".to_string()
    })?;

    serde_json::from_slice(&full).with_context(|| {
        let text = String::from_utf8_lossy(&full);
        ("<Failed to get text>".to_string());
        format!(
            "Failed to deserialize (supposedly JSON) to the specifyed type, \
             the text message is {:?}",
            text
        )
    })
}
