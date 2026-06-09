# Day 5 — `CouponEngine` Contract

Load context: `prompts/context/tech-stack.md`, `prompts/context/soroban-patterns.md`

## Goal

Implement the `CouponEngine` contract that calculates and distributes credit coupons to bondholders based on oracle-reported performance data.

## File

`contracts/coupon-engine/src/lib.rs` — replace the stub.

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
    /// PeriodInfo(bond_id: u64, period_index: u32)
    PeriodInfo(u64, u32),
    /// Total periods tracked per bond
    PeriodCount(u64),
    /// AccruedCredits(bond_id: u64, holder: Address)
    AccruedCredits(u64, Address),
    /// BondProject(bond_id: u64) -> project_id (BytesN<32>)
    BondProject(u64),
    /// Fixed-point precision constant
    Precision,
    /// BondIssuer contract address (for cross-contract calls)
    BondIssuerAddress,
    /// OracleConsumer contract address
    OracleConsumerAddress,
    /// Nonce(address)
    Nonce(Address),
}
```

### Structs

```rust
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
```

### Fixed-Point Precision

```rust
pub const FIXED_POINT: i128 = 10_000_000;  // 1e7
```

### Functions

```rust
#[contractimpl]
impl CouponEngine {
    /// Initialize with admin, bond_issuer_address, oracle_consumer_address.
    pub fn __constructor(
        env: Env,
        admin: Address,
        bond_issuer_address: Address,
        oracle_consumer_address: Address,
    )

    /// Register a bond for coupon tracking. Called by admin after bond issuance.
    /// Stores the bond_id -> project_id mapping.
    pub fn register_bond(
        env: Env,
        caller: Address,
        bond_id: u64,
        project_id: BytesN<32>,
        nonce: u64,
    ) -> Result<(), BondError>

    /// Distribute coupon for a given bond period.
    /// Reads the oracle report, calculates credits, distributes pro-rata.
    /// Returns the CouponResult with total credits and holder count.
    /// Must be called after an oracle report is verified for the bond's project.
    ///
    /// Calculation:
    ///   credits_per_period = (carbon_sequestered_kg / 1000) * conversion_factor
    ///   bondholder_credits = credits_per_period * (holder_tokens * FIXED_POINT / total_subscribed) / FIXED_POINT
    pub fn distribute_coupon(
        env: Env,
        caller: Address,
        bond_id: u64,
        period_index: u32,
        report: OracleReport,
        nonce: u64,
    ) -> Result<CouponResult, BondError>

    /// Query accrued credits for a bondholder.
    pub fn accrued_credits(
        env: Env,
        bond_id: u64,
        holder: Address,
    ) -> i128

    /// Claim accrued credits. Transfers credit tokens to holder.
    /// Currently a no-op that resets accrued to 0 (credit token minting comes in later phase).
    pub fn claim_credits(
        env: Env,
        caller: Address,
        bond_id: u64,
        nonce: u64,
    ) -> Result<i128, BondError>

    /// Get period info for a bond.
    pub fn get_period_info(
        env: Env,
        bond_id: u64,
        period_index: u32,
    ) -> Result<PeriodInfo, BondError>

    /// Get total periods distributed for a bond.
    pub fn get_period_count(env: Env, bond_id: u64) -> u32
}

#[derive(Clone)]
#[contracttype]
pub struct CouponResult {
    pub bond_id: u64,
    pub period_index: u32,
    pub total_credits: i128,
    pub holder_count: u32,
    pub credits_per_token: i128,
}
```

### Business Logic

**register_bond:**
1. Admin-only, nonce check
2. Store `BondProject(bond_id) -> project_id`

**distribute_coupon:**
1. `caller.require_auth()`, nonce check
2. Load `BondProject(bond_id)` — must exist
3. Validate: report.project_id matches bond's project, period not already distributed
4. Calculate credits:
   - `credits_per_period = (report.carbon_sequestered / 1000) * 1` (conversion_factor = 1 for now, simplify by dividing sequestered kg by 1000 to get tCO2e)
   - Wait — the formula from README: `credits_per_period = (carbon_sequestered_kg / 1000) * credit_conversion_factor`. For now, credit_conversion_factor is fixed at 1.
   - So `total_credits = report.carbon_sequestered / 1000` (integer division)
5. Read all holders by iterating — **note**: since we can't iterate all holders efficiently in Soroban, use a simplified approach:
   - The caller provides a list of holder addresses to distribute to
   - The function takes an additional parameter: `holders: Vec<Address>`
6. For each holder:
   - Query their balance from BondIssuer via cross-contract call
   - `holder_credits = total_credits * holder_balance * FIXED_POINT / total_supply / FIXED_POINT`
   - Add to `AccruedCredits(bond_id, holder)`
7. Store `PeriodInfo` with `distributed = true`
8. Increment `PeriodCount(bond_id)`
9. Return `CouponResult`

**Cross-contract call pattern** (to read BondIssuer holder balance):

```rust
fn get_holder_balance_from_bond_issuer(
    env: &Env,
    bond_id: u64,
    holder: Address,
) -> i128 {
    let bond_issuer: Address = env
        .storage()
        .instance()
        .get(&DataKey::BondIssuerAddress)
        .expect("bond issuer not set");

    let balance: i128 = env.invoke_contract(
        &bond_issuer,
        &Symbol::new(env, "get_holder_balance"),
        (bond_id, holder),
    );
    balance
}
```

Similarly for `total_supply`.

**claim_credits:**
1. `caller.require_auth()`, nonce check
2. Read accrued credits for caller
3. Set accrued to 0
4. Return claimed amount (actual token transfer is future work)

## Edge Cases

| Case | Expected Behavior |
|------|------------------|
| Distribute for unregistered bond | `BondNotFound` |
| Double-distribute same period | Allow? No — check `distributed` flag, return error or no-op |
| Period with zero holders | Return zero-credit result |
| Holder with zero balance gets skipped | Correct — they get 0 credits |
| Total credits is zero (no sequestration) | Distribute zero — valid, period is still recorded |
| Non-admin registers bond | `Unauthorized` |
| Caller not authorized to distribute | `Unauthorized` |

## Verification

```bash
cd contracts && cargo test -p nbbs-coupon-engine -- --nocapture
```

Expected: 8+ tests — register bond, distribute to single holder, distribute pro-rata to multiple holders, zero sequestration, double-distribute prevention, query accrued, claim credits resets balance, zero holders case.

## Commit Message

```
feat(contract): CouponEngine — pro-rata credit distribution with cross-contract calls
```
