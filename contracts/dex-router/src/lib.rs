#![no_std]
#![allow(deprecated)]
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, IntoVal, Symbol, Vec};
use nbbs_shared::DEXError;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Order(u64),
    OrderCount,
    SellerOrders(Address),
    BondOrders(u64),
    BondIssuerAddress,
    CouponEngineAddress,
    Nonce(Address),
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct Order {
    pub id: u64,
    pub seller: Address,
    pub bond_id: u64,
    pub amount: i128,
    pub price_per_token: i128,
    pub quote_asset: Symbol,
    pub status: OrderStatus,
    pub created_at: u64,
    pub expires_at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[contracttype]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Expired,
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), DEXError> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(DEXError::NotInitialized)?;
    if caller != &admin {
        return Err(DEXError::Unauthorized);
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

fn is_order_expired(env: &Env, order: &Order) -> bool {
    env.ledger().timestamp() > order.expires_at
}

fn verify_holder_balance(
    env: &Env,
    holder: &Address,
    bond_id: u64,
    required: i128,
) -> Result<(), DEXError> {
    let bond_issuer: Address = env
        .storage()
        .instance()
        .get(&DataKey::BondIssuerAddress)
        .ok_or(DEXError::NotInitialized)?;

    let balance: i128 = env.invoke_contract(
        &bond_issuer,
        &Symbol::new(env, "get_holder_balance"),
        vec![
            &env,
            bond_id.into_val(env),
            holder.clone().into_val(env),
        ],
    );

    if balance < required {
        return Err(DEXError::InsufficientBalance);
    }
    Ok(())
}

#[contract]
pub struct DEXRouter;

#[allow(clippy::too_many_arguments)]
#[contractimpl]
impl DEXRouter {
    pub fn __constructor(
        env: Env,
        admin: Address,
        bond_issuer_address: Address,
        coupon_engine_address: Address,
    ) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::BondIssuerAddress, &bond_issuer_address);
        env.storage()
            .instance()
            .set(&DataKey::CouponEngineAddress, &coupon_engine_address);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn list_bond_tokens(
        env: Env,
        seller: Address,
        bond_id: u64,
        amount: i128,
        price_per_token: i128,
        quote_asset: Symbol,
        expires_after_seconds: u64,
        nonce: u64,
    ) -> Result<u64, DEXError> {
        seller.require_auth();

        let expected_nonce = get_nonce(&env, &seller);
        if nonce != expected_nonce {
            return Err(DEXError::InvalidNonce);
        }
        set_nonce(&env, &seller, expected_nonce + 1);

        if amount <= 0 || price_per_token <= 0 {
            return Err(DEXError::InsufficientBalance);
        }

        verify_holder_balance(&env, &seller, bond_id, amount)?;

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::OrderCount)
            .unwrap_or(0);
        let order_id = count + 1;
        env.storage()
            .instance()
            .set(&DataKey::OrderCount, &order_id);

        let now = env.ledger().timestamp();
        let order = Order {
            id: order_id,
            seller: seller.clone(),
            bond_id,
            amount,
            price_per_token,
            quote_asset,
            status: OrderStatus::Open,
            created_at: now,
            expires_at: now + expires_after_seconds,
        };

        env.storage()
            .instance()
            .set(&DataKey::Order(order_id), &order);

        let mut seller_orders: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::SellerOrders(seller.clone()))
            .unwrap_or(vec![&env]);
        seller_orders.push_back(order_id);
        env.storage()
            .instance()
            .set(&DataKey::SellerOrders(seller.clone()), &seller_orders);

        let mut bond_orders: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::BondOrders(bond_id))
            .unwrap_or(vec![&env]);
        bond_orders.push_back(order_id);
        env.storage()
            .instance()
            .set(&DataKey::BondOrders(bond_id), &bond_orders);

        env.events().publish(
            (Symbol::new(&env, "order_listed"),),
            (order_id, seller, bond_id, amount, price_per_token),
        );

        Ok(order_id)
    }

    pub fn cancel_listing(
        env: Env,
        caller: Address,
        order_id: u64,
        nonce: u64,
    ) -> Result<(), DEXError> {
        caller.require_auth();

        let expected_nonce = get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(DEXError::InvalidNonce);
        }
        set_nonce(&env, &caller, expected_nonce + 1);

        let mut order: Order = env
            .storage()
            .instance()
            .get(&DataKey::Order(order_id))
            .ok_or(DEXError::OrderNotFound)?;

        if caller != order.seller {
            return Err(DEXError::Unauthorized);
        }

        if order.status != OrderStatus::Open
            && order.status != OrderStatus::PartiallyFilled
        {
            return Err(DEXError::OrderAlreadyFilled);
        }

        order.status = OrderStatus::Cancelled;
        env.storage()
            .instance()
            .set(&DataKey::Order(order_id), &order);

        env.events().publish(
            (Symbol::new(&env, "order_cancelled"),),
            (order_id, caller),
        );

        Ok(())
    }

    pub fn execute_purchase(
        env: Env,
        buyer: Address,
        order_id: u64,
        max_price: i128,
        amount: i128,
        nonce: u64,
    ) -> Result<(), DEXError> {
        buyer.require_auth();

        let expected_nonce = get_nonce(&env, &buyer);
        if nonce != expected_nonce {
            return Err(DEXError::InvalidNonce);
        }
        set_nonce(&env, &buyer, expected_nonce + 1);

        let mut order: Order = env
            .storage()
            .instance()
            .get(&DataKey::Order(order_id))
            .ok_or(DEXError::OrderNotFound)?;

        if order.status != OrderStatus::Open
            && order.status != OrderStatus::PartiallyFilled
        {
            return Err(DEXError::OrderAlreadyFilled);
        }

        if buyer == order.seller {
            return Err(DEXError::SelfBuyNotAllowed);
        }

        if is_order_expired(&env, &order) {
            return Err(DEXError::OrderExpired);
        }

        if amount > order.amount {
            return Err(DEXError::InsufficientBalance);
        }

        if max_price < order.price_per_token {
            return Err(DEXError::InsufficientBalance);
        }

        if amount == order.amount {
            order.status = OrderStatus::Filled;
        } else {
            order.status = OrderStatus::PartiallyFilled;
            order.amount -= amount;
        }

        env.storage()
            .instance()
            .set(&DataKey::Order(order_id), &order);

        env.events().publish(
            (Symbol::new(&env, "order_filled"),),
            (
                order_id,
                buyer,
                order.seller.clone(),
                amount,
                order.price_per_token,
            ),
        );

        Ok(())
    }

    pub fn get_order(env: Env, order_id: u64) -> Result<Order, DEXError> {
        env.storage()
            .instance()
            .get(&DataKey::Order(order_id))
            .ok_or(DEXError::OrderNotFound)
    }

    pub fn get_bond_orders(env: Env, bond_id: u64) -> Vec<u64> {
        env.storage()
            .instance()
            .get(&DataKey::BondOrders(bond_id))
            .unwrap_or(vec![&env])
    }

    pub fn get_seller_orders(env: Env, seller: Address) -> Vec<u64> {
        env.storage()
            .instance()
            .get(&DataKey::SellerOrders(seller))
            .unwrap_or(vec![&env])
    }

    pub fn order_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::OrderCount)
            .unwrap_or(0)
    }

    pub fn clean_expired_orders(
        env: Env,
        caller: Address,
        nonce: u64,
    ) -> Result<u32, DEXError> {
        caller.require_auth();

        let expected_nonce = get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(DEXError::InvalidNonce);
        }
        set_nonce(&env, &caller, expected_nonce + 1);

        require_admin(&env, &caller)?;

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::OrderCount)
            .unwrap_or(0);

        let mut cleaned: u32 = 0;
        for id in 1..=count {
            let key = DataKey::Order(id);
            if let Some(mut order) = env.storage().instance().get::<DataKey, Order>(&key) {
                if (order.status == OrderStatus::Open
                    || order.status == OrderStatus::PartiallyFilled)
                    && is_order_expired(&env, &order)
                {
                    order.status = OrderStatus::Expired;
                    env.storage().instance().set(&key, &order);
                    cleaned += 1;
                }
            }
        }

        env.events().publish(
            (Symbol::new(&env, "expired_orders_cleaned"),),
            (cleaned,),
        );

        Ok(cleaned)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        vec, BytesN, Env, Symbol,
    };

    fn create_project_id(env: &Env, value: u8) -> BytesN<32> {
        let mut arr = [0u8; 32];
        arr[31] = value;
        BytesN::from_array(env, &arr)
    }

    fn setup_bond_and_holder(
        env: &Env,
        bond_supply: i128,
        holder_subscribe: i128,
    ) -> (Address, Address, u64, Address) {
        let issuer_admin = Address::generate(env);
        let issuer_id = env.register(
            nbbs_bond_issuer::BondIssuer,
            (issuer_admin.clone(),),
        );
        let issuer_client =
            nbbs_bond_issuer::BondIssuerClient::new(env, &issuer_id);

        let project_id = create_project_id(env, 1);
        let bond_config = nbbs_shared::BondConfig {
            project_id,
            face_value: 1000,
            coupon_schedule: vec![env, 1_000_000u64, 2_000_000u64],
            credit_type: nbbs_shared::CreditType::Carbon,
            maturity_date: 3_000_000,
            total_supply: bond_supply,
        };

        let bond_id = issuer_client.issue_bond(&issuer_admin, &bond_config, &0);

        let holder = Address::generate(env);
        issuer_client.subscribe(&holder, &bond_id, &holder_subscribe, &0);

        (issuer_admin, issuer_id, bond_id, holder)
    }

    #[test]
    fn test_list_tokens() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let (_issuer_admin, issuer_id, bond_id, seller) =
            setup_bond_and_holder(&env, 10_000, 5_000);

        let contract_id = env.register(
            DEXRouter,
            (admin.clone(), issuer_id.clone(), Address::generate(&env)),
        );
        let client = DEXRouterClient::new(&env, &contract_id);

        let order_id = client.list_bond_tokens(
            &seller,
            &bond_id,
            &1_000i128,
            &100i128,
            &Symbol::new(&env, "USDC"),
            &3600u64,
            &0,
        );
        assert_eq!(order_id, 1);

        let order = client.get_order(&order_id);
        assert_eq!(order.seller, seller);
        assert_eq!(order.bond_id, bond_id);
        assert_eq!(order.amount, 1_000);
        assert_eq!(order.price_per_token, 100);
        assert_eq!(order.status, OrderStatus::Open);

        let bond_orders = client.get_bond_orders(&bond_id);
        assert_eq!(bond_orders.len(), 1);
        assert_eq!(bond_orders.get(0).unwrap(), order_id);

        let seller_orders = client.get_seller_orders(&seller);
        assert_eq!(seller_orders.len(), 1);
        assert_eq!(seller_orders.get(0).unwrap(), order_id);

        assert_eq!(client.order_count(), 1);
    }

    #[test]
    fn test_buy_full_order() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let buyer = Address::generate(&env);
        let (_issuer_admin, issuer_id, bond_id, seller) =
            setup_bond_and_holder(&env, 10_000, 5_000);

        let contract_id = env.register(
            DEXRouter,
            (admin.clone(), issuer_id, Address::generate(&env)),
        );
        let client = DEXRouterClient::new(&env, &contract_id);

        let order_id = client.list_bond_tokens(
            &seller,
            &bond_id,
            &1_000i128,
            &100i128,
            &Symbol::new(&env, "USDC"),
            &3600u64,
            &0,
        );

        client.execute_purchase(&buyer, &order_id, &100i128, &1_000i128, &0);

        let order = client.get_order(&order_id);
        assert_eq!(order.status, OrderStatus::Filled);
    }

    #[test]
    fn test_buy_partial_fill() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let buyer = Address::generate(&env);
        let (_issuer_admin, issuer_id, bond_id, seller) =
            setup_bond_and_holder(&env, 10_000, 5_000);

        let contract_id = env.register(
            DEXRouter,
            (admin.clone(), issuer_id, Address::generate(&env)),
        );
        let client = DEXRouterClient::new(&env, &contract_id);

        let order_id = client.list_bond_tokens(
            &seller,
            &bond_id,
            &1_000i128,
            &100i128,
            &Symbol::new(&env, "USDC"),
            &3600u64,
            &0,
        );

        client.execute_purchase(&buyer, &order_id, &100i128, &400i128, &0);

        let order = client.get_order(&order_id);
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert_eq!(order.amount, 600);

        client.execute_purchase(&buyer, &order_id, &100i128, &600i128, &1);

        let order = client.get_order(&order_id);
        assert_eq!(order.status, OrderStatus::Filled);
    }

    #[test]
    fn test_cancel_listing() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let (_issuer_admin, issuer_id, bond_id, seller) =
            setup_bond_and_holder(&env, 10_000, 5_000);

        let contract_id = env.register(
            DEXRouter,
            (admin.clone(), issuer_id, Address::generate(&env)),
        );
        let client = DEXRouterClient::new(&env, &contract_id);

        let order_id = client.list_bond_tokens(
            &seller,
            &bond_id,
            &1_000i128,
            &100i128,
            &Symbol::new(&env, "USDC"),
            &3600u64,
            &0,
        );

        client.cancel_listing(&seller, &order_id, &1);

        let order = client.get_order(&order_id);
        assert_eq!(order.status, OrderStatus::Cancelled);
    }

    #[test]
    fn test_cancel_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let stranger = Address::generate(&env);
        let (_issuer_admin, issuer_id, bond_id, seller) =
            setup_bond_and_holder(&env, 10_000, 5_000);

        let contract_id = env.register(
            DEXRouter,
            (admin.clone(), issuer_id, Address::generate(&env)),
        );
        let client = DEXRouterClient::new(&env, &contract_id);

        let order_id = client.list_bond_tokens(
            &seller,
            &bond_id,
            &1_000i128,
            &100i128,
            &Symbol::new(&env, "USDC"),
            &3600u64,
            &0,
        );

        let result = client.try_cancel_listing(&stranger, &order_id, &0);
        assert_eq!(result, Err(Ok(DEXError::Unauthorized)));
    }

    #[test]
    fn test_self_buy_reject() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let (_issuer_admin, issuer_id, bond_id, seller) =
            setup_bond_and_holder(&env, 10_000, 5_000);

        let contract_id = env.register(
            DEXRouter,
            (admin.clone(), issuer_id, Address::generate(&env)),
        );
        let client = DEXRouterClient::new(&env, &contract_id);

        let order_id = client.list_bond_tokens(
            &seller,
            &bond_id,
            &1_000i128,
            &100i128,
            &Symbol::new(&env, "USDC"),
            &3600u64,
            &0,
        );

        let result = client.try_execute_purchase(&seller, &order_id, &100i128, &1_000i128, &1);
        assert_eq!(result, Err(Ok(DEXError::SelfBuyNotAllowed)));
    }

    #[test]
    fn test_insufficient_balance() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let (_issuer_admin, issuer_id, bond_id, seller) =
            setup_bond_and_holder(&env, 10_000, 1_000);

        let contract_id = env.register(
            DEXRouter,
            (admin.clone(), issuer_id, Address::generate(&env)),
        );
        let client = DEXRouterClient::new(&env, &contract_id);

        let result = client.try_list_bond_tokens(
            &seller,
            &bond_id,
            &2_000i128,
            &100i128,
            &Symbol::new(&env, "USDC"),
            &3600u64,
            &0,
        );
        assert_eq!(result, Err(Ok(DEXError::InsufficientBalance)));
    }

    #[test]
    fn test_expired_order() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let buyer = Address::generate(&env);
        let (_issuer_admin, issuer_id, bond_id, seller) =
            setup_bond_and_holder(&env, 10_000, 5_000);

        let contract_id = env.register(
            DEXRouter,
            (admin.clone(), issuer_id, Address::generate(&env)),
        );
        let client = DEXRouterClient::new(&env, &contract_id);

        env.ledger().set_timestamp(1_000_000);

        let order_id = client.list_bond_tokens(
            &seller,
            &bond_id,
            &1_000i128,
            &100i128,
            &Symbol::new(&env, "USDC"),
            &100u64,
            &0,
        );

        env.ledger().set_timestamp(1_000_101);

        let result = client.try_execute_purchase(&buyer, &order_id, &100i128, &500i128, &0);
        assert_eq!(result, Err(Ok(DEXError::OrderExpired)));
    }

    #[test]
    fn test_nonexistent_order() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);

        let contract_id = env.register(
            DEXRouter,
            (admin.clone(), Address::generate(&env), Address::generate(&env)),
        );
        let client = DEXRouterClient::new(&env, &contract_id);

        let result = client.try_get_order(&999);
        assert_eq!(result, Err(Ok(DEXError::OrderNotFound)));
    }

    #[test]
    fn test_clean_expired_orders() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let (_issuer_admin, issuer_id, bond_id, seller) =
            setup_bond_and_holder(&env, 10_000, 5_000);

        let contract_id = env.register(
            DEXRouter,
            (admin.clone(), issuer_id, Address::generate(&env)),
        );
        let client = DEXRouterClient::new(&env, &contract_id);

        env.ledger().set_timestamp(1_000_000);

        let order_id = client.list_bond_tokens(
            &seller,
            &bond_id,
            &1_000i128,
            &100i128,
            &Symbol::new(&env, "USDC"),
            &100u64,
            &0,
        );

        client.list_bond_tokens(
            &seller,
            &bond_id,
            &500i128,
            &200i128,
            &Symbol::new(&env, "XLM"),
            &10_000u64,
            &1,
        );

        env.ledger().set_timestamp(1_000_200);

        let cleaned = client.clean_expired_orders(&admin, &0);
        assert_eq!(cleaned, 1);

        let order1 = client.get_order(&order_id);
        assert_eq!(order1.status, OrderStatus::Expired);

        let result = client.try_execute_purchase(
            &Address::generate(&env),
            &order_id,
            &100i128,
            &100i128,
            &0,
        );
        assert_eq!(result, Err(Ok(DEXError::OrderAlreadyFilled)));
    }

    #[test]
    fn test_buy_more_than_listed() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let buyer = Address::generate(&env);
        let (_issuer_admin, issuer_id, bond_id, seller) =
            setup_bond_and_holder(&env, 10_000, 5_000);

        let contract_id = env.register(
            DEXRouter,
            (admin.clone(), issuer_id, Address::generate(&env)),
        );
        let client = DEXRouterClient::new(&env, &contract_id);

        let order_id = client.list_bond_tokens(
            &seller,
            &bond_id,
            &1_000i128,
            &100i128,
            &Symbol::new(&env, "USDC"),
            &3600u64,
            &0,
        );

        let result = client.try_execute_purchase(&buyer, &order_id, &100i128, &2_000i128, &0);
        assert_eq!(result, Err(Ok(DEXError::InsufficientBalance)));
    }

    #[test]
    fn test_buy_with_low_max_price() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let buyer = Address::generate(&env);
        let (_issuer_admin, issuer_id, bond_id, seller) =
            setup_bond_and_holder(&env, 10_000, 5_000);

        let contract_id = env.register(
            DEXRouter,
            (admin.clone(), issuer_id, Address::generate(&env)),
        );
        let client = DEXRouterClient::new(&env, &contract_id);

        let order_id = client.list_bond_tokens(
            &seller,
            &bond_id,
            &1_000i128,
            &100i128,
            &Symbol::new(&env, "USDC"),
            &3600u64,
            &0,
        );

        let result = client.try_execute_purchase(&buyer, &order_id, &50i128, &500i128, &0);
        assert_eq!(result, Err(Ok(DEXError::InsufficientBalance)));
    }
}
