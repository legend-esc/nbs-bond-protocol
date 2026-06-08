#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, BytesN, Env, IntoVal, Symbol, Vec};
use nbbs_shared::{BondError, OracleReport};

pub const FIXED_POINT: i128 = 10_000_000;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    PeriodInfo(u64, u32),
    PeriodCount(u64),
    AccruedCredits(u64, Address),
    BondProject(u64),
    Precision,
    BondIssuerAddress,
    OracleConsumerAddress,
    Nonce(Address),
}

#[derive(Clone)]
#[contracttype]
pub struct PeriodInfo {
    pub period_index: u32,
    pub start_time: u64,
    pub end_time: u64,
    pub total_credits_earned: i128,
    pub distributed: bool,
    pub report_id: u64,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct CouponResult {
    pub bond_id: u64,
    pub period_index: u32,
    pub total_credits: i128,
    pub holder_count: u32,
    pub credits_per_token: i128,
}

#[contract]
pub struct CouponEngine;

#[contractimpl]
impl CouponEngine {
    pub fn __constructor(
        env: Env,
        admin: Address,
        bond_issuer_address: Address,
        oracle_consumer_address: Address,
    ) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::BondIssuerAddress, &bond_issuer_address);
        env.storage()
            .instance()
            .set(&DataKey::OracleConsumerAddress, &oracle_consumer_address);
        env.storage().instance().set(&DataKey::Precision, &FIXED_POINT);
    }

    pub fn register_bond(
        env: Env,
        caller: Address,
        bond_id: u64,
        project_id: BytesN<32>,
        nonce: u64,
    ) -> Result<(), BondError> {
        caller.require_auth();

        let expected_nonce = get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(BondError::InvalidNonce);
        }
        set_nonce(&env, &caller, expected_nonce + 1);

        require_admin(&env, &caller)?;

        env.storage()
            .instance()
            .set(&DataKey::BondProject(bond_id), &project_id);

        env.events().publish(
            (Symbol::new(&env, "bond_registered"),),
            (bond_id, project_id),
        );

        Ok(())
    }

    pub fn distribute_coupon(
        env: Env,
        caller: Address,
        bond_id: u64,
        period_index: u32,
        holders: Vec<Address>,
        report: OracleReport,
        nonce: u64,
    ) -> Result<CouponResult, BondError> {
        caller.require_auth();

        let expected_nonce = get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(BondError::InvalidNonce);
        }
        set_nonce(&env, &caller, expected_nonce + 1);

        let project_id: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::BondProject(bond_id))
            .ok_or(BondError::BondNotFound)?;

        if report.project_id != project_id {
            return Err(BondError::BondNotFound);
        }

        let existing: Option<PeriodInfo> = env
            .storage()
            .persistent()
            .get(&DataKey::PeriodInfo(bond_id, period_index));
        if let Some(info) = existing {
            if info.distributed {
                return Err(BondError::Overflow);
            }
        }

        let total_credits = report.carbon_sequestered / 1000;

        let bond_issuer: Address = env
            .storage()
            .instance()
            .get(&DataKey::BondIssuerAddress)
            .expect("bond issuer not set");

        let total_subscribed: i128 = env.invoke_contract(
            &bond_issuer,
            &Symbol::new(&env, "total_subscribed"),
            vec![&env, bond_id.into_val(&env)],
        );

        let mut total_holder_credits: i128 = 0;
        let mut holder_count: u32 = 0;

        let credits_per_token = if total_subscribed > 0 && total_credits > 0 {
            total_credits * FIXED_POINT / total_subscribed
        } else {
            0
        };

        for holder in holders.iter() {
            let balance: i128 = env.invoke_contract(
                &bond_issuer,
                &Symbol::new(&env, "get_holder_balance"),
                vec![&env, bond_id.into_val(&env), holder.clone().into_val(&env)],
            );

            if balance > 0 {
                let holder_credits = credits_per_token * balance / FIXED_POINT;
                if holder_credits > 0 {
                    total_holder_credits = total_holder_credits
                        .checked_add(holder_credits)
                        .ok_or(BondError::Overflow)?;

                    let key = DataKey::AccruedCredits(bond_id, holder.clone());
                    let accrued: i128 = env.storage().persistent().get(&key).unwrap_or(0);
                    env.storage()
                        .persistent()
                        .set(&key, &(accrued + holder_credits));
                    holder_count += 1;
                }
            }
        }

        let period_info = PeriodInfo {
            period_index,
            start_time: report.period_start,
            end_time: report.period_end,
            total_credits_earned: total_holder_credits,
            distributed: true,
            report_id: period_index as u64,
        };
        env.storage()
            .persistent()
            .set(&DataKey::PeriodInfo(bond_id, period_index), &period_info);

        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::PeriodCount(bond_id))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::PeriodCount(bond_id), &(count + 1));

        env.events().publish(
            (Symbol::new(&env, "coupon_distributed"),),
            (bond_id, period_index, total_holder_credits, holder_count),
        );

        Ok(CouponResult {
            bond_id,
            period_index,
            total_credits: total_holder_credits,
            holder_count,
            credits_per_token,
        })
    }

    pub fn accrued_credits(env: Env, bond_id: u64, holder: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::AccruedCredits(bond_id, holder))
            .unwrap_or(0)
    }

    pub fn claim_credits(
        env: Env,
        caller: Address,
        bond_id: u64,
        nonce: u64,
    ) -> Result<i128, BondError> {
        caller.require_auth();

        let expected_nonce = get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(BondError::InvalidNonce);
        }
        set_nonce(&env, &caller, expected_nonce + 1);

        let key = DataKey::AccruedCredits(bond_id, caller.clone());
        let accrued: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &0i128);

        env.events().publish(
            (Symbol::new(&env, "credits_claimed"),),
            (bond_id, caller, accrued),
        );

        Ok(accrued)
    }

    pub fn get_period_info(
        env: Env,
        bond_id: u64,
        period_index: u32,
    ) -> Result<PeriodInfo, BondError> {
        env.storage()
            .persistent()
            .get(&DataKey::PeriodInfo(bond_id, period_index))
            .ok_or(BondError::BondNotFound)
    }

    pub fn get_period_count(env: Env, bond_id: u64) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::PeriodCount(bond_id))
            .unwrap_or(0)
    }
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), BondError> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(BondError::NotInitialized)?;
    if caller != &admin {
        return Err(BondError::Unauthorized);
    }
    Ok(())
}

fn get_nonce(env: &Env, addr: &Address) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::Nonce(addr.clone()))
        .unwrap_or(0)
}

fn set_nonce(env: &Env, addr: &Address, nonce: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::Nonce(addr.clone()), &nonce);
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::Address as _, vec, BytesN, Env, Symbol,
    };

    fn create_project_id(env: &Env, value: u8) -> BytesN<32> {
        let mut arr = [0u8; 32];
        arr[31] = value;
        BytesN::from_array(env, &arr)
    }

    fn make_report(env: &Env, project_id: BytesN<32>, carbon: i128) -> OracleReport {
        OracleReport {
            project_id,
            period_start: 1000,
            period_end: 2000,
            carbon_sequestered: carbon,
            methodology: Symbol::new(env, "verra_vcs"),
            provider_signature: BytesN::from_array(env, &[0u8; 64]),
            ipfs_evidence_hash: BytesN::from_array(env, &[0u8; 32]),
        }
    }

    #[test]
    fn test_constructor_and_register_bond() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let oracle = Address::generate(&env);

        let contract_id = env.register(
            CouponEngine,
            (admin.clone(), issuer.clone(), oracle.clone()),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        let project_id = create_project_id(&env, 42);
        client.register_bond(&admin, &1, &project_id, &0);

        let count = client.get_period_count(&1);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_register_bond_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let oracle = Address::generate(&env);
        let user = Address::generate(&env);

        let contract_id = env.register(
            CouponEngine,
            (admin.clone(), issuer.clone(), oracle.clone()),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        let project_id = create_project_id(&env, 42);
        let result = client.try_register_bond(&user, &1, &project_id, &0);
        assert_eq!(result, Err(Ok(BondError::Unauthorized)));
    }

    #[test]
    fn test_register_bond_invalid_nonce() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let oracle = Address::generate(&env);

        let contract_id = env.register(
            CouponEngine,
            (admin.clone(), issuer.clone(), oracle.clone()),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        let project_id = create_project_id(&env, 42);
        let result = client.try_register_bond(&admin, &1, &project_id, &1);
        assert_eq!(result, Err(Ok(BondError::InvalidNonce)));
    }

    #[test]
    fn test_distribute_to_single_holder() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let issuer_admin = Address::generate(&env);
        let issuer_id = env.register(
            nbbs_bond_issuer::BondIssuer,
            (issuer_admin.clone(),),
        );
        let issuer_client = nbbs_bond_issuer::BondIssuerClient::new(&env, &issuer_id);

        let bond_config = nbbs_shared::BondConfig {
            project_id: project_id.clone(),
            face_value: 1000,
            coupon_schedule: vec![&env, 1000000u64, 2000000u64],
            credit_type: nbbs_shared::CreditType::Carbon,
            maturity_date: 3000000,
            total_supply: 10000,
        };

        let bond_id = issuer_client.issue_bond(&issuer_admin, &bond_config, &0);
        assert_eq!(bond_id, 1);

        let holder = Address::generate(&env);
        issuer_client.subscribe(&holder, &bond_id, &10000, &0);

        let contract_id = env.register(
            CouponEngine,
            (admin.clone(), issuer_id.clone(), oracle.clone()),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        client.register_bond(&admin, &bond_id, &project_id, &0);

        let report = make_report(&env, project_id, 100_000);
        let holders = vec![&env, holder.clone()];

        let result = client.distribute_coupon(
            &admin,
            &bond_id,
            &0,
            &holders,
            &report,
            &1,
        );

        assert_eq!(result.bond_id, bond_id);
        assert_eq!(result.period_index, 0);
        assert_eq!(result.total_credits, 100);
        assert_eq!(result.holder_count, 1);
        assert_eq!(result.credits_per_token, 100 * FIXED_POINT / 10000);

        let accrued = client.accrued_credits(&bond_id, &holder);
        assert_eq!(accrued, 100);

        let period_info = client.get_period_info(&bond_id, &0);
        assert!(period_info.distributed);
        assert_eq!(period_info.total_credits_earned, 100);
    }

    #[test]
    fn test_distribute_pro_rata_multiple_holders() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let issuer_admin = Address::generate(&env);
        let issuer_id = env.register(
            nbbs_bond_issuer::BondIssuer,
            (issuer_admin.clone(),),
        );
        let issuer_client = nbbs_bond_issuer::BondIssuerClient::new(&env, &issuer_id);

        let bond_config = nbbs_shared::BondConfig {
            project_id: project_id.clone(),
            face_value: 1000,
            coupon_schedule: vec![&env, 1000000u64, 2000000u64],
            credit_type: nbbs_shared::CreditType::Carbon,
            maturity_date: 3000000,
            total_supply: 10000,
        };

        let bond_id = issuer_client.issue_bond(&issuer_admin, &bond_config, &0);

        let holder1 = Address::generate(&env);
        let holder2 = Address::generate(&env);
        issuer_client.subscribe(&holder1, &bond_id, &3000, &0);
        issuer_client.subscribe(&holder2, &bond_id, &7000, &0);

        let contract_id = env.register(
            CouponEngine,
            (admin.clone(), issuer_id.clone(), oracle.clone()),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        client.register_bond(&admin, &bond_id, &project_id, &0);

        let report = make_report(&env, project_id, 100_000);
        let holders = vec![&env, holder1.clone(), holder2.clone()];

        let result = client.distribute_coupon(
            &admin,
            &bond_id,
            &0,
            &holders,
            &report,
            &1,
        );

        assert_eq!(result.total_credits, 100);
        assert_eq!(result.holder_count, 2);

        let total_sub = 10000i128;
        let credits_per_token = 100 * FIXED_POINT / total_sub;
        let expected_h1 = credits_per_token * 3000 / FIXED_POINT;
        let expected_h2 = credits_per_token * 7000 / FIXED_POINT;

        assert_eq!(client.accrued_credits(&bond_id, &holder1), expected_h1);
        assert_eq!(client.accrued_credits(&bond_id, &holder2), expected_h2);
        assert_eq!(expected_h1 + expected_h2, 100);
    }

    #[test]
    fn test_distribute_zero_sequestration() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let issuer_admin = Address::generate(&env);
        let issuer_id = env.register(
            nbbs_bond_issuer::BondIssuer,
            (issuer_admin.clone(),),
        );
        let issuer_client = nbbs_bond_issuer::BondIssuerClient::new(&env, &issuer_id);

        let bond_config = nbbs_shared::BondConfig {
            project_id: project_id.clone(),
            face_value: 1000,
            coupon_schedule: vec![&env, 1000000u64, 2000000u64],
            credit_type: nbbs_shared::CreditType::Carbon,
            maturity_date: 3000000,
            total_supply: 10000,
        };

        let bond_id = issuer_client.issue_bond(&issuer_admin, &bond_config, &0);

        let holder = Address::generate(&env);
        issuer_client.subscribe(&holder, &bond_id, &10000, &0);

        let contract_id = env.register(
            CouponEngine,
            (admin.clone(), issuer_id.clone(), oracle.clone()),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        client.register_bond(&admin, &bond_id, &project_id, &0);

        let report = make_report(&env, project_id, 0);
        let holders = vec![&env, holder.clone()];

        let result = client.distribute_coupon(
            &admin,
            &bond_id,
            &0,
            &holders,
            &report,
            &1,
        );

        assert_eq!(result.total_credits, 0);
        assert_eq!(result.holder_count, 0);
        assert_eq!(result.credits_per_token, 0);

        let accrued = client.accrued_credits(&bond_id, &holder);
        assert_eq!(accrued, 0);
    }

    #[test]
    fn test_prevent_double_distribute() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let issuer_admin = Address::generate(&env);
        let issuer_id = env.register(
            nbbs_bond_issuer::BondIssuer,
            (issuer_admin.clone(),),
        );
        let issuer_client = nbbs_bond_issuer::BondIssuerClient::new(&env, &issuer_id);

        let bond_config = nbbs_shared::BondConfig {
            project_id: project_id.clone(),
            face_value: 1000,
            coupon_schedule: vec![&env, 1000000u64, 2000000u64],
            credit_type: nbbs_shared::CreditType::Carbon,
            maturity_date: 3000000,
            total_supply: 10000,
        };

        let bond_id = issuer_client.issue_bond(&issuer_admin, &bond_config, &0);

        let holder = Address::generate(&env);
        issuer_client.subscribe(&holder, &bond_id, &10000, &0);

        let contract_id = env.register(
            CouponEngine,
            (admin.clone(), issuer_id.clone(), oracle.clone()),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        client.register_bond(&admin, &bond_id, &project_id, &0);

        let report = make_report(&env, project_id, 100_000);
        let holders = vec![&env, holder.clone()];

        client.distribute_coupon(&admin, &bond_id, &0, &holders, &report, &1);

        let result = client.try_distribute_coupon(
            &admin,
            &bond_id,
            &0,
            &holders,
            &report,
            &2,
        );
        assert_eq!(result, Err(Ok(BondError::Overflow)));
    }

    #[test]
    fn test_distribute_unregistered_bond() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let oracle = Address::generate(&env);

        let contract_id = env.register(
            CouponEngine,
            (admin.clone(), issuer.clone(), oracle.clone()),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        let project_id = create_project_id(&env, 1);
        let report = make_report(&env, project_id, 100_000);
        let holders = vec![&env];

        let result = client.try_distribute_coupon(
            &admin,
            &999,
            &0,
            &holders,
            &report,
            &0,
        );
        assert_eq!(result, Err(Ok(BondError::BondNotFound)));
    }

    #[test]
    fn test_claim_credits() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let issuer_admin = Address::generate(&env);
        let issuer_id = env.register(
            nbbs_bond_issuer::BondIssuer,
            (issuer_admin.clone(),),
        );
        let issuer_client = nbbs_bond_issuer::BondIssuerClient::new(&env, &issuer_id);

        let bond_config = nbbs_shared::BondConfig {
            project_id: project_id.clone(),
            face_value: 1000,
            coupon_schedule: vec![&env, 1000000u64, 2000000u64],
            credit_type: nbbs_shared::CreditType::Carbon,
            maturity_date: 3000000,
            total_supply: 10000,
        };

        let bond_id = issuer_client.issue_bond(&issuer_admin, &bond_config, &0);

        let holder = Address::generate(&env);
        issuer_client.subscribe(&holder, &bond_id, &10000, &0);

        let contract_id = env.register(
            CouponEngine,
            (admin.clone(), issuer_id.clone(), oracle.clone()),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        client.register_bond(&admin, &bond_id, &project_id, &0);

        let report = make_report(&env, project_id, 100_000);
        let holders = vec![&env, holder.clone()];
        client.distribute_coupon(&admin, &bond_id, &0, &holders, &report, &1);

        let claimed = client.claim_credits(&holder, &bond_id, &0);
        assert_eq!(claimed, 100);

        let accrued = client.accrued_credits(&bond_id, &holder);
        assert_eq!(accrued, 0);
    }

    #[test]
    fn test_zero_holders_case() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let issuer_admin = Address::generate(&env);
        let issuer_id = env.register(
            nbbs_bond_issuer::BondIssuer,
            (issuer_admin.clone(),),
        );
        let issuer_client = nbbs_bond_issuer::BondIssuerClient::new(&env, &issuer_id);

        let bond_config = nbbs_shared::BondConfig {
            project_id: project_id.clone(),
            face_value: 1000,
            coupon_schedule: vec![&env, 1000000u64, 2000000u64],
            credit_type: nbbs_shared::CreditType::Carbon,
            maturity_date: 3000000,
            total_supply: 10000,
        };

        let bond_id = issuer_client.issue_bond(&issuer_admin, &bond_config, &0);

        let contract_id = env.register(
            CouponEngine,
            (admin.clone(), issuer_id.clone(), oracle.clone()),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        client.register_bond(&admin, &bond_id, &project_id, &0);

        let report = make_report(&env, project_id, 100_000);
        let holders = vec![&env];

        let result = client.distribute_coupon(
            &admin,
            &bond_id,
            &0,
            &holders,
            &report,
            &1,
        );

        assert_eq!(result.total_credits, 0);
        assert_eq!(result.holder_count, 0);
        assert!(result.credits_per_token >= 0);

        let period_info = client.get_period_info(&bond_id, &0);
        assert!(period_info.distributed);
    }

    #[test]
    fn test_period_count_increments() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let issuer_admin = Address::generate(&env);
        let issuer_id = env.register(
            nbbs_bond_issuer::BondIssuer,
            (issuer_admin.clone(),),
        );
        let issuer_client = nbbs_bond_issuer::BondIssuerClient::new(&env, &issuer_id);

        let bond_config = nbbs_shared::BondConfig {
            project_id: project_id.clone(),
            face_value: 1000,
            coupon_schedule: vec![&env, 1000000u64, 2000000u64],
            credit_type: nbbs_shared::CreditType::Carbon,
            maturity_date: 3000000,
            total_supply: 10000,
        };

        let bond_id = issuer_client.issue_bond(&issuer_admin, &bond_config, &0);

        let holder = Address::generate(&env);
        issuer_client.subscribe(&holder, &bond_id, &10000, &0);

        let contract_id = env.register(
            CouponEngine,
            (admin.clone(), issuer_id.clone(), oracle.clone()),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        client.register_bond(&admin, &bond_id, &project_id, &0);

        assert_eq!(client.get_period_count(&bond_id), 0);

        let report = make_report(&env, project_id.clone(), 100_000);
        let holders = vec![&env, holder.clone()];

        client.distribute_coupon(&admin, &bond_id, &0, &holders, &report, &1);
        assert_eq!(client.get_period_count(&bond_id), 1);

        let report2 = make_report(&env, project_id, 200_000);
        client.distribute_coupon(&admin, &bond_id, &1, &holders, &report2, &2);
        assert_eq!(client.get_period_count(&bond_id), 2);
    }

    #[test]
    fn test_query_accrued_credits_zero() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let oracle = Address::generate(&env);

        let contract_id = env.register(
            CouponEngine,
            (admin, issuer, oracle),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        let holder = Address::generate(&env);
        let accrued = client.accrued_credits(&1, &holder);
        assert_eq!(accrued, 0);
    }

    #[test]
    fn test_claim_credits_invalid_nonce() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let oracle = Address::generate(&env);

        let contract_id = env.register(
            CouponEngine,
            (admin, issuer, oracle),
        );
        let client = CouponEngineClient::new(&env, &contract_id);

        let holder = Address::generate(&env);
        let result = client.try_claim_credits(&holder, &1, &1);
        assert_eq!(result, Err(Ok(BondError::InvalidNonce)));
    }
}
