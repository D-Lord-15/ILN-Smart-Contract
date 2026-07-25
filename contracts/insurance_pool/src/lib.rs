#![no_std]

//! Default-protection insurance pool — stub implementation (Issue #123).
//!
//! Liquidity providers (LPs) optionally opt into this pool by paying premiums.
//! When an invoice they funded defaults, the pool compensates them out of the
//! accumulated premium balance (up to a flat per-claim coverage cap).
//!
//! This is a **design-forward stub**: it implements the full
//! [`InsurancePoolInterface`] with correct storage, auth, events and accounting
//! semantics, but deliberately keeps the economics simple:
//!   * Premiums are tracked as pool *accounting* balance rather than via an
//!     actual token transfer (token settlement is a follow-up).
//!   * Compensation is a flat per-claim cap configured at init, not a
//!     risk-priced payout.
//!
//! See `docs/insurance-pool-design.md` for the integration design and the
//! follow-up work needed before mainnet.

mod insurance_interface;
#[cfg(test)]
mod test;

pub use insurance_interface::{InsurancePoolInterface, InsurancePoolInterfaceClient};

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env,
};

/// Timelock delay (in seconds) enforced between proposing and executing an
/// admin action, and before which a proposal may be cancelled (Issue #542).
pub const TIMELOCK_DELAY_SECONDS: u64 = 3 * 24 * 60 * 60; // 3 days

/// Errors surfaced by the insurance pool stub.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum InsuranceError {
    /// Contract has not been initialised with an admin.
    NotInitialized = 1,
    /// A claim has already been processed for this invoice.
    AlreadyClaimed = 2,
    /// Premium / coverage amount must be positive.
    InvalidAmount = 3,
    /// Pool has no balance available to pay a claim.
    PoolEmpty = 4,
    /// Contract is already initialised.
    AlreadyInitialized = 5,
    /// No pending proposal exists for the requested admin action.
    NoPendingProposal = 6,
    /// The proposal's timelock has not yet expired.
    TimelockNotExpired = 7,
}

/// Storage keys for the pool.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Admin authorised to report confirmed defaults (the liquidity contract).
    Admin,
    /// Total pool balance (sum of premiums minus payouts).
    Balance,
    /// Flat per-claim coverage cap configured at init.
    Coverage,
    /// Enrollment flag per LP.
    Enrolled(Address),
    /// Cumulative premium paid per LP.
    Premiums(Address),
    /// Whether a claim has been processed for a given invoice id.
    Claimed(u64),
    /// Proposed new coverage cap awaiting timelock expiry (Issue #542).
    PendingCoverage,
    /// Proposed new admin awaiting timelock expiry (Issue #542).
    PendingAdmin,
    /// Ledger timestamp at which the pending coverage change becomes executable.
    CoverageEta,
    /// Ledger timestamp at which the pending admin transfer becomes executable.
    AdminEta,
}

#[contract]
pub struct InsurancePool;

#[contractimpl]
impl InsurancePool {
    /// Initialise the pool.
    ///
    /// * `admin` — authorised to file claims (in production, the liquidity
    ///   contract address acting on a confirmed default).
    /// * `coverage` — flat per-claim compensation cap (in token stroops).
    pub fn initialize(env: Env, admin: Address, coverage: i128) -> Result<(), InsuranceError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(InsuranceError::AlreadyInitialized);
        }
        if coverage <= 0 {
            return Err(InsuranceError::InvalidAmount);
        }
        admin.require_auth();
        let storage = env.storage().instance();
        storage.set(&DataKey::Admin, &admin);
        storage.set(&DataKey::Balance, &0i128);
        storage.set(&DataKey::Coverage, &coverage);
        Ok(())
    }

    /// Total premium an LP has contributed over the pool's lifetime.
    pub fn get_premiums_paid(env: Env, lp: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Premiums(lp))
            .unwrap_or(0)
    }

    /// The configured flat per-claim coverage cap.
    pub fn get_coverage(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Coverage)
            .unwrap_or(0)
    }

    /// Returns `true` if a claim has already been processed for `invoice_id`.
    pub fn is_claimed(env: Env, invoice_id: u64) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Claimed(invoice_id))
            .unwrap_or(false)
    }

    // ── Issue #542: timelocked admin actions ────────────────────────────
    //
    // Coverage cap changes and admin transfers are sensitive, LP-affecting
    // parameters. Rather than applying immediately, they are queued behind a
    // `TIMELOCK_DELAY_SECONDS` delay so LPs have advance notice and a chance
    // to exit before the change takes effect. The current admin may cancel a
    // pending proposal at any time before it executes.

    /// Propose a new coverage cap. Requires current admin auth. Overwrites
    /// any previously pending coverage proposal.
    pub fn propose_coverage_change(env: Env, new_coverage: i128) -> Result<u64, InsuranceError> {
        Self::require_admin(&env);
        if new_coverage <= 0 {
            return Err(InsuranceError::InvalidAmount);
        }

        let eta = env
            .ledger()
            .timestamp()
            .saturating_add(TIMELOCK_DELAY_SECONDS);

        let storage = env.storage().instance();
        storage.set(&DataKey::PendingCoverage, &new_coverage);
        storage.set(&DataKey::CoverageEta, &eta);

        env.events().publish(
            (symbol_short!("cov_prop"),),
            (new_coverage, eta),
        );
        Ok(eta)
    }

    /// Execute a previously proposed coverage change once its timelock has
    /// expired. Callable by anyone once the delay has elapsed.
    pub fn execute_coverage_change(env: Env) -> Result<(), InsuranceError> {
        let storage = env.storage().instance();
        let new_coverage: i128 = storage
            .get(&DataKey::PendingCoverage)
            .ok_or(InsuranceError::NoPendingProposal)?;
        let eta: u64 = storage
            .get(&DataKey::CoverageEta)
            .ok_or(InsuranceError::NoPendingProposal)?;

        if env.ledger().timestamp() < eta {
            return Err(InsuranceError::TimelockNotExpired);
        }

        storage.set(&DataKey::Coverage, &new_coverage);
        storage.remove(&DataKey::PendingCoverage);
        storage.remove(&DataKey::CoverageEta);

        env.events()
            .publish((symbol_short!("cov_exec"),), new_coverage);
        Ok(())
    }

    /// Cancel a pending coverage change proposal. Requires current admin auth.
    pub fn cancel_coverage_change(env: Env) -> Result<(), InsuranceError> {
        Self::require_admin(&env);
        let storage = env.storage().instance();
        if !storage.has(&DataKey::PendingCoverage) {
            return Err(InsuranceError::NoPendingProposal);
        }
        storage.remove(&DataKey::PendingCoverage);
        storage.remove(&DataKey::CoverageEta);
        env.events().publish((symbol_short!("cov_cncl"),), ());
        Ok(())
    }

    /// Propose an admin transfer. Requires current admin auth. Overwrites any
    /// previously pending admin proposal.
    pub fn propose_admin_transfer(env: Env, new_admin: Address) -> Result<u64, InsuranceError> {
        Self::require_admin(&env);

        let eta = env
            .ledger()
            .timestamp()
            .saturating_add(TIMELOCK_DELAY_SECONDS);

        let storage = env.storage().instance();
        storage.set(&DataKey::PendingAdmin, &new_admin);
        storage.set(&DataKey::AdminEta, &eta);

        env.events()
            .publish((symbol_short!("adm_prop"), new_admin), eta);
        Ok(eta)
    }

    /// Execute a previously proposed admin transfer once its timelock has
    /// expired. Callable by anyone once the delay has elapsed.
    pub fn execute_admin_transfer(env: Env) -> Result<(), InsuranceError> {
        let storage = env.storage().instance();
        let new_admin: Address = storage
            .get(&DataKey::PendingAdmin)
            .ok_or(InsuranceError::NoPendingProposal)?;
        let eta: u64 = storage
            .get(&DataKey::AdminEta)
            .ok_or(InsuranceError::NoPendingProposal)?;

        if env.ledger().timestamp() < eta {
            return Err(InsuranceError::TimelockNotExpired);
        }

        storage.set(&DataKey::Admin, &new_admin);
        storage.remove(&DataKey::PendingAdmin);
        storage.remove(&DataKey::AdminEta);

        env.events()
            .publish((symbol_short!("adm_exec"),), new_admin);
        Ok(())
    }

    /// Cancel a pending admin transfer proposal. Requires current admin auth.
    pub fn cancel_admin_transfer(env: Env) -> Result<(), InsuranceError> {
        Self::require_admin(&env);
        let storage = env.storage().instance();
        if !storage.has(&DataKey::PendingAdmin) {
            return Err(InsuranceError::NoPendingProposal);
        }
        storage.remove(&DataKey::PendingAdmin);
        storage.remove(&DataKey::AdminEta);
        env.events().publish((symbol_short!("adm_cncl"),), ());
        Ok(())
    }

    /// Returns the pending coverage proposal (new cap, eta), if any.
    pub fn get_pending_coverage(env: Env) -> Option<(i128, u64)> {
        let storage = env.storage().instance();
        let new_coverage: i128 = storage.get(&DataKey::PendingCoverage)?;
        let eta: u64 = storage.get(&DataKey::CoverageEta)?;
        Some((new_coverage, eta))
    }

    /// Returns the pending admin transfer proposal (new admin, eta), if any.
    pub fn get_pending_admin(env: Env) -> Option<(Address, u64)> {
        let storage = env.storage().instance();
        let new_admin: Address = storage.get(&DataKey::PendingAdmin)?;
        let eta: u64 = storage.get(&DataKey::AdminEta)?;
        Some((new_admin, eta))
    }

    fn require_admin(env: &Env) -> Address {
        match env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
        {
            Some(admin) => {
                admin.require_auth();
                admin
            }
            None => panic_with_error!(env, InsuranceError::NotInitialized),
        }
    }
}

#[contractimpl]
impl InsurancePoolInterface for InsurancePool {
    fn enroll(env: Env, lp: Address) {
        lp.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::Enrolled(lp.clone()), &true);
        env.events().publish((symbol_short!("enrolled"), lp), ());
    }

    fn is_enrolled(env: Env, lp: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Enrolled(lp))
            .unwrap_or(false)
    }

    fn deposit_premium(env: Env, lp: Address, amount: i128) {
        lp.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, InsuranceError::InvalidAmount);
        }

        // Auto-enroll on first premium so a paying LP is always covered.
        env.storage()
            .persistent()
            .set(&DataKey::Enrolled(lp.clone()), &true);

        let prev_premium: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Premiums(lp.clone()))
            .unwrap_or(0);
        env.storage().persistent().set(
            &DataKey::Premiums(lp.clone()),
            &prev_premium.saturating_add(amount),
        );

        let balance: i128 = env.storage().instance().get(&DataKey::Balance).unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::Balance, &balance.saturating_add(amount));

        env.events().publish((symbol_short!("premium"), lp), amount);
    }

    fn claim(env: Env, invoice_id: u64) -> i128 {
        // Only the configured admin (the liquidity contract in production) may
        // report a confirmed default and trigger compensation.
        Self::require_admin(&env);

        if Self::is_claimed(env.clone(), invoice_id) {
            panic_with_error!(&env, InsuranceError::AlreadyClaimed);
        }

        let balance: i128 = env.storage().instance().get(&DataKey::Balance).unwrap_or(0);
        if balance <= 0 {
            panic_with_error!(&env, InsuranceError::PoolEmpty);
        }

        let coverage: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Coverage)
            .unwrap_or(0);
        // Stub payout: flat coverage cap, bounded by available balance.
        let payout = if coverage < balance {
            coverage
        } else {
            balance
        };

        env.storage()
            .instance()
            .set(&DataKey::Balance, &(balance - payout));
        env.storage()
            .persistent()
            .set(&DataKey::Claimed(invoice_id), &true);

        env.events()
            .publish((symbol_short!("claimed"), invoice_id), payout);
        payout
    }

    fn get_pool_balance(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::Balance).unwrap_or(0)
    }
}
