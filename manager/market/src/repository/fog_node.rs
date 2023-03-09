use anyhow::{anyhow, ensure, Result};
use model::dto::node::{Node, NodeRecord};
use model::view::auction::AcceptedBid;
use model::{FogNodeFaaSPortExternal, FogNodeHTTPPort, NodeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::IpAddr;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct FogNode {
    nodes: RwLock<HashMap<NodeId, Node<NodeRecord>>>,
}

impl FogNode {
    pub fn new() -> Self { Self { nodes: RwLock::new(HashMap::new()) } }

    async fn check_tree(&self) -> Result<()> {
        let mut roots = self
            .nodes
            .read()
            .await
            .iter()
            .filter(|(_id, node)| node.parent.is_none())
            .map(|(id, _node)| id.clone())
            .collect::<Vec<_>>();

        ensure!(!roots.is_empty(), "The tree doesn't have a root");
        ensure!(roots.len() == 1, "The tree have more than a single root");

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
                ensure!(
                    self.nodes.read().await.contains_key(child),
                    "Node {} child's doesn't exists: {}",
                    id,
                    child
                );
            }

            for child in node_children.iter() {
                if let Some(parent) =
                    &self.nodes.read().await.get(child).unwrap().parent
                {
                    ensure!(
                        *parent == id,
                        "Node {} parent's doesn't exists: {}",
                        id,
                        child
                    );
                }
            }

            stack.extend(node_children.iter().cloned());
        }

        Ok(())
    }

    // async fn print_tree(&self) {
    //     let to_print =
    //         serde_json::to_string_pretty(&*self.nodes.read().await).
    // unwrap();     trace!("{}", to_print);
    // }

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
    ) -> Result<()> {
        self.nodes
            .write()
            .await
            .get_mut(parent)
            .ok_or_else(|| {
                anyhow!(
                    "The parent of {} (which is supposedly {}) doesn't exist",
                    child,
                    parent
                )
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
                    anyhow!(
                        "The parent of {} (which is supposedly {}) doesn't \
                         exist",
                        child,
                        parent
                    )
                })?
                .children
                .iter()
                .rev()
                .position(|node| node == &child)
                .ok_or_else(|| {
                    anyhow!(
                        "The child of {} (which is supposedly {}) doesn't \
                         exist",
                        parent,
                        child,
                    )
                })?;
            self.nodes
                .write()
                .await
                .get_mut(parent)
                .ok_or_else(|| {
                    anyhow!(
                        "The parent of {} (which is supposedly {}) doesn't \
                         exist",
                        child,
                        parent
                    )
                })?
                .children
                .remove(pos);
            self.nodes.write().await.remove(&child);

            return result;
        }
        // self.print_tree().await;
        Ok(())
    }

    pub async fn append_root(
        &self,
        root: NodeId,
        ip: IpAddr,
        port_http: FogNodeHTTPPort,
        port_faas: FogNodeFaaSPortExternal,
        tags: &[String],
    ) -> Result<()> {
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
