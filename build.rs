use std::env;
use std::path::Path;

fn main() {
    // CI 环境跳过钩子检查
    if env::var("CELL_SKIP_HOOK_CHECK").is_ok() || env::var("CI").is_ok() {
        return;
    }

    // 检查 .git/hooks/pre-commit 是否存在且指向 .githooks/pre-commit
    let git_hooks_path = Path::new(".git/hooks/pre-commit");
    let project_hooks_path = Path::new(".githooks/pre-commit");

    if !project_hooks_path.exists() {
        // 项目钩子不存在可能是发布包，跳过
        return;
    }

    if !git_hooks_path.exists() {
        eprintln!();
        eprintln!("  ❌ Git hooks not installed!");
        eprintln!();
        eprintln!("  To ensure code quality, you must install the pre-commit hooks.");
        eprintln!();
        eprintln!("  Run one of these commands:");
        eprintln!("    make setup-hooks");
        eprintln!("    git config core.hooksPath .githooks");
        eprintln!();
        eprintln!("  To skip this check (CI only), set CELL_SKIP_HOOK_CHECK=1");
        eprintln!();
        panic!("Git hooks not installed. Run 'make setup-hooks' to install.");
    }

    // 检查是否是符号链接或者内容相同（简单检查：读取内容比较）
    let git_hook_content = std::fs::read_to_string(git_hooks_path).ok();
    let project_hook_content = std::fs::read_to_string(project_hooks_path).ok();

    if let (Some(git), Some(project)) = (git_hook_content, project_hook_content)
        && git.trim() != project.trim() {
            // 内容不同，警告但不强制失败（可能用户自定义了）
            println!("cargo:warning=Git hook differs from project hook. Consider running 'make setup-hooks' to update.");
        }

    // 告诉 cargo 不需要重新运行 build.rs 除非这些文件变了
    println!("cargo:rerun-if-changed=.githooks/pre-commit");
    println!("cargo:rerun-if-changed=build.rs");
}
