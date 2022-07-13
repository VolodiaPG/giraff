use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    OpeningFile(#[from] std::io::Error),
    #[error(transparent)]
    Deserialization(#[from] serde_json::Error),
    #[error(transparent)]
    Ron(#[from] ron::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait FromDisk {
    async fn from_disk(path: &Path) -> Result<Self, Error>
    where
        Self: Sized;
}
