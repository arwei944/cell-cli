use cell_domain::errors::CellResult;
use cell_domain::pattern_library::{Pattern, PatternCategory, PatternId, PatternLibrary, PatternStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSummary {
    pub id: String,
    pub name: String,
    pub category: PatternCategory,
    pub status: PatternStatus,
    pub description: String,
    pub tags: Vec<String>,
    pub rating: f64,
    pub usage_count: u64,
}

#[derive(Debug, Clone)]
pub struct PatternDetail {
    pub pattern: Pattern,
    pub related_patterns: Vec<(PatternSummary, String)>,
    pub version_count: usize,
    pub example_count: usize,
}

#[derive(Debug, Clone)]
pub struct PatternRecommendation {
    pub id: String,
    pub name: String,
    pub category: PatternCategory,
    pub reason: String,
    pub similarity_score: f64,
}

pub struct PatternLibraryService {
    library: PatternLibrary,
}

impl PatternLibraryService {
    pub fn new() -> Self {
        Self { library: PatternLibrary::new() }
    }

    pub fn with_library(library: PatternLibrary) -> Self {
        Self { library }
    }

    pub fn list_patterns(&self) -> CellResult<Vec<PatternSummary>> {
        let mut patterns: Vec<&Pattern> = self.library.list_patterns();
        patterns.sort_by_key(|p| std::cmp::Reverse(p.updated_at));
        Ok(patterns.into_iter().map(Self::to_summary).collect())
    }

    pub fn search_patterns(&self, keyword: &str) -> CellResult<Vec<PatternSummary>> {
        let results = self.library.search(keyword);
        Ok(results.into_iter().map(Self::to_summary).collect())
    }

    pub fn get_pattern_detail(&self, id: &str) -> CellResult<PatternDetail> {
        let pattern_id = PatternId(id.to_string());
        let pattern = self.library.get_pattern(&pattern_id)
            .ok_or_else(|| cell_domain::errors::CellError::NotFound(format!("Pattern {id} not found")))?;

        let related = self.library.related_patterns(&pattern_id);
        let related_patterns: Vec<(PatternSummary, String)> = related.into_iter()
            .map(|(p, r)| (Self::to_summary(p), r.description.clone()))
            .collect();

        Ok(PatternDetail {
            pattern: pattern.clone(),
            related_patterns,
            version_count: pattern.versions.len(),
            example_count: pattern.examples.len(),
        })
    }

    pub fn recommend_patterns(&self) -> CellResult<Vec<PatternRecommendation>> {
        let top_rated = self.library.top_rated(10);
        let mut recommendations = Vec::new();

        for (i, pattern) in top_rated.iter().enumerate() {
            if pattern.status != PatternStatus::Active {
                continue;
            }

            let reason = if i < 3 {
                "Top rated pattern".to_string()
            } else if pattern.usage_count > 100 {
                "High usage pattern".to_string()
            } else {
                "Recommended pattern".to_string()
            };

            recommendations.push(PatternRecommendation {
                id: pattern.id.0.clone(),
                name: pattern.name.clone(),
                category: pattern.category.clone(),
                reason,
                similarity_score: (i as f64).mul_add(-0.3, 5.0).max(2.0),
            });

            if recommendations.len() >= 8 {
                break;
            }
        }

        Ok(recommendations)
    }

    pub fn format_pattern_list(&self, patterns: &[PatternSummary]) -> String {
        let mut output = String::new();

        if patterns.is_empty() {
            output.push_str("  No patterns\n");
            return output;
        }

        for (i, p) in patterns.iter().enumerate() {
            let status_icon = match p.status {
                PatternStatus::Active => "✅",
                PatternStatus::Draft => "📝",
                PatternStatus::Deprecated => "🚫",
            };

            let cat_label = p.category.label();
            let tags: String = p.tags.iter().take(3).map(|t| format!("#{t}")).collect::<Vec<_>>().join(" ");

            output.push_str(&format!("  {}. {} {} - {}\n", i + 1, status_icon, p.name, cat_label));
            output.push_str(&format!("     ID: {}\n", p.id));
            output.push_str(&format!("     ⭐ {} · 📊 {}\n", p.rating, p.usage_count));
            if !p.description.is_empty() {
                output.push_str(&format!("     {}\n", p.description));
            }
            if !p.tags.is_empty() {
                output.push_str(&format!("     {tags}\n"));
            }
            output.push('\n');
        }

        output
    }

    pub fn format_pattern_detail(&self, detail: &PatternDetail) -> String {
        let p = &detail.pattern;
        let mut output = String::new();

        let status_icon = match p.status {
            PatternStatus::Active => "✅ Active",
            PatternStatus::Draft => "📝 Draft",
            PatternStatus::Deprecated => "🚫 Deprecated",
        };

        output.push_str(&format!("\n📋 {} ({})\n", p.name, status_icon));
        output.push_str(&"═".repeat(60));
        output.push('\n');

        output.push_str(&format!("\nID: {}\n", p.id.0));
        output.push_str(&format!("Category: {}\n", p.category.label()));
        output.push_str(&format!("Rating: ⭐ {} ({} votes)\n", p.rating.average_score, p.rating.total_ratings));
        output.push_str(&format!("Usage: {} times\n", p.usage_count));
        output.push_str(&format!("Versions: {} | Examples: {}\n", detail.version_count, detail.example_count));

        if !p.description.is_empty() {
            output.push_str("\n📖 Description:\n");
            output.push_str(&format!("  {}\n", p.description));
        }

        if !p.problem.is_empty() {
            output.push_str("\n🔍 Problem:\n");
            output.push_str(&format!("  {}\n", p.problem));
        }

        if !p.solution.is_empty() {
            output.push_str("\n💡 Solution:\n");
            output.push_str(&format!("  {}\n", p.solution));
        }

        if !p.tags.is_empty() {
            output.push_str("\n🏷️ Tags:\n");
            output.push_str(&format!("  {}\n", p.tags.join(", ")));
        }

        if !p.scenarios.is_empty() {
            output.push_str("\n🎯 Scenarios:\n");
            for (i, s) in p.scenarios.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, s));
            }
        }

        if !p.pros_cons.pros.is_empty() || !p.pros_cons.cons.is_empty() {
            output.push_str("\n📊 Pros & Cons:\n");
            if !p.pros_cons.pros.is_empty() {
                output.push_str("  ✅ Pros:\n");
                for pro in &p.pros_cons.pros {
                    output.push_str(&format!("     • {pro}\n"));
                }
            }
            if !p.pros_cons.cons.is_empty() {
                output.push_str("  ❌ Cons:\n");
                for con in &p.pros_cons.cons {
                    output.push_str(&format!("     • {con}\n"));
                }
            }
        }

        if !detail.related_patterns.is_empty() {
            output.push_str("\n🔗 Related Patterns:\n");
            for (i, (related, desc)) in detail.related_patterns.iter().enumerate() {
                output.push_str(&format!("  {}. {} ({})\n", i + 1, related.name, desc));
            }
        }

        output.push('\n');
        output
    }

    pub fn format_recommendations(&self, recommendations: &[PatternRecommendation]) -> String {
        let mut output = String::new();

        if recommendations.is_empty() {
            output.push_str("  No recommendations\n");
            return output;
        }

        for (i, rec) in recommendations.iter().enumerate() {
            output.push_str(&format!("  {}. {} - {}\n", i + 1, rec.name, rec.category.label()));
            output.push_str(&format!("     ID: {}\n", rec.id));
            output.push_str(&format!("     🎯 {}\n", rec.reason));
            output.push_str(&format!("     📊 Similarity: {:.2}\n", rec.similarity_score));
            output.push('\n');
        }

        output
    }

    fn to_summary(pattern: &Pattern) -> PatternSummary {
        PatternSummary {
            id: pattern.id.0.clone(),
            name: pattern.name.clone(),
            category: pattern.category.clone(),
            status: pattern.status.clone(),
            description: pattern.description.clone(),
            tags: pattern.tags.clone(),
            rating: pattern.rating.average_score,
            usage_count: pattern.usage_count,
        }
    }
}

impl Default for PatternLibraryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cell_domain::pattern_library::{PatternExample, PatternProsCons, PatternRating, PatternVersion};
    use chrono::Utc;

    fn create_test_pattern(id: &str, name: &str, category: PatternCategory) -> Pattern {
        Pattern {
            id: PatternId(id.to_string()),
            name: name.to_string(),
            category,
            status: PatternStatus::Active,
            description: format!("Description for {name}"),
            problem: format!("Problem for {name}"),
            solution: format!("Solution for {name}"),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            scenarios: vec!["Scenario 1".to_string()],
            pros_cons: PatternProsCons {
                pros: vec!["Pro 1".to_string()],
                cons: vec!["Con 1".to_string()],
            },
            examples: vec![PatternExample {
                title: "Example".to_string(),
                language: "rust".to_string(),
                code: "fn main() {}".to_string(),
                description: "Example description".to_string(),
            }],
            versions: vec![PatternVersion {
                version: "1.0.0".to_string(),
                description: "Initial version".to_string(),
                created_at: Utc::now(),
                changes: vec!["Initial release".to_string()],
            }],
            relations: Vec::new(),
            rating: PatternRating { average_score: 4.5, total_ratings: 10 },
            usage_count: 100,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_service() -> PatternLibraryService {
        let mut library = PatternLibrary::new();
        library.register_pattern(create_test_pattern("singleton", "Singleton", PatternCategory::Design)).unwrap();
        library.register_pattern(create_test_pattern("factory", "Factory", PatternCategory::Design)).unwrap();
        library.register_pattern(create_test_pattern("circuit-breaker", "Circuit Breaker", PatternCategory::Performance)).unwrap();
        PatternLibraryService::with_library(library)
    }

    #[test]
    fn test_list_patterns() {
        let service = create_test_service();
        let result = service.list_patterns().unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.iter().any(|p| p.name == "Singleton"));
    }

    #[test]
    fn test_search_patterns() {
        let service = create_test_service();
        let result = service.search_patterns("Singleton").unwrap();
        assert!(!result.is_empty());
        assert_eq!(result[0].name, "Singleton");
    }

    #[test]
    fn test_get_pattern_detail() {
        let service = create_test_service();
        let detail = service.get_pattern_detail("singleton").unwrap();
        assert_eq!(detail.pattern.name, "Singleton");
        assert_eq!(detail.version_count, 1);
        assert_eq!(detail.example_count, 1);
    }

    #[test]
    fn test_get_pattern_detail_not_found() {
        let service = create_test_service();
        let result = service.get_pattern_detail("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_recommend_patterns() {
        let service = create_test_service();
        let result = service.recommend_patterns().unwrap();
        assert!(!result.is_empty());
        assert!(result.iter().all(|r| r.similarity_score >= 1.0));
    }
}
