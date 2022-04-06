use core::fmt;
use std::{collections::HashMap, fs};

use anyhow::Result;
use chrono::{Duration, Utc};
use if_chain::if_chain;
use serde::Deserialize;
use sla::Sla;
use uuid::Uuid;

use crate::models::{AuctionStatus, BidId, BidRecord, NodeId, NodeRecord, NodeRecordDisk};

pub struct BidDataBase {
    database: HashMap<BidId, BidRecord>,
}

#[derive(Debug, Deserialize)]
pub struct Node<T> {
    parent: Option<NodeId>,
    children: Vec<NodeId>,

    /// The actual data which will be stored within the tree
    pub data: T,
}

impl From<Node<NodeRecordDisk>> for Node<NodeRecord> {
    fn from(disk: Node<NodeRecordDisk>) -> Self {
        Node {
            parent: disk.parent,
            children: disk.children,
            data: disk.data.into(),
        }
    }
}

pub struct NodesDataBase {
    nodes: HashMap<NodeId, Node<NodeRecord>>,
}

impl BidDataBase {
    pub fn new() -> Self {
        BidDataBase {
            database: HashMap::new(),
        }
    }

    pub fn insert(&mut self, bid: BidRecord) -> BidId {
        let uuid: BidId = Uuid::new_v4().into();
        self.database.insert(uuid.clone(), bid);
        uuid
    }

    pub fn get(&self, id: &BidId) -> Option<&BidRecord> {
        self.database.get(id)
    }

    pub fn update_auction(&mut self, id: &BidId, status: AuctionStatus) {
        if let Some(bid) = self.database.get_mut(id) {
            bid.auction = status;
        }
    }
}

#[derive(Debug)]
pub struct NodeIdList {
    list: Vec<NodeId>,
}

impl fmt::Display for NodeIdList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.list)
    }
}

impl From<Vec<NodeId>> for NodeIdList {
    fn from(list: Vec<NodeId>) -> Self {
        NodeIdList { list }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TreeError {
    #[error("Cannot find a root to the tree")]
    NoRoot,
    #[error("Node {0} parent doesn't exists: {1}")]
    ParentDoesntExist(NodeId, NodeId),
    #[error("Node {0} child doesn't exists: {1}")]
    ChildDoesntExist(NodeId, NodeId),
    #[error("Multiple roots found: {0}")]
    MultipleRoots(NodeIdList),
}

impl NodesDataBase {
    /// Create a new tree from a file path.
    ///
    /// E.g.,
    /// ```ron
    /// {
    /// "49aaea47-7af7-4c68-b29a-b445ef194d3a": (
    ///     parent: Some("e13f2a63-2934-480a-a448-b1b01af7e170"),
    ///     children: [],
    ///     data: (ip: "localhost:3001"),
    /// ),
    /// "e13f2a63-2934-480a-a448-b1b01af7e170": (
    ///     parent: None,
    ///     children: ["49aaea47-7af7-4c68-b29a-b445ef194d3a"],
    ///     data: (ip: "localhost:3002"),
    /// ),
    /// }
    /// ```
    pub fn new(path: String) -> Result<Self, TreeError> {
        let mut db = HashMap::<NodeId, Node<NodeRecord>>::new();
        let ret;
        if_chain! {
            if let Ok(content) = fs::read_to_string(path.clone());
            if let Ok(nodes) = ron::from_str::<HashMap<NodeId, Node<NodeRecordDisk>>>(&content);
            then
            {
                info!("Loading nodes from disk, path: {}", path);
                db.extend(nodes.into_iter().map(|(k, v)| (k, v.into())));

                ret = NodesDataBase {
                    nodes: db,
                };
                ret.check_tree()?;
                Ok(ret)
            }
            else
            {
                error!("No nodes found on disk, path: {}", path);
                Err(TreeError::NoRoot)
            }
        }
    }

    fn check_tree(&self) -> Result<(), TreeError> {
        let mut roots = Vec::new();
        for (id, node) in self.nodes.iter() {
            if node.parent.is_none() {
                roots.push(id.clone());
            }
        }

        if roots.is_empty() {
            return Err(TreeError::NoRoot);
        }

        if roots.len() > 1 {
            return Err(TreeError::MultipleRoots(roots.into()));
        }

        let root = roots.pop().unwrap();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let node = self.nodes.get(&id).unwrap();
            for child in node.children.iter() {
                if !self.nodes.contains_key(child) {
                    return Err(TreeError::ChildDoesntExist(id.clone(), child.clone()));
                }
            }

            for child in node.children.iter() {
                if let Some(parent) = &self.nodes.get(child).unwrap().parent {
                    if *parent != id {
                        return Err(TreeError::ParentDoesntExist(id.clone(), parent.clone()));
                    }
                }
            }

            stack.extend(node.children.iter().cloned());
        }

        Ok(())
    }

    // pub fn get_root(&self) -> &NodeId {
    //     self.nodes.iter()
    //         .find(|(_, node)| node.parent.is_none())
    //         .map(|(id, _)| id)
    //         .unwrap()
    // }

    // pub async fn insert(&mut self, root_id: NodeId, record: NodeRecord) -> Result<NodeId> {
    //     if self.nodes.contains_key(&root_id) {
    //         anyhow::bail!(anyhow::anyhow!("Node already exists"));
    //     }

    //     let uuid:NodeId = Uuid::new_v4().into();

    //     let node = Node {
    //         parent: Some(root_id.clone()),
    //         children: vec![],
    //         data: record,
    //     };

    //     self.nodes.insert(uuid.clone(), node);

    //     let root = self.nodes.get_mut(&root_id).unwrap();
    //     root.children.push(uuid.clone());

    //     Ok(uuid)
    // }

    pub fn get_mut(&mut self, id: &NodeId) -> Option<&mut NodeRecord> {
        self.nodes.get_mut(id).map(|node| &mut node.data)
    }

    pub fn get(&self, id: &NodeId) -> Option<&NodeRecord> {
        self.nodes.get(id).map(|node| &node.data)
    }

    pub fn get_bid_candidates(&self, sla: &Sla, leaf_node: NodeId) -> HashMap<NodeId, NodeRecord> {
        let mut candidates = HashMap::new();
        let mut next_raw = Some(&leaf_node);
        while let Some(next) = next_raw {
            if_chain! {
                if let Some(node) = self.nodes.get(next);
                if node.data.latency.get_avg() < sla.latency_max;
                if Utc::now() - node.data.latency.get_last_update() > Duration::seconds(10);
                then {
                    candidates.insert(next.clone(), node.data.clone());
                    next_raw = node.parent.as_ref();
                }
                else {
                    next_raw = None;
                }
            }
        }

        candidates
    }
}
