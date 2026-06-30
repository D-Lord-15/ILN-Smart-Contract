# ILN Protocol Glossary

Definitions for DeFi, invoice-factoring, and Stellar-specific terms used across ILN documentation and code. Terms are listed alphabetically.

---

## Basis Points (bps)

A basis point is one hundredth of a percent (1 bps = 0.01%). ILN stores discount rates, protocol fees, and yield figures as integer basis points to avoid floating-point arithmetic in Soroban contracts. See [ADR-003: Discount Rate in Basis Points](adr/ADR-003-discount-rate-basis-points.md) and [`sdk/src/utils/validate.ts`](../sdk/src/utils/validate.ts).

## Circuit Breaker

A resilience pattern that stops repeated calls to a failing dependency after a threshold of consecutive errors, then probes recovery after a cooldown. ILN's notification service uses a per-endpoint circuit breaker for webhook delivery. See [notifications README](../notifications/README.md) and [`notifications/src/delivery/circuitBreaker.ts`](../notifications/src/delivery/circuitBreaker.ts).

## Discount Rate

The percentage of an invoice's face value that the liquidity provider retains as compensation for early funding, expressed in basis points (e.g. 300 bps = 3%). The freelancer receives `amount − discount` at funding time; the discount portion is held in escrow until settlement. See [Architecture — fund_invoice()](Architecture.md#step-2--fund_invoice) and [ADR-003](adr/ADR-003-discount-rate-basis-points.md).

## Effective Yield

The annualised return an LP earns on a funded invoice, derived from the discount rate and days until the invoice due date (`effectiveYieldBps = discountRate × daysToMaturity / 365`). See [`sdk/src/methods/fundInvoice.ts`](../sdk/src/methods/fundInvoice.ts) (`computeEffectiveYieldBps`).

## HMAC

Hash-based Message Authentication Code — a cryptographic signature that proves a message was created by someone who knows a shared secret. ILN webhook notifications sign payloads with HMAC-SHA256 in the `x-iln-signature` header. See [notifications README](../notifications/README.md) and [`tests/e2e/webhookDelivery.test.ts`](../tests/e2e/webhookDelivery.test.ts).

## Horizon

Stellar's legacy REST API for querying accounts, transactions, and ledger history on the classic network. ILN integrators primarily use [Stellar RPC](https://developers.stellar.org/docs/data/rpc) for Soroban contract calls; Horizon remains useful for account balances and trustline setup. See [Developer Quickstart](developer-quickstart.md).

## Invoice Factoring

A financing arrangement where a business sells its unpaid invoices to a third party at a discount in exchange for immediate cash. ILN automates this on-chain: a freelancer (submitter) registers an invoice, an LP funds it early, and the payer settles the full amount later. See [Architecture](Architecture.md).

## Ledger

A closed batch of Stellar network transactions. Ledger sequence numbers and timestamps drive ILN logic such as reputation decay periods and default eligibility after the due date. See [Reputation](reputation.md) and [`claim_default()` in Architecture](Architecture.md#step-3b--claim_default--unhappy-path).

## Liquidity Provider (LP)

A funder who supplies capital to pay out a pending invoice at a discount, earning yield when the payer settles. LPs call `fund_invoice` and may call `claim_default` if the payer fails to pay by the due date. See [Architecture — System actors](Architecture.md#system-actors).

## Payer

The client or debtor who owes the invoice amount and settles it on-chain by calling `mark_paid`. Only the registered payer address may trigger settlement for a given invoice. See [Architecture — mark_paid()](Architecture.md#step-3a--mark_paid--happy-path).

## Quorum

The minimum fraction of governance-token supply that must participate in a vote before a proposal can pass. ILN governance defaults to 10% quorum (`min_quorum_bps = 1_000`). See [Governance — Quorum and majority rules](governance.md#6-quorum-and-majority-rules).

## Reputation Score

An on-chain creditworthiness rating (0–100, default 50) for payers and LPs based on payment history, defaults, and activity. LPs use payer reputation to assess funding risk. See [Reputation](reputation.md) and [Reputation Model](reputation-model.md).

## Settlement

The on-chain payment that closes a funded invoice — typically the payer transferring the full invoice amount via `mark_paid`, after which the LP receives principal plus escrowed discount (yield). See [Architecture — mark_paid()](Architecture.md#step-3a--mark_paid--happy-path) and [First Invoice Tutorial](tutorials/first-invoice.md).

## Soroban

Stellar's smart-contract platform; ILN contracts compile to Soroban WASM and run on the Stellar ledger with deterministic, metered execution. See [ADR-001: Soroban / Stellar Choice](adr/ADR-001-soroban-chain-choice.md) and [Developer Quickstart](developer-quickstart.md).

## Stellar Asset Contract (SAC)

A Soroban contract that wraps a classic Stellar asset (e.g. USDC) so it can be transferred in smart-contract calls. ILN invoices are denominated in SAC-backed tokens such as testnet USDC. See [Multi-Token Support](multi-token.md) and [SDK Integration Guide](sdk-integration.md).

## Submitter

The freelancer or SME who registers an unpaid invoice on-chain by calling `submit_invoice`. Also referred to as the **freelancer** in architecture docs. See [Architecture — submit_invoice()](Architecture.md#step-1--submit_invoice) and [First Invoice Tutorial](tutorials/first-invoice.md).

## Timelock

A mandatory delay between a governance proposal passing and its on-chain execution, giving the community time to react. ILN v1 has **no timelock**; the admin veto serves as an emergency brake instead. See [ADR-005: Governance Timelock](adr/ADR-005-governance-timelock.md) and [Governance](governance.md).

## Trustline

A Stellar account permission that allows holding a specific classic asset (e.g. USDC). Accounts must establish trustlines before receiving SAC-backed tokens used in ILN invoices. See [First Invoice Tutorial — USDC trustline](tutorials/first-invoice.md#1c-add-a-usdc-trustline-and-mint-tokens).

## XDR

External Data Representation — Stellar's binary encoding format for transactions, contract arguments, and RPC payloads. SDK clients serialise transactions to XDR before signing and submission. See [SDK Integration Guide](sdk-integration.md).

## Yield

The return a liquidity provider earns when an invoice is settled — equal to the discount amount escrowed at funding time. For annualised comparisons, see **Effective Yield**. See [Architecture — mark_paid() money flow](Architecture.md#step-3a--mark_paid--happy-path).
