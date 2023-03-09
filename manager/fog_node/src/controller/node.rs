use crate::NodeLife;
use anyhow::{Context, Result};
use model::view::node::RegisterNode;
use std::sync::Arc;

pub async fn register_child_node(
    register: RegisterNode,
    router: &Arc<NodeLife>,
) -> Result<()> {
    router
        .register_child_node(register)
        .await
        .context("Failed to register a child node to this one")
}
