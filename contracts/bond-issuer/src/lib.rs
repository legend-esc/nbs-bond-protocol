#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, Symbol};
use nbbs_shared::{BondConfig, BondError, BondStatus};

pub const MAX_SUPPLY: i128 = 1_000_000_000_000_000_000;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    BondConfig(u64),
    BondState(u64),
    HolderBalance(u64, Address),
    BondCount,
    Nonce(Address),
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct BondState {
    pub total_subscribed: i128,
    pub status: BondStatus,
    pub created_at: u64,
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

#[contract]
pub struct BondIssuer;

#[contractimpl]
impl BondIssuer {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn issue_bond(
        env: Env,
        caller: Address,
        config: BondConfig,
        nonce: u64,
    ) -> Result<u64, BondError> {
        caller.require_auth();

        let expected_nonce: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Nonce(caller.clone()))
            .unwrap_or(0);
        if nonce != expected_nonce {
            return Err(BondError::InvalidNonce);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Nonce(caller.clone()), &(expected_nonce + 1));

        require_admin(&env, &caller)?;

        if config.face_value <= 0 {
            return Err(BondError::ZeroAmount);
        }
        if config.total_supply <= 0 {
            return Err(BondError::ZeroAmount);
        }
        if config.maturity_date <= env.ledger().timestamp() {
            return Err(BondError::Overflow);
        }

        let schedule_len = config.coupon_schedule.len();
        if schedule_len == 0 {
            return Err(BondError::ZeroAmount);
        }
        for i in 0..schedule_len {
            let coupon_date = config.coupon_schedule.get(i).unwrap();
            if coupon_date >= config.maturity_date {
                return Err(BondError::ZeroAmount);
            }
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::BondCount)
            .unwrap_or(0);
        let bond_id = count + 1;
        env.storage()
            .instance()
            .set(&DataKey::BondCount, &bond_id);

        env.storage()
            .instance()
            .set(&DataKey::BondConfig(bond_id), &config);

        let state = BondState {
            total_subscribed: 0,
            status: BondStatus::Active,
            created_at: env.ledger().timestamp(),
        };
        env.storage()
            .instance()
            .set(&DataKey::BondState(bond_id), &state);

        env.events().publish(
            (Symbol::new(&env, "bond_issued"),),
            (bond_id, config.project_id),
        );

        Ok(bond_id)
    }

    pub fn subscribe(
        env: Env,
        investor: Address,
        bond_id: u64,
        amount: i128,
        nonce: u64,
    ) -> Result<(), BondError> {
        investor.require_auth();

        let expected_nonce: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Nonce(investor.clone()))
            .unwrap_or(0);
        if nonce != expected_nonce {
            return Err(BondError::InvalidNonce);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Nonce(investor.clone()), &(expected_nonce + 1));

        if amount <= 0 {
            return Err(BondError::ZeroAmount);
        }

        let config: BondConfig = env
            .storage()
            .instance()
            .get(&DataKey::BondConfig(bond_id))
            .ok_or(BondError::BondNotFound)?;

        let mut state: BondState = env
            .storage()
            .instance()
            .get(&DataKey::BondState(bond_id))
            .ok_or(BondError::BondNotFound)?;

        if state.status != BondStatus::Active {
            return Err(BondError::BondAlreadyMatured);
        }

        let new_total = state
            .total_subscribed
            .checked_add(amount)
            .ok_or(BondError::Overflow)?;
        if new_total > config.total_supply {
            return Err(BondError::InsufficientSupply);
        }

        let balance_key = DataKey::HolderBalance(bond_id, investor.clone());
        let current_balance: i128 = env
            .storage()
            .persistent()
            .get(&balance_key)
            .unwrap_or(0);
        let new_balance = current_balance
            .checked_add(amount)
            .ok_or(BondError::Overflow)?;
        env.storage()
            .persistent()
            .set(&balance_key, &new_balance);

        state.total_subscribed = new_total;
        env.storage()
            .instance()
            .set(&DataKey::BondState(bond_id), &state);

        env.events().publish(
            (Symbol::new(&env, "subscribed"),),
            (bond_id, investor, amount),
        );

        Ok(())
    }

    pub fn redeem(
        env: Env,
        holder: Address,
        bond_id: u64,
        amount: i128,
        nonce: u64,
    ) -> Result<(), BondError> {
        holder.require_auth();

        let expected_nonce: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Nonce(holder.clone()))
            .unwrap_or(0);
        if nonce != expected_nonce {
            return Err(BondError::InvalidNonce);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Nonce(holder.clone()), &(expected_nonce + 1));

        if amount <= 0 {
            return Err(BondError::ZeroAmount);
        }

        let mut state: BondState = env
            .storage()
            .instance()
            .get(&DataKey::BondState(bond_id))
            .ok_or(BondError::BondNotFound)?;

        if state.status != BondStatus::Matured {
            return Err(BondError::BondAlreadyMatured);
        }

        let balance_key = DataKey::HolderBalance(bond_id, holder.clone());
        let current_balance: i128 = env
            .storage()
            .persistent()
            .get(&balance_key)
            .unwrap_or(0);
        if current_balance < amount {
            return Err(BondError::InsufficientSupply);
        }

        let new_balance = current_balance
            .checked_sub(amount)
            .ok_or(BondError::Overflow)?;
        env.storage()
            .persistent()
            .set(&balance_key, &new_balance);

        state.total_subscribed = state
            .total_subscribed
            .checked_sub(amount)
            .ok_or(BondError::Overflow)?;
        env.storage()
            .instance()
            .set(&DataKey::BondState(bond_id), &state);

        env.events().publish(
            (Symbol::new(&env, "redeemed"),),
            (bond_id, holder, amount),
        );

        Ok(())
    }

    pub fn get_bond(env: Env, bond_id: u64) -> Result<BondConfig, BondError> {
        env.storage()
            .instance()
            .get(&DataKey::BondConfig(bond_id))
            .ok_or(BondError::BondNotFound)
    }

    pub fn get_bond_state(env: Env, bond_id: u64) -> Result<BondState, BondError> {
        env.storage()
            .instance()
            .get(&DataKey::BondState(bond_id))
            .ok_or(BondError::BondNotFound)
    }

    pub fn get_holder_balance(env: Env, bond_id: u64, holder: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::HolderBalance(bond_id, holder))
            .unwrap_or(0)
    }

    pub fn total_supply(env: Env, bond_id: u64) -> Result<i128, BondError> {
        let config: BondConfig = env
            .storage()
            .instance()
            .get(&DataKey::BondConfig(bond_id))
            .ok_or(BondError::BondNotFound)?;
        Ok(config.total_supply)
    }

    pub fn total_subscribed(env: Env, bond_id: u64) -> Result<i128, BondError> {
        let state: BondState = env
            .storage()
            .instance()
            .get(&DataKey::BondState(bond_id))
            .ok_or(BondError::BondNotFound)?;
        Ok(state.total_subscribed)
    }

    pub fn mature_bond(
        env: Env,
        caller: Address,
        bond_id: u64,
        nonce: u64,
    ) -> Result<(), BondError> {
        caller.require_auth();

        let expected_nonce: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Nonce(caller.clone()))
            .unwrap_or(0);
        if nonce != expected_nonce {
            return Err(BondError::InvalidNonce);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Nonce(caller.clone()), &(expected_nonce + 1));

        require_admin(&env, &caller)?;

        let mut state: BondState = env
            .storage()
            .instance()
            .get(&DataKey::BondState(bond_id))
            .ok_or(BondError::BondNotFound)?;

        if state.status != BondStatus::Active {
            return Err(BondError::BondAlreadyMatured);
        }

        state.status = BondStatus::Matured;
        env.storage()
            .instance()
            .set(&DataKey::BondState(bond_id), &state);

        env.events().publish(
            (Symbol::new(&env, "bond_matured"),),
            (bond_id,),
        );

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, BytesN};

    fn create_project_id(env: &Env, value: u8) -> BytesN<32> {
        let mut arr = [0u8; 32];
        arr[31] = value;
        BytesN::from_array(env, &arr)
    }

    fn make_config(env: &Env) -> BondConfig {
        BondConfig {
            project_id: create_project_id(env, 1),
            face_value: 1000,
            coupon_schedule: vec![&env, 1000000u64, 2000000u64],
            credit_type: nbbs_shared::CreditType::Carbon,
            maturity_date: 3000000,
            total_supply: 10000,
        }
    }

    fn setup() -> (Env, BondIssuerClient<'static>, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let contract_id = env.register(BondIssuer, (&admin,));
        let client = BondIssuerClient::new(&env, &contract_id);
        (env, client, admin, user)
    }

    #[test]
    fn test_issue_bond() {
        let (env, client, admin, _user) = setup();
        let config = make_config(&env);

        let bond_id = client.issue_bond(&admin, &config, &0);
        assert_eq!(bond_id, 1);

        let stored = client.get_bond(&bond_id);
        assert_eq!(stored.face_value, 1000);
        assert_eq!(stored.total_supply, 10000);
        assert_eq!(stored.maturity_date, 3000000);

        let state = client.get_bond_state(&bond_id);
        assert_eq!(state.total_subscribed, 0);
        assert_eq!(state.status, BondStatus::Active);
    }

    #[test]
    fn test_issue_bond_past_maturity() {
        let (env, client, admin, _user) = setup();
        env.ledger().set_timestamp(1000);
        let mut config = make_config(&env);
        config.maturity_date = 500;

        let result = client.try_issue_bond(&admin, &config, &0);
        assert_eq!(result, Err(Ok(BondError::Overflow)));
    }

    #[test]
    fn test_issue_bond_empty_schedule() {
        let (env, client, admin, _user) = setup();
        let mut config = make_config(&env);
        config.coupon_schedule = vec![&env];

        let result = client.try_issue_bond(&admin, &config, &0);
        assert_eq!(result, Err(Ok(BondError::ZeroAmount)));
    }

    #[test]
    fn test_subscribe_partial() {
        let (env, client, admin, user) = setup();
        let config = make_config(&env);
        let bond_id = client.issue_bond(&admin, &config, &0);

        client.subscribe(&user, &bond_id, &500, &0);

        let state = client.get_bond_state(&bond_id);
        assert_eq!(state.total_subscribed, 500);

        let balance = client.get_holder_balance(&bond_id, &user);
        assert_eq!(balance, 500);
    }

    #[test]
    fn test_subscribe_full() {
        let (env, client, admin, user) = setup();
        let config = make_config(&env);
        let bond_id = client.issue_bond(&admin, &config, &0);

        client.subscribe(&user, &bond_id, &10000, &0);

        let state = client.get_bond_state(&bond_id);
        assert_eq!(state.total_subscribed, 10000);
    }

    #[test]
    fn test_subscribe_exceeds_supply() {
        let (env, client, admin, user) = setup();
        let config = make_config(&env);
        let bond_id = client.issue_bond(&admin, &config, &0);

        let result = client.try_subscribe(&user, &bond_id, &10001, &0);
        assert_eq!(result, Err(Ok(BondError::InsufficientSupply)));
    }

    #[test]
    fn test_subscribe_zero_amount() {
        let (env, client, admin, user) = setup();
        let config = make_config(&env);
        let bond_id = client.issue_bond(&admin, &config, &0);

        let result = client.try_subscribe(&user, &bond_id, &0, &0);
        assert_eq!(result, Err(Ok(BondError::ZeroAmount)));
    }

    #[test]
    fn test_subscribe_non_existent_bond() {
        let (_env, client, _admin, user) = setup();
        let result = client.try_subscribe(&user, &999, &500, &0);
        assert_eq!(result, Err(Ok(BondError::BondNotFound)));
    }

    #[test]
    fn test_mature_bond() {
        let (env, client, admin, user) = setup();
        let config = make_config(&env);
        let bond_id = client.issue_bond(&admin, &config, &0);

        client.subscribe(&user, &bond_id, &5000, &0);
        client.mature_bond(&admin, &bond_id, &1);

        let state = client.get_bond_state(&bond_id);
        assert_eq!(state.status, BondStatus::Matured);
    }

    #[test]
    fn test_redeem_after_maturity() {
        let (env, client, admin, user) = setup();
        let config = make_config(&env);
        let bond_id = client.issue_bond(&admin, &config, &0);

        client.subscribe(&user, &bond_id, &3000, &0);
        client.mature_bond(&admin, &bond_id, &1);

        client.redeem(&user, &bond_id, &1000, &1);

        let balance = client.get_holder_balance(&bond_id, &user);
        assert_eq!(balance, 2000);

        let state = client.get_bond_state(&bond_id);
        assert_eq!(state.total_subscribed, 2000);
    }

    #[test]
    fn test_redeem_before_maturity() {
        let (env, client, admin, user) = setup();
        let config = make_config(&env);
        let bond_id = client.issue_bond(&admin, &config, &0);

        client.subscribe(&user, &bond_id, &3000, &0);

        let result = client.try_redeem(&user, &bond_id, &1000, &1);
        assert_eq!(result, Err(Ok(BondError::BondAlreadyMatured)));
    }

    #[test]
    fn test_redeem_more_than_owned() {
        let (env, client, admin, user) = setup();
        let config = make_config(&env);
        let bond_id = client.issue_bond(&admin, &config, &0);

        client.subscribe(&user, &bond_id, &1000, &0);
        client.mature_bond(&admin, &bond_id, &1);

        let result = client.try_redeem(&user, &bond_id, &2000, &1);
        assert_eq!(result, Err(Ok(BondError::InsufficientSupply)));
    }

    #[test]
    fn test_invalid_nonce() {
        let (env, client, admin, _user) = setup();
        let config = make_config(&env);

        let result = client.try_issue_bond(&admin, &config, &1);
        assert_eq!(result, Err(Ok(BondError::InvalidNonce)));
    }

    #[test]
    fn test_unauthorized() {
        let (env, client, _admin, user) = setup();
        let config = make_config(&env);

        let result = client.try_issue_bond(&user, &config, &0);
        assert_eq!(result, Err(Ok(BondError::Unauthorized)));
    }

    #[test]
    fn test_multiple_investors() {
        let (env, client, admin, user) = setup();
        let user2 = Address::generate(&env);
        let config = make_config(&env);
        let bond_id = client.issue_bond(&admin, &config, &0);

        client.subscribe(&user, &bond_id, &2000, &0);
        client.subscribe(&user2, &bond_id, &3000, &0);

        assert_eq!(client.get_holder_balance(&bond_id, &user), 2000);
        assert_eq!(client.get_holder_balance(&bond_id, &user2), 3000);

        let state = client.get_bond_state(&bond_id);
        assert_eq!(state.total_subscribed, 5000);
    }

    #[test]
    fn test_total_supply_and_subscribed() {
        let (env, client, admin, user) = setup();
        let config = make_config(&env);
        let bond_id = client.issue_bond(&admin, &config, &0);

        assert_eq!(client.total_supply(&bond_id), 10000);
        assert_eq!(client.total_subscribed(&bond_id), 0);

        client.subscribe(&user, &bond_id, &4000, &0);
        assert_eq!(client.total_subscribed(&bond_id), 4000);
    }
}
