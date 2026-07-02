#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup_test(
    env: &Env,
) -> (
    InvoiceLiquidityContractClient<'_>,
    Address,
    Address,
    Address,
) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InvoiceLiquidityContract);
    let client = InvoiceLiquidityContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let usdc_admin = Address::generate(env);
    let usdc_id = env.register_stellar_asset_contract_v2(usdc_admin);
    let usdc = usdc_id.address();

    let xlm_admin = Address::generate(env);
    let xlm_id = env.register_stellar_asset_contract_v2(xlm_admin);
    let xlm = xlm_id.address();

    client.initialize(&admin, &usdc, &xlm);

    (client, usdc, Address::generate(env), Address::generate(env))
}

#[test]
fn submit_invoice_at_minimum_succeeds() {
    let env = Env::default();
    let (client, token, freelancer, payer) = setup_test(&env);

    let amount = MIN_INVOICE_AMOUNT;
    let due_date = env.ledger().timestamp() + 100000;
    let discount = 100u32;

    let id = client.submit_invoice(&freelancer, &payer, &amount, &due_date, &discount, &token, &ReferralCode::None);

    let invoice = client.get_invoice(&id);
    assert_eq!(invoice.amount, amount);
}

#[test]
fn submit_invoice_below_minimum_rejected() {
    let env = Env::default();
    let (client, token, freelancer, payer) = setup_test(&env);

    let amount = MIN_INVOICE_AMOUNT - 1;
    let due_date = env.ledger().timestamp() + 100000;
    let discount = 100u32;

    let result = client.try_submit_invoice(&freelancer, &payer, &amount, &due_date, &discount, &token, &ReferralCode::None);
    assert_eq!(result, Err(Ok(ContractError::AmountTooSmall)));
}

#[test]
fn submit_invoice_zero_rejected() {
    let env = Env::default();
    let (client, token, freelancer, payer) = setup_test(&env);

    let amount = 0i128;
    let due_date = env.ledger().timestamp() + 100000;
    let discount = 100u32;

    let result = client.try_submit_invoice(&freelancer, &payer, &amount, &due_date, &discount, &token, &ReferralCode::None);
    assert_eq!(result, Err(Ok(ContractError::AmountTooSmall)));
}
