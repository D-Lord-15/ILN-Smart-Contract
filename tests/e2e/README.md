# E2E Tests

End-to-end tests for the Invoice Liquidity Network.

## Running Tests

```bash
# Run all E2E tests
npm run test:e2e

# Run with Allure reporting
npm run test:e2e:allure

# Watch mode
npm run test:e2e:watch
```

## Allure Reporting

Tests generate Allure reports for visual test analysis.

### Generate HTML report

```bash
npm run allure:generate
npm run allure:open
```

### CI Integration

The `e2e-allure.yml` workflow automatically:
- Runs all E2E tests with Allure reporter
- Generates HTML report
- Uploads report as GitHub Actions artifact
- Adds test summary to PR step summary

### Viewing reports locally

1. Run tests: `npm run test:e2e:allure`
2. Generate report: `npm run allure:generate`
3. Open report: `npm run allure:open`

## Test Files

| File | Description |
|------|-------------|
| `lifecycle.test.ts` | Full invoice lifecycle (submit → fund → pay) |
| `multiToken.test.ts` | Multi-token invoice scenarios |
| `reputation.test.ts` | Payer reputation scoring |
| `cli.test.ts` | CLI command integration |
| `smoke.test.ts` | Quick smoke test |
| `indexerConsistency.test.ts` | Indexer data consistency |
| `webhookDelivery.test.ts` | Webhook notification delivery |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SOROBAN_RPC_URL` | Soroban RPC endpoint | `https://soroban-testnet.stellar.org` |
| `CONTRACT_ID` | Invoice Liquidity contract ID | (required) |
| `ILN_GOVERNANCE_ID` | Governance contract ID | (optional) |
| `NETWORK_PASSPHRASE` | Stellar network passphrase | Testnet |
