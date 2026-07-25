#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

const INVOICE_AMOUNT: i128 = 1_000_000_000;
const DISCOUNT_RATE: u32 = 300;
const DUE_DATE_OFFSET: u64 = 60 * 60 * 24 * 30;

struct TestEnv {
    env: Env,
    contract: InvoiceLiquidityContractClient<'static>,
    token: TokenClient<'static>,
    token_address: Address,
    admin: Address,
    freelancer: Address,
    payer: Address,
    funder: Address,
}

fn setup() -> TestEnv {
    let env = Env::default();
    env.mock_all_auths();

    let usdc_admin = Address::generate(&env);
    let usdc_contract_id = env.register_stellar_asset_contract_v2(usdc_admin.clone());
    let usdc_address = usdc_contract_id.address();
    let token = TokenClient::new(&env, &usdc_address);
    let token_admin = StellarAssetClient::new(&env, &usdc_address);

    let admin = Address::generate(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let funder = Address::generate(&env);

    token_admin.mint(&funder, &(INVOICE_AMOUNT * 10));
    token_admin.mint(&payer, &(INVOICE_AMOUNT * 10));

    let contract_id = env.register_contract(None, InvoiceLiquidityContract);
    let contract = InvoiceLiquidityContractClient::new(&env, &contract_id);
    token_admin.mint(&contract.address, &(INVOICE_AMOUNT * 100));

    let xlm_admin = Address::generate(&env);
    let xlm_contract_id = env.register_stellar_asset_contract_v2(xlm_admin);
    let xlm_address = xlm_contract_id.address();

    let eurc_address = Address::generate(&env);
    contract.initialize(&admin, &usdc_address, &eurc_address, &xlm_address);

    let mut ledger_info = env.ledger().get();
    ledger_info.timestamp = 1_700_000_000;
    env.ledger().set(ledger_info);

    TestEnv {
        env,
        contract,
        token,
        token_address: usdc_address,
        admin,
        freelancer,
        payer,
        funder,
    }
}

fn due_date(t: &TestEnv) -> u64 {
    t.env.ledger().timestamp() + DUE_DATE_OFFSET
}

fn submit_standard(t: &TestEnv) -> u64 {
    t.contract.submit_invoice(
        &t.freelancer,
        &t.payer,
        &INVOICE_AMOUNT,
        &due_date(t),
        &DISCOUNT_RATE,
        &t.token_address,
        &ReferralCode::None,
    )
}

// ================================================================
// update_invoice token-aware validation tests (Issue #489)
// ================================================================

#[test]
fn test_update_invoice_preserves_token() {
    let t = setup();
    let id = submit_standard(&t);

    let invoice_before = t.contract.get_invoice(&id);
    let original_token = invoice_before.token.clone();

    let new_amount = INVOICE_AMOUNT + 250_000_000;
    let new_due = due_date(&t) + DUE_DATE_OFFSET;
    t.contract.update_invoice(&t.freelancer, &id, &new_amount, &new_due, &(DISCOUNT_RATE + 100));

    let invoice_after = t.contract.get_invoice(&id);
    assert_eq!(invoice_after.token, original_token);
    assert_eq!(invoice_after.amount, new_amount);
}

#[test]
fn test_update_invoice_valid_token_aware_amount() {
    let t = setup();
    let id = submit_standard(&t);

    let valid_amount = 2_000_000i128;
    let new_due = due_date(&t) + DUE_DATE_OFFSET;
    t.contract
        .update_invoice(&t.freelancer, &id, &valid_amount, &new_due, &DISCOUNT_RATE);

    let invoice = t.contract.get_invoice(&id);
    assert_eq!(invoice.amount, valid_amount);
}

#[test]
fn test_update_invoice_below_token_minimum_rejected() {
    let t = setup();
    let id = submit_standard(&t);

    let below_min = 500_000i128;
    let new_due = due_date(&t) + DUE_DATE_OFFSET;
    let result = t.contract.try_update_invoice(
        &t.freelancer,
        &id,
        &below_min,
        &new_due,
        &DISCOUNT_RATE,
    );

    assert_eq!(result, Err(Ok(ContractError::InvalidAmount)));
}

#[test]
fn test_update_invoice_xlm_token_aware_rejection() {
    let t = setup();

    let xlm_admin = Address::generate(&t.env);
    let xlm_id = t.env.register_stellar_asset_contract_v2(xlm_admin.clone());
    let xlm_address = xlm_id.address();
    let xlm_sac = StellarAssetClient::new(&t.env, &xlm_address);
    xlm_sac.mint(&t.funder, &1_000_000_000_000);
    // Admin needs tokens on the new token so add_token() can verify it.
    xlm_sac.mint(&t.admin, &10_000_000);
    t.contract.add_token(&xlm_address, &7);

    let id = t.contract.submit_invoice(
        &t.freelancer,
        &t.payer,
        &10_000_000,
        &due_date(&t),
        &DISCOUNT_RATE,
        &xlm_address,
        &ReferralCode::None,
    );

    let below_xlm_min = 5_000_000i128;
    let new_due = due_date(&t) + DUE_DATE_OFFSET;
    let result = t.contract.try_update_invoice(
        &t.freelancer,
        &id,
        &below_xlm_min,
        &new_due,
        &DISCOUNT_RATE,
    );

    assert_eq!(result, Err(Ok(ContractError::InvalidAmount)));
}

#[test]
fn test_update_invoice_xlm_at_minimum_accepted() {
    let t = setup();

    let xlm_admin = Address::generate(&t.env);
    let xlm_id = t.env.register_stellar_asset_contract_v2(xlm_admin.clone());
    let xlm_address = xlm_id.address();
    let xlm_sac = StellarAssetClient::new(&t.env, &xlm_address);
    xlm_sac.mint(&t.funder, &1_000_000_000_000);
    // Admin needs tokens on the new token so add_token() can verify it.
    xlm_sac.mint(&t.admin, &10_000_000);
    t.contract.add_token(&xlm_address, &7);

    let id = t.contract.submit_invoice(
        &t.freelancer,
        &t.payer,
        &100_000_000,
        &due_date(&t),
        &DISCOUNT_RATE,
        &xlm_address,
        &ReferralCode::None,
    );

    let xlm_min = 10_000_000i128;
    let new_due = due_date(&t) + DUE_DATE_OFFSET;
    t.contract.update_invoice(&t.freelancer, &id, &xlm_min, &new_due, &DISCOUNT_RATE);

    let invoice = t.contract.get_invoice(&id);
    assert_eq!(invoice.amount, xlm_min);
    assert_eq!(invoice.token, xlm_address);
}