use model::dto::node::{Node, NodeIdList, NodeRecord};
use model::view::auction::AcceptedBid;
use model::{FogNodeFaaSPortExternal, FogNodeHTTPPort, NodeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::IpAddr;
use tokio::sync::RwLock;

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

#[derive(Debug)]
pub struct FogNode {
    nodes: RwLock<HashMap<NodeId, Node<NodeRecord>>>,
}

impl FogNode {
    pub fn new() -> Self { Self { nodes: RwLock::new(HashMap::new()) } }

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

    pub async fn get(&self, id: &NodeId) -> Option<Node<NodeRecord>> {
        return self.nodes.read().await.get(id).cloned();
    }

    pub async fn update(&self, id: &NodeId, record: NodeRecord) {
        if let Some(node) = self.nodes.write().await.get_mut(id) {
            node.data = record;
        }
    }

    pub async fn append_new_child(
        &self,
        parent: &NodeId,
        child: NodeId,
        ip: IpAddr,
        port_http: FogNodeHTTPPort,
        port_faas: FogNodeFaaSPortExternal,
        tags: &[String],
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
                data:     NodeRecord::new(ip, port_http, port_faas, tags),
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

    pub async fn append_root(
        &self,
        root: NodeId,
        ip: IpAddr,
        port_http: FogNodeHTTPPort,
        port_faas: FogNodeFaaSPortExternal,
        tags: &[String],
    ) -> Result<(), Error> {
        self.nodes.write().await.insert(
            root.clone(),
            Node {
                parent:   None,
                children: vec![],
                data:     NodeRecord::new(ip, port_http, port_faas, tags),
            },
        );
        let res = self.check_tree().await;
        if res.is_err() {
            self.nodes.write().await.remove(&root);
        }
        res
    }

    pub async fn get_records(&self) -> HashMap<NodeId, Vec<AcceptedBid>> {
        let mut records: HashMap<NodeId, Vec<AcceptedBid>> = HashMap::new();
        for (node, data) in &*self.nodes.read().await {
            records.insert(
                node.clone(),
                data.data.accepted_bids.values().cloned().collect(),
            );
        }
        records
    }

    pub async fn get_nodes(&self) -> Vec<(NodeId, NodeRecord)> {
        return self
            .nodes
            .read()
            .await
            .iter()
            .map(|(id, record)| (id.clone(), record.data.clone()))
            .collect();
    }
}
