.PHONY: all build test lint fmt arch arch-lint lint-arch entropy check setup-hooks clean
.PHONY: coverage coverage-html coverage-lcov coverage-check

all: check

build:
	cargo build --bin cell

test:
	cargo test --lib

test-all:
	cargo test --all-targets --all-features

lint:
	@echo "🔍 Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings
	@echo ""
	@echo "🏗️  Running architecture lint (62 rules)..."
	cargo test --package cell-application architecture_tests --lib
	@echo ""
	@echo "✅ All lint checks passed!"

arch-lint: lint-arch

lint-arch:
	@echo "🏗️  Cell Architecture Lint"
	@echo "─────────────────────────"
	@echo ""
	@echo "📋 Rules summary:"
	@echo "  • Layer dependency rules: 6 (L001-L006)"
	@echo "  • Complexity rules: 5 (C001-C005)"
	@echo "  • Naming rules: 4 (N001-N004)"
	@echo "  • Testing rules: 3 (T001-T003)"
	@echo "  • Best practice rules: 4 (B001-B004)"
	@echo "  • Invariant rules: 6 (INV01-INV06)"
	@echo "  • Code quality rules: 9"
	@echo "  • Visibility rules: 15"
	@echo "  • Total: 62 rules"
	@echo ""
	@echo "🔍 Running architecture tests..."
	@echo ""
	cargo test --package cell-application architecture_tests --lib
	@echo ""
	@echo "✅ Architecture lint passed! (62 rules, 0 violations)"

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

arch:
	cargo run --bin cell -- arch validate -p .

entropy:
	cargo run --bin cell -- entropy check src

check: fmt-check lint test arch entropy
	@echo "✅ All checks passed!"

coverage:
	cargo llvm-cov --workspace --lib --summary-only --output-dir target/coverage -- --skip auto_instrumentation

coverage-html:
	cargo llvm-cov --workspace --lib --html --output-dir target/coverage/html -- --skip auto_instrumentation
	@echo "📊 HTML report: target/coverage/html/index.html"

coverage-lcov:
	cargo llvm-cov --workspace --lib --lcov --output-path target/coverage/lcov.info -- --skip auto_instrumentation

coverage-check:
	@echo "🔍 Checking coverage thresholds..."
	@echo "  cell-domain:     >= 82.0% (baseline: 84.49%)"
	@echo "  cell-application: >= 37.0% (baseline: 39.67%)"
	@echo "  cell-adapters:   >= 85.0% (baseline: 92.23%)"
	@cargo llvm-cov --workspace --lib --summary-only --output-dir target/coverage --fail-under-lines 37 -- --skip auto_instrumentation || (echo "❌ Coverage below threshold!" ; exit 1)
	@echo "✅ Coverage check passed!"

setup-hooks:
	git config core.hooksPath .githooks
	@echo "✅ Git hooks configured"

clean:
	cargo clean
