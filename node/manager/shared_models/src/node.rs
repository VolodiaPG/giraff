use core::fmt;

use crate::BidId;

use super::NodeId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// #[derive(Debug, Serialize, Deserialize, Clone, Validate)]
// pub struct RegisterNode {
//     pub ip: String,
// }

/// changes information of a node
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostNode {
    #[serde(rename = "createdAt")]
    #[serde_as(as = "super::DateTimeHelper")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "lastAnsweredAt")]
    #[serde_as(as = "Option<super::DateTimeHelper>")]
    pub last_answered_at: Option<DateTime<Utc>>,

    #[serde(rename = "lastAnswerReceivedAt")]
    #[serde_as(as = "Option<super::DateTimeHelper>")]
    pub last_answer_received_at: Option<DateTime<Utc>>,

    pub from: NodeId,
}

/// The answer to the post node request
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostNodeResponse {
    #[serde(rename = "answeredAt")]
    #[serde_as(as = "super::DateTimeHelper")]
    pub answered_at: DateTime<Utc>,
}

#[derive(Error, Debug)]
pub enum RoutingStackError {
    #[error("The stack is empty for function {0}")]
    Empty(BidId),
    #[error("The current id in the stack is not mine {current}. The stack left is {stack}")]
    CurrentIdIsNotMine { current: NodeId, stack: RouteStack },
    #[error("Next node in the stack is not a child of mine. Stack is: {0}")]
    NextNodeIsNotAChildOfMine(RouteStack),
    #[error("A skip comes after a redirectio, which is not viable. Occurring at node {0}")]
    SkipMispositioned(NodeId)
}

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
