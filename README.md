# ILN Smart Contract

Soroban smart contracts for the Invoice Liquidity Network — a two-sided protocol on Stellar that connects invoice holders with liquidity providers.

## Contracts

| Contract | Description |
|----------|-------------|
| `invoice_liquidity` | Core escrow: submit, fund, and settle invoices |
| `iln_governance` | On-chain governance: proposals, voting, delegation |
| `iln_distribution` | Yield distribution for LPs and freelancers |
| `reputation_bonus` | Reputation-based discount bonuses |

## Documentation

| Doc | Description |
|-----|-------------|
| [SDK Integration Guide](docs/sdk-integration.md) | TypeScript examples for every contract interaction |
| [Architecture](docs/Architecture.md) | System design, money flow, and security model |
| [Contract ABI](docs/contract-abi.md) | Function signatures and error codes |
| [Events](docs/events.md) | All emitted events and their payloads |
| [Governance](docs/governance.md) | Proposal lifecycle and voting mechanics |
| [Storage Layout](docs/storage-layout.md) | On-chain storage key reference |
| [Threat Model](docs/threat-model.md) | Security assumptions and known risks |

## Quickstart

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
# Build all contracts
make build

# Run tests
make test

# Run fuzz suite (directly via cargo)
cargo test -p iln_fuzz
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full test and contribution guide.

## Testnet

Contract ID: `CD3TE3IAHM737P236XZL2OYU275ZKD6MN7YH7PYYAXYIGEH55OPEWYJC`

See the [SDK Integration Guide](docs/sdk-integration.md) for TypeScript examples tested against this deployment.
