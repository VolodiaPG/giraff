use super::{FaaSBackend, RemovableFunctionRecord};
use anyhow::Result;
use model::dto::function::{Paid, Provisioned};
use model::SlaId;
use std::fmt::Debug;
use std::time::Duration;
use tracing::warn;

#[derive(Debug)]
pub struct FaaSBackendOfflineImpl {
    online_delay: Duration,
}

impl FaaSBackendOfflineImpl {
    pub fn new(online_delay: Duration) -> Self {
        warn!("Using offline faas backend");
        Self { online_delay }
    }
}
#[async_trait::async_trait]
impl FaaSBackend for FaaSBackendOfflineImpl {
    async fn provision_function(
        &self,
        id: SlaId,
        bid: Paid,
    ) -> Result<Provisioned> {
        let function_name = format!("fogfn-{}", id); // Respect DNS-1035 formatting (letter as first char of name)
        let bid = bid.to_provisioned(function_name);
        Ok(bid)
    }

    async fn check_is_live(&self, _function: &Provisioned) -> Result<()> {
        tokio::time::sleep(self.online_delay).await;
        Ok(())
    }

    async fn remove_function(
        &self,
        _function: RemovableFunctionRecord,
    ) -> Result<()> {
        Ok(())
    }
}
