# Day 4 — `BondIssuer` Contract

Load context: `prompts/context/tech-stack.md`, `prompts/context/soroban-patterns.md`

## Goal

Implement the `BondIssuer` contract that handles bond creation, tranche configuration, and investor token minting.

## File

`contracts/bond-issuer/src/lib.rs` — replace the stub.

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
    /// BondConfig(bond_id: u64)
    BondConfig(u64),
    /// BondState(bond_id: u64)
    BondState(u64),
    /// HolderBalance(bond_id: u64, holder: Address)
    HolderBalance(u64, Address),
    /// Total bond count
    BondCount,
    /// Nonce(address)
    Nonce(Address),
}
```

### Structs

```rust
#[derive(Clone)]
#[contracttype]
pub struct BondState {
    pub total_subscribed: i128,
    pub status: BondStatus,
    pub created_at: u64,
}
```

BondConfig from `nbbs-shared` is used directly.

### Functions

```rust
#[contractimpl]
impl BondIssuer {
    /// Initialize with admin address.
    pub fn __constructor(env: Env, admin: Address)

    /// Issue a new bond tranche. Admin only.
    /// Returns the new bond ID.
    pub fn issue_bond(
        env: Env,
        caller: Address,
        config: BondConfig,
        nonce: u64,
    ) -> Result<u64, BondError>

    /// Subscribe to a bond tranche. Investor buys bond tokens.
    /// Updates holder balance and total subscribed.
    /// Fails if bond is matured or fully subscribed.
    pub fn subscribe(
        env: Env,
        investor: Address,
        bond_id: u64,
        amount: i128,
        nonce: u64,
    ) -> Result<(), BondError>

    /// Redeem bond tokens at maturity. Burns tokens, records redemption.
    /// Only callable when bond status is Matured.
    pub fn redeem(
        env: Env,
        holder: Address,
        bond_id: u64,
        amount: i128,
        nonce: u64,
    ) -> Result<(), BondError>

    /// Get bond configuration.
    pub fn get_bond(env: Env, bond_id: u64) -> Result<BondConfig, BondError>

    /// Get bond state (total subscribed, status).
    pub fn get_bond_state(env: Env, bond_id: u64) -> Result<BondState, BondError>

    /// Get holder balance for a specific bond.
    pub fn get_holder_balance(env: Env, bond_id: u64, holder: Address) -> i128

    /// Get total supply for a bond.
    pub fn total_supply(env: Env, bond_id: u64) -> Result<i128, BondError>

    /// Get total subscribed amount for a bond.
    pub fn total_subscribed(env: Env, bond_id: u64) -> Result<i128, BondError>

    /// Mark a bond as matured. Admin only. Called after maturity_date passes.
    pub fn mature_bond(
        env: Env,
        caller: Address,
        bond_id: u64,
        nonce: u64,
    ) -> Result<(), BondError>
}
```

### Business Logic

**issue_bond:**
1. `caller.require_auth()`, verify admin, verify nonce
2. Validate `config.face_value > 0`, `config.total_supply > 0`, `config.maturity_date > env.ledger().timestamp()`
3. `coupon_schedule` must not be empty, and all entries must be `< maturity_date`
4. Increment `BondCount`, assign as bond_id
5. Store `BondConfig` and `BondState` (total_subscribed: 0, status: Active, created_at: now)
6. Emit event via `env.events().publish()`: `{"bond_id": id, "event": "Issued", "project_id": config.project_id}`

**subscribe:**
1. `investor.require_auth()`, verify nonce
2. Load `BondConfig` and `BondState` — fail if not found or status != Active
3. `amount > 0`, `total_subscribed + amount <= total_supply`
4. Use `checked_add` on all arithmetic
5. Update `HolderBalance(bond_id, investor)` — add amount
6. Update `total_subscribed`
7. Emit event: `{"bond_id": id, "investor": investor, "amount": amount}`

**redeem:**
1. `holder.require_auth()`, verify nonce
2. Bond must be Matured
3. Load `HolderBalance` — must have >= `amount`
4. Subtract from balance and total_subscribed
5. Emit event

**mature_bond:**
1. Admin-only
2. Bond must be Active
3. Set status to Matured

### Important Constants

```rust
pub const MAX_SUPPLY: i128 = 1_000_000_000_000_000_000;  // 1e18 (compatible with i128)
```

## Edge Cases

| Case | Expected Behavior |
|------|------------------|
| Subscribe to non-existent bond | `BondNotFound` |
| Subscribe beyond total supply | `InsufficientSupply` |
| Subscribe with zero amount | `ZeroAmount` |
| Subscribe to matured bond | `BondAlreadyMatured` |
| Redeem from active bond (not matured) | `BondAlreadyMatured` (error — use this code meaning "not in redeemable state") |
| Redeem more than owned | `InsufficientSupply` |
| Issue bond with past maturity | Return `BondError::Overflow` — reject if maturity_date <= ledger time |
| Issue bond with empty coupon_schedule | Return `BondError::ZeroAmount` — reject |
| Double nonce replay | `InvalidNonce` |
| Overflow on total_subscribed | `Overflow` |

## Verification

```bash
cd contracts && cargo test -p nbbs-bond-issuer -- --nocapture
```

Expected: 9+ tests — issue, subscribe (full), subscribe (partial), subscribe exceeds supply, redeem after maturity, redeem before maturity (fail), mature bond, nonce replay guard, admin guard.

## Commit Message

```
feat(contract): BondIssuer — issue, subscribe, redeem, mature with nonce protection
```
