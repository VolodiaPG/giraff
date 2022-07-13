use crate::NodeLife;
use manager::model::view::node::RegisterNode;
use std::sync::Arc;

pub async fn register_child_node(
    register: RegisterNode,
    router: &Arc<dyn NodeLife>,
) -> anyhow::Result<()> {
    router.register_child_node(register).await.map_err(|e| anyhow::anyhow!(e))
}
