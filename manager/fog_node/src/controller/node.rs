use crate::{service, NodeLife};
use model::view::node::RegisterNode;
use std::sync::Arc;

pub async fn register_child_node(
    register: RegisterNode,
    router: &Arc<NodeLife>,
) -> Result<(), service::node_life::Error> {
    router.register_child_node(register).await
}
