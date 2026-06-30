.PHONY: help install build build-rust build-sdk test test-rust test-e2e fuzz lint \
        fmt fmt-check clippy deploy-testnet seed reset reset-testnet docs spec \
        changelog soroban-optimize health

# Package manager for the TypeScript workspaces (sdk, indexer, notifications,
# tests/e2e). Prefer pnpm when available, otherwise fall back to npm.
PKG := $(shell command -v pnpm >/dev/null 2>&1 && echo pnpm || echo npm)
NODE_PKGS := sdk indexer notifications tests/e2e

# Default target: list everything.
.DEFAULT_GOAL := help

help: ## Show this help message
	@echo "ILN Smart Contract — developer workflows"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| sort \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'

install: ## Install Node dependencies across all TypeScript packages
	@for pkg in $(NODE_PKGS); do \
		echo "📦 installing $$pkg ($(PKG))"; \
		(cd $$pkg && $(PKG) install) || exit 1; \
	done

build: build-rust build-sdk ## Build everything (contracts + SDK)

build-rust: ## Build optimized contract WASM (wasm32v1-none release)
	cargo build --target wasm32v1-none --release

build-sdk: ## Build the @iln/sdk TypeScript package
	cd sdk && $(PKG) run build

soroban-optimize: ## Build WASM via the wasm32-unknown-unknown target
	cargo build --release --target wasm32-unknown-unknown

test: test-rust ## Run the Rust unit/integration test suite
test-rust: ## Run cargo test for the whole workspace
	cargo test

test-e2e: ## Run the end-to-end test suite (tests/e2e)
	cd tests/e2e && $(PKG) run test:e2e

fuzz: ## Run the property/fuzz test suite
	cargo test -p iln_fuzz

lint: fmt-check clippy ## Lint everything (rustfmt check + clippy)

fmt: ## Format all Rust code in place
	cargo fmt --all

fmt-check: ## Verify Rust formatting without modifying files
	cargo fmt --all -- --check

clippy: ## Run clippy with warnings denied
	cargo clippy --all-targets -- -D warnings

deploy-testnet: ## Deploy all contracts to Stellar testnet
	bash scripts/deploy-testnet.sh

seed: ## Seed the testnet deployment with sample data
	npx tsx scripts/seed.ts

reset: reset-testnet ## Alias for reset-testnet
reset-testnet: ## Reset local/testnet state
	bash scripts/reset-testnet.sh

health: ## Run the deployment health check (JSON metrics)
	npx tsx scripts/check-contract-health.ts --pretty

docs: ## Generate SDK API documentation (typedoc)
	cd sdk && $(PKG) run docs

# Generate the contract ABI/spec JSON (Issue #111).
# Every `pub fn` in the `#[contractimpl]` block is exported into the Soroban
# spec automatically. When the `stellar` CLI and a built WASM are available we
# emit the canonical embedded spec; otherwise we fall back to a toolchain-free
# source generator that produces an equivalent JSON. Output: docs/contract-spec.json
spec: ## Generate the contract ABI/spec JSON (docs/contract-spec.json)
	@if command -v stellar >/dev/null 2>&1 && [ -f target/wasm32v1-none/release/invoice_liquidity.wasm ]; then \
		stellar contract inspect --wasm target/wasm32v1-none/release/invoice_liquidity.wasm --output json > docs/contract-spec.json && \
		echo "✅ spec from 'stellar contract inspect' -> docs/contract-spec.json"; \
	else \
		echo "ℹ️  stellar CLI/WASM not found; generating spec from source"; \
		npx --yes tsx scripts/gen-spec.ts; \
	fi

# Generate CHANGELOG.md from conventional commits using git-cliff.
# Install: cargo install git-cliff
# Usage:
#   make changelog            # update for unreleased commits
#   make changelog TAG=v1.0.0 # generate up to a specific tag
changelog: ## Generate CHANGELOG.md from conventional commits (git-cliff)
	git cliff $(if $(TAG),--tag $(TAG)) --output CHANGELOG.md
