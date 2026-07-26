use soroban_sdk::{contracttype, Address, BytesN, Env, Symbol};

use crate::config::Config;
use crate::invoice::{AppealRecord, Invoice, LpFundRequest, ReputationScore};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataKey {
    // Instance Storage
    Admin,
    Config,
    FeeRate,
    MaxDiscountRate,
    DistributionContract,
    Paused,
    /// Minimum payer reputation required to fund an invoice (Issue #28). Default 0.
    MinPayerReputation,
    NextInvoiceId,

    // Persistent Storage
    Invoice(u64),
    InvoiceCount,
    Token,
    PayerScore(Address),
    InvoiceFunders(u64),
    ApprovedToken(Address),
    TokenList,
    /// Decimal precision for each allowlisted token (e.g. 6 for USDC, 7 for XLM).
    TokenDecimals(Address),
    /// Detailed reputation profile per address (Issue #26).
    Reputation(Address),
    Appeal(u64),
    PreDefaultPayerScore(u64),
    LpScore(Address),
    FundQueue(u64),
    QueueResolution(u64),

    // Stats (Persistent)
    TotalInvoices,
    TotalFunded,
    TotalPaid,
    TotalVolumeUsdc,
    TotalVolumeEurc,
    TotalVolumeXlm,
    TokenVolume(Address),
    /// Referral counts keyed by fixed-size code
    ReferralCount(BytesN<32>),
    Dispute(u64),
    SubmitterInvoices(Address),
    LpInvoices(Address),
    /// Fixed-size min-heap of the top payers by reputation score (Issue #77).
    TopPayersHeap,
    /// NFT Metadata storage (Issue #423)
    InvoiceNft(u64),
    /// NFT Owner tracking (Issue #423)
    InvoiceNftOwner(u64),
    /// Reentrancy guard lock (Issue #535)
    ReentrancyLock,
    /// Last ledger sequence when each rate-limited function was called (Issue #541).
    /// Keyed by a Symbol representing the function name.
    RateLimit(Symbol),
}

// ----------------------------------------------------------------
// Config Helpers
// ----------------------------------------------------------------

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_config(env: &Env) -> Option<Config> {
    env.storage().instance().get(&DataKey::Config)
}

pub fn set_config(env: &Env, config: &Config) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}

// ----------------------------------------------------------------
// Invoice Helpers
// ----------------------------------------------------------------

pub fn save_invoice(env: &Env, invoice: &Invoice) {
    let key = DataKey::Invoice(invoice.id);
    env.storage().persistent().set(&key, invoice);
    env.storage()
        .persistent()
        .extend_ttl(&key, 1_000_000, 2_000_000);
}

pub fn load_invoice(env: &Env, id: u64) -> Invoice {
    env.storage()
        .persistent()
        .get(&DataKey::Invoice(id))
        .expect("invoice not found")
}

pub fn invoice_exists(env: &Env, id: u64) -> bool {
    env.storage().persistent().has(&DataKey::Invoice(id))
}

pub fn read_next_invoice_id(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::NextInvoiceId)
        .unwrap_or(1)
}

pub fn write_next_invoice_id(env: &Env, id: u64) {
    env.storage().instance().set(&DataKey::NextInvoiceId, &id);
}

pub fn next_invoice_id(env: &Env) -> Result<u64, crate::errors::ContractError> {
    let current_id = read_next_invoice_id(env);
    let next_id = current_id
        .checked_add(1)
        .ok_or(crate::errors::ContractError::ArithmeticOverflow)?;

    write_next_invoice_id(env, next_id);

    Ok(current_id)
}

// ----------------------------------------------------------------
// Funder List Helpers
// ----------------------------------------------------------------

pub fn get_invoice_funders(env: &Env, id: u64) -> soroban_sdk::Vec<(Address, i128)> {
    env.storage()
        .persistent()
        .get(&DataKey::InvoiceFunders(id))
        .unwrap_or_else(|| soroban_sdk::Vec::new(env))
}

pub fn save_invoice_funders(env: &Env, id: u64, funders: &soroban_sdk::Vec<(Address, i128)>) {
    env.storage()
        .persistent()
        .set(&DataKey::InvoiceFunders(id), funders);
}

// ----------------------------------------------------------------
// Reputation Helpers
// ----------------------------------------------------------------

pub fn get_payer_score(env: &Env, payer: &Address) -> u32 {
    match env
        .storage()
        .persistent()
        .get::<DataKey, ReputationScore>(&DataKey::PayerScore(payer.clone()))
    {
        Some(mut rep) => {
            if let Some(decay_config) = get_config(env) {
                let current_ledger = env.ledger().sequence() as u64;
                let ledgers_since_activity =
                    current_ledger.saturating_sub(rep.last_activity_ledger.into());

                if ledgers_since_activity >= decay_config.decay_period_ledgers
                    && decay_config.decay_period_ledgers > 0
                    && decay_config.decay_rate_bps > 0
                {
                    let periods_passed = ledgers_since_activity / decay_config.decay_period_ledgers;
                    let mut decayed_score = rep.score as u64;
                    for _ in 0..periods_passed {
                        let decay_amount =
                            (decayed_score * decay_config.decay_rate_bps as u64) / 10_000;
                        decayed_score = decayed_score.saturating_sub(decay_amount);
                    }
                    rep.score = (decayed_score.min(100)) as u32;
                }
            }
            rep.score
        }
        None => 50,
    }
}

pub fn set_payer_score(env: &Env, payer: &Address, score: u32) {
    let score = score.min(100);
    // Note: To preserve `last_activity_ledger`, we should actually retrieve the old Rep or create a new one.
    // In `invoice.rs` the old function was `set_payer_score(env: &Env, payer: &Address, score: u32) { env.storage().persistent().set(..., &rep) }` which didn't compile correctly in the snippet I saw (`&rep` not defined). Let's fix that.
    let current_ledger = env.ledger().sequence() as u64;
    let rep = ReputationScore {
        score,
        last_activity_ledger: current_ledger as u32,
    };
    env.storage()
        .persistent()
        .set(&DataKey::PayerScore(payer.clone()), &rep);
}

pub fn get_lp_score(env: &Env, lp: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::LpScore(lp.clone()))
        .unwrap_or(50)
}

pub fn set_lp_score(env: &Env, lp: &Address, score: u32) {
    let score = score.min(100);
    env.storage()
        .persistent()
        .set(&DataKey::LpScore(lp.clone()), &score);
}

// ----------------------------------------------------------------
// LP Queue Helpers
// ----------------------------------------------------------------

pub fn get_fund_queue(env: &Env, invoice_id: u64) -> soroban_sdk::Vec<LpFundRequest> {
    env.storage()
        .persistent()
        .get(&DataKey::FundQueue(invoice_id))
        .unwrap_or_else(|| soroban_sdk::Vec::new(env))
}

pub fn save_fund_queue(env: &Env, invoice_id: u64, queue: &soroban_sdk::Vec<LpFundRequest>) {
    env.storage()
        .persistent()
        .set(&DataKey::FundQueue(invoice_id), queue);
}

pub fn get_queue_resolution(env: &Env, invoice_id: u64) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::QueueResolution(invoice_id))
}

pub fn save_queue_resolution(env: &Env, invoice_id: u64, approved_lp: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::QueueResolution(invoice_id), approved_lp);
}

// ----------------------------------------------------------------
// Appeal Helpers
// ----------------------------------------------------------------

pub fn get_appeal(env: &Env, invoice_id: u64) -> Option<AppealRecord> {
    env.storage().persistent().get(&DataKey::Appeal(invoice_id))
}

pub fn save_appeal(env: &Env, invoice_id: u64, record: &AppealRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Appeal(invoice_id), record);
}

pub fn save_pre_default_payer_score(env: &Env, invoice_id: u64, score: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::PreDefaultPayerScore(invoice_id), &score);
}

pub fn get_pre_default_payer_score(env: &Env, invoice_id: u64) -> Option<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::PreDefaultPayerScore(invoice_id))
}

// ----------------------------------------------------------------
// Contract Stats Helpers
// ----------------------------------------------------------------

/// In-memory stats accumulator for optimizing batch updates.
/// Use this struct to accumulate multiple stat changes and commit
/// them with a single storage operation for better gas efficiency.
#[derive(Clone, Debug)]
pub struct StatsAccumulator {
    pub invoices_delta: i64,
    pub funded_delta: i64,
    pub paid_delta: i64,
}

impl StatsAccumulator {
    pub fn new() -> Self {
        StatsAccumulator {
            invoices_delta: 0,
            funded_delta: 0,
            paid_delta: 0,
        }
    }

    pub fn add_invoice(&mut self) {
        self.invoices_delta = self.invoices_delta.saturating_add(1);
    }

    pub fn add_funded(&mut self) {
        self.funded_delta = self.funded_delta.saturating_add(1);
    }

    pub fn add_paid(&mut self) {
        self.paid_delta = self.paid_delta.saturating_add(1);
    }

    /// Commit accumulated deltas to persistent storage.
    /// This is more efficient than calling increment_* functions multiple times.
    pub fn commit(self, env: &Env) {
        if self.invoices_delta > 0 {
            let current: u64 = env
                .storage()
                .persistent()
                .get(&DataKey::TotalInvoices)
                .unwrap_or(0);
            env.storage().persistent().set(
                &DataKey::TotalInvoices,
                &current.saturating_add(self.invoices_delta as u64),
            );
        }
        if self.funded_delta > 0 {
            let current: u64 = env
                .storage()
                .persistent()
                .get(&DataKey::TotalFunded)
                .unwrap_or(0);
            env.storage().persistent().set(
                &DataKey::TotalFunded,
                &current.saturating_add(self.funded_delta as u64),
            );
        }
        if self.paid_delta > 0 {
            let current: u64 = env
                .storage()
                .persistent()
                .get(&DataKey::TotalPaid)
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&DataKey::TotalPaid, &current.saturating_add(self.paid_delta as u64));
        }
    }
}

pub fn increment_total_invoices(env: &Env) {
    let current: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::TotalInvoices)
        .unwrap_or(0);
    env.storage()
        .persistent()
        .set(&DataKey::TotalInvoices, &current.saturating_add(1));
}

pub fn increment_total_funded(env: &Env) {
    let current: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::TotalFunded)
        .unwrap_or(0);
    env.storage()
        .persistent()
        .set(&DataKey::TotalFunded, &current.saturating_add(1));
}

pub fn increment_total_paid(env: &Env) {
    let current: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::TotalPaid)
        .unwrap_or(0);
    env.storage()
        .persistent()
        .set(&DataKey::TotalPaid, &current.saturating_add(1));
}

// add_volume moved to invoice.rs where the configured token addresses are available

/// Get current total invoices count.
pub fn get_total_invoices(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalInvoices)
        .unwrap_or(0)
}

/// Get current total funded count.
pub fn get_total_funded(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalFunded)
        .unwrap_or(0)
}

/// Get current total paid count.
pub fn get_total_paid(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalPaid)
        .unwrap_or(0)
}
