use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FeatureId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeatureStatus {
    Draft,
    Active,
    Deprecated,
    Removed,
}

impl FeatureStatus {
    pub fn label(&self) -> &str {
        match self {
            FeatureStatus::Draft => "Draft",
            FeatureStatus::Active => "Active",
            FeatureStatus::Deprecated => "Deprecated",
            FeatureStatus::Removed => "Removed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDef {
    pub id: FeatureId,
    pub name: String,
    pub description: String,
    pub status: FeatureStatus,
    pub category: String,
    pub tags: Vec<String>,
    pub dependencies: Vec<FeatureId>,
    pub conflicts_with: Vec<FeatureId>,
    pub requires_cells: Vec<String>,
    pub complexity_cost: f64,
    pub entropy_contribution: f64,
    pub created_at: String,
    pub updated_at: String,
}

impl FeatureDef {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: FeatureId(id.into()),
            name: name.into(),
            description: String::new(),
            status: FeatureStatus::Draft,
            category: category.into(),
            tags: Vec::new(),
            dependencies: Vec::new(),
            conflicts_with: Vec::new(),
            requires_cells: Vec::new(),
            complexity_cost: 10.0,
            entropy_contribution: 5.0,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_dependency(mut self, dep: FeatureId) -> Self {
        self.dependencies.push(dep);
        self
    }

    pub fn with_conflict(mut self, conflict: FeatureId) -> Self {
        self.conflicts_with.push(conflict);
        self
    }

    pub fn with_cell(mut self, cell: impl Into<String>) -> Self {
        self.requires_cells.push(cell.into());
        self
    }

    pub fn with_complexity(mut self, cost: f64, entropy: f64) -> Self {
        self.complexity_cost = cost;
        self.entropy_contribution = entropy;
        self
    }

    pub fn activate(&mut self) {
        self.status = FeatureStatus::Active;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn deprecate(&mut self) {
        self.status = FeatureStatus::Deprecated;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCombination {
    pub id: String,
    pub name: String,
    pub features: Vec<FeatureId>,
    pub total_complexity: f64,
    pub total_entropy: f64,
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
    pub required_cells: Vec<String>,
}

impl FeatureCombination {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            features: Vec::new(),
            total_complexity: 0.0,
            total_entropy: 0.0,
            is_valid: false,
            validation_errors: Vec::new(),
            required_cells: Vec::new(),
        }
    }

    pub fn add_feature(&mut self, feature_id: FeatureId) {
        if !self.features.contains(&feature_id) {
            self.features.push(feature_id);
        }
    }

    pub fn remove_feature(&mut self, feature_id: &FeatureId) {
        self.features.retain(|f| f != feature_id);
    }
}

pub struct FeatureComposer {
    features: HashMap<FeatureId, FeatureDef>,
    combinations: HashMap<String, FeatureCombination>,
}

impl FeatureComposer {
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
            combinations: HashMap::new(),
        }
    }

    pub fn register_feature(&mut self, feature: FeatureDef) {
        self.features.insert(feature.id.clone(), feature);
    }

    pub fn get_feature(&self, id: &FeatureId) -> Option<&FeatureDef> {
        self.features.get(id)
    }

    pub fn list_features(&self) -> Vec<&FeatureDef> {
        self.features.values().collect()
    }

    pub fn active_features(&self) -> Vec<&FeatureDef> {
        self.features
            .values()
            .filter(|f| f.status == FeatureStatus::Active)
            .collect()
    }

    pub fn features_by_category(&self, category: &str) -> Vec<&FeatureDef> {
        self.features
            .values()
            .filter(|f| f.category == category)
            .collect()
    }

    pub fn features_by_tag(&self, tag: &str) -> Vec<&FeatureDef> {
        self.features
            .values()
            .filter(|f| f.tags.iter().any(|t| t == tag))
            .collect()
    }

    pub fn validate_combination(&self, features: &[FeatureId]) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut total_complexity = 0.0;
        let mut total_entropy = 0.0;
        let mut all_cells = HashSet::new();
        let mut resolved_features = HashSet::new();

        let mut to_visit: Vec<FeatureId> = features.to_vec();
        let mut visited = HashSet::new();

        while let Some(feature_id) = to_visit.pop() {
            if !visited.insert(feature_id.clone()) {
                continue;
            }

            match self.features.get(&feature_id) {
                None => {
                    errors.push(format!("Feature not found: {:?}", feature_id));
                    continue;
                }
                Some(feature) => {
                    if feature.status != FeatureStatus::Active && feature.status != FeatureStatus::Draft {
                        warnings.push(format!("Feature '{}' is {}", feature.name, feature.status.label()));
                    }

                    resolved_features.insert(feature_id.clone());
                    total_complexity += feature.complexity_cost;
                    total_entropy += feature.entropy_contribution;

                    for cell in &feature.requires_cells {
                        all_cells.insert(cell.clone());
                    }

                    for dep in &feature.dependencies {
                        if !visited.contains(dep) {
                            to_visit.push(dep.clone());
                        }
                    }

                    for conflict in &feature.conflicts_with {
                        if features.contains(conflict) || resolved_features.contains(conflict) {
                            errors.push(format!(
                                "Conflict: feature '{:?}' conflicts with '{:?}'",
                                feature_id, conflict
                            ));
                        }
                    }
                }
            }
        }

        for feature_id in features {
            if let Some(feature) = self.features.get(feature_id) {
                for dep in &feature.dependencies {
                    if !resolved_features.contains(dep) {
                        errors.push(format!(
                            "Missing dependency: '{:?}' requires '{:?}'",
                            feature_id, dep
                        ));
                    }
                }
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            total_complexity,
            total_entropy,
            required_cells: all_cells.into_iter().collect(),
            resolved_count: resolved_features.len(),
        }
    }

    pub fn create_combination(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        features: &[FeatureId],
    ) -> Result<&FeatureCombination, String> {
        let id_str = id.into();

        if self.combinations.contains_key(&id_str) {
            return Err(format!("Combination '{}' already exists", id_str));
        }

        let validation = self.validate_combination(features);
        let required_cells = validation.required_cells.clone();

        let combination = FeatureCombination {
            id: id_str.clone(),
            name: name.into(),
            features: features.to_vec(),
            total_complexity: validation.total_complexity,
            total_entropy: validation.total_entropy,
            is_valid: validation.is_valid,
            validation_errors: validation.errors,
            required_cells,
        };

        self.combinations.insert(id_str.clone(), combination);
        Ok(self.combinations.get(&id_str).unwrap())
    }

    pub fn get_combination(&self, id: &str) -> Option<&FeatureCombination> {
        self.combinations.get(id)
    }

    pub fn list_combinations(&self) -> Vec<&FeatureCombination> {
        self.combinations.values().collect()
    }

    pub fn valid_combinations(&self) -> Vec<&FeatureCombination> {
        self.combinations
            .values()
            .filter(|c| c.is_valid)
            .collect()
    }

    pub fn recommend_combinations(&self, target_complexity: f64) -> Vec<Vec<&FeatureDef>> {
        let active: Vec<&FeatureDef> = self.active_features();
        let mut results = Vec::new();

        for i in 0..active.len() {
            let mut combo = vec![active[i]];
            let mut complexity = active[i].complexity_cost;

            for j in (i + 1)..active.len() {
                let feat = active[j];
                let has_conflict = combo.iter().any(|f| {
                    f.conflicts_with.contains(&feat.id)
                        || feat.conflicts_with.contains(&f.id)
                });

                if !has_conflict && complexity + feat.complexity_cost <= target_complexity {
                    combo.push(feat);
                    complexity += feat.complexity_cost;
                }
            }

            if combo.len() >= 2 {
                results.push(combo);
            }
        }

        results.sort_by(|a, b| b.len().cmp(&a.len()));
        results.into_iter().take(10).collect()
    }

    pub fn dependency_chain(&self, feature_id: &FeatureId) -> Vec<FeatureId> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = vec![feature_id.clone()];

        while let Some(id) = stack.pop() {
            if !visited.insert(id.clone()) {
                continue;
            }
            chain.push(id.clone());

            if let Some(feature) = self.features.get(&id) {
                for dep in &feature.dependencies {
                    stack.push(dep.clone());
                }
            }
        }

        chain
    }

    pub fn reverse_dependencies(&self, feature_id: &FeatureId) -> Vec<FeatureId> {
        let mut dependents = Vec::new();

        for feat in self.features.values() {
            if feat.dependencies.contains(feature_id) {
                dependents.push(feat.id.clone());
            }
        }

        dependents
    }

    pub fn detect_circular_dependencies(&self) -> Vec<Vec<FeatureId>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        for feature_id in self.features.keys() {
            if !visited.contains(feature_id) {
                self.detect_cycles_dfs(feature_id, &mut visited, &mut stack, &mut cycles);
            }
        }

        cycles
    }

    fn detect_cycles_dfs(
        &self,
        current: &FeatureId,
        visited: &mut HashSet<FeatureId>,
        stack: &mut Vec<FeatureId>,
        cycles: &mut Vec<Vec<FeatureId>>,
    ) {
        visited.insert(current.clone());
        stack.push(current.clone());

        if let Some(feature) = self.features.get(current) {
            for dep in &feature.dependencies {
                if stack.contains(dep) {
                    let cycle_start = stack.iter().position(|x| x == dep).unwrap();
                    let cycle: Vec<FeatureId> = stack[cycle_start..].to_vec();
                    cycles.push(cycle);
                } else if !visited.contains(dep) {
                    self.detect_cycles_dfs(dep, visited, stack, cycles);
                }
            }
        }

        stack.pop();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub total_complexity: f64,
    pub total_entropy: f64,
    pub required_cells: Vec<String>,
    pub resolved_count: usize,
}

impl Default for FeatureComposer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_feature(id: &str, name: &str) -> FeatureDef {
        FeatureDef::new(id, name, "core")
    }

    #[test]
    fn test_feature_creation() {
        let feature = create_test_feature("f1", "Feature 1");
        assert_eq!(feature.id.0, "f1");
        assert_eq!(feature.name, "Feature 1");
        assert_eq!(feature.status, FeatureStatus::Draft);
    }

    #[test]
    fn test_feature_activate() {
        let mut feature = create_test_feature("f1", "Feature 1");
        feature.activate();
        assert_eq!(feature.status, FeatureStatus::Active);
    }

    #[test]
    fn test_register_feature() {
        let mut composer = FeatureComposer::new();
        let feature = create_test_feature("f1", "Feature 1");
        composer.register_feature(feature);

        assert_eq!(composer.list_features().len(), 1);
        assert!(composer.get_feature(&FeatureId("f1".to_string())).is_some());
    }

    #[test]
    fn test_active_features() {
        let mut composer = FeatureComposer::new();

        let mut f1 = create_test_feature("f1", "Active 1");
        f1.activate();
        composer.register_feature(f1);

        let f2 = create_test_feature("f2", "Draft 1");
        composer.register_feature(f2);

        assert_eq!(composer.active_features().len(), 1);
    }

    #[test]
    fn test_validate_valid_combination() {
        let mut composer = FeatureComposer::new();

        let mut f1 = create_test_feature("f1", "Feature 1");
        f1.activate();
        composer.register_feature(f1);

        let mut f2 = create_test_feature("f2", "Feature 2");
        f2.activate();
        composer.register_feature(f2);

        let result = composer.validate_combination(&[
            FeatureId("f1".to_string()),
            FeatureId("f2".to_string()),
        ]);

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_with_conflict() {
        let mut composer = FeatureComposer::new();

        let f1 = FeatureDef::new("f1", "Feature 1", "core")
            .with_conflict(FeatureId("f2".to_string()));
        composer.register_feature(f1);

        let f2 = create_test_feature("f2", "Feature 2");
        composer.register_feature(f2);

        let result = composer.validate_combination(&[
            FeatureId("f1".to_string()),
            FeatureId("f2".to_string()),
        ]);

        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_missing_dependency() {
        let mut composer = FeatureComposer::new();

        let f1 = FeatureDef::new("f1", "Feature 1", "core")
            .with_dependency(FeatureId("missing".to_string()));
        composer.register_feature(f1);

        let result = composer.validate_combination(&[FeatureId("f1".to_string())]);

        assert!(!result.is_valid);
    }

    #[test]
    fn test_complexity_calculation() {
        let mut composer = FeatureComposer::new();

        let f1 = FeatureDef::new("f1", "Feature 1", "core")
            .with_complexity(20.0, 10.0);
        composer.register_feature(f1);

        let f2 = FeatureDef::new("f2", "Feature 2", "core")
            .with_complexity(30.0, 15.0);
        composer.register_feature(f2);

        let result = composer.validate_combination(&[
            FeatureId("f1".to_string()),
            FeatureId("f2".to_string()),
        ]);

        assert_eq!(result.total_complexity, 50.0);
        assert_eq!(result.total_entropy, 25.0);
    }

    #[test]
    fn test_create_combination() {
        let mut composer = FeatureComposer::new();

        let mut f1 = create_test_feature("f1", "Feature 1");
        f1.activate();
        composer.register_feature(f1);

        let mut f2 = create_test_feature("f2", "Feature 2");
        f2.activate();
        composer.register_feature(f2);

        let combo = composer
            .create_combination(
                "combo1",
                "Combo 1",
                &[FeatureId("f1".to_string()), FeatureId("f2".to_string())],
            )
            .unwrap();

        assert_eq!(combo.features.len(), 2);
        assert!(combo.is_valid);
    }

    #[test]
    fn test_duplicate_combination() {
        let mut composer = FeatureComposer::new();
        composer
            .create_combination("c1", "Combo 1", &[])
            .unwrap();

        let result = composer.create_combination("c1", "Combo 1", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_dependency_chain() {
        let mut composer = FeatureComposer::new();

        let f3 = create_test_feature("f3", "Feature 3");
        composer.register_feature(f3);

        let f2 = FeatureDef::new("f2", "Feature 2", "core")
            .with_dependency(FeatureId("f3".to_string()));
        composer.register_feature(f2);

        let f1 = FeatureDef::new("f1", "Feature 1", "core")
            .with_dependency(FeatureId("f2".to_string()));
        composer.register_feature(f1);

        let chain = composer.dependency_chain(&FeatureId("f1".to_string()));
        assert!(chain.len() >= 3);
    }

    #[test]
    fn test_reverse_dependencies() {
        let mut composer = FeatureComposer::new();

        let f1 = create_test_feature("f1", "Feature 1");
        composer.register_feature(f1);

        let f2 = FeatureDef::new("f2", "Feature 2", "core")
            .with_dependency(FeatureId("f1".to_string()));
        composer.register_feature(f2);

        let dependents = composer.reverse_dependencies(&FeatureId("f1".to_string()));
        assert_eq!(dependents.len(), 1);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut composer = FeatureComposer::new();

        let f1 = FeatureDef::new("f1", "Feature 1", "core")
            .with_dependency(FeatureId("f2".to_string()));
        composer.register_feature(f1);

        let f2 = FeatureDef::new("f2", "Feature 2", "core")
            .with_dependency(FeatureId("f1".to_string()));
        composer.register_feature(f2);

        let cycles = composer.detect_circular_dependencies();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_features_by_category() {
        let mut composer = FeatureComposer::new();
        composer.register_feature(FeatureDef::new("f1", "F1", "auth"));
        composer.register_feature(FeatureDef::new("f2", "F2", "billing"));
        composer.register_feature(FeatureDef::new("f3", "F3", "auth"));

        assert_eq!(composer.features_by_category("auth").len(), 2);
    }

    #[test]
    fn test_valid_combinations() {
        let mut composer = FeatureComposer::new();

        let mut f1 = create_test_feature("f1", "Feature 1");
        f1.activate();
        composer.register_feature(f1);

        let mut f2 = create_test_feature("f2", "Feature 2");
        f2.activate();
        composer.register_feature(f2);

        composer
            .create_combination("valid", "Valid", &[FeatureId("f1".to_string())])
            .unwrap();

        composer
            .create_combination("invalid", "Invalid", &[FeatureId("missing".to_string())])
            .ok();

        assert_eq!(composer.valid_combinations().len(), 1);
    }

    #[test]
    fn test_feature_deprecate() {
        let mut feature = create_test_feature("f1", "Feature 1");
        feature.activate();
        assert_eq!(feature.status, FeatureStatus::Active);
        feature.deprecate();
        assert_eq!(feature.status, FeatureStatus::Deprecated);
    }

    #[test]
    fn test_resolved_count() {
        let mut composer = FeatureComposer::new();

        let f2 = create_test_feature("f2", "Feature 2");
        composer.register_feature(f2);

        let f1 = FeatureDef::new("f1", "Feature 1", "core")
            .with_dependency(FeatureId("f2".to_string()));
        composer.register_feature(f1);

        let result = composer.validate_combination(&[FeatureId("f1".to_string())]);
        assert_eq!(result.resolved_count, 2);
    }
}
