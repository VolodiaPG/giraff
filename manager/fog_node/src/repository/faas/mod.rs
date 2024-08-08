use anyhow::Result;
use model::dto::function::{Live, Paid, Provisioned};
use model::SlaId;
use nutype::nutype;
use std::fmt::Debug;

#[async_trait::async_trait]
pub trait FaaSBackend: Debug + Sync + Send {
    async fn provision_function(
        &self,
        id: SlaId,
        bid: Paid,
    ) -> Result<Provisioned>;

    async fn check_is_live(&self, function: &Provisioned) -> Result<()>;

    async fn remove_function(
        &self,
        function: RemovableFunctionRecord,
    ) -> Result<()>;
}

#[derive(Debug)]
pub struct RemovableFunctionRecord {
    #[allow(dead_code)]
    function_name: String,
}

impl From<Provisioned> for RemovableFunctionRecord {
    fn from(value: Provisioned) -> Self {
        Self { function_name: value.function_name }
    }
}

impl From<Live> for RemovableFunctionRecord {
    fn from(value: Live) -> Self {
        Self { function_name: value.function_name }
    }
}

#[nutype(derive(Clone, Debug), validate(greater_or_equal = 0))]
pub struct FunctionTimeout(u64);

#[cfg(not(feature = "offline"))]
pub mod faas;
#[cfg(not(feature = "offline"))]
pub use faas::*;

#[cfg(feature = "offline")]
pub mod faas_offline;
#[cfg(feature = "offline")]
pub use faas_offline::*;
