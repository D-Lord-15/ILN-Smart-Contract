#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token::StellarAssetClient, Address, Env,
};

const DEFAULT_HALF_TOKEN: i128 = 5_000_000;
const DEFAULT_HUNDRED_USDC_STROOPS: i128 = 1_000_000_000;
const DEFAULT_LP_MULTIPLIER: i128 = 10_000_000;

#[contracttype]
pub enum StorageKey {
    Initialized,
    IlnContract,
    GovToken,
    LpFundedVolume(Address),
    FreelancerSettled(Address),
    PayerOnTimeSettled(Address),
    Claimed(Address),
    /// Issue #544: configurable reward parameters
    HalfToken,
    HundredUsdcStroops,
    LpMultiplier,
}

/// Emitted once, when the contract is initialised (Issue #538).
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ContractInitialized {
    pub iln_contract: Address,
    pub gov_token: Address,
}

/// Emitted when an LP's funded volume accrual increases (Issue #538).
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct LpVolumeAccrued {
    pub lp: Address,
    pub amount_usdc_equivalent: i128,
}

/// Emitted when a settlement is recorded for a freelancer/payer (Issue #538).
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct SettlementAccrued {
    pub freelancer: Address,
    pub payer: Address,
    pub settled_on_time: bool,
}

/// Emitted when a participant claims accrued governance tokens (Issue #538).
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct TokensClaimed {
    pub claimer: Address,
    pub amount: i128,
}

#[contract]
pub struct IlnDistribution;

#[contractimpl]
impl IlnDistribution {
    pub fn initialize(env: Env, iln_contract: Address, gov_token: Address) {
        if env.storage().instance().has(&StorageKey::Initialized) {
            panic!("already initialized");
        }

        env.storage()
            .instance()
            .set(&StorageKey::Initialized, &true);
        env.storage()
            .instance()
            .set(&StorageKey::IlnContract, &iln_contract);
        env.storage()
            .instance()
            .set(&StorageKey::GovToken, &gov_token);

        env.events().publish(
            (symbol_short!("init"),),
            ContractInitialized {
                iln_contract,
                gov_token,
            },
        );
    }

    pub fn accrue_lp(env: Env, lp: Address, amount_usdc_equivalent: i128) {
        Self::require_iln_invoker(&env);

        if amount_usdc_equivalent <= 0 {
            return;
        }

        let key = StorageKey::LpFundedVolume(lp.clone());
        let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&key, &current.saturating_add(amount_usdc_equivalent));

        env.events().publish(
            (symbol_short!("lp_accr"), lp.clone()),
            LpVolumeAccrued {
                lp,
                amount_usdc_equivalent,
            },
        );
    }

    pub fn accrue_settlement(env: Env, freelancer: Address, payer: Address, settled_on_time: bool) {
        Self::require_iln_invoker(&env);

        let freelancer_key = StorageKey::FreelancerSettled(freelancer.clone());
        let freelancer_count: u64 = env
            .storage()
            .persistent()
            .get(&freelancer_key)
            .unwrap_or(0_u64);
        env.storage()
            .persistent()
            .set(&freelancer_key, &freelancer_count.saturating_add(1));

        if settled_on_time {
            let payer_key = StorageKey::PayerOnTimeSettled(payer.clone());
            let payer_count: u64 = env.storage().persistent().get(&payer_key).unwrap_or(0_u64);
            env.storage()
                .persistent()
                .set(&payer_key, &payer_count.saturating_add(1));
        }

        env.events().publish(
            (
                symbol_short!("settled"),
                freelancer.clone(),
                payer.clone(),
            ),
            SettlementAccrued {
                freelancer,
                payer,
                settled_on_time,
            },
        );
    }

    pub fn claim_tokens(env: Env, claimer: Address) -> i128 {
        claimer.require_auth();

        let total_earned = Self::total_earned(&env, &claimer);
        let claimed_key = StorageKey::Claimed(claimer.clone());
        let already_claimed: i128 = env.storage().persistent().get(&claimed_key).unwrap_or(0);

        let claimable = total_earned.saturating_sub(already_claimed);
        if claimable <= 0 {
            return 0;
        }

        let gov_token: Address = env.storage().instance().get(&StorageKey::GovToken).unwrap();
        StellarAssetClient::new(&env, &gov_token).mint(&claimer, &claimable);

        env.storage()
            .persistent()
            .set(&claimed_key, &already_claimed.saturating_add(claimable));

        env.events().publish(
            (symbol_short!("claimed"), claimer.clone()),
            TokensClaimed {
                claimer,
                amount: claimable,
            },
        );

        claimable
    }

    pub fn get_accrual(env: Env, participant: Address) -> i128 {
        Self::total_earned(&env, &participant)
    }

    /// Issue #544: Update reward parameters. Only callable by the ILN contract
    /// (acting on a governance proposal). The ILN contract address is the
    /// authority for all state-changing calls to this contract.
    pub fn update_reward_params(
        env: Env,
        half_token: i128,
        hundred_usdc_stroops: i128,
        lp_multiplier: i128,
    ) {
        Self::require_iln_invoker(&env);

        let old_half = Self::get_half_token(&env);
        let old_hundred = Self::get_hundred_usdc_stroops(&env);
        let old_lp = Self::get_lp_multiplier(&env);

        env.storage()
            .instance()
            .set(&StorageKey::HalfToken, &half_token);
        env.storage()
            .instance()
            .set(&StorageKey::HundredUsdcStroops, &hundred_usdc_stroops);
        env.storage()
            .instance()
            .set(&StorageKey::LpMultiplier, &lp_multiplier);

        env.events().publish(
            (symbol_short!("rwrd_upd"),),
            (
                old_half, half_token,
                old_hundred, hundred_usdc_stroops,
                old_lp, lp_multiplier,
            ),
        );
    }

    fn get_half_token(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&StorageKey::HalfToken)
            .unwrap_or(DEFAULT_HALF_TOKEN)
    }

    fn get_hundred_usdc_stroops(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&StorageKey::HundredUsdcStroops)
            .unwrap_or(DEFAULT_HUNDRED_USDC_STROOPS)
    }

    fn get_lp_multiplier(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&StorageKey::LpMultiplier)
            .unwrap_or(DEFAULT_LP_MULTIPLIER)
    }

    fn total_earned(env: &Env, participant: &Address) -> i128 {
        let lp_volume: i128 = env
            .storage()
            .persistent()
            .get(&StorageKey::LpFundedVolume(participant.clone()))
            .unwrap_or(0);
        let freelancer_settled: u64 = env
            .storage()
            .persistent()
            .get(&StorageKey::FreelancerSettled(participant.clone()))
            .unwrap_or(0_u64);
        let payer_on_time: u64 = env
            .storage()
            .persistent()
            .get(&StorageKey::PayerOnTimeSettled(participant.clone()))
            .unwrap_or(0_u64);

        let hundred_usdc = Self::get_hundred_usdc_stroops(env);
        let lp_mult = Self::get_lp_multiplier(env);
        let half_token = Self::get_half_token(env);

        let lp_reward = if hundred_usdc > 0 {
            (lp_volume / hundred_usdc).saturating_mul(lp_mult)
        } else {
            0
        };
        let freelancer_reward = (freelancer_settled as i128).saturating_mul(half_token);
        let payer_reward = (payer_on_time as i128).saturating_mul(half_token);

        lp_reward
            .saturating_add(freelancer_reward)
            .saturating_add(payer_reward)
    }

    fn require_iln_invoker(env: &Env) {
        let iln_contract: Address = env
            .storage()
            .instance()
            .get(&StorageKey::IlnContract)
            .unwrap();
        iln_contract.require_auth();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, token::Client as TokenClient, Address};

    #[cfg(test)]
    use super::{DEFAULT_HALF_TOKEN as HALF_TOKEN, DEFAULT_HUNDRED_USDC_STROOPS as HUNDRED_USDC_STROOPS};

    #[contract]
    pub struct MockIln;

    #[contractimpl]
    impl MockIln {
        pub fn accrue_lp(env: Env, dist: Address, lp: Address, amount: i128) {
            IlnDistributionClient::new(&env, &dist).accrue_lp(&lp, &amount);
        }

        pub fn accrue_settlement(
            env: Env,
            dist: Address,
            freelancer: Address,
            payer: Address,
            on_time: bool,
        ) {
            IlnDistributionClient::new(&env, &dist).accrue_settlement(
                &freelancer,
                &payer,
                &on_time,
            );
        }
    }

    #[test]
    fn lp_earns_on_funding_and_cannot_double_claim() {
        let env = Env::default();
        env.mock_all_auths();

        let iln_id = env.register_contract(None, MockIln);
        let dist_id = env.register_contract(None, IlnDistribution);
        let dist = IlnDistributionClient::new(&env, &dist_id);
        let iln = MockIlnClient::new(&env, &iln_id);

        let gov_token_id = env.register_stellar_asset_contract_v2(dist_id.clone());
        let gov_token = gov_token_id.address();
        let token_client = TokenClient::new(&env, &gov_token);

        dist.initialize(&iln_id, &gov_token);

        let lp = Address::generate(&env);
        iln.accrue_lp(&dist_id, &lp, &HUNDRED_USDC_STROOPS);

        let claimed = dist.claim_tokens(&lp);
        assert_eq!(claimed, 10_000_000);
        assert_eq!(token_client.balance(&lp), 10_000_000);

        let second_claim = dist.claim_tokens(&lp);
        assert_eq!(second_claim, 0);
        assert_eq!(token_client.balance(&lp), 10_000_000);
    }

    #[test]
    fn freelancer_and_payer_earn_on_settlement() {
        let env = Env::default();
        env.mock_all_auths();

        let iln_id = env.register_contract(None, MockIln);
        let dist_id = env.register_contract(None, IlnDistribution);
        let dist = IlnDistributionClient::new(&env, &dist_id);
        let iln = MockIlnClient::new(&env, &iln_id);

        let gov_token_id = env.register_stellar_asset_contract_v2(dist_id.clone());
        let gov_token = gov_token_id.address();
        let token_client = TokenClient::new(&env, &gov_token);

        dist.initialize(&iln_id, &gov_token);

        let freelancer = Address::generate(&env);
        let payer = Address::generate(&env);

        iln.accrue_settlement(&dist_id, &freelancer, &payer, &true);

        assert_eq!(dist.claim_tokens(&freelancer), HALF_TOKEN);
        assert_eq!(dist.claim_tokens(&payer), HALF_TOKEN);
        assert_eq!(token_client.balance(&freelancer), HALF_TOKEN);
        assert_eq!(token_client.balance(&payer), HALF_TOKEN);
    }

    #[test]
    fn late_settlement_does_not_reward_payer() {
        let env = Env::default();
        env.mock_all_auths();

        let iln_id = env.register_contract(None, MockIln);
        let dist_id = env.register_contract(None, IlnDistribution);
        let dist = IlnDistributionClient::new(&env, &dist_id);
        let iln = MockIlnClient::new(&env, &iln_id);

        let gov_token_id = env.register_stellar_asset_contract_v2(dist_id.clone());
        let gov_token = gov_token_id.address();

        dist.initialize(&iln_id, &gov_token);

        let freelancer = Address::generate(&env);
        let payer = Address::generate(&env);

        iln.accrue_settlement(&dist_id, &freelancer, &payer, &false);

        assert_eq!(dist.claim_tokens(&freelancer), HALF_TOKEN);
        assert_eq!(dist.claim_tokens(&payer), 0);
    }

    #[test]
    fn update_reward_params_changes_earned_amounts() {
        let env = Env::default();
        env.mock_all_auths();

        let iln_id = env.register_contract(None, MockIln);
        let dist_id = env.register_contract(None, IlnDistribution);
        let dist = IlnDistributionClient::new(&env, &dist_id);
        let iln = MockIlnClient::new(&env, &iln_id);

        let gov_token_id = env.register_stellar_asset_contract_v2(dist_id.clone());
        let gov_token = gov_token_id.address();

        dist.initialize(&iln_id, &gov_token);

        let freelancer = Address::generate(&env);
        let payer = Address::generate(&env);

        // Default params: freelancer gets HALF_TOKEN per settlement
        iln.accrue_settlement(&dist_id, &freelancer, &payer, &true);
        assert_eq!(dist.get_accrual(&freelancer), HALF_TOKEN);
        assert_eq!(dist.get_accrual(&payer), HALF_TOKEN);

        // Update params: double the half_token to 10_000_000
        dist.update_reward_params(&10_000_000, &1_000_000_000, &10_000_000);

        // New accruals use the updated params
        let freelancer2 = Address::generate(&env);
        let payer2 = Address::generate(&env);
        iln.accrue_settlement(&dist_id, &freelancer2, &payer2, &true);
        assert_eq!(dist.get_accrual(&freelancer2), 10_000_000);
        assert_eq!(dist.get_accrual(&payer2), 10_000_000);
    }

    #[test]
    fn update_reward_params_affects_lp_rewards() {
        let env = Env::default();
        env.mock_all_auths();

        let iln_id = env.register_contract(None, MockIln);
        let dist_id = env.register_contract(None, IlnDistribution);
        let dist = IlnDistributionClient::new(&env, &dist_id);
        let iln = MockIlnClient::new(&env, &iln_id);

        let gov_token_id = env.register_stellar_asset_contract_v2(dist_id.clone());
        let gov_token = gov_token_id.address();

        dist.initialize(&iln_id, &gov_token);

        let lp = Address::generate(&env);

        // Default: 100 USDC → 10_000_000 tokens
        iln.accrue_lp(&dist_id, &lp, &HUNDRED_USDC_STROOPS);
        assert_eq!(dist.get_accrual(&lp), 10_000_000);

        // Update LP multiplier to 20_000_000
        dist.update_reward_params(&5_000_000, &1_000_000_000, &20_000_000);

        let lp2 = Address::generate(&env);
        iln.accrue_lp(&dist_id, &lp2, &HUNDRED_USDC_STROOPS);
        assert_eq!(dist.get_accrual(&lp2), 20_000_000);
    }
}
