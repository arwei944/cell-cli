use crate::application::rule_engine_service::RuleEngineService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;
use std::collections::HashMap;

pub fn cmd_rule(args: RuleArgs) -> CellResult<()> {
    let mut service = RuleEngineService::new();

    match args.sub {
        RuleSub::List { rule_type } => {
            println!("\n📋 Business Rules\n{}", "─".repeat(50));

            let all_rules = service.list_rules();
            let filtered: Vec<_> = if let Some(rt) = rule_type {
                all_rules.into_iter()
                    .filter(|r| format!("{:?}", r.rule_type).to_lowercase() == rt.to_lowercase())
                    .collect()
            } else {
                all_rules
            };

            println!("\nTotal rules: {}\n", filtered.len());
            for (i, rule) in filtered.iter().enumerate() {
                println!("  {}. [{}] {} ({}) - {}",
                    i + 1,
                    rule.rule_type,
                    rule.name,
                    rule.status,
                    rule.id
                );
            }

            println!("{}", "─".repeat(50));
            println!("Use `cell rule evaluate <id>` to evaluate a rule");
        }

        RuleSub::Evaluate { id, data } => {
            println!("\n⚙️ Evaluating Rule: {}\n{}", id, "─".repeat(50));

            let rule_id = id.parse::<uuid::Uuid>().map_err(|e| {
                crate::domain::errors::CellError::Validation(format!("Invalid rule ID: {}", e))
            })?;

            let input: HashMap<String, serde_json::Value> = data
                .map(|d| serde_json::from_str(&d).unwrap_or_default())
                .unwrap_or_default();

            let result = service.evaluate_rule(rule_id, input)?;
            println!("\n  Rule: {} ({})", result.rule_name, result.rule_id);
            println!("  Matched: {}", result.matched);
            println!("  Action results: {}", result.action_results.len());

            println!("{}", "─".repeat(50));
        }

        RuleSub::Activate { id } => {
            println!("\n✨ Activating Rule: {}\n{}", id, "─".repeat(50));

            let rule_id = id.parse::<uuid::Uuid>().map_err(|e| {
                crate::domain::errors::CellError::Validation(format!("Invalid rule ID: {}", e))
            })?;

            service.activate_rule(rule_id)?;
            println!("\n✅ Rule '{}' activated successfully!", id);

            println!("{}", "─".repeat(50));
        }

        RuleSub::Version { id } => {
            println!("\n📜 Version History: {}\n{}", id, "─".repeat(50));

            let rule_id = id.parse::<uuid::Uuid>().map_err(|e| {
                crate::domain::errors::CellError::Validation(format!("Invalid rule ID: {}", e))
            })?;

            let versions = service.get_rule_version_history(rule_id)?;
            for (i, v) in versions.iter().enumerate() {
                println!("  {}. v{} - {} ({})",
                    i + 1,
                    v.version,
                    v.change_note,
                    v.created_at
                );
            }

            println!("{}", "─".repeat(50));
        }
    }

    Ok(())
}
