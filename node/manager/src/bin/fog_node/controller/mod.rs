#[derive(thiserror::Error, Debug)]
pub enum ControllerError {
    #[error(transparent)]
    OpenFaaS(#[from] crate::service::faas::Error),
    #[error(transparent)]
    Auction(#[from] crate::service::auction::Error),
    #[error(transparent)]
    Function(#[from] crate::service::function_life::Error),
}

pub(crate) mod auction;
pub(crate) mod node;
pub(crate) mod ping;
pub(crate) mod routing;
