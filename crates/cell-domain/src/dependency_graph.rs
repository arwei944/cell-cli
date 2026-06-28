use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeType {
    Cell,
    Service,
    Module,
    Function,
    Database,
    Queue,
    Cache,
    ExternalAPI,
}

impl NodeType {
    pub fn label(&self) -> &str {
        match self {
            Self::Cell => "Cell",
            Self::Service => "Service",
            Self::Module => "Module",
            Self::Function => "Function",
            Self::Database => "Database",
            Self::Queue => "Queue",
            Self::Cache => "Cache",
            Self::ExternalAPI => "External API",
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub id: NodeId,
    pub name: String,
    pub node_type: NodeType,
    pub metadata: HashMap<String, String>,
}

impl DependencyNode {
    pub fn new(id: impl Into<String>, name: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: NodeId(id.into()),
            name: name.into(),
            node_type,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeType {
    Calls,
    DependsOn,
    Publishes,
    Subscribes,
    ReadsFrom,
    WritesTo,
    Uses,
}

impl EdgeType {
    pub fn label(&self) -> &str {
        match self {
            Self::Calls => "calls",
            Self::DependsOn => "depends on",
            Self::Publishes => "publishes",
            Self::Subscribes => "subscribes to",
            Self::ReadsFrom => "reads from",
            Self::WritesTo => "writes to",
            Self::Uses => "uses",
        }
    }

    pub fn is_directional(&self) -> bool {
        true
    }
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub edge_type: EdgeType,
    pub weight: f64,
    pub metadata: HashMap<String, String>,
}

impl DependencyEdge {
    pub fn new(from: NodeId, to: NodeId, edge_type: EdgeType) -> Self {
        Self {
            from,
            to,
            edge_type,
            weight: 1.0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    nodes: HashMap<NodeId, DependencyNode>,
    outgoing_edges: HashMap<NodeId, Vec<DependencyEdge>>,
    incoming_edges: HashMap<NodeId, Vec<DependencyEdge>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            outgoing_edges: HashMap::new(),
            incoming_edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: DependencyNode) {
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        self.outgoing_edges.entry(id.clone()).or_default();
        self.incoming_edges.entry(id).or_default();
    }

    pub fn add_edge(&mut self, edge: DependencyEdge) {
        let from = edge.from.clone();
        let to = edge.to.clone();

        self.outgoing_edges
            .entry(from.clone())
            .or_default()
            .push(edge.clone());
        self.incoming_edges
            .entry(to.clone())
            .or_default()
            .push(edge);

        if !self.nodes.contains_key(&from) {
            self.add_node(DependencyNode::new(
                from.0.clone(),
                from.0,
                NodeType::Module,
            ));
        }
        if !self.nodes.contains_key(&to) {
            self.add_node(DependencyNode::new(
                to.0.clone(),
                to.0,
                NodeType::Module,
            ));
        }
    }

    pub fn get_node(&self, id: &NodeId) -> Option<&DependencyNode> {
        self.nodes.get(id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.outgoing_edges.values().map(std::vec::Vec::len).sum()
    }

    pub fn get_outgoing_edges(&self, node_id: &NodeId) -> Vec<&DependencyEdge> {
        self.outgoing_edges
            .get(node_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_incoming_edges(&self, node_id: &NodeId) -> Vec<&DependencyEdge> {
        self.incoming_edges
            .get(node_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_dependencies(&self, node_id: &NodeId) -> Vec<&DependencyNode> {
        self.get_outgoing_edges(node_id)
            .iter()
            .filter_map(|e| self.nodes.get(&e.to))
            .collect()
    }

    pub fn get_dependents(&self, node_id: &NodeId) -> Vec<&DependencyNode> {
        self.get_incoming_edges(node_id)
            .iter()
            .filter_map(|e| self.nodes.get(&e.from))
            .collect()
    }

    pub fn transitive_dependencies(&self, node_id: &NodeId) -> HashSet<NodeId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        for edge in self.get_outgoing_edges(node_id) {
            if !visited.contains(&edge.to) {
                visited.insert(edge.to.clone());
                queue.push_back(edge.to.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            for edge in self.get_outgoing_edges(&current) {
                if !visited.contains(&edge.to) {
                    visited.insert(edge.to.clone());
                    queue.push_back(edge.to.clone());
                }
            }
        }

        visited
    }

    pub fn transitive_dependents(&self, node_id: &NodeId) -> HashSet<NodeId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        for edge in self.get_incoming_edges(node_id) {
            if !visited.contains(&edge.from) {
                visited.insert(edge.from.clone());
                queue.push_back(edge.from.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            for edge in self.get_incoming_edges(&current) {
                if !visited.contains(&edge.from) {
                    visited.insert(edge.from.clone());
                    queue.push_back(edge.from.clone());
                }
            }
        }

        visited
    }

    pub fn impact_analysis(&self, changed_node: &NodeId) -> ImpactAnalysisResult {
        let dependents = self.transitive_dependents(changed_node);
        let direct_dependents = self.get_dependents(changed_node);

        let mut by_type: HashMap<NodeType, usize> = HashMap::new();
        for dep_id in &dependents {
            if let Some(node) = self.nodes.get(dep_id) {
                *by_type.entry(node.node_type.clone()).or_insert(0) += 1;
            }
        }

        ImpactAnalysisResult {
            changed_node: changed_node.clone(),
            direct_impact_count: direct_dependents.len(),
            total_impact_count: dependents.len(),
            impacted_nodes: dependents,
            impact_by_type: by_type,
        }
    }

    pub fn find_circular_dependencies(&self) -> Vec<Vec<NodeId>> {
        let mut visited = HashSet::new();
        let mut recursion_stack = Vec::new();
        let mut cycles = Vec::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                self.dfs_find_cycles(node_id, &mut visited, &mut recursion_stack, &mut cycles);
            }
        }

        cycles
    }

    fn dfs_find_cycles(
        &self,
        node_id: &NodeId,
        visited: &mut HashSet<NodeId>,
        recursion_stack: &mut Vec<NodeId>,
        cycles: &mut Vec<Vec<NodeId>>,
    ) {
        visited.insert(node_id.clone());
        recursion_stack.push(node_id.clone());

        for edge in self.get_outgoing_edges(node_id) {
            let neighbor = &edge.to;

            if !visited.contains(neighbor) {
                self.dfs_find_cycles(neighbor, visited, recursion_stack, cycles);
            } else if let Some(pos) = recursion_stack.iter().position(|x| x == neighbor) {
                let cycle: Vec<NodeId> = recursion_stack[pos..].to_vec();
                cycles.push(cycle);
            }
        }

        recursion_stack.pop();
    }

    pub fn top_nodes_by_out_degree(&self, n: usize) -> Vec<(NodeId, usize)> {
        let mut nodes: Vec<(NodeId, usize)> = self
            .outgoing_edges
            .iter()
            .map(|(id, edges)| (id.clone(), edges.len()))
            .collect();

        nodes.sort_by_key(|n| std::cmp::Reverse(n.1));
        nodes.into_iter().take(n).collect()
    }

    pub fn top_nodes_by_in_degree(&self, n: usize) -> Vec<(NodeId, usize)> {
        let mut nodes: Vec<(NodeId, usize)> = self
            .incoming_edges
            .iter()
            .map(|(id, edges)| (id.clone(), edges.len()))
            .collect();

        nodes.sort_by_key(|n| std::cmp::Reverse(n.1));
        nodes.into_iter().take(n).collect()
    }

    pub fn all_nodes(&self) -> Vec<&DependencyNode> {
        self.nodes.values().collect()
    }

    pub fn nodes_by_type(&self, node_type: &NodeType) -> Vec<&DependencyNode> {
        self.nodes
            .values()
            .filter(|n| &n.node_type == node_type)
            .collect()
    }

    pub fn shortest_path(&self, from: &NodeId, to: &NodeId) -> Option<Vec<NodeId>> {
        if from == to {
            return Some(vec![from.clone()]);
        }

        let mut visited = HashSet::new();
        let mut parent: HashMap<NodeId, NodeId> = HashMap::new();
        let mut queue = VecDeque::new();

        visited.insert(from.clone());
        queue.push_back(from.clone());

        while let Some(current) = queue.pop_front() {
            for edge in self.get_outgoing_edges(&current) {
                if !visited.contains(&edge.to) {
                    visited.insert(edge.to.clone());
                    parent.insert(edge.to.clone(), current.clone());
                    queue.push_back(edge.to.clone());

                    if &edge.to == to {
                        let mut path = Vec::new();
                        let mut node = to.clone();
                        path.push(node.clone());

                        while let Some(p) = parent.get(&node) {
                            node = p.clone();
                            path.push(node.clone());
                        }

                        path.reverse();
                        return Some(path);
                    }
                }
            }
        }

        None
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysisResult {
    pub changed_node: NodeId,
    pub direct_impact_count: usize,
    pub total_impact_count: usize,
    pub impacted_nodes: HashSet<NodeId>,
    pub impact_by_type: HashMap<NodeType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> DependencyGraph {
        let mut graph = DependencyGraph::new();

        graph.add_node(DependencyNode::new("cell-a", "Cell A", NodeType::Cell));
        graph.add_node(DependencyNode::new("cell-b", "Cell B", NodeType::Cell));
        graph.add_node(DependencyNode::new("cell-c", "Cell C", NodeType::Cell));
        graph.add_node(DependencyNode::new("service-1", "Service 1", NodeType::Service));
        graph.add_node(DependencyNode::new("service-2", "Service 2", NodeType::Service));
        graph.add_node(DependencyNode::new("db-main", "Main DB", NodeType::Database));
        graph.add_node(DependencyNode::new("queue-events", "Events Queue", NodeType::Queue));

        graph.add_edge(DependencyEdge::new(
            NodeId("cell-a".to_string()),
            NodeId("cell-b".to_string()),
            EdgeType::Calls,
        ));
        graph.add_edge(DependencyEdge::new(
            NodeId("cell-b".to_string()),
            NodeId("cell-c".to_string()),
            EdgeType::Calls,
        ));
        graph.add_edge(DependencyEdge::new(
            NodeId("cell-a".to_string()),
            NodeId("service-1".to_string()),
            EdgeType::Uses,
        ));
        graph.add_edge(DependencyEdge::new(
            NodeId("service-1".to_string()),
            NodeId("db-main".to_string()),
            EdgeType::WritesTo,
        ));
        graph.add_edge(DependencyEdge::new(
            NodeId("service-2".to_string()),
            NodeId("db-main".to_string()),
            EdgeType::ReadsFrom,
        ));
        graph.add_edge(DependencyEdge::new(
            NodeId("cell-b".to_string()),
            NodeId("queue-events".to_string()),
            EdgeType::Publishes,
        ));
        graph.add_edge(DependencyEdge::new(
            NodeId("cell-c".to_string()),
            NodeId("queue-events".to_string()),
            EdgeType::Subscribes,
        ));

        graph
    }

    #[test]
    fn test_graph_creation() {
        let graph = create_test_graph();
        assert_eq!(graph.node_count(), 7);
        assert_eq!(graph.edge_count(), 7);
    }

    #[test]
    fn test_get_node() {
        let graph = create_test_graph();
        let node = graph.get_node(&NodeId("cell-a".to_string()));
        assert!(node.is_some());
        assert_eq!(node.unwrap().name, "Cell A");
        assert_eq!(node.unwrap().node_type, NodeType::Cell);
    }

    #[test]
    fn test_outgoing_edges() {
        let graph = create_test_graph();
        let edges = graph.get_outgoing_edges(&NodeId("cell-a".to_string()));
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_incoming_edges() {
        let graph = create_test_graph();
        let edges = graph.get_incoming_edges(&NodeId("db-main".to_string()));
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_direct_dependencies() {
        let graph = create_test_graph();
        let deps = graph.get_dependencies(&NodeId("cell-a".to_string()));
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_transitive_dependencies() {
        let graph = create_test_graph();
        let deps = graph.transitive_dependencies(&NodeId("cell-a".to_string()));
        assert!(deps.contains(&NodeId("cell-b".to_string())));
        assert!(deps.contains(&NodeId("cell-c".to_string())));
        assert!(deps.contains(&NodeId("service-1".to_string())));
        assert!(deps.contains(&NodeId("db-main".to_string())));
        assert!(deps.contains(&NodeId("queue-events".to_string())));
    }

    #[test]
    fn test_transitive_dependents() {
        let graph = create_test_graph();
        let deps = graph.transitive_dependents(&NodeId("db-main".to_string()));
        assert!(deps.contains(&NodeId("service-1".to_string())));
        assert!(deps.contains(&NodeId("cell-a".to_string())));
    }

    #[test]
    fn test_impact_analysis() {
        let graph = create_test_graph();
        let impact = graph.impact_analysis(&NodeId("db-main".to_string()));

        assert_eq!(impact.direct_impact_count, 2);
        assert!(impact.total_impact_count >= 2);
    }

    #[test]
    fn test_circular_dependencies_none() {
        let graph = create_test_graph();
        let cycles = graph.find_circular_dependencies();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_circular_dependencies_found() {
        let mut graph = DependencyGraph::new();
        graph.add_node(DependencyNode::new("a", "A", NodeType::Module));
        graph.add_node(DependencyNode::new("b", "B", NodeType::Module));
        graph.add_node(DependencyNode::new("c", "C", NodeType::Module));

        graph.add_edge(DependencyEdge::new(NodeId("a".into()), NodeId("b".into()), EdgeType::Calls));
        graph.add_edge(DependencyEdge::new(NodeId("b".into()), NodeId("c".into()), EdgeType::Calls));
        graph.add_edge(DependencyEdge::new(NodeId("c".into()), NodeId("a".into()), EdgeType::Calls));

        let cycles = graph.find_circular_dependencies();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_top_nodes_by_degree() {
        let graph = create_test_graph();

        let top_out = graph.top_nodes_by_out_degree(3);
        assert!(!top_out.is_empty());

        let top_in = graph.top_nodes_by_in_degree(3);
        assert!(!top_in.is_empty());
    }

    #[test]
    fn test_nodes_by_type() {
        let graph = create_test_graph();
        let cells = graph.nodes_by_type(&NodeType::Cell);
        assert_eq!(cells.len(), 3);
    }

    #[test]
    fn test_shortest_path() {
        let graph = create_test_graph();
        let path = graph.shortest_path(
            &NodeId("cell-a".to_string()),
            &NodeId("cell-c".to_string()),
        );
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 3);
    }

    #[test]
    fn test_shortest_path_none() {
        let graph = create_test_graph();
        let path = graph.shortest_path(
            &NodeId("cell-c".to_string()),
            &NodeId("cell-a".to_string()),
        );
        assert!(path.is_none());
    }

    #[test]
    fn test_edge_weight() {
        let mut graph = DependencyGraph::new();
        graph.add_node(DependencyNode::new("a", "A", NodeType::Module));
        graph.add_node(DependencyNode::new("b", "B", NodeType::Module));

        let edge = DependencyEdge::new(
            NodeId("a".into()),
            NodeId("b".into()),
            EdgeType::Calls,
        )
        .with_weight(5.0);

        graph.add_edge(edge);

        let edges = graph.get_outgoing_edges(&NodeId("a".into()));
        assert_eq!(edges[0].weight, 5.0);
    }

    #[test]
    fn test_node_metadata() {
        let node = DependencyNode::new("test", "Test", NodeType::Service)
            .with_metadata("version", "1.0")
            .with_metadata("owner", "team-a");

        assert_eq!(node.metadata.get("version").unwrap(), "1.0");
        assert_eq!(node.metadata.get("owner").unwrap(), "team-a");
    }
}
