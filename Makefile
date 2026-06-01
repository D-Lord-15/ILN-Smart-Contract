# =============================================================================
# ILN Smart Contract — Developer Makefile
# =============================================================================
# Usage:
#   make build          Build optimised WASM for all contracts
#   make test           Run the full test suite (native target)
#   make fmt            Format all Rust source code
#   make lint           Run Clippy linter across the workspace
#   make deploy-testnet Deploy all contracts to Stellar testnet
#   make coverage       Run tarpaulin code-coverage report
#   make clean          Remove build artefacts
#   make help           Print this help message
# =============================================================================

.PHONY: build test fmt lint deploy-testnet coverage clean help

# Default target
.DEFAULT_GOAL := help

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

## build: Compile all contracts to optimised WASM (wasm32v1-none, release profile)
build:
	cargo build-wasm
	@echo "--- Built WASM artefacts ---"
	@ls -lh target/wasm32v1-none/release/*.wasm

# ---------------------------------------------------------------------------
# Test
# ---------------------------------------------------------------------------

## test: Run the full test suite on the native target (includes unit + integration tests)
test:
	cargo test-native

# ---------------------------------------------------------------------------
# Format
# ---------------------------------------------------------------------------

## fmt: Format all Rust source files using rustfmt
fmt:
	cargo fmt --all

# ---------------------------------------------------------------------------
# Lint
# ---------------------------------------------------------------------------

## lint: Run Clippy with workspace-wide checks and deny warnings
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# ---------------------------------------------------------------------------
# Deploy
# ---------------------------------------------------------------------------

## deploy-testnet: Deploy all contracts to the Stellar testnet
##   Requires the Stellar CLI to be installed and STELLAR_ACCOUNT to be set.
##   Example: STELLAR_ACCOUNT=my-account make deploy-testnet
deploy-testnet: build
	@if [ -z "$$STELLAR_ACCOUNT" ]; then \
		echo "Error: STELLAR_ACCOUNT is not set. Export your Stellar account alias before deploying."; \
		exit 1; \
	fi
	@echo "--- Deploying invoice_liquidity ---"
	stellar contract deploy \
		--wasm target/wasm32v1-none/release/invoice_liquidity.wasm \
		--source "$$STELLAR_ACCOUNT" \
		--network testnet
	@echo "--- Deploying iln_governance ---"
	stellar contract deploy \
		--wasm target/wasm32v1-none/release/iln_governance.wasm \
		--source "$$STELLAR_ACCOUNT" \
		--network testnet
	@echo "--- Deploying iln_distribution ---"
	stellar contract deploy \
		--wasm target/wasm32v1-none/release/iln_distribution.wasm \
		--source "$$STELLAR_ACCOUNT" \
		--network testnet
	@echo "--- Deploying reputation_bonus ---"
	stellar contract deploy \
		--wasm target/wasm32v1-none/release/reputation_bonus.wasm \
		--source "$$STELLAR_ACCOUNT" \
		--network testnet

# ---------------------------------------------------------------------------
# Coverage
# ---------------------------------------------------------------------------

## coverage: Run cargo-tarpaulin and produce an HTML coverage report
##   Requires cargo-tarpaulin: cargo install cargo-tarpaulin
coverage:
	cargo tarpaulin \
		--target x86_64-unknown-linux-gnu \
		--workspace \
		--exclude iln_fuzz \
		--out Html \
		--output-dir coverage/
	@echo "--- Coverage report written to coverage/tarpaulin-report.html ---"

# ---------------------------------------------------------------------------
# Clean
# ---------------------------------------------------------------------------

## clean: Remove all build artefacts
clean:
	cargo clean

# ---------------------------------------------------------------------------
# Help
# ---------------------------------------------------------------------------

## help: Print all documented Makefile targets
help:
	@echo ""
	@echo "ILN Smart Contract — available targets:"
	@echo ""
	@grep -E '^## ' Makefile | sed 's/^## /  /'
	@echo ""
