//! Comprehensive tests covering remaining view functions, governance actions,
//! dispute resolution, appeal resolution, and storage migration in invoice_liquidity.

#![cfg(test)]

use super::*;
use crate::invoice::ReferralCode;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    vec, Address, BytesN, Env,
};

const INVOICE_AMOUNT: i128 = 10_000_000; // 10 USDC
const DISCOUNT_RATE: u32 = 300;
const DUE_DATE_OFFSET: u64 = 60 * 60 * 24 * 30; // 30 days

#[allow(dead_code)]
struct BoosterTestEnv {
    env: Env,
    contract: InvoiceLiquidityContractClient<'static>,
    token: TokenClient<'static>,
    admin: Address,
    freelancer: Address,
    payer: Address,
    funder: Address,
}

fn setup_booster() -> BoosterTestEnv {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc_id = env.register_stellar_asset_contract_v2(admin.clone());
    let usdc_addr = usdc_id.address();

    let token = TokenClient::new(&env, &usdc_addr);
    let token_admin = StellarAssetClient::new(&env, &usdc_addr);

    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let funder = Address::generate(&env);

    token_admin.mint(&funder, &(INVOICE_AMOUNT * 20));
    token_admin.mint(&payer, &(INVOICE_AMOUNT * 20));

    let contract_id = env.register(InvoiceLiquidityContract, ());
    let contract = InvoiceLiquidityContractClient::new(&env, &contract_id);
    token_admin.mint(&contract.address, &(INVOICE_AMOUNT * 100));

    let xlm_admin = Address::generate(&env);
    let xlm_id = env.register_stellar_asset_contract_v2(xlm_admin);
    let eurc_admin = Address::generate(&env);
    let eurc_id = env.register_stellar_asset_contract_v2(eurc_admin);

    contract.initialize(&admin, &xlm_id.address(), &usdc_addr, &eurc_id.address());

    BoosterTestEnv {
        env,
        contract,
        token,
        admin,
        freelancer,
        payer,
        funder,
    }
}

fn advance_rate_limit(env: &Env) {
    let mut info = env.ledger().get();
    info.sequence_number += 5000;
    env.ledger().set(info);
}

#[test]
fn test_storage_version_and_migration() {
    let t = setup_booster();
    assert_eq!(t.contract.get_storage_version(), 1);

    let v = t.contract.migrate();
    assert_eq!(v, crate::constants::CURRENT_STORAGE_VERSION);
    assert_eq!(
        t.contract.get_storage_version(),
        crate::constants::CURRENT_STORAGE_VERSION
    );

    // Idempotent migration
    let v2 = t.contract.migrate();
    assert_eq!(v2, crate::constants::CURRENT_STORAGE_VERSION);
}

#[test]
fn test_fee_tiers_management() {
    let t = setup_booster();
    advance_rate_limit(&t.env);
    assert_eq!(t.contract.get_fee_tiers().len(), 0);

    let tiers = vec![
        &t.env,
        (1_000_000, 300),
        (10_000_000, 200),
        (50_000_000, 100),
    ];
    t.contract.update_fee_tiers(&tiers);
    let loaded = t.contract.get_fee_tiers();
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded.get(0).unwrap(), (1_000_000, 300));
}

#[test]
fn test_list_invoices_pagination() {
    let t = setup_booster();
    let now = t.env.ledger().timestamp();
    let due = now + DUE_DATE_OFFSET;

    let id1 = t.contract.submit_invoice(
        &t.freelancer,
        &t.payer,
        &INVOICE_AMOUNT,
        &due,
        &DISCOUNT_RATE,
        &t.token.address,
        &ReferralCode::None,
    );
    let id2 = t.contract.submit_invoice(
        &t.freelancer,
        &t.payer,
        &INVOICE_AMOUNT,
        &due,
        &DISCOUNT_RATE,
        &t.token.address,
        &ReferralCode::None,
    );
    let id3 = t.contract.submit_invoice(
        &t.freelancer,
        &t.payer,
        &INVOICE_AMOUNT,
        &due,
        &DISCOUNT_RATE,
        &t.token.address,
        &ReferralCode::None,
    );

    // List by submitter pagination (0-indexed page)
    let page0 = t.contract.list_invoices_by_submitter(&t.freelancer, &0, &2);
    assert_eq!(page0.len(), 2);
    assert_eq!(page0.get(0).unwrap().id, id1);
    assert_eq!(page0.get(1).unwrap().id, id2);

    let page1 = t.contract.list_invoices_by_submitter(&t.freelancer, &1, &2);
    assert_eq!(page1.len(), 1);
    assert_eq!(page1.get(0).unwrap().id, id3);

    // Empty page
    let page2 = t.contract.list_invoices_by_submitter(&t.freelancer, &2, &2);
    assert_eq!(page2.len(), 0);

    // Fund one invoice
    t.contract
        .fund_invoice(&t.funder, &id1, &INVOICE_AMOUNT, &false);
    let lp_page = t.contract.list_invoices_by_lp(&t.funder, &0, &10);
    assert_eq!(lp_page.len(), 1);
    assert_eq!(lp_page.get(0).unwrap().id, id1);
}

#[test]
fn test_governance_setters_and_views() {
    let t = setup_booster();
    advance_rate_limit(&t.env);

    // update_decay_params
    t.contract.update_decay_params(&100, &2000);
    let cfg = t.contract.get_config();
    assert_eq!(cfg.decay_rate_bps, 100);
    assert_eq!(cfg.decay_period_ledgers, 2000);

    // set_distribution_contract
    let dist_addr = Address::generate(&t.env);
    advance_rate_limit(&t.env);
    t.contract.set_distribution_contract(&dist_addr);

    // insurance pool
    assert_eq!(t.contract.get_insurance_pool(), None);
    let pool_addr = Address::generate(&t.env);
    t.contract.set_insurance_pool(&pool_addr);
    assert_eq!(t.contract.get_insurance_pool(), Some(pool_addr));

    // token decimals
    let new_token_admin = Address::generate(&t.env);
    let new_token_id = t.env.register_stellar_asset_contract_v2(new_token_admin);
    let new_token = new_token_id.address();
    let new_token_client = StellarAssetClient::new(&t.env, &new_token);
    new_token_client.mint(&t.admin, &10_000_000);
    advance_rate_limit(&t.env);
    t.contract.add_token(&new_token, &6);
    assert_eq!(t.contract.get_token_decimals(&new_token), Some(6));
    let unk = Address::generate(&t.env);
    assert_eq!(t.contract.get_token_decimals(&unk), None);

    // oracle age & getters
    assert_eq!(t.contract.get_max_oracle_age(), 17280);
    advance_rate_limit(&t.env);
    t.contract.set_max_oracle_age(&20000);
    assert_eq!(t.contract.get_max_oracle_age(), 20000);

    // min payer reputation
    assert_eq!(t.contract.min_payer_reputation(), 0);
    advance_rate_limit(&t.env);
    t.contract.set_min_payer_reputation(&40);
    assert_eq!(t.contract.min_payer_reputation(), 40);

    // suggested discount rate
    let sugg = t.contract.suggested_discount_rate(&t.payer);
    assert!(sugg > 0);

    // reputation profile
    let rep = t.contract.get_reputation(&t.payer);
    assert_eq!(rep.score, 0);

    // score getters
    assert_eq!(t.contract.payer_score(&t.payer), 50);
    assert_eq!(t.contract.lp_score(&t.funder), 50);

    // NFT queries
    assert_eq!(t.contract.query_nft_metadata(&999), None);
    assert_eq!(t.contract.query_nft_owner(&999), None);
}

#[test]
fn test_dispute_and_resolution_flow() {
    let t = setup_booster();
    let now = t.env.ledger().timestamp();
    let due = now + DUE_DATE_OFFSET;

    let id = t.contract.submit_invoice(
        &t.freelancer,
        &t.payer,
        &INVOICE_AMOUNT,
        &due,
        &DISCOUNT_RATE,
        &t.token.address,
        &ReferralCode::None,
    );

    let reason_hash = BytesN::from_array(&t.env, &[7u8; 32]);
    let resolution_hash = BytesN::from_array(&t.env, &[8u8; 32]);

    // Dispute pending invoice
    t.contract.dispute_invoice(&id, &reason_hash);
    let inv = t.contract.get_invoice(&id);
    assert_eq!(inv.status, InvoiceStatus::Disputed);

    // Resolve dispute with resolution 2 (Rejected -> Freelancer wins -> status back to Pending)
    t.contract.resolve_dispute(&id, &resolution_hash, &2);
    let inv = t.contract.get_invoice(&id);
    assert_eq!(inv.status, InvoiceStatus::Pending);

    // Fund it
    t.contract
        .fund_invoice(&t.funder, &id, &INVOICE_AMOUNT, &false);

    // Submit and fund another invoice to test Upheld resolution on Funded status
    let id2 = t.contract.submit_invoice(
        &t.freelancer,
        &t.payer,
        &INVOICE_AMOUNT,
        &due,
        &DISCOUNT_RATE,
        &t.token.address,
        &ReferralCode::None,
    );
    t.contract
        .fund_invoice(&t.funder, &id2, &INVOICE_AMOUNT, &false);
    t.contract.dispute_invoice(&id2, &reason_hash);
    let inv2 = t.contract.get_invoice(&id2);
    assert_eq!(inv2.status, InvoiceStatus::Disputed);

    // Resolve dispute with resolution 1 (Upheld -> Payer wins -> status Cancelled, LP refunded)
    t.contract.resolve_dispute(&id2, &resolution_hash, &1);
    let inv2 = t.contract.get_invoice(&id2);
    assert_eq!(inv2.status, InvoiceStatus::Cancelled);
}

#[test]
fn test_auto_resolve_dispute_timeout() {
    let t = setup_booster();
    let now = t.env.ledger().timestamp();
    let due = now + DUE_DATE_OFFSET;

    let id = t.contract.submit_invoice(
        &t.freelancer,
        &t.payer,
        &INVOICE_AMOUNT,
        &due,
        &DISCOUNT_RATE,
        &t.token.address,
        &ReferralCode::None,
    );
    let reason_hash = BytesN::from_array(&t.env, &[5u8; 32]);
    t.contract.dispute_invoice(&id, &reason_hash);

    // Timeout not reached
    assert!(t.contract.try_auto_resolve_dispute(&id).is_err());

    // Advance ledger past timeout (10000 ledgers)
    let mut ledger = t.env.ledger().get();
    ledger.sequence_number += 20000;
    t.env.ledger().set(ledger);

    t.contract.auto_resolve_dispute(&id);
    let inv = t.contract.get_invoice(&id);
    assert_eq!(inv.status, InvoiceStatus::Pending);
}

#[test]
fn test_appeal_and_resolution_flow() {
    let t = setup_booster();
    let now = t.env.ledger().timestamp();
    let due = now + DUE_DATE_OFFSET;

    let id = t.contract.submit_invoice(
        &t.freelancer,
        &t.payer,
        &INVOICE_AMOUNT,
        &due,
        &DISCOUNT_RATE,
        &t.token.address,
        &ReferralCode::None,
    );
    t.contract
        .fund_invoice(&t.funder, &id, &INVOICE_AMOUNT, &false);

    // Advance time past due date
    let mut ledger = t.env.ledger().get();
    ledger.timestamp = due + 10;
    t.env.ledger().set(ledger);

    // LP claims default
    t.contract.claim_default(&t.funder, &id);
    let inv = t.contract.get_invoice(&id);
    assert_eq!(inv.status, InvoiceStatus::Defaulted);

    // Payer appeals
    let evidence_hash = BytesN::from_array(&t.env, &[9u8; 32]);
    t.contract.appeal_default(&id, &evidence_hash);
    let inv = t.contract.get_invoice(&id);
    assert_eq!(inv.status, InvoiceStatus::Appealed);

    // Admin resolves appeal (upheld)
    t.contract.resolve_appeal(&id, &true);
    let inv = t.contract.get_invoice(&id);
    assert_eq!(inv.status, InvoiceStatus::Defaulted);
}

#[test]
fn test_claim_yield_and_referral_stats() {
    let t = setup_booster();
    let now = t.env.ledger().timestamp();
    let due = now + DUE_DATE_OFFSET;

    let ref_code = BytesN::from_array(&t.env, &[42u8; 32]);
    let id = t.contract.submit_invoice(
        &t.freelancer,
        &t.payer,
        &INVOICE_AMOUNT,
        &due,
        &DISCOUNT_RATE,
        &t.token.address,
        &ReferralCode::Present(ref_code.clone()),
    );
    assert_eq!(t.contract.get_referral_stats(&ref_code), 1);

    // Unfunded invoice yield is 0 (or NothingToClaim if no funder)
    assert_eq!(
        t.contract.try_claim_yield(&id),
        Err(Ok(ContractError::NothingToClaim))
    );

    t.contract
        .fund_invoice(&t.funder, &id, &INVOICE_AMOUNT, &false);
    assert_eq!(t.contract.claim_yield(&id), 0);

    t.contract.mark_paid(&id, &INVOICE_AMOUNT);
    let y = t.contract.claim_yield(&id);
    assert_eq!(y, 300_000);
}
