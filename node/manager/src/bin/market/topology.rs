pub struct NodeContent {
    id: NodeId,
    functions: Vec<BidId>,
}

pub enum Node {
    Leaf(&Node, NodeContent),
    Node(&Node, NodeContent, Vec<Node>),
    Root(NodeContent, Vec<Node>),
}

pub struct Topology {
    root: Node,
    nodes: HashMap<NodeId, Node>,
}

impl Topology {
    pub fn new(root: Node) {
        Topology {
            root,
            nodes: HashMap::new(),
        }
    }

    pub fn add(&mut self, anchorage: NodeId, node: Node) {
        self.nodes.insert(node.id(), &node);
        if let Some(anchorage) = self.nodes.get(&anchorage) {
            anchorage.add(node);
        }
    }
}

impl Node {
    pub fn add(&mut self, node: Node) {
        match self {
            Node::Root(content, children) => {
                children.push(node);
            }
            Node::Node(parent, content, children) => {
                children.push(node);
            }
            Node::Leaf(parent, content) => {
                std::mem::replace(parent, Node::Node(self, content, vec![node]));
            }
        }
    }
}
