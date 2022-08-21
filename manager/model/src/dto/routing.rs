use crate::NodeId;

#[derive(Debug, Clone)]
pub enum Direction {
    NextNode(NodeId),
    CurrentNode,
}
