# Day 7 — `DEXRouter` Contract

Load context: `prompts/context/tech-stack.md`, `prompts/context/soroban-patterns.md`

## Goal

Implement the `DEXRouter` contract that manages bond token listings, order routing, and settlement on the Stellar DEX.

## File

`contracts/dex-router/src/lib.rs` — replace the stub.

Dependency in `Cargo.toml`:
```toml
nbbs-shared = { path = "../shared" }
```

## Technical Spec

### DataKey

```rust
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    /// Order(order_id: u64) -> Order
    Order(u64),
    /// Order count
    OrderCount,
    /// SellerOrders(seller: Address) -> Vec<u64>
    SellerOrders(Address),
    /// BondOrders(bond_id: u64) -> Vec<u64>
    BondOrders(u64),
    /// BondIssuer contract address
    BondIssuerAddress,
    /// CouponEngine contract address
    CouponEngineAddress,
    /// Nonce(address)
    Nonce(Address),
}
```

### Structs

```rust
#[derive(Clone)]
#[contracttype]
pub struct Order {
    pub id: u64,
    pub seller: Address,
    pub bond_id: u64,
    pub amount: i128,
    pub price_per_token: i128,
    pub quote_asset: Symbol,     // e.g., "USDC" or "XLM"
    pub status: OrderStatus,
    pub created_at: u64,
    pub expires_at: u64,
}

#[derive(Clone, Copy, PartialEq)]
#[contracttype]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Expired,
}
```

### Functions

```rust
#[contractimpl]
impl DEXRouter {
    /// Initialize with admin, bond_issuer_address, coupon_engine_address.
    pub fn __constructor(
        env: Env,
        admin: Address,
        bond_issuer_address: Address,
        coupon_engine_address: Address,
    )

    /// List bond tokens for sale. Creates a sell order.
    /// - `seller` must hold at least `amount` tokens in BondIssuer
    /// - Amount is locked/escrowed (in a real implementation; for now we just verify balance)
    pub fn list_bond_tokens(
        env: Env,
        seller: Address,
        bond_id: u64,
        amount: i128,
        price_per_token: i128,
        quote_asset: Symbol,
        expires_after_seconds: u64,
        nonce: u64,
    ) -> Result<u64, DEXError>

    /// Cancel a listing. Only the seller can cancel.
    pub fn cancel_listing(
        env: Env,
        caller: Address,
        order_id: u64,
        nonce: u64,
    ) -> Result<(), DEXError>

    /// Execute a purchase. Buyer sends payment.
    /// - Verifies buyer has sufficient balance (simplified: just checks buyer != seller)
    /// - Marks order as filled (or partially filled)
    pub fn execute_purchase(
        env: Env,
        buyer: Address,
        order_id: u64,
        max_price: i128,
        amount: i128,
        nonce: u64,
    ) -> Result<(), DEXError>

    /// Get order details.
    pub fn get_order(env: Env, order_id: u64) -> Result<Order, DEXError>

    /// List all orders for a bond.
    pub fn get_bond_orders(env: Env, bond_id: u64) -> Vec<u64>

    /// List all orders by a seller.
    pub fn get_seller_orders(env: Env, seller: Address) -> Vec<u64>

    /// Get the total number of orders.
    pub fn order_count(env: Env) -> u64

    /// Clean up expired orders. Admin only.
    /// Sets expired orders' status to Expired.
    pub fn clean_expired_orders(env: Env, caller: Address, nonce: u64) -> Result<u32, DEXError>
}
```

### Business Logic

**list_bond_tokens:**
1. `seller.require_auth()`, nonce check
2. Validate `amount > 0`, `price_per_token > 0`
3. Verify seller's bond balance via cross-contract call to BondIssuer (`get_holder_balance`)
4. Must have at least `amount` tokens
5. Increment `OrderCount`
6. Create `Order` with `status: Open`, `created_at: now`, `expires_at: now + expires_after_seconds`
7. Store under `DataKey::Order(id)`
8. Append to `SellerOrders(seller)` and `BondOrders(bond_id)`
9. Return order_id

**cancel_listing:**
1. `caller.require_auth()`, nonce check
2. Load order — must be `Open` or `PartiallyFilled`
3. `caller` must be `order.seller`
4. Set `status = Cancelled`

**execute_purchase:**
1. `buyer.require_auth()`, nonce check
2. Load order — must be `Open` or `PartiallyFilled`
3. `buyer != order.seller` (no self-buying)
4. Check `!expired`
5. `amount <= order.amount` (no over-buy)
6. Validate `max_price >= order.price_per_token` (buyer accepts the price)
7. If `amount == order.amount` → `status = Filled`
8. If `amount < order.amount` → `status = PartiallyFilled`, reduce `order.amount`
9. Emit event with buyer, seller, order_id, amount, price
10. **Note:** Actual Stellar asset transfer is simulated — in this phase, we just track state. The real DEX integration will use Stellar path payments.

### Order Expiry

```rust
fn is_order_expired(env: &Env, order: &Order) -> bool {
    env.ledger().timestamp() > order.expires_at
}
```

## Edge Cases

| Case | Expected Behavior |
|------|------------------|
| List more tokens than owned | `InsufficientBalance` |
| List zero tokens | Return `DEXError::InsufficientBalance` or check `amount > 0` |
| Cancel someone else's order | `Unauthorized` |
| Buy own listing | `SelfBuyNotAllowed` |
| Buy more than listed | Return error — calculate remaining and reject if `amount > order.amount` |
| Buy from filled/cancelled order | `OrderAlreadyFilled` |
| Buy from expired order | Return `OrderExpired` — buyer should check status first |
| Price exceeds max_price | Return error — price doesn't match |
| Non-existent order | `OrderNotFound` |
| List with zero price | Return error — use `DEXError::InsufficientBalance` or similar |

## Cross-Contract Call Pattern

```rust
fn verify_holder_balance(env: &Env, holder: &Address, bond_id: u64, required: i128) -> Result<(), DEXError> {
    let bond_issuer: Address = env
        .storage()
        .instance()
        .get(&DataKey::BondIssuerAddress)
        .ok_or(DEXError::NotInitialized)?;

    let balance: i128 = env.invoke_contract(
        &bond_issuer,
        &Symbol::new(env, "get_holder_balance"),
        (bond_id, holder.clone()),
    );

    if balance < required {
        return Err(DEXError::InsufficientBalance);
    }
    Ok(())
}
```

## Verification

```bash
cd contracts && cargo test -p nbbs-dex-router -- --nocapture
```

Expected: 8+ tests — list tokens, buy full order, buy partial fill, cancel listing, self-buy reject, insufficient balance, expired order, non-existent order.

## Commit Message

```
feat(contract): DEXRouter — bond token listing, purchase, cancellation with order book
```
