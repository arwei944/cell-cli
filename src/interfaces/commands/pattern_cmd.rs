use crate::application::pattern_library_service::PatternLibraryService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_pattern(args: PatternArgs) -> CellResult<()> {
    let service = PatternLibraryService::new();

    match args.sub {
        PatternSub::List { category } => {
            println!("\n📋 Pattern Library\n{}", "─".repeat(50));

            if let Some(cat) = category {
                let patterns = service.search_patterns(&cat)?;
                println!("\nCategory: {}\n", cat);
                println!("{}", service.format_pattern_list(&patterns));
            } else {
                let patterns = service.list_patterns()?;
                println!("\nTotal patterns: {}\n", patterns.len());
                println!("{}", service.format_pattern_list(&patterns));
            }

            println!("{}", "─".repeat(50));
            println!("Use `cell pattern detail <id>` to view details");
        }

        PatternSub::Search { keyword } => {
            println!("\n🔍 Search Results for: '{}'\n{}", keyword, "─".repeat(50));

            let patterns = service.search_patterns(&keyword)?;
            println!("\nFound {} pattern(s)\n", patterns.len());
            println!("{}", service.format_pattern_list(&patterns));

            println!("{}", "─".repeat(50));
        }

        PatternSub::Detail { id } => {
            let detail = service.get_pattern_detail(&id)?;
            println!("{}", service.format_pattern_detail(&detail));
        }

        PatternSub::Recommend { id } => {
            println!("\n💡 Pattern Recommendations\n{}", "─".repeat(50));

            if let Some(pattern_id) = id {
                let detail = service.get_pattern_detail(&pattern_id)?;
                let related: Vec<_> = detail.related_patterns.iter()
                    .map(|(p, reason)| crate::application::pattern_library_service::PatternRecommendation {
                        id: p.id.clone(),
                        name: p.name.clone(),
                        category: p.category.clone(),
                        reason: reason.clone(),
                        similarity_score: 0.0,
                    })
                    .collect();
                println!("\nRelated patterns for '{}':\n", detail.pattern.name);
                println!("{}", service.format_recommendations(&related));
            } else {
                let recommendations = service.recommend_patterns()?;
                println!("\n{}", service.format_recommendations(&recommendations));
            }

            println!("{}", "─".repeat(50));
            println!("Use `cell pattern detail <id>` to view details");
        }
    }

    Ok(())
}
