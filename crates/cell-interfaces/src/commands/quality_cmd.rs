use cell_application::config_service::{CellConfig, ConfigService};
use cell_application::coverage_service::CoverageService;
use cell_application::fast_verify_service::FastVerifyService;
use cell_application::progress_bar::StepProgress;
use cell_application::simplicity_checker::SimplicityChecker;
use cell_domain::errors::{CellError, CellResult};
use crate::cli::{TestArgs, TestSub, VerifyArgs, LintArgs, ConfigArgs, ConfigSub};
use std::path::Path;

pub fn cmd_test(args: TestArgs) -> CellResult<()> {
    match args.sub {
        TestSub::Coverage { path } => {
            let path = path.unwrap_or_else(|| ".".to_string());
            let service = CoverageService::new(&path);
            let report = service.analyze()?;
            println!("{}", service.format_report(&report));
        }
        TestSub::Missing { path } => {
            let path = path.unwrap_or_else(|| ".".to_string());
            let service = CoverageService::new(&path);
            let report = service.analyze()?;
            println!("\n📋 缺少测试的文件 ({}/{})\n", report.uncovered_files.len(), report.total_files);
            for f in &report.uncovered_files {
                println!("  • {f}");
            }
            println!();
        }
    }
    Ok(())
}

pub fn cmd_verify(args: VerifyArgs) -> CellResult<()> {
    let path = args.path.unwrap_or_else(|| ".".to_string());
    let service = FastVerifyService::new();
    let is_deep = args.deep;

    let steps = if is_deep {
        vec![
            ("编译检查", "运行 cargo check"),
            ("单元测试", "运行 cargo test --lib"),
            ("架构验证", "检查架构约束"),
            ("代码质量", "简洁性 & 质量检测"),
            ("熵值门禁", "熵值阈值检查"),
        ]
    } else {
        vec![
            ("编译检查", "运行 cargo check"),
            ("快速测试", "运行 cargo test --lib (快速模式)"),
            ("架构验证", "检查架构约束"),
        ]
    };

    let mut progress = StepProgress::new(steps);

    run_verify_step(&mut progress, || service.check_compilation(&path), "编译检查")?;
    run_verify_step(&mut progress, || service.check_tests(&path, is_deep), "测试")?;
    run_verify_step(&mut progress, || service.check_architecture(&path), "架构验证")?;

    if is_deep {
        run_verify_step(&mut progress, || run_lint_check(&path), "代码质量")?;
        run_verify_step(&mut progress, || service.check_entropy_gate(&path, 50.0), "熵值门禁")?;
    }

    progress.render_summary();
    let result = if is_deep { service.deep_check(&path)? } else { service.quick_check(&path)? };
    println!("\n{}", service.format_result(&result));
    if !result.passed { std::process::exit(1); }
    Ok(())
}

fn run_verify_step<F>(progress: &mut StepProgress, check: F, name: &str) -> CellResult<()>
where
    F: FnOnce() -> CellResult<()>,
{
    progress.start_next();
    match check() {
        Ok(()) => progress.complete_current(),
        Err(e) => {
            progress.fail_current(&e.to_string());
            progress.render_summary();
            eprintln!("[{name}] 失败: {e}");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn run_lint_check(path: &str) -> CellResult<()> {
    let checker = SimplicityChecker::new();
    let report = checker.check(path)?;
    if report.score < 60.0 {
        return Err(CellError::Config(
            format!("代码质量评分 {:.1} 低于 60 分阈值", report.score)
        ));
    }
    Ok(())
}

pub fn cmd_lint(args: LintArgs) -> CellResult<()> {
    let path = args.path.unwrap_or_else(|| ".".to_string());
    let mut checker = if args.strict {
        SimplicityChecker::strict()
    } else {
        SimplicityChecker::new()
    };

    if let Some(max_file) = args.max_file_lines {
        checker = checker.with_max_file_lines(max_file);
    }
    if let Some(max_fn) = args.max_fn_lines {
        checker = checker.with_max_fn_lines(max_fn);
    }

    let report = checker.check(&path)?;
    println!("{}", checker.format(&report));
    Ok(())
}

pub fn cmd_config(args: ConfigArgs) -> CellResult<()> {
    match args.sub {
        ConfigSub::Show { path } => {
            let path = path.unwrap_or_else(|| ".".to_string());
            let service = ConfigService::new(&path);
            let config = service.load()?;
            println!("{}", service.format_show(&config));
        }
        ConfigSub::Get { key, path } => {
            let path = path.unwrap_or_else(|| ".".to_string());
            let service = ConfigService::new(&path);
            match service.get(&key)? {
                Some(value) => println!("{key} = {value}"),
                None => println!("未找到配置项: {key}"),
            }
        }
        ConfigSub::Set { key, value, path } => {
            let path = path.unwrap_or_else(|| ".".to_string());
            let service = ConfigService::new(&path);
            service.set(&key, &value)?;
            println!("✅ 已设置: {key} = {value}");
        }
        ConfigSub::Init { path, force } => {
            let path = path.unwrap_or_else(|| ".".to_string());
            let service = ConfigService::new(&path);
            let config_path = service.config_path();
            if Path::new(&config_path).exists() && !force {
                println!("⚠️  配置文件已存在: {config_path}");
                println!("   使用 --force 覆盖现有配置");
                return Ok(());
            }
            let config = CellConfig::default();
            service.save(&config)?;
            println!("✅ 已创建默认配置: {config_path}");
        }
        ConfigSub::Validate { path } => {
            let path = path.unwrap_or_else(|| ".".to_string());
            let service = ConfigService::new(&path);
            let config = service.load()?;
            let result = config.validate();

            println!("\n{}", "=".repeat(50));
            println!("  📐 配置文件验证");
            println!("{}", "=".repeat(50));
            println!("  配置文件: {}", service.config_path());

            if result.errors.is_empty() && result.warnings.is_empty() {
                println!("\n✅ 配置验证通过！所有配置项都有效。\n");
            } else {
                if !result.errors.is_empty() {
                    println!("\n❌ 错误 ({}):", result.errors.len());
                    for e in &result.errors {
                        println!("   • {e}");
                    }
                }
                if !result.warnings.is_empty() {
                    println!("\n⚠️  警告 ({}):", result.warnings.len());
                    for w in &result.warnings {
                        println!("   • {w}");
                    }
                }
                println!();
                if !result.valid {
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}
