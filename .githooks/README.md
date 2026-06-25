# Git Hooks Setup

Cell 架构项目使用 githooks 强制在提交前运行架构检查。

## 安装

### Linux / macOS
```bash
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit
```

### Windows (PowerShell)
```powershell
git config core.hooksPath .githooks
```

## 检查内容

每次提交前自动运行：
1. `cargo fmt --check` - 代码格式检查
2. `cargo clippy -- -D warnings` - Lint 检查（零警告）
3. `cargo test --lib` - 单元测试
4. `cargo run -- arch validate -p .` - 架构分层验证
5. `cargo run -- entropy check src` - 熵值门禁（阈值 5.0）

任何一项失败，提交将被阻止。

## 跳过检查（紧急情况）

```bash
git commit --no-verify
```

⚠️ 仅在紧急情况下使用，CI 仍然会运行所有检查。
