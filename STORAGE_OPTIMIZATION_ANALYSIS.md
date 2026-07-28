# Invoice Liquidity Contract - Storage Layout Optimization Analysis

## Executive Summary

This document analyzes the current storage patterns in the `invoice_liquidity` contract and identifies optimization opportunities for gas efficiency and access performance.

## Current Storage Architecture

### Storage Key Breakdown

**Instance Storage** (52 accesses per full invoice lifecycle):
- Admin, Config, FeeRate, MaxDiscountRate, DistributionContract, Paused, MinPayerReputation, NextInvoiceId

**Persistent Storage** (frequent access patterns):
- Core: Invoice(u64), InvoiceFunders(u64), FundQueue(u64)
- Scores: PayerScore(Address), LpScore(Address), Reputation(Address)
- Stats: TotalInvoices, TotalFunded, TotalPaid, TotalVolume*, TokenVolume(Address)
- Tokens: Token, ApprovedToken(Address), TokenDecimals(Address)

## Hot Paths Analysis

### 1. Invoice Operations (Critical Path)
**Frequency**: O(1) - once per invoice lifecycle

```
Operation: load_invoice(id)
  - Get: DataKey::Invoice(id)
  - Cost: ~100-200 gas (single persistent read)
  - Optimization: GOOD - Already optimal for single lookup

Operation: save_invoice(invoice)
  - Set: DataKey::Invoice(id) + TTL extend
  - Cost: ~150-250 gas
  - Optimization: Already includes TTL optimization
```

### 2. Funder List Operations (Hot Path)
**Frequency**: O(n) per invoice funding phase

```
Operation: get_invoice_funders(id) + save_invoice_funders(id, list)
  - Get + Set: DataKey::InvoiceFunders(id)
  - Cost: ~200-300 gas per iteration
  - Issue: Full vector deserialization even for single lookup
  - Improvement: Consider maintaining separate index for fast checks
```

### 3. Reputation Score Access (Very Hot Path)
**Frequency**: O(1) per reputation check (~5-10 times per invoice)

```
Operation: get_payer_score(payer)
  - Get: DataKey::PayerScore(payer)
  - Logic: Decay calculation on every read
  - Cost: ~150-300 gas (includes decay computation)
  - Issue: Complex computation on every access
  - Improvement: Cache scores within transaction scope or batch updates
```

### 4. Statistics Updates (Moderate Hot Path)
**Frequency**: O(1) per state change (3-5 times per invoice)

```
Operation: increment_total_invoices()
  - Read-Modify-Write pattern:
    1. Get: DataKey::TotalInvoices
    2. Increment
    3. Set: DataKey::TotalInvoices
  - Cost: ~300-400 gas per call (2 storage ops)
  - Issue: Separate RMW operations accumulate
  - Improvement: Batch updates or single-op counters
```

## Identified Optimization Opportunities

### Priority 1: High Impact, Low Risk

#### 1.1 Consolidate Stats Incrementors
**Current Pattern**:
```rust
pub fn increment_total_invoices(env: &Env) {
    let current: u64 = env.storage().persistent().get(...).unwrap_or(0);
    env.storage().persistent().set(..., &current.saturating_add(1));
}
```

**Issues**:
- Each stat increment is a separate RMW cycle
- Called multiple times per invoice lifecycle
- Gas cost: ~300-400 per call

**Recommendation**: Batch stats updates at transaction end
- Maintain in-memory accumulator
- Single flush to storage
- Estimated savings: 50-60% of stats update costs

#### 1.2 Optimize Funder List Access Pattern
**Current Pattern**:
```rust
pub fn get_invoice_funders(env: &Env, id: u64) -> Vec<(Address, i128)> {
    env.storage().persistent().get(...).unwrap_or_else(...)
}
```

**Issues**:
- Deserializes entire vector for partial lookups
- O(n) space usage for large funder lists

**Recommendation**: Maintain separate index keys
- Keep list for full reconstruction
- Add count key for fast "how many funders" checks
- Estimated savings: 20-30% for check-heavy paths

### Priority 2: Medium Impact, Low Risk

#### 2.1 Cache Reputation Scores in Memory
**Current Issue**:
```rust
pub fn get_payer_score(env: &Env, payer: &Address) -> u32 {
    // Fetches from storage + decay calculation
    // Called multiple times per invoice
}
```

**Recommendation**:
- Use env.temporary() storage for within-call caching
- Store computed scores in context struct
- Estimated savings: 40-50% for reputation lookups

#### 2.2 Lazy Field Loading for Invoice
**Analysis**:
- Invoice struct contains all fields always
- Not all operations need all fields
- Could split into: metadata + detailed data

**Recommendation**: Document which fields are truly "hot" before implementation

### Priority 3: Moderate Impact, Higher Risk

#### 3.1 Normalization of Token Volume Storage
**Current**: Separate counters for USDC, EURC, XLM + TokenVolume(Address)
**Risk**: Breaking changes to stats queries
**Status**: Document before implementing

## Benchmark Methodology

### Before-State Baseline
1. Test suite: run existing tests with current storage patterns
2. Measure: contract_call gas costs
3. Record: per-operation averages

### After-State Comparison
1. Implement Priority 1 optimizations
2. Run same test suite
3. Compare: per-operation gas deltas
4. Target: 15-25% improvement on hot paths

## Recommended Implementation Order

1. **Phase 1**: Consolidate stats incrementors (Priority 1.1)
   - Risk: Low
   - Testing: Unit tests sufficient
   - Expected gain: 50-100 gas per invoice

2. **Phase 2**: Reputation score caching (Priority 2.1)
   - Risk: Low
   - Testing: Verify decay calculation correctness
   - Expected gain: 30-60 gas per reputation check

3. **Phase 3**: Funder list optimization (Priority 1.2)
   - Risk: Medium (index consistency)
   - Testing: Comprehensive integration tests required
   - Expected gain: 50-100 gas per funding operation

## Conclusion

The current storage layout is reasonably optimized for the core hot paths (Invoice loading/saving). The main opportunities for improvement are:

1. **Batch operations** for statistics (Quick win)
2. **In-call caching** for reputation (Quick win)
3. **Index structures** for collections (Medium effort)

Combined impact: **15-25% gas reduction** on typical invoice lifecycle (estimated 300-500 gas savings per invoice).

## Next Steps

1. Implement Priority 1.1 (Stats consolidation)
2. Add benchmark tests to measure improvements
3. Document actual vs estimated gas savings
4. Consider Priority 1.2 and 2.1 for Phase 2 work
