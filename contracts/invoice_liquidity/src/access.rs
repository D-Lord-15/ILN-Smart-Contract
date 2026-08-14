use crate::errors::ContractError;
use crate::invoice::{get_invoice_funders, invoice_exists, load_invoice, StorageKey};
use soroban_sdk::{Address, Env, Symbol};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Submitter,
    Payer,
    LP,
    Admin,
    Governance,
    Anyone,
}

pub fn require_admin(env: &Env) -> Result<(), ContractError> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&StorageKey::Admin)
        .ok_or(ContractError::Unauthorized)?;
    admin.require_auth();
    Ok(())
}

pub fn require_submitter(_env: &Env, caller: &Address) -> Result<(), ContractError> {
    caller.require_auth();
    Ok(())
}

pub fn require_submitter_by_id(
    env: &Env,
    caller: &Address,
    invoice_id: u64,
) -> Result<(), ContractError> {
    if !invoice_exists(env, invoice_id) {
        return Err(ContractError::InvoiceNotFound);
    }
    let invoice = load_invoice(env, invoice_id);
    caller.require_auth();
    if caller != &invoice.freelancer {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

pub fn require_payer_by_id(env: &Env, invoice_id: u64) -> Result<(), ContractError> {
    if !invoice_exists(env, invoice_id) {
        return Err(ContractError::InvoiceNotFound);
    }
    let invoice = load_invoice(env, invoice_id);
    invoice.payer.require_auth();
    Ok(())
}

pub fn require_lp(_env: &Env, caller: &Address) -> Result<(), ContractError> {
    caller.require_auth();
    Ok(())
}

pub fn require_lp_by_id(env: &Env, caller: &Address, invoice_id: u64) -> Result<(), ContractError> {
    if !invoice_exists(env, invoice_id) {
        return Err(ContractError::InvoiceNotFound);
    }
    caller.require_auth();

    let funders = get_invoice_funders(env, invoice_id);
    let mut is_funder = false;
    for i in 0..funders.len() {
        if funders.get(i).unwrap().0 == *caller {
            is_funder = true;
            break;
        }
    }
    if !is_funder {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

pub fn require_governance(_env: &Env) -> Result<(), ContractError> {
    // Currently no governance implemented, always reject
    Err(ContractError::Unauthorized)
}

// ----------------------------------------------------------------
// Reentrancy Guard (Issue #535)
// ----------------------------------------------------------------
// CEI (Checks-Effects-Interactions) pattern enforcement:
//   All state-changing functions MUST perform state mutations BEFORE
//   any external calls (token transfers, cross-contract invocations).
//   The guards below provide defense-in-depth for critical paths.

/// Activate the reentrancy lock. Returns `Reentrancy` error if already locked.
pub fn lock_reentrancy(env: &Env) -> Result<(), ContractError> {
    let locked: bool = env
        .storage()
        .instance()
        .get(&StorageKey::ReentrancyLock)
        .unwrap_or(false);
    if locked {
        return Err(ContractError::Reentrancy);
    }
    env.storage()
        .instance()
        .set(&StorageKey::ReentrancyLock, &true);
    Ok(())
}

/// Deactivate the reentrancy lock.
pub fn unlock_reentrancy(env: &Env) {
    env.storage()
        .instance()
        .set(&StorageKey::ReentrancyLock, &false);
}

// ----------------------------------------------------------------
// Rate Limiting (Issue #541)
// ----------------------------------------------------------------
//
// Design:
//   Each rate-limited function is keyed by a Symbol (its function name).
//   On each call, we check whether enough ledgers have elapsed since the
//   last recorded call. If not, the call is rejected with RateLimited.
//
//   Cooldown defaults are set per function category:
//     - Admin transfer: ADMIN_CHANGE_COOLDOWN_LEDGERS (720 ledgers ≈ 1h)
//     - Contract upgrade: UPGRADE_COOLDOWN_LEDGERS (1440 ledgers ≈ 2h)
//     - Economic params: ECONOMIC_PARAM_COOLDOWN_LEDGERS (360 ledgers ≈ 30min)
//     - General: DEFAULT_RATE_LIMIT_LEDGERS (120 ledgers ≈ 10min)
//
//   Emergency functions (pause/unpause) are deliberately exempt.

/// Check whether the given rate-limited function may be called.
/// Returns `RateLimited` if the cooldown has not yet elapsed.
/// Otherwise records the current ledger as the last call time.
pub fn check_rate_limit(
    env: &Env,
    fn_name: &str,
    cooldown_ledgers: u64,
) -> Result<(), ContractError> {
    let key = StorageKey::RateLimit(Symbol::new(env, fn_name));
    let last_ledger: u32 = env.storage().instance().get(&key).unwrap_or(0);
    let current_ledger = env.ledger().sequence();

    if current_ledger < last_ledger.saturating_add(cooldown_ledgers as u32) {
        return Err(ContractError::RateLimited);
    }

    env.storage().instance().set(&key, &current_ledger);
    Ok(())
}

/// Clear a rate-limit record (for testing or emergency bypass by admin).
pub fn clear_rate_limit(env: &Env, fn_name: &str) {
    let key = StorageKey::RateLimit(Symbol::new(env, fn_name));
    env.storage().instance().remove(&key);
}
