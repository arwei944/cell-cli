use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PatternId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternCategory {
    Architectural,
    Design,
    Integration,
    Observability,
    Performance,
    AntiPattern,
}

impl PatternCategory {
    pub fn label(&self) -> &str {
        match self {
            Self::Architectural => "Architectural",
            Self::Design => "Design",
            Self::Integration => "Integration",
            Self::Observability => "Observability",
            Self::Performance => "Performance",
            Self::AntiPattern => "AntiPattern",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternStatus {
    Draft,
    Active,
    Deprecated,
}

impl PatternStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Draft => "Draft",
            Self::Active => "Active",
            Self::Deprecated => "Deprecated",
        }
    }

    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Draft | Self::Deprecated, Self::Active) |
(Self::Active, Self::Deprecated)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternRelationType {
    Alternative,
    Composition,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRelation {
    pub target_id: PatternId,
    pub relation_type: PatternRelationType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExample {
    pub title: String,
    pub language: String,
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternProsCons {
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRating {
    pub average_score: f64,
    pub total_ratings: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternVersion {
    pub version: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: PatternId,
    pub name: String,
    pub category: PatternCategory,
    pub status: PatternStatus,
    pub description: String,
    pub problem: String,
    pub solution: String,
    pub tags: Vec<String>,
    pub scenarios: Vec<String>,
    pub pros_cons: PatternProsCons,
    pub examples: Vec<PatternExample>,
    pub versions: Vec<PatternVersion>,
    pub relations: Vec<PatternRelation>,
    pub rating: PatternRating,
    pub usage_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Pattern {
    pub fn new(id: impl Into<String>, name: impl Into<String>, category: PatternCategory) -> Self {
        let now = Utc::now();
        Self {
            id: PatternId(id.into()), name: name.into(), category,
            status: PatternStatus::Draft,
            description: String::new(), problem: String::new(), solution: String::new(),
            tags: Vec::new(), scenarios: Vec::new(),
            pros_cons: PatternProsCons { pros: Vec::new(), cons: Vec::new() },
            examples: Vec::new(), versions: Vec::new(), relations: Vec::new(),
            rating: PatternRating { average_score: 0.0, total_ratings: 0 },
            usage_count: 0, created_at: now, updated_at: now,
        }
    }

    pub fn with_description(mut self, v: impl Into<String>) -> Self { self.description = v.into(); self }
    pub fn with_problem(mut self, v: impl Into<String>) -> Self { self.problem = v.into(); self }
    pub fn with_solution(mut self, v: impl Into<String>) -> Self { self.solution = v.into(); self }
    pub fn add_tag(mut self, v: impl Into<String>) -> Self { self.tags.push(v.into()); self }
    pub fn add_scenario(mut self, v: impl Into<String>) -> Self { self.scenarios.push(v.into()); self }
    pub fn add_pro(mut self, v: impl Into<String>) -> Self { self.pros_cons.pros.push(v.into()); self }
    pub fn add_con(mut self, v: impl Into<String>) -> Self { self.pros_cons.cons.push(v.into()); self }
    pub fn add_example(mut self, v: PatternExample) -> Self { self.examples.push(v); self }
    pub fn add_version(mut self, v: PatternVersion) -> Self { self.versions.push(v); self }
    pub fn add_relation(mut self, v: PatternRelation) -> Self { self.relations.push(v); self }

    pub fn set_status(&mut self, status: PatternStatus) -> bool {
        if self.status.can_transition_to(&status) {
            self.status = status; self.updated_at = Utc::now(); true
        } else { false }
    }

    pub fn rate(&mut self, score: u8) {
        let score = f64::from(score.clamp(1, 5));
        let total = f64::from(self.rating.total_ratings);
        self.rating.average_score = self.rating.average_score.mul_add(total, score) / (total + 1.0);
        self.rating.total_ratings += 1; self.updated_at = Utc::now();
    }

    pub fn increment_usage(&mut self) { self.usage_count += 1; self.updated_at = Utc::now(); }
    pub fn has_tag(&self, tag: &str) -> bool { self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) }
}

pub struct PatternLibrary { patterns: HashMap<PatternId, Pattern> }

impl PatternLibrary {
    pub fn new() -> Self { Self { patterns: HashMap::new() } }

    pub fn register_pattern(&mut self, pattern: Pattern) -> Result<(), String> {
        if self.patterns.contains_key(&pattern.id) {
            return Err(format!("Pattern {} already exists", pattern.id.0));
        }
        self.patterns.insert(pattern.id.clone(), pattern); Ok(())
    }

    pub fn get_pattern(&self, id: &PatternId) -> Option<&Pattern> { self.patterns.get(id) }
    pub fn get_pattern_mut(&mut self, id: &PatternId) -> Option<&mut Pattern> { self.patterns.get_mut(id) }
    pub fn list_patterns(&self) -> Vec<&Pattern> { self.patterns.values().collect() }
    pub fn count(&self) -> usize { self.patterns.len() }

    pub fn search(&self, query: &str) -> Vec<&Pattern> {
        let q = query.to_lowercase();
        let mut results: Vec<&Pattern> = self.patterns.values()
            .filter(|p| {
                p.name.to_lowercase().contains(&q)
                    || p.description.to_lowercase().contains(&q)
                    || p.problem.to_lowercase().contains(&q)
                    || p.solution.to_lowercase().contains(&q)
                    || p.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect();
        results.sort_by(|a, b| {
            let sa = Self::relevance_score(a, &q);
            let sb = Self::relevance_score(b, &q);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    fn relevance_score(p: &Pattern, q: &str) -> f64 {
        let mut s = 0.0;
        if p.name.to_lowercase().contains(q) { s += 10.0; }
        if p.description.to_lowercase().contains(q) { s += 3.0; }
        if p.tags.iter().any(|t| t.to_lowercase() == q) { s += 5.0; }
        s = p.rating.average_score.mul_add(0.5, s);
        s += (p.usage_count as f64).log10().max(0.0);
        s
    }

    pub fn by_category(&self, c: &PatternCategory) -> Vec<&Pattern> {
        self.patterns.values().filter(|p| &p.category == c).collect()
    }

    pub fn by_status(&self, s: &PatternStatus) -> Vec<&Pattern> {
        self.patterns.values().filter(|p| &p.status == s).collect()
    }

    pub fn by_tags(&self, tags: &[String]) -> Vec<&Pattern> {
        let set: HashSet<String> = tags.iter().map(|t| t.to_lowercase()).collect();
        self.patterns.values().filter(|p| p.tags.iter().any(|t| set.contains(&t.to_lowercase()))).collect()
    }

    pub fn version_history(&self, id: &PatternId) -> Option<&[PatternVersion]> {
        self.patterns.get(id).map(|p| p.versions.as_slice())
    }

    pub fn related_patterns(&self, id: &PatternId) -> Vec<(&Pattern, &PatternRelation)> {
        let mut r = Vec::new();
        if let Some(p) = self.patterns.get(id) {
            for rel in &p.relations {
                if let Some(t) = self.patterns.get(&rel.target_id) { r.push((t, rel)); }
            }
        }
        r
    }

    pub fn recommend_related(&self, id: &PatternId, limit: usize) -> Vec<&Pattern> {
        let mut recs = Vec::new();
        let mut seen = HashSet::new();
        if let Some(pattern) = self.patterns.get(id) {
            seen.insert(&pattern.id);
            for rel in &pattern.relations {
                if let Some(target) = self.patterns.get(&rel.target_id) {
                    if seen.insert(&target.id) { recs.push(target); }
                    for r2 in &target.relations {
                        if let Some(snd) = self.patterns.get(&r2.target_id)
                            && seen.insert(&snd.id) { recs.push(snd); }
                    }
                }
            }
            let cat = pattern.category.clone();
            let tag_set: HashSet<String> = pattern.tags.iter().map(|t| t.to_lowercase()).collect();
            let mut cat_matches: Vec<&Pattern> = self.patterns.values()
                .filter(|p| !seen.contains(&p.id) && p.category == cat && p.status == PatternStatus::Active)
                .filter(|p| p.tags.iter().any(|t| tag_set.contains(&t.to_lowercase())))
                .collect();
            cat_matches.sort_by(|a, b| {
                b.rating.average_score.partial_cmp(&a.rating.average_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(b.usage_count.cmp(&a.usage_count))
            });
            for p in cat_matches { if recs.len() >= limit { break; } recs.push(p); }
        }
        recs.into_iter().take(limit).collect()
    }

    pub fn top_rated(&self, limit: usize) -> Vec<&Pattern> {
        let mut v: Vec<&Pattern> = self.patterns.values().collect();
        v.sort_by(|a, b| {
            b.rating.average_score.partial_cmp(&a.rating.average_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.rating.total_ratings.cmp(&a.rating.total_ratings))
        });
        v.into_iter().take(limit).collect()
    }

    pub fn most_used(&self, limit: usize) -> Vec<&Pattern> {
        let mut v: Vec<&Pattern> = self.patterns.values().collect();
        v.sort_by_key(|p| std::cmp::Reverse(p.usage_count));
        v.into_iter().take(limit).collect()
    }
}

impl Default for PatternLibrary { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_pat(id: &str, name: &str, cat: PatternCategory) -> Pattern {
        Pattern::new(id, name, cat)
            .with_description("d").with_problem("p").with_solution("s")
            .add_tag("t").add_scenario("sc").add_pro("good").add_con("bad")
    }

    fn mk_lib() -> PatternLibrary {
        let mut lib = PatternLibrary::new();
        let s = mk_pat("singleton", "Singleton Pattern", PatternCategory::Design)
            .add_tag("creational").add_tag("gang-of-four");
        let f = mk_pat("factory", "Factory Method", PatternCategory::Design)
            .add_tag("creational").add_tag("gang-of-four");
        let o = mk_pat("observer", "Observer Pattern", PatternCategory::Design)
            .add_tag("behavioral");
        let cb = mk_pat("circuit-breaker", "Circuit Breaker", PatternCategory::Performance)
            .add_tag("resilience");
        let go = mk_pat("god-object", "God Object", PatternCategory::AntiPattern);
        let m = mk_pat("metrics", "Metrics Collection", PatternCategory::Observability)
            .add_tag("monitoring");
        lib.register_pattern(s).unwrap();
        lib.register_pattern(f).unwrap();
        lib.register_pattern(o).unwrap();
        lib.register_pattern(cb).unwrap();
        lib.register_pattern(go).unwrap();
        lib.register_pattern(m).unwrap();
        lib
    }

    #[test]
    fn test_register_pattern() {
        let mut lib = PatternLibrary::new();
        assert!(lib.register_pattern(mk_pat("t", "Test", PatternCategory::Design)).is_ok());
        assert_eq!(lib.count(), 1);
    }

    #[test]
    fn test_register_duplicate() {
        let mut lib = PatternLibrary::new();
        lib.register_pattern(mk_pat("dup", "D1", PatternCategory::Design)).unwrap();
        assert!(lib.register_pattern(mk_pat("dup", "D2", PatternCategory::Architectural)).is_err());
    }

    #[test]
    fn test_search_patterns() {
        let lib = mk_lib();
        let r = lib.search("singleton");
        assert!(!r.is_empty());
        assert_eq!(r[0].name, "Singleton Pattern");
    }

    #[test]
    fn test_search_by_tag() {
        let lib = mk_lib();
        assert!(lib.search("creational").len() >= 2);
    }

    #[test]
    fn test_by_category() {
        let lib = mk_lib();
        assert_eq!(lib.by_category(&PatternCategory::Design).len(), 3);
        assert_eq!(lib.by_category(&PatternCategory::AntiPattern).len(), 1);
    }

    #[test]
    fn test_version_management() {
        let mut lib = PatternLibrary::new();
        let p = mk_pat("v", "VTest", PatternCategory::Design).add_version(PatternVersion {
            version: "1.0.0".into(), description: "init".into(),
            created_at: Utc::now(), changes: vec!["release".into()],
        });
        lib.register_pattern(p).unwrap();
        let h = lib.version_history(&PatternId("v".into())).unwrap();
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].version, "1.0.0");
    }

    #[test]
    fn test_pattern_relations() {
        let mut lib = PatternLibrary::new();
        let mono = mk_pat("monostate", "Monostate", PatternCategory::Design);
        let sing = mk_pat("singleton", "Singleton", PatternCategory::Design).add_relation(
            PatternRelation {
                target_id: PatternId("monostate".into()),
                relation_type: PatternRelationType::Alternative,
                description: "alt".into(),
            },
        );
        lib.register_pattern(sing).unwrap();
        lib.register_pattern(mono).unwrap();
        let r = lib.related_patterns(&PatternId("singleton".into()));
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].0.name, "Monostate");
    }

    #[test]
    fn test_tag_filtering() {
        let lib = mk_lib();
        assert_eq!(lib.by_tags(&["gang-of-four".into()]).len(), 2);
        assert_eq!(lib.by_tags(&["resilience".into(), "monitoring".into()]).len(), 2);
    }

    #[test]
    fn test_usage_statistics() {
        let mut lib = PatternLibrary::new();
        lib.register_pattern(mk_pat("u", "U", PatternCategory::Design)).unwrap();
        lib.get_pattern_mut(&PatternId("u".into())).unwrap().increment_usage();
        lib.get_pattern_mut(&PatternId("u".into())).unwrap().increment_usage();
        assert_eq!(lib.get_pattern(&PatternId("u".into())).unwrap().usage_count, 2);
        assert_eq!(lib.most_used(1)[0].usage_count, 2);
    }

    #[test]
    fn test_pattern_rating() {
        let mut lib = PatternLibrary::new();
        lib.register_pattern(mk_pat("r", "R", PatternCategory::Design)).unwrap();
        {
            let p = lib.get_pattern_mut(&PatternId("r".into())).unwrap();
            p.rate(5); p.rate(4); p.rate(3);
        }
        let p = lib.get_pattern(&PatternId("r".into())).unwrap();
        assert_eq!(p.rating.total_ratings, 3);
        assert!((p.rating.average_score - 4.0).abs() < f64::EPSILON);
        assert_eq!(lib.top_rated(1).len(), 1);
    }

    #[test]
    fn test_recommend_related() {
        let mut lib = PatternLibrary::new();
        let p1 = mk_pat("p1", "P1", PatternCategory::Design).add_tag("common");
        let p2 = mk_pat("p2", "P2", PatternCategory::Design).add_tag("common");
        let p3 = mk_pat("p3", "P3", PatternCategory::Design).add_tag("common");
        let p1 = p1.add_relation(PatternRelation {
            target_id: PatternId("p2".into()),
            relation_type: PatternRelationType::Composition,
            description: "comp".into(),
        });
        lib.register_pattern(p1).unwrap();
        lib.register_pattern(p2).unwrap();
        lib.register_pattern(p3).unwrap();
        assert!(!lib.recommend_related(&PatternId("p1".into()), 5).is_empty());
    }

    #[test]
    fn test_status_transitions() {
        let mut p = mk_pat("s", "S", PatternCategory::Design);
        assert_eq!(p.status, PatternStatus::Draft);
        assert!(p.set_status(PatternStatus::Active));
        assert_eq!(p.status, PatternStatus::Active);
        assert!(p.set_status(PatternStatus::Deprecated));
        assert_eq!(p.status, PatternStatus::Deprecated);
        assert!(p.set_status(PatternStatus::Active));
    }

    #[test]
    fn test_invalid_status_transition() {
        let mut p = mk_pat("s2", "S2", PatternCategory::Design);
        assert!(!p.set_status(PatternStatus::Deprecated));
        assert_eq!(p.status, PatternStatus::Draft);
    }

    #[test]
    fn test_pattern_examples_and_labels() {
        let p = mk_pat("ex", "Ex", PatternCategory::Design).add_example(PatternExample {
            title: "rust".into(), language: "rust".into(),
            code: "fn main() {}".into(), description: "ex".into(),
        });
        assert_eq!(p.examples.len(), 1);
        assert_eq!(p.examples[0].language, "rust");
        assert_eq!(PatternCategory::Architectural.label(), "Architectural");
        assert_eq!(PatternCategory::Integration.label(), "Integration");
        assert_eq!(PatternCategory::AntiPattern.label(), "AntiPattern");
    }

    #[test]
    fn test_by_status_and_rating_clamps() {
        let mut lib = PatternLibrary::new();
        let mut p2 = mk_pat("a", "Active", PatternCategory::Design);
        p2.set_status(PatternStatus::Active);
        lib.register_pattern(mk_pat("d", "Draft", PatternCategory::Design)).unwrap();
        lib.register_pattern(p2).unwrap();
        assert_eq!(lib.by_status(&PatternStatus::Draft).len(), 1);
        assert_eq!(lib.by_status(&PatternStatus::Active).len(), 1);

        let mut p = mk_pat("c", "C", PatternCategory::Design);
        p.rate(0); p.rate(10);
        assert_eq!(p.rating.total_ratings, 2);
        assert!(p.rating.average_score >= 1.0 && p.rating.average_score <= 5.0);
    }
}
