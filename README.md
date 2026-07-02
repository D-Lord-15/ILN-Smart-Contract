# ILN Smart Contract

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Soroban smart contracts for the **Invoice Liquidity Network (ILN)** — a two-sided protocol on [Stellar](https://stellar.org) that connects invoice holders (freelancers, SMEs) with liquidity providers (LPs). Contracts act as trustless escrow: funds are held on-chain, payment terms are enforced by code, and settlement follows a strict state machine.

> **New here?** Start with the [Developer Quickstart](docs/developer-quickstart.md), then read [Architecture](docs/Architecture.md) for the full money flow.
For the full documentation map, see [Documentation Index](docs/index.md) and [Glossary](docs/glossary.md).

---

## Table of Contents

- [How it works](#how-it-works)
- [Contracts](#contracts)
- [Architecture](#architecture)
- [Repository layout](#repository-layout)
- [Quick start](#quick-start)
- [Testnet deployment](#testnet-deployment)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

---

## How it works

| Actor | Role |
|-------|------|
| **Freelancer** | Submits unpaid invoices and receives early liquidity |
| **Payer** | The client who owes the invoice; settles on-chain |
| **Liquidity provider (LP)** | Funds invoices at a discount; earns yield when the payer pays |
| **ILN contracts** | Hold escrow, enforce rules, emit events, and route token transfers |

Typical lifecycle:

1. Freelancer calls `submit_invoice` → invoice is **Pending**.
2. LP calls `fund_invoice` → freelancer receives `(amount − discount)`; invoice is **Funded**.
3. Payer calls `mark_paid` → LP receives principal + yield; invoice is **Paid** (terminal).

Alternative paths include partial funding, defaults, disputes, appeals, and governance-controlled parameter updates. See [Architecture](docs/Architecture.md) for diagrams and edge cases.

---

## Contracts

| Crate | Path | Responsibility |
|-------|------|----------------|
| **`invoice_liquidity`** | `contracts/invoice_liquidity/` | Core escrow: submit, fund, settle, cancel, and default invoices; reputation scores; multi-token support; optional payer oracle |
| **`iln_governance`** | `contracts/iln_governance/` | On-chain governance: proposals, voting, delegation, quorum, and admin veto |
| **`iln_distribution`** | `contracts/iln_distribution/` | Yield and incentive distribution for LPs, freelancers, and payers (linked to governance token) |
| **`reputation_bonus`** | `contracts/reputation_bonus/` | Reputation-based discount bonuses and related invoice hooks |
| **`iln_fuzz`** | `contracts/fuzz/` | Property-based fuzz tests against core invoice flows |
| **Integration tests** | `contracts/tests/` | Cross-contract tests with mock tokens and oracles |

All contracts compile to Soroban WASM (`wasm32v1-none`) and are tested natively via `soroban-sdk` test utilities (no live network required for `cargo test`).

| Doc | Description |
|-----|-------------|
| [First Invoice Tutorial](docs/tutorials/first-invoice.md) | Hands-on walkthrough: submit, fund, settle, and query an invoice on testnet |
| [Local Development Guide](docs/local-development.md) | Docker setup, local Stellar node, deploying contracts locally, running tests |
| [Developer Quickstart](docs/developer-quickstart.md) | Rust toolchain setup, building, testing, and deploying to testnet |
| [Documentation Index](docs/index.md) | Complete map of protocol, integration, operations, and contributor docs |
| [Glossary](docs/glossary.md) | Definitions for protocol, DeFi, invoice factoring, and Stellar terms |
| [SDK Integration Guide](docs/sdk-integration.md) | TypeScript examples for every contract interaction |
| [Architecture](docs/Architecture.md) | System design, money flow, and security model |
| [Contract ABI](docs/contract-abi.md) | Function signatures and error codes |
| [Events](docs/events.md) | All emitted events and their payloads |
| [Governance](docs/governance.md) | Proposal lifecycle and voting mechanics |
| [Storage Layout](docs/storage-layout.md) | On-chain storage key reference |
| [Threat Model](docs/threat-model.md) | Security assumptions and known risks |
---

## Architecture

High-level component view (one deployment per network; `invoice_liquidity` is the primary integration surface):

```
                         ┌─────────────────────────────────────────┐
                         │           Stellar / Soroban             │
                         └─────────────────────────────────────────┘
    Freelancer                    Payer                      LP
        │                          │                          │
        │ submit_invoice           │ mark_paid                │ fund_invoice
        ▼                          ▼                          ▼
┌───────────────────────────────────────────────────────────────────────┐
│                     invoice_liquidity (escrow)                        │
│  Pending → PartiallyFunded → Funded → Paid / Defaulted / Disputed   │
│  + reputation scores · multi-token · optional price oracle            │
└───────────────┬───────────────────────────────┬───────────────────────┘
                │                               │
        ┌───────▼────────┐              ┌───────▼────────┐
        │ iln_governance │              │reputation_bonus│
        │ proposals/votes│              │ discount rules │
        └───────┬────────┘              └────────────────┘
                │
        ┌───────▼────────┐
        │iln_distribution│
        │ yield / claims │
        └────────────────┘
                │
        ┌───────▼────────┐
        │  USDC / XLM    │  Stellar Asset Contracts (SAC)
        │  (test tokens) │
        └────────────────┘
```

**Design notes**

- No backend server; state transitions are driven by signed on-chain invocations.
- Persistent storage uses Soroban `StorageKey` patterns (see [Storage Layout](docs/storage-layout.md)).
- Major design decisions are recorded as [ADRs](docs/adr/README.md).

---

## Repository layout

```
ILN-Smart-Contract/
├── contracts/
│   ├── invoice_liquidity/   # Core escrow contract + unit/integration tests
│   ├── iln_governance/       # Governance contract
│   ├── iln_distribution/   # Distribution / rewards contract
│   ├── reputation_bonus/   # Reputation bonus contract
│   ├── fuzz/                 # Fuzz / property tests (iln_fuzz)
│   └── tests/                # Workspace-level integration tests & mocks
├── docs/                     # Technical documentation (see below)
├── scripts/
│   ├── deploy-local.sh       # Deploy all contracts (local or testnet)
│   ├── setup-local-env.sh    # Docker + Stellar CLI local setup
│   ├── local-test.sh         # Local integration test runner
│   └── gen-abi.ts            # ABI generation helper
├── .github/workflows/        # CI (build, test, benchmarks)
├── Cargo.toml                # Rust workspace manifest
├── Makefile                  # build · test · fuzz · changelog
├── docker-compose.yml        # Local Stellar node for development
├── CONTRIBUTING.md           # Contribution guide
└── README.md                 # This file
```

---

## Quick start

### Prerequisites

| Tool | Version |
|------|---------|
| [Rust](https://rustup.rs/) | ≥ 1.74 |
| `wasm32v1-none` target | `rustup target add wasm32v1-none` |
| [Stellar CLI](https://developers.stellar.org/docs/tools/cli) | For deploy / invoke (optional for tests) |

### Clone, build, and test

A `Makefile` at the repo root provides all common developer commands:

| Command | Description |
|---------|-------------|
| `make build` | Compile all contracts to optimised WASM |
| `make test` | Run the full test suite |
| `make fmt` | Format all Rust source files |
| `make lint` | Run Clippy with denied warnings |
| `make deploy-testnet` | Deploy all contracts to Stellar testnet |
| `make coverage` | Generate a tarpaulin HTML coverage report |
| `make clean` | Remove build artefacts |
| `make help` | List all available targets |

```bash
git clone https://github.com/Invoice-Liquidity-Network/ILN-Smart-Contract.git
cd ILN-Smart-Contract

# Install WASM target (first time only)
rustup target add wasm32v1-none

# Build optimized WASM for all contracts
make build
# or: cargo build --target wasm32v1-none --release

# Run the full test suite
make test
# or: cargo test

# Run property-based fuzz tests
make fuzz
# or: cargo test -p iln_fuzz
```

### Local network (optional)

For integration testing against a local Stellar node:

```bash
./scripts/setup-local-env.sh   # Docker + CLI config
./scripts/deploy-local.sh      # Build & deploy all contracts
./scripts/local-test.sh        # Smoke tests against local node
```

See [Local Development Guide](docs/local-development.md) for troubleshooting and CI notes.

### Deploy to testnet

Full step-by-step instructions (fund accounts, upload WASM, deploy, initialize) are in [Developer Quickstart §7](docs/developer-quickstart.md#7-deploying-to-testnet). Quick deploy of all workspace contracts:

```bash
cargo build --target wasm32v1-none --release
./scripts/deploy-local.sh testnet alice
```

---

## Testnet deployment

<!-- TESTNET_CONTRACT_IDS_START -->
| Resource | Contract ID | Notes |
|----------|-------------|-------|
| **`invoice_liquidity`** | `CD3TE3IAHM737P236XZL2OYU275ZKD6MN7YH7PYYAXYIGEH55OPEWYJC` | Primary integration contract; used in [SDK examples](docs/sdk-integration.md) |
| **`iln_governance`** | `C2AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA` | Governance proposals and voting |
| **`iln_distribution`** | `C2AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB` | Rewards distribution |
| **`reputation_bonus`** | `C2AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC` | Reputation-based bonus rules |
| **Testnet USDC (SAC)** | `CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA` | Referenced in SDK integration guide |
<!-- TESTNET_CONTRACT_IDS_END -->

| Network | RPC | Passphrase |
|---------|-----|------------|
| Testnet | `https://soroban-testnet.stellar.org` | `Test SDF Network ; September 2015` |

`iln_governance`, `iln_distribution`, and `reputation_bonus` are deployed per environment via `scripts/deploy-local.sh`. Save the printed contract IDs after deploy for your integration config.

**Explorer:** [Stellar Expert](https://stellar.expert/explorer/testnet) — search by contract ID.

---

## Documentation

### Getting started

| Document | Description |
|----------|-------------|
| [Developer Quickstart](docs/developer-quickstart.md) | Toolchain, build, test, and testnet deploy |
| [Local Development Guide](docs/local-development.md) | Docker Stellar node, scripts, and workflow |
| [Documentation Index](docs/index.md) | Full documentation map |
| [Glossary](docs/glossary.md) | Protocol and Stellar terminology |
| [SDK Integration Guide](docs/sdk-integration.md) | TypeScript / Stellar SDK examples (testnet) |
| [SDK Usage Guide](sdk/README.md) | Complete NPM package usage guide for @iln/sdk |
| [FAQ](docs/faq.md) | Frequently asked questions for users and developers |

### Architecture & security

| Document | Description |
|----------|-------------|
| [Architecture](docs/Architecture.md) | Actors, money flow, state machine, deployment |
| [Threat Model](docs/threat-model.md) | Security assumptions, risks, and mitigations |
| [Security Policy](docs/security.md) | Reporting process, severity, safe harbor, and component-specific vulnerability classes |
| [Access Control](docs/access-control.md) | Roles, auth requirements, and admin functions |
| [Storage Layout](docs/storage-layout.md) | On-chain keys and data structures |
| [Upgrade Guide](docs/upgrade-guide.md) | Contract upgrade process and safeguards |
| [Mainnet Launch Checklist](docs/mainnet-launch-checklist.md) | Launch readiness owners, statuses, and sign-off |
| [Architecture Decision Records](docs/adr/README.md) | ADR index (Soroban choice, governance timelock, etc.) |

### Contract reference

| Document | Description |
|----------|-------------|
| [Contract ABI](docs/contract-abi.md) | Public functions and **error codes** |
| [Error Codes](docs/error-codes.md) | Numeric error reference with causes and remediation |
| [Events](docs/events.md) | Emitted events and payloads |
| [Governance](docs/governance.md) | Proposals, voting, delegation, timelock |
| [Multi-Token Support](docs/multi-token.md) | USDC, XLM, and token configuration |
| [Reputation](docs/reputation.md) | Reputation system overview |
| [Reputation Model](docs/reputation-model.md) | Scoring formulas and decay |
| [Oracle Design](docs/oracle-design.md) | Optional payer-verification oracle |
| [Oracle Integration](docs/oracle-integration.md) | Deploy and register a compatible oracle |
| [Benchmarks](docs/benchmarks.md) | Gas / resource usage benchmarks |

---

## Contributing

We welcome contributions — bug fixes, tests, documentation, and new features.

1. Read [CONTRIBUTING.md](CONTRIBUTING.md) for environment setup, testing expectations, and PR requirements.
2. Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `test:`, etc.).
3. Open a PR against `main` with a clear description and linked issue (e.g. `Fixes #107`).

For documentation-only changes:

```bash
git checkout -b docs/readme
# edit README.md
git commit -m "docs: write comprehensive README for ILN-Smart-Contract repo"
```

---

## License

This project is licensed under the [MIT License](LICENSE).
