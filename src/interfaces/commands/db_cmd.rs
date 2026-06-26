use crate::application::db_migration_service::{DbMigrationService, ChangeType, MigrationStatus};
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_db(args: DbArgs) -> CellResult<()> {
    let service = DbMigrationService::new();
    let project_path = ".";

    match args.sub {
        DbSub::Create { name, description } => {
            let changes = vec![
                crate::application::db_migration_service::SchemaChange {
                    type_: ChangeType::CreateTable,
                    target: "example_table".to_string(),
                    details: "创建示例表".to_string(),
                },
            ];
            
            let migration = service.create_migration(project_path, &name, &description, changes)?;
            
            println!("\n✅ 迁移已创建\n");
            println!("  ID: {}", migration.id);
            println!("  版本: {}", migration.version);
            println!("  名称: {}", migration.name);
            println!("  描述: {}", migration.description);
            println!("  状态: {}", migration.status.label());
            println!();
        }
        DbSub::List {} => {
            let migrations = service.list_migrations(project_path)?;
            
            println!("\n📦 迁移列表\n");
            if migrations.is_empty() {
                println!("  暂无迁移");
            } else {
                println!("  {:<36} {:<10} {:<10} {:<20}", "ID", "版本", "状态", "创建时间");
                println!("  {}", "-".repeat(70));
                for m in &migrations {
                    println!("  {:<36} {:<10} {:<10} {:<20}", 
                        m.id, m.version, m.status.label(), 
                        m.created_at.split('T').next().unwrap_or(""));
                }
            }
            println!();
        }
        DbSub::Migrate { version } => {
            let applied = service.migrate(project_path, version.as_deref())?;
            
            println!("\n✅ 迁移已执行\n");
            if applied.is_empty() {
                println!("  无待执行的迁移");
            } else {
                println!("  已应用 {} 个迁移:", applied.len());
                for m in &applied {
                    println!("    • {} ({})", m.id, m.version);
                }
            }
            println!();
        }
        DbSub::Rollback { version } => {
            let rolled_back = service.rollback(project_path, &version)?;
            
            println!("\n✅ 迁移已回滚\n");
            if rolled_back.is_empty() {
                println!("  无可回滚的迁移");
            } else {
                println!("  已回滚 {} 个迁移:", rolled_back.len());
                for m in &rolled_back {
                    println!("    • {} ({})", m.id, m.version);
                }
            }
            println!();
        }
        DbSub::Status {} => {
            let history = service.status(project_path)?;
            
            println!("\n📦 迁移状态\n");
            println!("  当前版本: {}", history.current_version);
            println!("  Schema Hash: {}", history.schema_hash);
            println!("  更新时间: {}", history.last_updated);
            println!("  迁移总数: {}", history.migrations.len());
            
            let pending = history.migrations.iter().filter(|m| m.status == MigrationStatus::Pending).count();
            let applied = history.migrations.iter().filter(|m| m.status == MigrationStatus::Applied).count();
            
            println!("\n  待执行: {}", pending);
            println!("  已应用: {}", applied);
            println!();
        }
        DbSub::Validate { id } => {
            let valid = service.validate_migration(project_path, &id)?;
            
            println!("\n🔍 迁移验证\n");
            println!("  ID: {}", id);
            println!("  状态: {}", if valid { "✅ 有效" } else { "❌ 无效" });
            println!();
        }
        DbSub::Drift { schema } => {
            let result = service.detect_drift(project_path, &schema)?;
            
            println!("\n🔍 Schema 漂移检测\n");
            println!("  期望 Hash: {}", result.expected_hash);
            println!("  实际 Hash: {}", result.actual_hash);
            println!("  漂移状态: {}", if result.has_drift { "⚠️  存在漂移" } else { "✅ 无漂移" });
            
            if result.has_drift {
                println!("\n  差异:");
                for d in &result.differences {
                    println!("    • {}", d);
                }
            }
            println!();
        }
    }

    Ok(())
}