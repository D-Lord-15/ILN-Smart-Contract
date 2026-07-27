# Contract ABI Documentation

## `invoice_liquidity` — Standalone Functions

| Function | Parameters | Returns | Access | Description |
|----------|------------|---------|--------|-------------|
| `initialize` | `env: Env, token: Address` | `Result<(), ContractError>` | Deployer (once) | Initialize the contract with a primary token address |
| `payer_score` | `env: Env, payer: Address` | `u32` | Anyone | Query a payer's current reputation score |
| `suggested_discount_rate` | `env: Env, payer: Address` | `u32` | Anyone | Compute suggested discount rate based on payer reputation |

---

## `insurance_pool` — Standalone Functions (`InsurancePool`)

| Function | Parameters | Returns | Access | Description |
|----------|------------|---------|--------|-------------|
| `initialize` | `env: Env, admin: Address, coverage: i128` | `Result<(), InsuranceError>` | Deployer (once) | Initialize pool with admin (the liquidity contract) and flat per-claim coverage cap. Fails with `AlreadyInitialized` if called twice; `InvalidAmount` if coverage ≤ 0 |
| `get_premiums_paid` | `env: Env, lp: Address` | `i128` | Anyone | Total premium an LP has contributed over the pool's lifetime |
| `get_coverage` | `env: Env` | `i128` | Anyone | The configured flat per-claim coverage cap |
| `is_claimed` | `env: Env, invoice_id: u64` | `bool` | Anyone | Whether a claim has already been processed for an invoice |
| `propose_coverage_change` | `env: Env, new_coverage: i128` | `Result<u64, InsuranceError>` | Admin | Queue a new coverage cap behind a 3-day timelock. Returns the eta (unix timestamp). `InvalidAmount` if ≤ 0; overwrites any prior pending proposal |
| `execute_coverage_change` | `env: Env` | `Result<(), InsuranceError>` | Anyone (after timelock) | Apply the pending coverage change once the timelock has expired. `NoPendingProposal` if none queued; `TimelockNotExpired` if too early |
| `cancel_coverage_change` | `env: Env` | `Result<(), InsuranceError>` | Admin | Cancel a pending coverage change before it executes. `NoPendingProposal` if none |
| `propose_admin_transfer` | `env: Env, new_admin: Address` | `Result<u64, InsuranceError>` | Admin | Queue a new admin address behind a 3-day timelock. Returns the eta. Overwrites any prior pending proposal |
| `execute_admin_transfer` | `env: Env` | `Result<(), InsuranceError>` | Anyone (after timelock) | Apply the pending admin transfer once the timelock has expired. `NoPendingProposal` if none queued; `TimelockNotExpired` if too early |
| `cancel_admin_transfer` | `env: Env` | `Result<(), InsuranceError>` | Admin | Cancel a pending admin transfer before it executes. `NoPendingProposal` if none |
| `get_pending_coverage` | `env: Env` | `Option<(i128, u64)>` | Anyone | Returns `(new_coverage, eta)` for the pending coverage proposal, or `None` |
| `get_pending_admin` | `env: Env` | `Option<(Address, u64)>` | Anyone | Returns `(new_admin, eta)` for the pending admin transfer, or `None` |

---

## `insurance_pool` — Trait-Implemented Functions (`InsurancePoolInterface`)

| Function | Parameters | Returns | Access | Description |
|----------|------------|---------|--------|-------------|
| `enroll` | `env: Env, lp: Address` | `()` | LP (`lp.require_auth()`) | Enroll LP in insurance program so future defaults on invoices they fund become eligible for compensation |
| `is_enrolled` | `env: Env, lp: Address` | `bool` | Anyone | Check LP enrollment status |
| `deposit_premium` | `env: Env, lp: Address, amount: i128` | `()` | LP (`lp.require_auth()`) | Record premium payment. Auto-enrolls LP if not already enrolled. `InvalidAmount` if ≤ 0 |
| `claim` | `env: Env, invoice_id: u64` | `i128` | Admin (`require_admin`) | File claim for a defaulted invoice. Returns the compensation amount (flat coverage cap, bounded by available balance). `AlreadyClaimed` if invoice already claimed; `PoolEmpty` if no funds |
| `get_pool_balance` | `env: Env` | `i128` | Anyone | Current total balance held by the pool (sum of premiums minus payouts) |

---

## Contract Errors

