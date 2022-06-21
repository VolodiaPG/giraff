use crate::model::NodeId;

#[derive(Debug, Clone)]
pub enum Direction {
    NextNode(NodeId),
    CurrentNode,
}
