.PHONY: all build test lint fmt arch entropy check setup-hooks clean

all: check

build:
	cargo build --bin cell

test:
	cargo test --lib

test-all:
	cargo test --all-targets --all-features

lint:
	cargo clippy --all-targets --all-features -- -D warnings

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

setup-hooks:
	git config core.hooksPath .githooks
	@echo "✅ Git hooks configured"

clean:
	cargo clean
