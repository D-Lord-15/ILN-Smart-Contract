#![cfg(test)]

//! Comprehensive access control tests for admin-restricted functions.
//!
//! These tests verify that every admin-only function properly rejects calls
//! from non-admin accounts. Tests use `try_` variants and expect
//! `ContractError::Unauthorized` (or the appropriate auth error).
//!
//! Issue #540 — Access Control Audit

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token::{Client as TokenClient, StellarAssetClient},
    Address, BytesN, Env, IntoVal,
};

fn setup_env() -> (
    Env,
    Address,
    Address,
    InvoiceLiquidityContractClient<'static>,
) {
    let env = Env::default();
    let admin = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let usdc_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = usdc_contract.address();

    let xlm_admin = Address::generate(&env);
    let xlm_contract = env.register_stellar_asset_contract_v2(xlm_admin.clone());
    let xlm_address = xlm_contract.address();

    let contract_id = env.register_contract(None, InvoiceLiquidityContract);
    let client = InvoiceLiquidityContractClient::new(&env, &contract_id);

    client.initialize(&admin, &token_address, &token_address, &xlm_address);

    let mut ledger = env.ledger().get();
    ledger.timestamp = 1_700_000_000;
    env.ledger().set(ledger);

    (env, admin, token_address, client)
}

// ----------------------------------------------------------------
// Admin function violations
// ----------------------------------------------------------------

#[test]
fn test_set_admin_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);
    let new_admin = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_admin",
            args: (new_admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let res = client.try_set_admin(&new_admin);
    assert!(res.is_err(), "set_admin should fail for non-admin caller");
}

#[test]
fn test_update_fee_rate_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_fee_rate",
            args: (250u32,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let res = client.try_update_fee_rate(&250);
    assert!(res.is_err(), "update_fee_rate should fail for non-admin caller");
}

#[test]
fn test_update_max_discount_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_max_discount",
            args: (4000u32,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let res = client.try_update_max_discount(&4000);
    assert!(res.is_err(), "update_max_discount should fail for non-admin caller");
}

#[test]
fn test_pause_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "pause",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let res = client.try_pause();
    assert!(res.is_err(), "pause should fail for non-admin caller");
}

#[test]
fn test_unpause_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "unpause",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);

    // First pause as admin
    client.pause();
    // Now try unpause as imposter
    let res = client.try_unpause();
    assert!(res.is_err(), "unpause should fail for non-admin caller");
}

#[test]
fn test_add_token_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);
    let new_token = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "add_token",
            args: (new_token.clone(), 6u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let res = client.try_add_token(&new_token, &6);
    assert!(res.is_err(), "add_token should fail for non-admin caller");
}

#[test]
fn test_remove_token_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);
    let token = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "remove_token",
            args: (token.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let res = client.try_remove_token(&token);
    assert!(res.is_err(), "remove_token should fail for non-admin caller");
}

#[test]
fn test_set_distribution_contract_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);
    let dist = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_distribution_contract",
            args: (dist.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let res = client.try_set_distribution_contract(&dist);
    assert!(res.is_err(), "set_distribution_contract should fail for non-admin caller");
}

#[test]
fn test_set_min_payer_reputation_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_min_payer_reputation",
            args: (10u32,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let res = client.try_set_min_payer_reputation(&10);
    assert!(res.is_err(), "set_min_payer_reputation should fail for non-admin caller");
}

#[test]
fn test_upgrade_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);
    let dummy_hash = BytesN::from_array(&env, &[0u8; 32]);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "upgrade",
            args: (dummy_hash.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let res = client.try_upgrade(&dummy_hash);
    assert!(res.is_err(), "upgrade should fail for non-admin caller");
}

#[test]
fn test_resolve_appeal_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "resolve_appeal",
            args: (1u64, true).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let res = client.try_resolve_appeal(&1, &true);
    assert!(res.is_err(), "resolve_appeal should fail for non-admin caller");
}

#[test]
fn test_set_price_oracle_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);
    let oracle = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_price_oracle",
            args: (oracle.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let res = client.try_set_price_oracle(&oracle);
    assert!(res.is_err(), "set_price_oracle should fail for non-admin caller");
}

#[test]
fn test_set_max_oracle_age_unauthorized_caller() {
    let (env, _admin, _, client) = setup_env();
    let imposter = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_max_oracle_age",
            args: (10000u64,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let res = client.try_set_max_oracle_age(&10000);
    assert!(res.is_err(), "set_max_oracle_age should fail for non-admin caller");
}

// ----------------------------------------------------------------
// Public methods should succeed for anyone
// ----------------------------------------------------------------

#[test]
fn test_public_methods() {
    let (_env, _admin, _, client) = setup_env();

    // Anyone can read contract stats without mock auth
    let stats = client.get_contract_stats();
    assert_eq!(stats.total_invoices, 0);

    let count = client.get_invoice_count();
    assert_eq!(count, 0);
}
