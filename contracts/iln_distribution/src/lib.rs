#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, token::StellarAssetClient, Address, Env};

const HALF_TOKEN: i128 = 5_000_000;
const HUNDRED_USDC_STROOPS: i128 = 1_000_000_000;

#[contracttype]
pub enum StorageKey {
    Initialized,
    IlnContract,
    GovToken,
    LpFundedVolume(Address),
    FreelancerSettled(Address),
    PayerOnTimeSettled(Address),
    Claimed(Address),
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
    }

    pub fn accrue_lp(env: Env, lp: Address, amount_usdc_equivalent: i128) {
        Self::require_iln_invoker(&env);

        if amount_usdc_equivalent <= 0 {
            return;
        }

        let key = StorageKey::LpFundedVolume(lp);
        let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&key, &(current + amount_usdc_equivalent));
    }

    pub fn accrue_settlement(env: Env, freelancer: Address, payer: Address, settled_on_time: bool) {
        Self::require_iln_invoker(&env);

        let freelancer_key = StorageKey::FreelancerSettled(freelancer);
        let freelancer_count: u64 = env
            .storage()
            .persistent()
            .get(&freelancer_key)
            .unwrap_or(0_u64);
        env.storage()
            .persistent()
            .set(&freelancer_key, &(freelancer_count + 1));

        if settled_on_time {
            let payer_key = StorageKey::PayerOnTimeSettled(payer);
            let payer_count: u64 = env.storage().persistent().get(&payer_key).unwrap_or(0_u64);
            env.storage()
                .persistent()
                .set(&payer_key, &(payer_count + 1));
        }
    }

    pub fn claim_tokens(env: Env, claimer: Address) -> i128 {
        claimer.require_auth();

        let total_earned = Self::total_earned(&env, &claimer);
        let claimed_key = StorageKey::Claimed(claimer.clone());
        let already_claimed: i128 = env.storage().persistent().get(&claimed_key).unwrap_or(0);

        let claimable = total_earned - already_claimed;
        if claimable <= 0 {
            return 0;
        }

        let gov_token: Address = env.storage().instance().get(&StorageKey::GovToken).unwrap();
        StellarAssetClient::new(&env, &gov_token).mint(&claimer, &claimable);

        env.storage()
            .persistent()
            .set(&claimed_key, &(already_claimed + claimable));

        claimable
    }

    pub fn get_accrual(env: Env, participant: Address) -> i128 {
        Self::total_earned(&env, &participant)
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

        let lp_reward = lp_volume / HUNDRED_USDC_STROOPS * 10_000_000;
        let freelancer_reward = (freelancer_settled as i128) * HALF_TOKEN;
        let payer_reward = (payer_on_time as i128) * HALF_TOKEN;

        lp_reward + freelancer_reward + payer_reward
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
}
