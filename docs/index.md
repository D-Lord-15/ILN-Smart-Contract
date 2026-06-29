# ILN Documentation

Central index for Invoice Liquidity Network (ILN) smart-contract documentation.

---

## Getting started

| Document | Description |
|----------|-------------|
| [Developer Quickstart](developer-quickstart.md) | Toolchain, build, test, and testnet deploy |
| [Local Development Guide](local-development.md) | Docker Stellar node, scripts, and workflow |
| [First Invoice Tutorial](tutorials/first-invoice.md) | Hands-on walkthrough on testnet |
| [SDK Integration Guide](sdk-integration.md) | TypeScript / Stellar SDK examples |
| [SDK Usage Guide](../sdk/README.md) | Complete `@iln/sdk` package reference |

## Reference

| Document | Description |
|----------|-------------|
| [Protocol Glossary](glossary.md) | Definitions for invoice factoring, basis points, yield, and Stellar terms |
| [Contract ABI](contract-abi.md) | Public functions and error codes |
| [Error Codes](error-codes.md) | Numeric error reference with remediation |
| [Events](events.md) | Emitted events and payloads |
| [Storage Layout](storage-layout.md) | On-chain keys and data structures |

## Architecture & security

| Document | Description |
|----------|-------------|
| [Architecture](Architecture.md) | Actors, money flow, state machine, deployment |
| [Threat Model](threat-model.md) | Security assumptions, risks, and mitigations |
| [Access Control](access-control.md) | Roles, auth requirements, and admin functions |
| [Upgrade Guide](upgrade-guide.md) | Contract upgrade process and safeguards |
| [Architecture Decision Records](adr/README.md) | ADR index |

## Protocol features

| Document | Description |
|----------|-------------|
| [Governance](governance.md) | Proposals, voting, delegation, timelock |
| [Multi-Token Support](multi-token.md) | USDC, XLM, and token configuration |
| [Reputation](reputation.md) | Reputation system overview |
| [Reputation Model](reputation-model.md) | Scoring formulas and decay |
| [Oracle Design](oracle-design.md) | Optional payer-verification oracle |
| [Oracle Integration](oracle-integration.md) | Deploy and register a compatible oracle |
| [Benchmarks](benchmarks.md) | Gas / resource usage benchmarks |
