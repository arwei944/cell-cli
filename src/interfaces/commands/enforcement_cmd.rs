use crate::application::enforcement_service::{EnforcementService, PolicyLevel, EnforcementReport};
use crate::application::ci_template_service::CiTemplateService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_enforcement(args: EnforcementArgs) -> CellResult<()> {
    let service = EnforcementService::new();
    let project_path = ".";

    match args.sub {
        EnforcementSub::Status {} => {
            let config = service.get_config(project_path)?;
            
            println!("\n🛡️  强约束系统状态\n");
            println!("  总开关: {}", if config.enabled { "✅ 启用" } else { "❌ 禁用" });
            println!();
            
            println!("  📋 策略配置:");
            println!("    {:<25} {:<10}", "规则", "级别");
            println!("    {}", "-".repeat(35));
            println!("    {:<25} {:<10}", "熵值退化", config.policy.entropy_degradation.label());
            println!("    {:<25} {:<10}", "架构违规", config.policy.architecture_violations.label());
            println!("    {:<25} {:<10}", "测试失败", config.policy.test_failure.label());
            println!("    {:<25} {:<10}", "命名违规", config.policy.naming_violations.label());
            println!("    {:<25} {:<10}", "循环依赖", config.policy.circular_dependency.label());
            println!("    {:<25} {:<10}", "未追踪决策", config.policy.untracked_decisions.label());
            println!();
            
            println!("  🪝 Git Hooks:");
            println!("    pre-commit:  {}", if config.git_hooks.pre_commit { "✅" } else { "⬜" });
            println!("    pre-push:    {}", if config.git_hooks.pre_push { "✅" } else { "⬜" });
            println!("    commit-msg:  {}", if config.git_hooks.commit_msg { "✅" } else { "⬜" });
            println!("    pre-rebase:  {}", if config.git_hooks.pre_rebase { "✅" } else { "⬜" });
            println!();
            
            println!("  🏗️  构建守卫:");
            println!("    启用: {}", if config.build_guard.enabled { "✅" } else { "⬜" });
            println!("    架构检查: {}", if config.build_guard.check_architecture { "✅" } else { "⬜" });
            println!("    熵值检查: {}", if config.build_guard.check_entropy { "✅" } else { "⬜" });
            println!("    测试检查: {}", if config.build_guard.check_tests { "✅" } else { "⬜" });
            println!();
        }
        EnforcementSub::PreCommit {} => {
            println!("\n🔍 运行 pre-commit 检查...\n");
            let report = service.run_pre_commit_check(project_path)?;
            print_report(&report);
            
            if !report.passed {
                std::process::exit(1);
            }
        }
        EnforcementSub::PrePush {} => {
            println!("\n🔍 运行 pre-push 检查...\n");
            let report = service.run_pre_push_check(project_path)?;
            print_report(&report);
            
            if !report.passed {
                std::process::exit(1);
            }
        }
        EnforcementSub::BuildGuard {} => {
            println!("\n🔍 运行构建守卫检查...\n");
            let report = service.run_build_guard(project_path)?;
            print_report(&report);
            
            if !report.passed {
                std::process::exit(1);
            }
        }
        EnforcementSub::InstallHooks {} => {
            println!("\n🪝 安装 Git Hooks...\n");
            let installed = service.install_git_hooks(project_path)?;
            
            if installed.is_empty() {
                println!("  没有需要安装的钩子");
            } else {
                for hook in &installed {
                    println!("  ✅ 已安装: {}", hook);
                }
            }
            println!();
            println!("  💡 现在 git commit / git push 会自动触发检查");
            println!();
        }
        EnforcementSub::UninstallHooks {} => {
            println!("\n🪝 卸载 Git Hooks...\n");
            let removed = service.uninstall_git_hooks(project_path)?;
            
            if removed.is_empty() {
                println!("  没有找到 Cell 管理的钩子");
            } else {
                for hook in &removed {
                    println!("  🗑️  已卸载: {}", hook);
                }
            }
            println!();
        }
        EnforcementSub::SetPolicy { policy, level } => {
            let level_enum = match level.to_lowercase().as_str() {
                "allow" | "permit" => PolicyLevel::Allow,
                "warn" | "warning" => PolicyLevel::Warn,
                "block" | "deny" => PolicyLevel::Block,
                _ => return Err(crate::domain::errors::CellError::Config(
                    format!("Invalid level: {}. Use allow, warn, or block.", level)
                )),
            };
            
            service.set_policy(project_path, &policy, level_enum.clone())?;
            println!("\n✅ 策略已更新\n");
            println!("  规则: {}", policy);
            println!("  级别: {}", level_enum.label());
            println!();
        }
        EnforcementSub::Enable {} => {
            let mut config = service.get_config(project_path)?;
            config.enabled = true;
            service.save_config(project_path, &config)?;
            println!("\n✅ 强约束系统已启用\n");
        }
        EnforcementSub::Disable {} => {
            let mut config = service.get_config(project_path)?;
            config.enabled = false;
            service.save_config(project_path, &config)?;
            println!("\n⚠️  强约束系统已禁用\n");
            println!("  建议: 仅在紧急情况下禁用，用完立即恢复");
            println!();
        }
        EnforcementSub::Ci { sub } => {
            let ci_service = CiTemplateService::new();
            match sub {
                CiSub::Generate { provider } => {
                    println!("\n🚀 生成 CI/CD 模板\n");
                    
                    match provider.to_lowercase().as_str() {
                        "github" => {
                            let template = ci_service.generate_github_actions(project_path)?;
                            print_ci_template(&template);
                        }
                        "gitlab" => {
                            let template = ci_service.generate_gitlab_ci(project_path)?;
                            print_ci_template(&template);
                        }
                        "jenkins" => {
                            let template = ci_service.generate_jenkinsfile(project_path)?;
                            print_ci_template(&template);
                        }
                        "gitee" => {
                            let template = ci_service.generate_gitee_workflow(project_path)?;
                            print_ci_template(&template);
                        }
                        "all" => {
                            let templates = ci_service.generate_all(project_path)?;
                            for t in &templates {
                                print_ci_template(t);
                            }
                        }
                        _ => {
                            println!("  未知 provider: {}", provider);
                            println!("  支持: github, gitlab, jenkins, gitee, all");
                        }
                    }
                    println!();
                }
                CiSub::Apply { provider } => {
                    println!("\n🚀 应用 CI/CD 模板\n");
                    
                    let provider_lower = provider.to_lowercase();
                    if provider_lower == "all" {
                        let applied = ci_service.apply_all(project_path)?;
                        for t in &applied {
                            println!("  ✅ {} → {}", t.name, t.path);
                        }
                        println!();
                        return Ok(());
                    }
                    
                    let provider_enum = match provider_lower.as_str() {
                        "github" => crate::application::enforcement_service::CiProvider::Github,
                        "gitlab" => crate::application::enforcement_service::CiProvider::Gitlab,
                        "jenkins" => crate::application::enforcement_service::CiProvider::Jenkins,
                        "gitee" => crate::application::enforcement_service::CiProvider::Gitee,
                        _ => return Err(crate::domain::errors::CellError::Config(
                            format!("Unknown provider: {}", provider)
                        )),
                    };
                    
                    let template = ci_service.apply_template(project_path, &provider_enum)?;
                    println!("  ✅ {} → {}", template.name, template.path);
                    println!();
                }
                CiSub::List {} => {
                    println!("\n🚀 支持的 CI/CD 提供商\n");
                    println!("  {:<15} {}", "提供商", "文件路径");
                    println!("  {}", "-".repeat(50));
                    println!("  {:<15} .github/workflows/cell-gate.yml", "GitHub Actions");
                    println!("  {:<15} .gitlab-ci.yml", "GitLab CI");
                    println!("  {:<15} Jenkinsfile", "Jenkins");
                    println!("  {:<15} .gitee/workflows/cell-gate.yml", "Gitee Go");
                    println!();
                }
            }
        }
    }

    Ok(())
}

fn print_report(report: &EnforcementReport) {
    println!("  ┌─────────────────────────────────────┐");
    println!("  │ {}", if report.passed { "✅ 全部通过" } else { "❌ 检查失败" });
    println!("  │ 阻断: {}  警告: {}", report.block_count, report.warn_count);
    println!("  └─────────────────────────────────────┘");
    println!();

    for check in &report.checks {
        let icon = if check.passed { "✅" } else if check.level == PolicyLevel::Block { "🔴" } else { "🟡" };
        println!("  {} {} [{}]", icon, check.name, check.level.label());
        println!("    {}", check.message);
        for detail in &check.details {
            println!("      • {}", detail);
        }
        println!();
    }

    if !report.passed {
        println!("  ❌ 存在阻断性问题，请修复后重试");
        println!();
    }
}

fn print_ci_template(template: &crate::application::ci_template_service::CiTemplate) {
    println!("\n  📄 {} → {}", template.name, template.path);
    println!("  {}", "-".repeat(50));
    for line in template.content.lines().take(15) {
        println!("  {}", line);
    }
    if template.content.lines().count() > 15 {
        println!("  ... (共 {} 行)", template.content.lines().count());
    }
}