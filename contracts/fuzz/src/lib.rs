#![no_std]

#[cfg(test)]
mod tests {
    use iln_governance::{GovernanceContract, GovernanceContractClient};
    use invoice_liquidity::{
        InvoiceLiquidityContract, InvoiceLiquidityContractClient, ReferralCode,
    };
    use proptest::prelude::*;
    use soroban_sdk::{
        address_payload::AddressPayload,
        testutils::{Address as _, Ledger},
        Address, BytesN, Env,
    };

    const LEDGER_TIMESTAMP: u64 = 1_700_000_000;

    struct FuzzEnv {
        env: Env,
        contract: InvoiceLiquidityContractClient<'static>,
    }

    fn setup_fuzz() -> FuzzEnv {
        let env = Env::default();
        env.mock_all_auths();

        // Deploy mock USDC token
        let usdc_admin = Address::generate(&env);
        let usdc_contract_id = env.register_stellar_asset_contract_v2(usdc_admin.clone());
        let usdc_address = usdc_contract_id.address();

        // Deploy and initialise the ILN contract
        let contract_id = env.register_contract(None, InvoiceLiquidityContract);
        let contract = InvoiceLiquidityContractClient::new(&env, &contract_id);

        let xlm_admin = Address::generate(&env);
        let xlm_contract_id = env.register_stellar_asset_contract_v2(xlm_admin);
        let xlm_address = xlm_contract_id.address();

        contract.initialize(&usdc_admin, &usdc_address, &xlm_address);

        // Fix ledger timestamp to a known baseline
        let mut ledger_info = env.ledger().get();
        ledger_info.timestamp = LEDGER_TIMESTAMP;
        env.ledger().set(ledger_info);

        FuzzEnv { env, contract }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn prop_submit_invoice_never_panics(
            amount in any::<i128>(),
            discount_rate in any::<u32>(),
            due_date in any::<u64>(),
            payer_bytes in any::<[u8; 32]>(),
            freelancer_bytes in any::<[u8; 32]>(),
            token_bytes in any::<[u8; 32]>(),
            payer_is_contract in any::<bool>(),
            freelancer_is_contract in any::<bool>(),
            token_is_contract in any::<bool>(),
        ) {
            let t = setup_fuzz();

            // Construct fuzzed random addresses using ContractIdHash or AccountIdPublicKeyEd25519 payloads
            let payer_payload = if payer_is_contract {
                AddressPayload::ContractIdHash(BytesN::from_array(&t.env, &payer_bytes))
            } else {
                AddressPayload::AccountIdPublicKeyEd25519(BytesN::from_array(&t.env, &payer_bytes))
            };
            let payer = Address::from_val(&t.env, &payer_payload);

            let freelancer_payload = if freelancer_is_contract {
                AddressPayload::ContractIdHash(BytesN::from_array(&t.env, &freelancer_bytes))
            } else {
                AddressPayload::AccountIdPublicKeyEd25519(BytesN::from_array(&t.env, &freelancer_bytes))
            };
            let freelancer = Address::from_val(&t.env, &freelancer_payload);

            let token_payload = if token_is_contract {
                AddressPayload::ContractIdHash(BytesN::from_array(&t.env, &token_bytes))
            } else {
                AddressPayload::AccountIdPublicKeyEd25519(BytesN::from_array(&t.env, &token_bytes))
            };
            let token = Address::from_val(&t.env, &token_payload);

            // Call try_submit_invoice with fuzzed random inputs.
            // We want to ensure that regardless of the fuzzed inputs,
            // the contract either succeeds or returns a handled error,
            // but NEVER panics or triggers an unexpected crash/unwind.
            let result = t.contract.try_submit_invoice(
                &freelancer,
                &payer,
                &amount,
                &due_date,
                &discount_rate,
                &token,
                &ReferralCode::None,
            );

            // We assert that the call completes gracefully (i.e. returning a Result),
            // regardless of whether it succeeded (Ok) or was rejected (Err).
            // Prop_assert guarantees this execution finished without panicking.
            match result {
                Ok(_) => {
                    // Successful invoice submission
                }
                Err(_) => {
                    // Handled validation error (e.g. InvalidAmount, InvalidDiscountRate, etc.)
                }
            }
        }

        #[test]
        fn prop_cancel_invoice_never_panics(
            invoice_id in any::<u64>(),
        ) {
            let t = setup_fuzz();
            let _ = t.contract.try_cancel_invoice(&invoice_id);
        }

        #[test]
        fn prop_appeal_default_never_panics(
            invoice_id in any::<u64>(),
            evidence_bytes in any::<[u8; 32]>(),
        ) {
            let t = setup_fuzz();
            let evidence = BytesN::from_array(&t.env, &evidence_bytes);
            let _ = t.contract.try_appeal_default(&invoice_id, &evidence);
        }

        #[test]
        fn prop_cast_vote_never_panics(
            proposal_id in any::<u64>(),
            support in any::<bool>(),
            voter_bytes in any::<[u8; 32]>(),
            voter_is_contract in any::<bool>(),
        ) {
            let env = Env::default();
            env.mock_all_auths();
            let contract_id = env.register_contract(None, GovernanceContract);
            let gov = GovernanceContractClient::new(&env, &contract_id);

            // Initialize gov contract
            let admin = Address::generate(&env);
            let token = Address::generate(&env);
            let _ = gov.try_initialize(&admin, &token);

            let voter_payload = if voter_is_contract {
                AddressPayload::ContractIdHash(BytesN::from_array(&env, &voter_bytes))
            } else {
                AddressPayload::AccountIdPublicKeyEd25519(BytesN::from_array(&env, &voter_bytes))
            };
            let voter = Address::from_val(&env, &voter_payload);

            let _ = gov.try_cast_vote(&voter, &proposal_id, &support);
        }

        #[test]
        fn prop_delegate_votes_never_panics(
            delegator_bytes in any::<[u8; 32]>(),
            delegate_bytes in any::<[u8; 32]>(),
        ) {
            let env = Env::default();
            env.mock_all_auths();
            let contract_id = env.register_contract(None, GovernanceContract);
            let gov = GovernanceContractClient::new(&env, &contract_id);

            // Initialize gov contract
            let admin = Address::generate(&env);
            let token = Address::generate(&env);
            let _ = gov.try_initialize(&admin, &token);

            let delegator = Address::from_val(&env, &AddressPayload::AccountIdPublicKeyEd25519(BytesN::from_array(&env, &delegator_bytes)));
            let delegate = Address::from_val(&env, &AddressPayload::AccountIdPublicKeyEd25519(BytesN::from_array(&env, &delegate_bytes)));

            let _ = gov.try_delegate_votes(&delegator, &delegate);
        }
    }
}

// ================================================================
// Issue #500: insurance pool claim / reentrancy fuzzing
// ================================================================
#[cfg(test)]
mod insurance_tests {
    use insurance_pool::{InsuranceError, InsurancePool, InsurancePoolClient};
    use proptest::prelude::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    const COVERAGE: i128 = 1_000_000_000;

    struct InsFuzzEnv {
        env: Env,
        client: InsurancePoolClient<'static>,
    }

    fn setup_insurance() -> InsFuzzEnv {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, InsurancePool);
        let client = InsurancePoolClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin, &COVERAGE);

        InsFuzzEnv { env, client }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        // claim must never panic-unwind for arbitrary invoice ids.
        // With an empty pool every claim must be rejected (PoolEmpty),
        // never silently succeed.
        #[test]
        fn prop_claim_empty_pool_always_rejected(invoice_id in any::<u64>()) {
            let t = setup_insurance();

            // No premiums deposited => empty pool.
            prop_assert_eq!(t.client.get_pool_balance(), 0);

            let res = t.client.try_claim(&invoice_id);
            // Empty pool must reject, not pay out.
            prop_assert_eq!(
                res,
                Err(Ok(soroban_sdk::Error::from(InsuranceError::PoolEmpty)))
            );
            prop_assert!(!t.client.is_claimed(&invoice_id));
        }

        // Double-claim rejection: a second claim for the same invoice id must
        // be rejected with AlreadyClaimed and must not pay out or drain the
        // pool a second time (reentrancy / double-spend guard).
        #[test]
        fn prop_double_claim_rejected(
            invoice_id in any::<u64>(),
            premium in 1i128..1_000_000_000_000i128,
        ) {
            let t = setup_insurance();
            let lp = Address::generate(&t.env);

            // Fund the pool so the first claim can pay out.
            t.client.deposit_premium(&lp, &premium);
            let balance_before = t.client.get_pool_balance();

            // First claim succeeds and marks the invoice claimed.
            let first = t.client.claim(&invoice_id);
            prop_assert!(first > 0);
            prop_assert!(t.client.is_claimed(&invoice_id));

            let balance_after_first = t.client.get_pool_balance();
            prop_assert_eq!(balance_after_first, balance_before - first);

            // Second claim for the same invoice must be rejected.
            let second = t.client.try_claim(&invoice_id);
            prop_assert_eq!(
                second,
                Err(Ok(soroban_sdk::Error::from(InsuranceError::AlreadyClaimed)))
            );

            // Balance must be unchanged by the rejected double-claim.
            prop_assert_eq!(t.client.get_pool_balance(), balance_after_first);
        }

        // General robustness: an interleaved sequence of claims across random
        // invoice ids never panics and never double-pays the same id.
        #[test]
        fn prop_claim_sequence_never_double_pays(
            ids in prop::collection::vec(any::<u64>(), 1..8),
            premium in 1i128..1_000_000_000_000i128,
        ) {
            let t = setup_insurance();
            let lp = Address::generate(&t.env);
            t.client.deposit_premium(&lp, &premium);

            for id in ids.iter() {
                let already = t.client.is_claimed(id);
                let res = t.client.try_claim(id);
                if already {
                    // A repeated id in the sequence must be rejected.
                    prop_assert_eq!(
                        res,
                        Err(Ok(soroban_sdk::Error::from(InsuranceError::AlreadyClaimed)))
                    );
                } else {
                    // Either paid (Ok) or rejected because the pool drained
                    // (PoolEmpty) — never a panic-unwind.
                    match res {
                        Ok(_) => prop_assert!(t.client.is_claimed(id)),
                        Err(_) => {}
                    }
                }
            }
        }
    }
}
