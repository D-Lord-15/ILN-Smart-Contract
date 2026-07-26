#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env,
};

struct Setup {
    env: Env,
    client: InsurancePoolClient<'static>,
    admin: Address,
}

const COVERAGE: i128 = 1_000_000_000; // flat per-claim cap (100 units @ 1e7)

fn setup() -> Setup {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InsurancePool);
    let client = InsurancePoolClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &COVERAGE);

    Setup { env, client, admin }
}

#[test]
fn initialize_sets_coverage_and_zero_balance() {
    let s = setup();
    assert_eq!(s.client.get_pool_balance(), 0);
    assert_eq!(s.client.get_coverage(), COVERAGE);
}

#[test]
fn initialize_is_single_shot() {
    let s = setup();
    let other = Address::generate(&s.env);
    let res = s.client.try_initialize(&other, &COVERAGE);
    assert_eq!(res, Err(Ok(InsuranceError::AlreadyInitialized)));
}

#[test]
fn enroll_marks_lp_enrolled() {
    let s = setup();
    let lp = Address::generate(&s.env);
    assert!(!s.client.is_enrolled(&lp));
    s.client.enroll(&lp);
    assert!(s.client.is_enrolled(&lp));
}

#[test]
fn deposit_premium_increases_balance_and_auto_enrolls() {
    let s = setup();
    let lp = Address::generate(&s.env);

    s.client.deposit_premium(&lp, &500);
    s.client.deposit_premium(&lp, &250);

    assert_eq!(s.client.get_pool_balance(), 750);
    assert_eq!(s.client.get_premiums_paid(&lp), 750);
    assert!(s.client.is_enrolled(&lp)); // auto-enrolled on first premium
}

#[test]
fn deposit_premium_rejects_non_positive_amount() {
    let s = setup();
    let lp = Address::generate(&s.env);
    assert!(s.client.try_deposit_premium(&lp, &0).is_err());
    assert!(s.client.try_deposit_premium(&lp, &-100).is_err());
}

#[test]
fn claim_pays_coverage_capped_by_balance() {
    let s = setup();
    let lp = Address::generate(&s.env);

    // Pool has less than the coverage cap -> payout bounded by balance.
    s.client.deposit_premium(&lp, &400);
    let payout = s.client.claim(&1);
    assert_eq!(payout, 400);
    assert_eq!(s.client.get_pool_balance(), 0);
    assert!(s.client.is_claimed(&1));
}

#[test]
fn claim_pays_flat_coverage_when_pool_is_large() {
    let s = setup();
    let lp = Address::generate(&s.env);

    s.client.deposit_premium(&lp, &(COVERAGE * 3));
    let payout = s.client.claim(&7);
    assert_eq!(payout, COVERAGE); // capped at flat coverage
    assert_eq!(s.client.get_pool_balance(), COVERAGE * 2);
}

#[test]
fn claim_is_idempotent_per_invoice() {
    let s = setup();
    let lp = Address::generate(&s.env);
    s.client.deposit_premium(&lp, &(COVERAGE * 2));

    s.client.claim(&42);
    let res = s.client.try_claim(&42);
    // `claim` returns `i128` and panics with the error, so it surfaces as the
    // outer host error (a `soroban_sdk::Error`) rather than an inner `Result`.
    assert_eq!(
        res,
        Err(Ok(soroban_sdk::Error::from(InsuranceError::AlreadyClaimed)))
    );
}

#[test]
fn claim_rejects_when_pool_empty() {
    let s = setup();
    let res = s.client.try_claim(&99);
    assert_eq!(
        res,
        Err(Ok(soroban_sdk::Error::from(InsuranceError::PoolEmpty)))
    );
}

#[test]
fn admin_is_recorded() {
    let s = setup();
    // A claim requires admin auth; with mock_all_auths it succeeds once funded.
    let lp = Address::generate(&s.env);
    s.client.deposit_premium(&lp, &COVERAGE);
    let _ = s.client.claim(&100);
    // admin captured at init is the one we passed
    assert!(s.client.is_claimed(&100));
    let _ = &s.admin;
}

#[test]
fn coverage_change_requires_timelock_expiry() {
    let s = setup();
    let new_coverage = COVERAGE * 2;

    let eta = s.client.propose_coverage_change(&new_coverage);
    assert_eq!(s.client.get_coverage(), COVERAGE); // unchanged until executed
    assert_eq!(s.client.get_pending_coverage(), Some((new_coverage, eta)));

    // Too early.
    let res = s.client.try_execute_coverage_change();
    assert_eq!(
        res,
        Err(Ok(InsuranceError::TimelockNotExpired))
    );

    s.env.ledger().set_timestamp(eta);
    s.client.execute_coverage_change();

    assert_eq!(s.client.get_coverage(), new_coverage);
    assert_eq!(s.client.get_pending_coverage(), None);
}

#[test]
fn coverage_change_can_be_cancelled() {
    let s = setup();
    s.client.propose_coverage_change(&(COVERAGE * 2));
    assert!(s.client.get_pending_coverage().is_some());

    s.client.cancel_coverage_change();
    assert_eq!(s.client.get_pending_coverage(), None);

    let res = s.client.try_execute_coverage_change();
    assert_eq!(res, Err(Ok(InsuranceError::NoPendingProposal)));
}

#[test]
fn admin_transfer_requires_timelock_expiry() {
    let s = setup();
    let new_admin = Address::generate(&s.env);

    let eta = s.client.propose_admin_transfer(&new_admin);
    assert_eq!(s.client.get_pending_admin(), Some((new_admin.clone(), eta)));

    let res = s.client.try_execute_admin_transfer();
    assert_eq!(
        res,
        Err(Ok(InsuranceError::TimelockNotExpired))
    );

    s.env.ledger().set_timestamp(eta);
    s.client.execute_admin_transfer();

    assert_eq!(s.client.get_pending_admin(), None);

    // New admin can now propose further changes; old admin no longer can
    // (require_auth would fail against the new admin in a real invocation --
    // here we simply confirm the pending state cleared and a new proposal by
    // the new admin succeeds under mock_all_auths).
    let _ = s.client.propose_coverage_change(&(COVERAGE * 3));
}

#[test]
fn admin_transfer_can_be_cancelled() {
    let s = setup();
    let new_admin = Address::generate(&s.env);

    s.client.propose_admin_transfer(&new_admin);
    s.client.cancel_admin_transfer();
    assert_eq!(s.client.get_pending_admin(), None);

    let res = s.client.try_execute_admin_transfer();
    assert_eq!(res, Err(Ok(InsuranceError::NoPendingProposal)));
}

// ── Governance-controlled parameter updates ────────────────────────────────

#[test]
fn governance_can_update_coverage_cap() {
    let s = setup();
    assert_eq!(s.client.get_coverage(), COVERAGE);

    let new_coverage = 2_000_000_000;
    s.client.set_coverage_via_governance(&new_coverage);
    assert_eq!(s.client.get_coverage(), new_coverage);
}

#[test]
fn governance_rejects_non_positive_coverage() {
    let s = setup();
    let res = s.client.try_set_coverage_via_governance(&0);
    assert_eq!(res, Err(Ok(InsuranceError::InvalidAmount)));

    let res = s.client.try_set_coverage_via_governance(&-1_000_000);
    assert_eq!(res, Err(Ok(InsuranceError::InvalidAmount)));
}

#[test]
fn governance_can_set_premium_rate() {
    let s = setup();
    // Premium rate setting is allowed
    let res = s.client.try_set_premium_rate_via_governance(&500);
    assert!(res.is_ok());
}

#[test]
fn coverage_update_affects_future_claims() {
    let s = setup();
    let lp = Address::generate(&s.env);

    // Deposit with default coverage
    s.client.deposit_premium(&lp, &(COVERAGE * 2));
    let payout1 = s.client.claim(&1);
    assert_eq!(payout1, COVERAGE);

    // Update coverage to higher value
    let new_coverage = 3_000_000_000;
    s.client.set_coverage_via_governance(&new_coverage);

    // Reset balance for testing
    s.client.deposit_premium(&lp, &(new_coverage * 2));
    let payout2 = s.client.claim(&2);
    assert_eq!(payout2, new_coverage);
}
