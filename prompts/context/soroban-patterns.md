# Soroban Contract Patterns — Reference

Use this for ALL contract implementations.

## Cargo.toml Template

```toml
[package]
name = "nbbs-{contract-name}"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
soroban-sdk = "26.0.1"

[dev-dependencies]
soroban-sdk = { version = "26.0.1", features = ["testutils"] }

[features]
testutils = ["soroban-sdk/testutils"]
```

## Workspace Cargo.toml Template

```toml
[workspace]
members = [
    "shared",
    "bond-issuer",
    "coupon-engine",
    "oracle-consumer",
    "dex-router",
    "project-registry",
    "credit-retirement",
    "tests",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
```

## Core Patterns

### Storage Key Pattern

```rust
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Key for storing a {thing} by its ID
    Thing(BytesN<32>),
    /// Singleton admin address
    Admin,
    /// Counter for {resource}
    Counter,
}
```

### Contract Function Pattern

```rust
#[contractimpl]
impl ContractName {
    pub fn create_thing(
        env: Env,
        caller: Address,
        param: Type,
        nonce: u64,
    ) -> Result<IdType, ErrorType> {
        caller.require_auth();
        // verify nonce
        let expected_nonce = Self::get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(ErrorType::InvalidNonce);
        }
        // increment nonce
        Self::set_nonce(&env, &caller, expected_nonce + 1);
        // ... logic
        Ok(id)
    }
}
```

### Nonce Management

```rust
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
```

### Admin Guard

```rust
fn require_admin(env: &Env, caller: &Address) -> Result<(), ErrorType> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(ErrorType::NotInitialized)?;
    if caller != &admin {
        return Err(ErrorType::Unauthorized);
    }
    Ok(())
}
```

### Test Pattern

```rust
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, vec, BytesN, Env, IntoVal, Symbol};

    #[test]
    fn test_create_fails_with_invalid_nonce() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MyContract);
        let client = MyContractClient::new(&env, &contract_id);
        // ...
    }
}
```

## Storage Lifecycle Guidance

| Data Kind | Storage Type | When to Use |
|-----------|-------------|-------------|
| Configuration (Admin, fees) | `env.storage().instance()` | Set once, rarely changed |
| User data (nonces, balances) | `env.storage().persistent()` | Lives as long as the contract |
| Temporary (challenge windows) | `env.storage().temporary()` | TTL-managed, auto-expires |
