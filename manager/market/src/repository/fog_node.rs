use std::collections::HashMap;
use std::fmt::Debug;
use std::net::IpAddr;

use async_trait::async_trait;

use tokio::sync::RwLock;

use model::dto::node::{Node, NodeIdList, NodeRecord};
use model::view::auction::AcceptedBid;
use model::NodeId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cannot find a root to the tree")]
    NoRoot,
    #[error("Node {0} parent's doesn't exists: {1}")]
    ParentDoesntExist(NodeId, NodeId),
    #[error("Node {0} child's doesn't exists: {1}")]
    ChildDoesntExist(NodeId, NodeId),
    #[error("Multiple roots found: {0}")]
    MultipleRoots(NodeIdList),
}

#[async_trait]
pub trait FogNode: Debug + Sync + Send {
    async fn get(&self, id: &NodeId) -> Option<Node<NodeRecord>>;
    async fn update(&self, id: &NodeId, node: NodeRecord);
    /// Append a new child to the current node, if fails, then doesn't append
    async fn append_new_child(
        &self,
        parent: &NodeId,
        child: NodeId,
        tags: Vec<String>,
    ) -> Result<(), Error>;
    /// Append the root of the tree, i.e., will fail if not the first node in
    /// the whole tree, and will fail thereafter
    async fn append_root(
        &self,
        root: NodeId,
        ip: IpAddr,
        port: u16,
        tags: Vec<String>,
    ) -> Result<(), Error>;
    /// Get all the [NodeId] up to the target node (included).
    /// Return the stack, meaning the destination is at the bottom and the next
    /// node is at the top.
    async fn get_route_to_node(&self, to: NodeId) -> Vec<NodeId>;
    // TODO make it a hashSet?

    async fn get_records(&self) -> HashMap<NodeId, Vec<AcceptedBid>>;

    /// Get all the connected nodes
    async fn get_nodes(&self) -> Vec<(NodeId, NodeRecord)>;

    /// Get a node hosting a function designated by its name
    async fn get_node_from_function(&self, name: &str) -> Option<NodeId>;
}

#[derive(Debug)]
pub struct FogNodeImpl {
    nodes: RwLock<HashMap<NodeId, Node<NodeRecord>>>,
}

impl FogNodeImpl {
    pub fn new() -> Self { FogNodeImpl { nodes: RwLock::new(HashMap::new()) } }

    async fn check_tree(&self) -> Result<(), Error> {
        let mut roots = self
            .nodes
            .read()
            .await
            .iter()
            .filter(|(_id, node)| node.parent.is_none())
            .map(|(id, _node)| id.clone())
            .collect::<Vec<_>>();

        if roots.is_empty() {
            return Err(Error::NoRoot);
        }

        if roots.len() > 1 {
            return Err(Error::MultipleRoots(roots.into()));
        }

        let root = roots.pop().unwrap();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let node_children = self
                .nodes
                .read()
                .await
                .get(&id)
                .map(|node| node.children.clone())
                .unwrap();
            for child in node_children.iter() {
                if !self.nodes.read().await.contains_key(child) {
                    return Err(Error::ChildDoesntExist(
                        id.clone(),
                        child.clone(),
                    ));
                }
            }

            for child in node_children.iter() {
                if let Some(parent) =
                    &self.nodes.read().await.get(child).unwrap().parent
                {
                    if *parent != id {
                        return Err(Error::ParentDoesntExist(
                            id.clone(),
                            parent.clone(),
                        ));
                    }
                }
            }

            stack.extend(node_children.iter().cloned());
        }

        Ok(())
    }

    async fn print_tree(&self) {
        let to_print =
            serde_json::to_string_pretty(&*self.nodes.read().await).unwrap();
        trace!("{}", to_print);
    }
}

#[async_trait]
impl FogNode for FogNodeImpl {
    async fn get(&self, id: &NodeId) -> Option<Node<NodeRecord>> {
        return self.nodes.read().await.get(id).cloned();
    }

    async fn update(&self, id: &NodeId, record: NodeRecord) {
        if let Some(node) = self.nodes.write().await.get_mut(id) {
            node.data = record;
        }
    }

    async fn append_new_child(
        &self,
        parent: &NodeId,
        child: NodeId,
        tags: Vec<String>,
    ) -> Result<(), Error> {
        self.nodes
            .write()
            .await
            .get_mut(parent)
            .ok_or_else(|| {
                Error::ParentDoesntExist(parent.clone(), child.clone())
            })?
            .children
            .push(child.clone());
        self.nodes.write().await.insert(
            child.clone(),
            Node {
                parent:   Some(parent.clone()),
                children: vec![],
                data:     NodeRecord { tags, ..NodeRecord::default() },
            },
        );
        let result = self.check_tree().await;
        if result.is_err() {
            let pos = self
                .nodes
                .read()
                .await
                .get(parent)
                .ok_or_else(|| {
                    Error::ParentDoesntExist(parent.clone(), child.clone())
                })?
                .children
                .iter()
                .rev()
                .position(|node| node == &child)
                .ok_or_else(|| {
                    Error::ChildDoesntExist(parent.clone(), child.clone())
                })?;
            self.nodes
                .write()
                .await
                .get_mut(parent)
                .ok_or_else(|| {
                    Error::ParentDoesntExist(parent.clone(), child.clone())
                })?
                .children
                .remove(pos);
            self.nodes.write().await.remove(&child);

            return result;
        }
        self.print_tree().await;
        Ok(())
    }

    async fn append_root(
        &self,
        root: NodeId,
        ip: IpAddr,
        port: u16,
        tags: Vec<String>,
    ) -> Result<(), Error> {
        self.nodes.write().await.insert(
            root.clone(),
            Node {
                parent:   None,
                children: vec![],
                data:     NodeRecord {
                    ip: Some(ip),
                    port: Some(port),
                    tags,
                    ..NodeRecord::default()
                },
            },
        );
        let res = self.check_tree().await;
        if res.is_err() {
            self.nodes.write().await.remove(&root);
        }
        res
    }

    async fn get_route_to_node(&self, to: NodeId) -> Vec<NodeId> {
        let mut current_cursor = Some(to);
        let mut route_stack = vec![]; // bottom: dest, top: next
        while let Some(current) = &current_cursor {
            if let Some(node) = self.get(current).await {
                route_stack.push(current.clone());
                current_cursor = node.parent;
            }
        }

        route_stack
    }

    async fn get_records(&self) -> HashMap<NodeId, Vec<AcceptedBid>> {
        let mut records: HashMap<NodeId, Vec<AcceptedBid>> = HashMap::new();
        for (node, data) in &*self.nodes.read().await {
            records.insert(
                node.clone(),
                data.data.accepted_bids.values().cloned().collect(),
            );
        }
        records
    }

    async fn get_nodes(&self) -> Vec<(NodeId, NodeRecord)> {
        return self
            .nodes
            .read()
            .await
            .iter()
            .map(|(id, record)| (id.clone(), record.data.clone()))
            .collect();
    }

    async fn get_node_from_function(&self, name: &str) -> Option<NodeId> {
        self.get_records()
            .await
            .iter()
            .find(|(_, rec)| {
                rec.iter().any(|bid| bid.sla.function_live_name.eq(name))
            })
            .map(|(id, _)| id.clone())
    }
}
