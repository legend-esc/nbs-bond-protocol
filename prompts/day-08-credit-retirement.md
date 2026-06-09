# Day 8 — `CreditRetirement` Contract + Integration Tests

Load context: `prompts/context/tech-stack.md`, `prompts/context/soroban-patterns.md`

## Goal

Implement the `CreditRetirement` contract (on-chain credit retirement with certificate NFT) and write the full cross-contract integration test suite.

## Files

- `contracts/credit-retirement/src/lib.rs` — replace the stub
- `contracts/tests/src/lib.rs` — replace the placeholder

Dependency in `credit-retirement/Cargo.toml`:
```toml
nbbs-shared = { path = "../shared" }
```

---

## Part 1: `CreditRetirement` Contract

### DataKey

```rust
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    /// Retirement(id: u64) -> RetirementRecord
    Retirement(u64),
    /// Retirement count
    RetirementCount,
    /// HolderRetirements(holder: Address) -> Vec<u64>
    HolderRetirements(Address),
    /// RetiredCredits(holder: Address) -> total i128
    RetiredCredits(Address),
    /// Nonce(address)
    Nonce(Address),
}
```

### Structs

```rust
#[derive(Clone)]
#[contracttype]
pub struct RetirementRecord {
    pub id: u64,
    pub holder: Address,
    pub bond_id: u64,
    pub amount: i128,
    pub credit_type: CreditType,
    pub retired_at: u64,
    pub certificate_ipfs_hash: BytesN<32>,
}

#[derive(Clone)]
#[contracttype]
pub struct RetirementCertificate {
    pub record_id: u64,
    pub holder: Address,
    pub bond_id: u64,
    pub amount: i128,
    pub credit_type: CreditType,
    pub retired_at: u64,
    pub certificate_hash: BytesN<32>,
}
```

### Functions

```rust
#[contractimpl]
impl CreditRetirement {
    /// Initialize with admin address.
    pub fn __constructor(env: Env, admin: Address)

    /// Retire credits. Burns the holder's accrued credits from CouponEngine
    /// and creates a permanent retirement record.
    /// - `holder` authorizes the retirement
    /// - Checks CouponEngine for accrued credits
    /// - Creates retirement record + certificate
    pub fn retire_credits(
        env: Env,
        holder: Address,
        bond_id: u64,
        amount: i128,
        credit_type: CreditType,
        certificate_hash: BytesN<32>,
        nonce: u64,
    ) -> Result<u64, CreditError>

    /// Get a retirement record by ID.
    pub fn get_retirement_record(
        env: Env,
        retirement_id: u64,
    ) -> Result<RetirementRecord, CreditError>

    /// Get retirement certificate data.
    pub fn get_retirement_certificate(
        env: Env,
        retirement_id: u64,
    ) -> Result<RetirementCertificate, CreditError>

    /// Get all retirement IDs for a holder.
    pub fn get_holder_retirements(env: Env, holder: Address) -> Vec<u64>

    /// Get total retired credits for a holder.
    pub fn get_total_retired(env: Env, holder: Address) -> i128

    /// Total retirements across the protocol.
    pub fn total_retirements(env: Env) -> u64
}
```

### Business Logic

**retire_credits:**
1. `holder.require_auth()`, nonce check
2. Validate `amount > 0`
3. **Simplification:** In this phase, we don't do a cross-contract call to CouponEngine to deduct credits. Instead, we assume the caller has already claimed them. The contract just records the retirement.
4. Increment `RetirementCount`
5. Create `RetirementRecord` with `id = count`, `retired_at = env.ledger().timestamp()`
6. Store under `DataKey::Retirement(id)`
7. Update `RetiredCredits(holder)` — add amount
8. Append to `HolderRetirements(holder)`
9. Emit event: `{"event": "CreditsRetired", "holder": holder, "amount": amount, "type": credit_type}`
10. Return retirement ID

## Edge Cases

| Case | Expected Behavior |
|------|------------------|
| Retire zero credits | `InsufficientCredits` |
| Retire from unregistered bond | Allow it (we don't validate bond_id) |
| Query non-existent retirement | Return `CreditError` — use `InsufficientCredits` or similar |
| Double-retire same credits | No protection needed — each call is independent |

Testing: 5+ unit tests covering create + query retirement, multiple retirements, holder query, total tracking.

---

## Part 2: Integration Tests

In `contracts/tests/src/lib.rs`, write a comprehensive integration test module.

### Test Setup

```rust
use soroban_sdk::{testutils::Address as _, vec, Env, Address, BytesN, Symbol};
use nbbs_project_registry::{ProjectRegistry, ProjectRegistryClient};
use nbbs_bond_issuer::{BondIssuer, BondIssuerClient};
use nbbs_coupon_engine::{CouponEngine, CouponEngineClient};
use nbbs_oracle_consumer::{OracleConsumer, OracleConsumerClient};
use nbbs_dex_router::{DEXRouter, DEXRouterClient};
use nbbs_credit_retirement::{CreditRetirement, CreditRetirementClient};
use nbbs_shared::{BondConfig, CreditType, OracleReport, BondError, RegistryError};
```

### Test Scenarios

**Scenario 1: Happy path — Full bond lifecycle**
1. Deploy all 6 contracts
2. Set up admin address + test addresses (alice, bob, oracle_provider)
3. Register a project via ProjectRegistry (alice registers)
4. Admin approves the project
5. Issue a bond via BondIssuer (admin issues, backed by project)
6. Bob subscribes to the bond (buys 1000 tokens)
7. Register an oracle provider
8. Submit an oracle report for the project
9. Verify the report
10. Distribute coupon via CouponEngine
11. Check Bob's accrued credits > 0
12. Retire credits via CreditRetirement
13. Verify retirement record exists

**Scenario 2: Insufficient supply rejection**
1. Register project, approve, issue bond with total_supply = 1000
2. Alice subscribes 1000 (fills it)
3. Bob tries to subscribe 1 — expect `BondError::InsufficientSupply`

**Scenario 3: Oracle challenge flow**
1. Register project, register provider, submit report
2. Alice challenges the report within window
3. Admin resolves challenge, sets report to Rejected
4. Verify report status is Rejected

**Scenario 4: DEX order lifecycle**
1. Issue bond, Alice subscribes
2. Alice lists 100 bond tokens on DEX
3. Bob buys 50 tokens — order is PartiallyFilled
4. Bob buys remaining 50 — order is Filled
5. Verify order statuses

**Scenario 5: Nonce replay protection**
1. Register project with nonce = 0
2. Try to register again with nonce = 0 — expect `RegistryError::InvalidNonce`
3. Register with nonce = 1 — success

**Scenario 6: Permission checks**
1. Non-admin tries to approve project — expect `RegistryError::Unauthorized`
2. Non-admin tries to issue bond — expect `BondError::Unauthorized`
3. Non-registered provider submits report — expect `OracleError::ProviderNotFound`

### Test Organization

```rust
#[cfg(test)]
mod integration {
    use super::*;

    struct TestEnv {
        env: Env,
        admin: Address,
        alice: Address,
        bob: Address,
        oracle: Address,
        // ... clients
    }

    impl TestEnv {
        fn setup() -> Self { ... }
    }

    mod full_lifecycle {
        #[test] fn test_happy_path() { ... }
        #[test] fn test_insufficient_supply() { ... }
    }

    mod oracle {
        #[test] fn test_challenge_flow() { ... }
    }

    mod dex {
        #[test] fn test_order_lifecycle() { ... }
    }

    mod security {
        #[test] fn test_nonce_replay() { ... }
        #[test] fn test_permission_checks() { ... }
    }
}
```

### Contract Deployment Helper

```rust
fn deploy_contracts(env: &Env, admin: &Address) -> TestContracts {
    // Deploy ProjectRegistry
    let pr_addr = env.register_contract(None, ProjectRegistry);
    let pr_client = ProjectRegistryClient::new(env, &pr_addr);
    pr_client.__constructor(admin);

    // Deploy BondIssuer
    let bi_addr = env.register_contract(None, BondIssuer);
    let bi_client = BondIssuerClient::new(env, &bi_addr);
    bi_client.__constructor(admin);

    // Deploy CouponEngine (needs BondIssuer + OracleConsumer addresses — deploy OracleConsumer first)
    let oc_addr = env.register_contract(None, OracleConsumer);
    let oc_client = OracleConsumerClient::new(env, &oc_addr);
    oc_client.__constructor(admin);

    let ce_addr = env.register_contract(None, CouponEngine);
    let ce_client = CouponEngineClient::new(env, &ce_addr);
    ce_client.__constructor(admin, &bi_addr, &oc_addr);

    // Deploy DEXRouter
    let dr_addr = env.register_contract(None, DEXRouter);
    let dr_client = DEXRouterClient::new(env, &dr_addr);
    dr_client.__constructor(admin, &bi_addr, &ce_addr);

    // Deploy CreditRetirement
    let cr_addr = env.register_contract(None, CreditRetirement);
    let cr_client = CreditRetirementClient::new(env, &cr_addr);
    cr_client.__constructor(admin);

    TestContracts { pr_client, bi_client, ce_client, oc_client, dr_client, cr_client }
}
```

## Verification

```bash
cd contracts && cargo test -- --nocapture
```

Expected: All 38 unit tests + 8 integration tests passing.

```bash
cd contracts && cargo clippy --all-targets -- -D warnings
```

Expected: Zero warnings.

## Commit Message

```
feat(contract): CreditRetirement + full integration test suite covering all 6 contracts
```
