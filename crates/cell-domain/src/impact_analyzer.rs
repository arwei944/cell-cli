use crate::dependency_graph::{DependencyGraph, NodeId, NodeType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactScope {
    pub changed_nodes: Vec<NodeId>,
    pub direct_impact: Vec<ImpactNode>,
    pub transitive_impact: Vec<ImpactNode>,
    pub blast_radius: usize,
    pub risk_level: RiskLevel,
    pub affected_services: Vec<String>,
    pub affected_databases: Vec<String>,
    pub affected_queues: Vec<String>,
    pub estimated_recovery_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactNode {
    pub id: NodeId,
    pub name: String,
    pub node_type: NodeType,
    pub distance: u32,
    pub impact_score: f64,
    pub criticality: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn label(&self) -> &str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }

    pub fn from_score(score: f64) -> Self {
        if score < 20.0 {
            Self::Low
        } else if score < 50.0 {
            Self::Medium
        } else if score < 80.0 {
            Self::High
        } else {
            Self::Critical
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRequest {
    pub id: String,
    pub description: String,
    pub changed_nodes: Vec<NodeId>,
    pub change_type: ChangeType,
    pub priority: ChangePriority,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    CodeChange,
    ConfigChange,
    SchemaChange,
    DependencyUpgrade,
    Infrastructure,
}

impl ChangeType {
    pub fn label(&self) -> &str {
        match self {
            Self::CodeChange => "Code Change",
            Self::ConfigChange => "Config Change",
            Self::SchemaChange => "Schema Change",
            Self::DependencyUpgrade => "Dependency Upgrade",
            Self::Infrastructure => "Infrastructure",
        }
    }

    pub fn impact_multiplier(&self) -> f64 {
        match self {
            Self::CodeChange => 1.0,
            Self::ConfigChange => 0.8,
            Self::SchemaChange => 1.5,
            Self::DependencyUpgrade => 1.2,
            Self::Infrastructure => 1.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangePriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl ChangePriority {
    pub fn label(&self) -> &str {
        match self {
            Self::Low => "Low",
            Self::Normal => "Normal",
            Self::High => "High",
            Self::Urgent => "Urgent",
        }
    }
}

pub struct ImpactAnalyzer {
    criticality_map: HashMap<NodeId, f64>,
}

impl ImpactAnalyzer {
    pub fn new() -> Self {
        Self {
            criticality_map: HashMap::new(),
        }
    }

    pub fn set_criticality(&mut self, node_id: NodeId, criticality: f64) {
        self.criticality_map.insert(node_id, criticality);
    }

    pub fn get_criticality(&self, node_id: &NodeId) -> f64 {
        self.criticality_map.get(node_id).copied().unwrap_or(50.0)
    }

    pub fn analyze_change(
        &self,
        graph: &DependencyGraph,
        change: &ChangeRequest,
    ) -> ImpactScope {
        let mut all_direct = HashSet::new();
        let mut all_transitive = HashSet::new();
        let mut distances: HashMap<NodeId, u32> = HashMap::new();

        for changed_node in &change.changed_nodes {
            let direct = graph.get_dependents(changed_node);
            for dep in &direct {
                all_direct.insert(dep.id.clone());
                distances.insert(dep.id.clone(), 1);
            }

            let result = graph.impact_analysis(changed_node);
            for imp in &result.impacted_nodes {
                if imp != changed_node {
                    all_transitive.insert(imp.clone());
                    let existing = *distances.get(imp).unwrap_or(&u32::MAX);
                    let dist = self.estimate_distance(graph, changed_node, imp);
                    if dist < existing {
                        distances.insert(imp.clone(), dist);
                    }
                }
            }
        }

        let multiplier = change.change_type.impact_multiplier();

        let mut direct_impact: Vec<ImpactNode> = all_direct
            .iter()
            .map(|id| self.create_impact_node(graph, id, *distances.get(id).unwrap_or(&1), multiplier))
            .collect();

        let mut transitive_impact: Vec<ImpactNode> = all_transitive
            .iter()
            .filter(|id| !all_direct.contains(id))
            .map(|id| self.create_impact_node(graph, id, *distances.get(id).unwrap_or(&2), multiplier))
            .collect();

        direct_impact.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap_or(std::cmp::Ordering::Equal));
        transitive_impact.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap_or(std::cmp::Ordering::Equal));

        let total_impact_score: f64 = direct_impact
            .iter()
            .chain(transitive_impact.iter())
            .map(|n| n.impact_score)
            .sum();

        let blast_radius = all_transitive.len() + change.changed_nodes.len();

        let risk_level = RiskLevel::from_score(total_impact_score.min(100.0));

        let mut affected_services = Vec::new();
        let mut affected_databases = Vec::new();
        let mut affected_queues = Vec::new();

        for node_id in all_transitive.iter().chain(all_direct.iter()) {
            if let Some(node) = graph.get_node(node_id) {
                match node.node_type {
                    NodeType::Service | NodeType::Cell => {
                        affected_services.push(node.name.clone());
                    }
                    NodeType::Database => {
                        affected_databases.push(node.name.clone());
                    }
                    NodeType::Queue => {
                        affected_queues.push(node.name.clone());
                    }
                    _ => {}
                }
            }
        }

        affected_services.sort();
        affected_services.dedup();
        affected_databases.sort();
        affected_databases.dedup();
        affected_queues.sort();
        affected_queues.dedup();

        let estimated_recovery = blast_radius as f64 * 15.0;

        ImpactScope {
            changed_nodes: change.changed_nodes.clone(),
            direct_impact,
            transitive_impact,
            blast_radius,
            risk_level,
            affected_services,
            affected_databases,
            affected_queues,
            estimated_recovery_minutes: estimated_recovery,
        }
    }

    fn create_impact_node(
        &self,
        graph: &DependencyGraph,
        id: &NodeId,
        distance: u32,
        multiplier: f64,
    ) -> ImpactNode {
        let criticality = self.get_criticality(id);
        let distance_factor = 1.0 / f64::from(distance);
        let impact_score = criticality * distance_factor * multiplier;

        let (name, node_type) = graph
            .get_node(id)
            .map_or_else(
                || (id.0.clone(), NodeType::Module),
                |n| (n.name.clone(), n.node_type.clone()),
            );

        ImpactNode {
            id: id.clone(),
            name,
            node_type,
            distance,
            impact_score,
            criticality,
        }
    }

    fn estimate_distance(
        &self,
        graph: &DependencyGraph,
        from: &NodeId,
        to: &NodeId,
    ) -> u32 {
        graph
            .shortest_path(from, to)
            .map_or(10, |path| path.len() as u32)
    }

    pub fn compare_changes(
        &self,
        graph: &DependencyGraph,
        changes: &[ChangeRequest],
    ) -> Vec<ChangeImpactComparison> {
        let mut results: Vec<ChangeImpactComparison> = changes
            .iter()
            .map(|c| {
                let scope = self.analyze_change(graph, c);
                ChangeImpactComparison {
                    change_id: c.id.clone(),
                    description: c.description.clone(),
                    blast_radius: scope.blast_radius,
                    risk_level: scope.risk_level,
                    total_impact_score: scope
                        .direct_impact
                        .iter()
                        .chain(scope.transitive_impact.iter())
                        .map(|n| n.impact_score)
                        .sum(),
                    estimated_recovery_minutes: scope.estimated_recovery_minutes,
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.total_impact_score
                .partial_cmp(&a.total_impact_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    pub fn high_risk_downstream(&self, graph: &DependencyGraph, node_id: &NodeId, threshold: f64) -> Vec<ImpactNode> {
        let analysis = graph.impact_analysis(node_id);
        let mut high_risk = Vec::new();

        for imp_id in &analysis.impacted_nodes {
            let criticality = self.get_criticality(imp_id);
            if criticality >= threshold {
                let dist = self.estimate_distance(graph, node_id, imp_id);
                let node = graph.get_node(imp_id);
                high_risk.push(ImpactNode {
                    id: imp_id.clone(),
                    name: node.map_or_else(|| imp_id.0.clone(), |n| n.name.clone()),
                    node_type: node.map_or(NodeType::Module, |n| n.node_type.clone()),
                    distance: dist,
                    impact_score: criticality,
                    criticality,
                });
            }
        }

        high_risk.sort_by(|a, b| b.criticality.partial_cmp(&a.criticality).unwrap_or(std::cmp::Ordering::Equal));
        high_risk
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeImpactComparison {
    pub change_id: String,
    pub description: String,
    pub blast_radius: usize,
    pub risk_level: RiskLevel,
    pub total_impact_score: f64,
    pub estimated_recovery_minutes: f64,
}

impl Default for ImpactAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency_graph::{DependencyEdge, DependencyGraph, DependencyNode, EdgeType};

    fn create_test_graph() -> DependencyGraph {
        let mut graph = DependencyGraph::new();

        graph.add_node(DependencyNode::new("api-gateway", "API Gateway", NodeType::Service));
        graph.add_node(DependencyNode::new("user-service", "User Service", NodeType::Service));
        graph.add_node(DependencyNode::new("order-service", "Order Service", NodeType::Service));
        graph.add_node(DependencyNode::new("payment-service", "Payment Service", NodeType::Service));
        graph.add_node(DependencyNode::new("user-db", "User DB", NodeType::Database));
        graph.add_node(DependencyNode::new("order-db", "Order DB", NodeType::Database));
        graph.add_node(DependencyNode::new("event-queue", "Event Queue", NodeType::Queue));

        graph.add_edge(DependencyEdge::new(NodeId("api-gateway".to_string()), NodeId("user-service".to_string()), EdgeType::Calls));
        graph.add_edge(DependencyEdge::new(NodeId("api-gateway".to_string()), NodeId("order-service".to_string()), EdgeType::Calls));
        graph.add_edge(DependencyEdge::new(NodeId("user-service".to_string()), NodeId("user-db".to_string()), EdgeType::ReadsFrom));
        graph.add_edge(DependencyEdge::new(NodeId("order-service".to_string()), NodeId("order-db".to_string()), EdgeType::ReadsFrom));
        graph.add_edge(DependencyEdge::new(NodeId("order-service".to_string()), NodeId("payment-service".to_string()), EdgeType::Calls));
        graph.add_edge(DependencyEdge::new(NodeId("order-service".to_string()), NodeId("event-queue".to_string()), EdgeType::Publishes));
        graph.add_edge(DependencyEdge::new(NodeId("payment-service".to_string()), NodeId("event-queue".to_string()), EdgeType::Publishes));

        graph
    }

    #[test]
    fn test_analyze_change() {
        let graph = create_test_graph();
        let analyzer = ImpactAnalyzer::new();

        let change = ChangeRequest {
            id: "chg-001".to_string(),
            description: "Update user service".to_string(),
            changed_nodes: vec![NodeId("user-service".to_string())],
            change_type: ChangeType::CodeChange,
            priority: ChangePriority::Normal,
        };

        let scope = analyzer.analyze_change(&graph, &change);
        assert!(scope.blast_radius > 0);
        assert_eq!(scope.changed_nodes.len(), 1);
    }

    #[test]
    fn test_risk_level_calculation() {
        let graph = create_test_graph();
        let analyzer = ImpactAnalyzer::new();

        let change = ChangeRequest {
            id: "chg-001".to_string(),
            description: "Small change".to_string(),
            changed_nodes: vec![NodeId("payment-service".to_string())],
            change_type: ChangeType::ConfigChange,
            priority: ChangePriority::Low,
        };

        let scope = analyzer.analyze_change(&graph, &change);
        assert_ne!(scope.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_schema_change_higher_impact() {
        let graph = create_test_graph();
        let analyzer = ImpactAnalyzer::new();

        let code_change = ChangeRequest {
            id: "code".to_string(),
            description: "Code change".to_string(),
            changed_nodes: vec![NodeId("user-service".to_string())],
            change_type: ChangeType::CodeChange,
            priority: ChangePriority::Normal,
        };

        let schema_change = ChangeRequest {
            id: "schema".to_string(),
            description: "Schema change".to_string(),
            changed_nodes: vec![NodeId("user-service".to_string())],
            change_type: ChangeType::SchemaChange,
            priority: ChangePriority::Normal,
        };

        let code_scope = analyzer.analyze_change(&graph, &code_change);
        let schema_scope = analyzer.analyze_change(&graph, &schema_change);

        let code_impact: f64 = code_scope
            .direct_impact
            .iter()
            .map(|n| n.impact_score)
            .sum();
        let schema_impact: f64 = schema_scope
            .direct_impact
            .iter()
            .map(|n| n.impact_score)
            .sum();

        assert!(schema_impact > code_impact);
    }

    #[test]
    fn test_criticality_affects_impact() {
        let graph = create_test_graph();
        let mut analyzer = ImpactAnalyzer::new();

        let change = ChangeRequest {
            id: "chg-001".to_string(),
            description: "Test".to_string(),
            changed_nodes: vec![NodeId("user-service".to_string())],
            change_type: ChangeType::CodeChange,
            priority: ChangePriority::Normal,
        };

        let low_scope = analyzer.analyze_change(&graph, &change);

        analyzer.set_criticality(NodeId("order-service".to_string()), 95.0);
        let high_scope = analyzer.analyze_change(&graph, &change);

        let low_total: f64 = low_scope.transitive_impact.iter().map(|n| n.impact_score).sum();
        let high_total: f64 = high_scope.transitive_impact.iter().map(|n| n.impact_score).sum();

        assert!(high_total >= low_total);
    }

    #[test]
    fn test_affected_by_category() {
        let graph = create_test_graph();
        let analyzer = ImpactAnalyzer::new();

        let change = ChangeRequest {
            id: "chg-001".to_string(),
            description: "DB change".to_string(),
            changed_nodes: vec![NodeId("user-db".to_string())],
            change_type: ChangeType::SchemaChange,
            priority: ChangePriority::High,
        };

        let scope = analyzer.analyze_change(&graph, &change);
        assert!(!scope.affected_services.is_empty());
    }

    #[test]
    fn test_compare_changes() {
        let graph = create_test_graph();
        let analyzer = ImpactAnalyzer::new();

        let small_change = ChangeRequest {
            id: "small".to_string(),
            description: "Small".to_string(),
            changed_nodes: vec![NodeId("payment-service".to_string())],
            change_type: ChangeType::ConfigChange,
            priority: ChangePriority::Low,
        };

        let big_change = ChangeRequest {
            id: "big".to_string(),
            description: "Big".to_string(),
            changed_nodes: vec![NodeId("order-service".to_string())],
            change_type: ChangeType::CodeChange,
            priority: ChangePriority::High,
        };

        let comparison = analyzer.compare_changes(&graph, &[small_change, big_change]);
        assert_eq!(comparison.len(), 2);
        assert_eq!(comparison[0].change_id, "big");
    }

    #[test]
    fn test_high_risk_downstream() {
        let graph = create_test_graph();
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.set_criticality(NodeId("payment-service".to_string()), 90.0);
        analyzer.set_criticality(NodeId("order-service".to_string()), 80.0);

        let high_risk = analyzer.high_risk_downstream(&graph, &NodeId("order-db".to_string()), 75.0);
        assert!(!high_risk.is_empty());
    }

    #[test]
    fn test_change_type_labels() {
        assert_eq!(ChangeType::CodeChange.label(), "Code Change");
        assert_eq!(ChangeType::SchemaChange.impact_multiplier(), 1.5);
    }

    #[test]
    fn test_risk_level_from_score() {
        assert_eq!(RiskLevel::from_score(10.0), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(30.0), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(60.0), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(90.0), RiskLevel::Critical);
    }

    #[test]
    fn test_direct_vs_transitive() {
        let graph = create_test_graph();
        let analyzer = ImpactAnalyzer::new();

        let change = ChangeRequest {
            id: "chg-001".to_string(),
            description: "Test".to_string(),
            changed_nodes: vec![NodeId("user-db".to_string())],
            change_type: ChangeType::CodeChange,
            priority: ChangePriority::Normal,
        };

        let scope = analyzer.analyze_change(&graph, &change);
        assert!(scope.transitive_impact.len() >= scope.direct_impact.len());
    }

    #[test]
    fn test_blast_radius_includes_changed() {
        let graph = create_test_graph();
        let analyzer = ImpactAnalyzer::new();

        let change = ChangeRequest {
            id: "chg-001".to_string(),
            description: "Test".to_string(),
            changed_nodes: vec![NodeId("user-service".to_string()), NodeId("order-service".to_string())],
            change_type: ChangeType::CodeChange,
            priority: ChangePriority::Normal,
        };

        let scope = analyzer.analyze_change(&graph, &change);
        assert!(scope.blast_radius >= 2);
    }

    #[test]
    fn test_estimated_recovery() {
        let graph = create_test_graph();
        let analyzer = ImpactAnalyzer::new();

        let change = ChangeRequest {
            id: "chg-001".to_string(),
            description: "Test".to_string(),
            changed_nodes: vec![NodeId("api-gateway".to_string())],
            change_type: ChangeType::CodeChange,
            priority: ChangePriority::Normal,
        };

        let scope = analyzer.analyze_change(&graph, &change);
        assert!(scope.estimated_recovery_minutes > 0.0);
    }
}
