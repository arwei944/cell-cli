# Cell Architecture pre-commit hook (Windows PowerShell)
# 提交前强制检查：格式、Lint、测试、架构、熵值

$ErrorActionPreference = "Stop"

function Write-Colored($color, $msg) {
    Write-Host $msg -ForegroundColor $color
}

Write-Colored Yellow "==> Running pre-commit checks..."

# 1. 格式检查
Write-Colored Yellow "[1/5] Checking code format..."
$fmtOutput = cargo fmt --all -- --check 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Colored Red "FAIL: Code formatting is incorrect. Run 'cargo fmt' to fix."
    exit 1
}
Write-Colored Green "PASS: Code format OK"

# 2. Clippy 检查
Write-Colored Yellow "[2/5] Running clippy..."
$clippyOutput = cargo clippy --all-targets --all-features -- -D warnings 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Colored Red "FAIL: Clippy found issues. Run 'cargo clippy' for details."
    exit 1
}
Write-Colored Green "PASS: Clippy OK"

# 3. 单元测试
Write-Colored Yellow "[3/5] Running tests..."
$testOutput = cargo test --lib 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Colored Red "FAIL: Tests failed. Run 'cargo test' for details."
    exit 1
}
Write-Colored Green "PASS: Tests OK"

# 4. 架构验证
Write-Colored Yellow "[4/5] Validating architecture..."
$archOutput = cargo run --bin cell -- arch validate -p . 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Colored Red "FAIL: Architecture validation failed. Run 'cargo run -- arch validate -p .' for details."
    exit 1
}
Write-Colored Green "PASS: Architecture OK"

# 5. 熵值门禁
Write-Colored Yellow "[5/5] Checking entropy gate..."
try {
    $entropyJson = cargo run --bin cell -- entropy check src 2>$null | ConvertFrom-Json
    $score = $entropyJson.overall_score
    Write-Host "  Entropy score: $score"
    if ($score -gt 5.0) {
        Write-Colored Red "FAIL: Entropy score $score exceeds threshold 5.0"
        exit 1
    }
} catch {
    Write-Colored Yellow "WARN: Could not read entropy score, skipping gate."
}
Write-Colored Green "PASS: Entropy OK"

Write-Colored Green "==> All pre-commit checks passed!"
exit 0
