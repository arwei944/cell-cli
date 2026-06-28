use cell_application::benchmark_service::BenchmarkService;
use cell_domain::errors::CellResult;
use crate::cli::{BenchArgs, BenchSub};

pub fn cmd_bench(args: BenchArgs) -> CellResult<()> {
    let service = BenchmarkService::new();
    let project_path = ".";

    match args.sub {
        BenchSub::Run {} => {
            println!("\n⚡ 运行性能基准测试...\n");
            let results = service.run_benchmarks(project_path)?;
            
            println!("  {:<30} {:>10}", "基准测试", "耗时(ms)");
            println!("  {}", "-".repeat(42));
            for r in &results {
                println!("  {:<30} {:>10}", r.name, r.duration_ms);
            }
            println!();
        }
        BenchSub::Compare { name } => {
            match service.compare_with_baseline(project_path, &name) {
                Ok(comp) => {
                    println!("\n📊 基准比较: {}\n", comp.benchmark_name);
                    println!("  基线平均: {:.2} ms", comp.baseline_avg);
                    println!("  当前平均: {:.2} ms", comp.current_avg);
                    println!("  变化比例: {:+.2}%", comp.change_percent);
                    println!("  阈值: {}%", comp.threshold_percent);
                    if comp.is_regression {
                        println!("\n  ⚠️  检测到性能回归!\n");
                    } else {
                        println!("\n  ✅ 性能正常\n");
                    }
                }
                Err(e) => {
                    println!("\n❌ {e}\n");
                }
            }
        }
        BenchSub::List {} => {
            let benchmarks = service.list_benchmarks(project_path)?;
            println!("\n📋 可用的基准测试\n");
            if benchmarks.is_empty() {
                println!("  暂无基准测试历史");
            } else {
                for b in &benchmarks {
                    println!("  • {b}");
                }
            }
            println!();
        }
    }

    Ok(())
}
