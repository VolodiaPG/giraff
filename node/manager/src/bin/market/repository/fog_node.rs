use std::collections::HashMap;
use std::fmt::Debug;
use std::net::IpAddr;

use async_trait::async_trait;
use tokio::sync::RwLock;

use manager::model::dto::node::{Node, NodeIdList, NodeRecord};
use manager::model::NodeId;

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
    async fn append_new_child(&self, parent: &NodeId, child: NodeId) -> Result<(), Error>;
    /// Append the root of the tree, i.e., will fail if not the first node in the whole tree, and will fail thereafter
    async fn append_root(&self, root: NodeId, ip: IpAddr, port: u16) -> Result<(), Error>;
    /// Get all the [NodeId] up to the target node (included).
    /// Return the stack, meaning the destination is at the bottom and the next node is at the top.
    async fn get_route_to_node(&self, to: NodeId) -> Vec<NodeId>;
}

#[derive(Debug)]
pub struct FogNodeImpl {
    nodes: RwLock<HashMap<NodeId, Node<NodeRecord>>>,
}

impl FogNodeImpl {
    pub fn new() -> Self {
        FogNodeImpl {
            nodes: RwLock::new(HashMap::new()),
        }
    }
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
                    return Err(Error::ChildDoesntExist(id.clone(), child.clone()));
                }
            }

            for child in node_children.iter() {
                if let Some(parent) = &self.nodes.read().await.get(child).unwrap().parent {
                    if *parent != id {
                        return Err(Error::ParentDoesntExist(id.clone(), parent.clone()));
                    }
                }
            }

            stack.extend(node_children.iter().cloned());
        }

        Ok(())
    }

    async fn print_tree(&self) {
        let to_print = serde_json::to_string_pretty(&*self.nodes.read().await).unwrap();
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

    async fn append_new_child(&self, parent: &NodeId, child: NodeId) -> Result<(), Error> {
        self.nodes
            .write()
            .await
            .get_mut(parent)
            .ok_or_else(|| Error::ParentDoesntExist(parent.clone(), child.clone()))?
            .children
            .push(child.clone());
        self.nodes.write().await.insert(
            child.clone(),
            Node {
                parent: Some(parent.clone()),
                children: vec![],
                data: NodeRecord::default(),
            },
        );
        let result = self.check_tree().await;
        if result.is_err() {
            let pos = self
                .nodes
                .read()
                .await
                .get(parent)
                .ok_or_else(|| Error::ParentDoesntExist(parent.clone(), child.clone()))?
                .children
                .iter()
                .rev()
                .position(|node| node == &child)
                .ok_or_else(|| Error::ChildDoesntExist(parent.clone(), child.clone()))?;
            self.nodes
                .write()
                .await
                .get_mut(parent)
                .ok_or_else(|| Error::ParentDoesntExist(parent.clone(), child.clone()))?
                .children
                .remove(pos);
            self.nodes.write().await.remove(&child);

            return result;
        }
        self.print_tree().await;
        Ok(())
    }

    async fn append_root(&self, root: NodeId, ip: IpAddr, port: u16) -> Result<(), Error> {
        self.nodes.write().await.insert(
            root.clone(),
            Node {
                parent: None,
                children: vec![],
                data: NodeRecord {
                    ip: Some(ip),
                    port: Some(port),
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
            if let Some(node) = self.get(&current).await {
                route_stack.push(current.clone());
                current_cursor = node.parent;
            }
        }

        route_stack
    }
}

// #[async_trait]
// impl FromDisk for FogNodeImpl {
//     /// Create a new tree from a file path.
//     ///
//     /// E.g.,
//     /// ```ron
//     /// {
//     /// "49aaea47-7af7-4c68-b29a-b445ef194d3a": (
//     ///     parent: Some("e13f2a63-2934-480a-a448-b1b01af7e170"),
//     ///     children: [],
//     ///     data: (ip: "localhost:3001"),
//     /// ),
//     /// "e13f2a63-2934-480a-a448-b1b01af7e170": (
//     ///     parent: None,
//     ///     children: ["49aaea47-7af7-4c68-b29a-b445ef194d3a"],
//     ///     data: (ip: "localhost:3002"),
//     /// ),
//     /// }
//     /// ```
//     async fn from_disk(path: &Path) -> Result<Self, manager::helper::from_disk::Error>
//     where
//         Self: Sized,
//     {
//         let mut db = HashMap::<NodeId, Node<NodeRecord>>::new();
//         let content = fs::read_to_string(path.clone())?;
//         let nodes = ron::from_str::<HashMap<NodeId, Node<NodeRecordDisk>>>(&content)?;
//
//         db.extend(nodes.into_iter().map(|(k, v)| (k, v.into())));
//
//         let ret = Self {
//             nodes: RwLock::new(db),
//         };
//
//         ret.check_tree().await.map_err(anyhow::Error::from)?;
//
//         Ok(ret)
//     }
// }

// /// Get the oriented path between two nodes using the lowest common ancestor
// /// The first element of the list is the "to" node and the last the "from" node
// pub fn get_path(&self, from: &NodeId, to: &NodeId) -> Option<Vec<NodeId>> {
//     if self.nodes.get(from).is_none() || self.nodes.get(to).is_none() || from == to {
//         return None;
//     }

//     let mut from_ancestors_hash = HashSet::new();
//     let mut from_ancestors_stack = vec![from];
//     let mut cursor = self.nodes.get(from);
//     while let Some(parent) = cursor.map(|node| node.parent.as_ref()).flatten() {
//         from_ancestors_hash.insert(parent);
//         cursor = self.nodes.get(parent);
//         from_ancestors_stack.push(parent);
//     }

//     let mut to_ancestors_stack = Vec::new();
//     let mut cursor = to;
//     while !from_ancestors_hash.contains(cursor) {
//         to_ancestors_stack.push(cursor.clone());
//         if let Some(parent) = self
//             .nodes
//             .get(cursor)
//             .map(|node| node.parent.as_ref())
//             .flatten()
//         {
//             cursor = parent;
//         }
//     }

//     to_ancestors_stack.extend(
//         from_ancestors_stack
//             .into_iter()
//             .rev()
//             .take_while(|id| id != &cursor)
//             .map(|id| id.clone()),
//     );

//     return Some(to_ancestors_stack);
// }
