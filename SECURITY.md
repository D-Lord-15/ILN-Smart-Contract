# Security Policy

## Supported Versions

Currently, the Invoice Liquidity Network (ILN) is in experimental/testnet phase. There are no formally supported "secure" versions for mainnet use yet. 

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Responsible Disclosure

If you discover a potential vulnerability in the ILN smart contracts, SDK, or indexer, please do not disclose it publicly.

Please report any security issues to **security@invoice-liquidity-network.local** (or open a confidential GitHub Security Advisory in the repository). 
We aim to acknowledge receipt of vulnerability reports within 48 hours and provide a timeline for resolution.

## Indexer Security Architecture

The ILN Indexer is designed to be **read-only from the network**. It strictly polls the Stellar/Soroban blockchain for state changes and persists them locally. 

By design, **the indexer does not expose any authenticated or unauthenticated write/state-change API routes** (`POST`, `PUT`, `DELETE`). All state changes must occur on-chain via the Soroban smart contract. Therefore, API keys or JWTs are not required for the indexer API.

The public read API is protected with request throttling by default. Clients that need higher throughput can send an `X-API-Key` header that matches one of the comma-separated values in `API_KEYS`; authenticated clients bypass the public rate limit.

## CodeQL Security Scanning

We use GitHub CodeQL to automatically scan our TypeScript/JavaScript packages for known vulnerabilities.
CodeQL findings appear in the **Security** tab of the repository.

To triage a finding:
1. Review the alert details in the Security tab.
2. Determine if it is a false positive or a legitimate issue.
3. If valid, open a PR to patch the issue.
4. If a false positive, dismiss the alert with a clear justification.
