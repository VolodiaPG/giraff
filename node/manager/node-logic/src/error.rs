#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Inherited an error when contacting the k8s API: {0}")]
    Kube(kube::Error),
    #[error("Unable to obtain the current key: {0}")]
    MissingKey(&'static str),
    #[error("Unable to parse the quantity: {0}")]
    QuantityParsing(String),
}