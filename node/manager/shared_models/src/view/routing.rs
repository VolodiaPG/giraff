use core::fmt;

use crate::BidId;

use crate::NodeId;

use serde::{Deserialize, Serialize};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RoutingStackError {
    #[error("The stack is empty for function {0}")]
    Empty(BidId),
    #[error("The current id in the stack is not mine {current}. The stack left is {stack}")]
    CurrentIdIsNotMine { current: NodeId, stack: RouteStack },
    #[error("Next node in the stack is not a child of mine. Stack is: {0}")]
    NextNodeIsNotAChildOfMine(RouteStack),
    #[error("A skip comes after a redirectio, which is not viable. Occurring at node {0}")]
    SkipMispositioned(NodeId),
}

/// Actions that can be taken by the routing stack
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RouteAction {
    Assign {
        node: NodeId,
        next: NodeId,
    },
    Skip {
        node: NodeId,
    },
    Divide {
        node: NodeId,
        next_from_side: Box<RouteStack>,
        next_to_side: Box<RouteStack>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RouteStack {
    pub routes: Vec<RouteAction>,
}

impl fmt::Display for RouteStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.routes)
    }
}
