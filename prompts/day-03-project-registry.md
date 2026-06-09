# Day 3 — `ProjectRegistry` Contract

Load context: `prompts/context/tech-stack.md`, `prompts/context/soroban-patterns.md`

## Goal

Implement the `ProjectRegistry` smart contract that maintains the canonical on-chain registry of all NbS projects.

## File

`contracts/project-registry/src/lib.rs` — replace the stub.

## Technical Spec

### DataKey

```rust
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    /// Project(project_id: BytesN<32>)
    Project(BytesN<32>),
    /// Seq counter for project IDs
    ProjectCount,
    /// ProjectId(address_index: u64)
    ProjectId(u64),
    /// Nonce(address)
    Nonce(Address),
    /// Owner project list: OwnerProjects(owner: Address) -> Vec<u64>
    OwnerProjects(Address),
}
```

### Structs

```rust
#[derive(Clone)]
#[contracttype]
pub struct Project {
    pub id: u64,
    pub owner: Address,
    pub metadata_ipfs_hash: BytesN<32>,
    pub status: ProjectStatus,
    pub methodology: Symbol,
    pub country: Symbol,
}

#[derive(Clone)]
#[contracttype]
pub struct ProjectSummary {
    pub id: u64,
    pub name: Symbol,
    pub status: ProjectStatus,
    pub country: Symbol,
}
```

### Functions

```rust
#[contractimpl]
impl ProjectRegistry {
    /// Initialize the contract with an admin address.
    pub fn __constructor(env: Env, admin: Address)

    /// Register a new NbS project. Returns the project ID.
    /// - `caller` must sign (require_auth)
    /// - `nonce` must match caller's current nonce
    /// - `metadata_ipfs_hash` must be non-zero (32 bytes, not all zero)
    pub fn register_project(
        env: Env,
        caller: Address,
        metadata_ipfs_hash: BytesN<32>,
        methodology: Symbol,
        country: Symbol,
        nonce: u64,
    ) -> Result<u64, RegistryError>

    /// Approve a pending project. Admin only.
    pub fn approve_project(
        env: Env,
        caller: Address,
        project_id: u64,
        nonce: u64,
    ) -> Result<(), RegistryError>

    /// Reject a pending project. Admin only.
    pub fn reject_project(
        env: Env,
        caller: Address,
        project_id: u64,
        nonce: u64,
    ) -> Result<(), RegistryError>

    /// Get full project details.
    pub fn get_project(env: Env, project_id: u64) -> Result<Project, RegistryError>

    /// List all project IDs. Returns a vector of project summaries (id, name, status, country).
    /// Pagination via `page` (0-indexed) and `page_size` (max 50).
    pub fn list_projects(env: Env, page: u32, page_size: u32) -> Vec<ProjectSummary>

    /// Get the number of registered projects.
    pub fn project_count(env: Env) -> u64

    /// Get projects owned by a specific address.
    pub fn get_owner_projects(env: Env, owner: Address) -> Vec<u64>
}
```

### Business Logic

**register_project:**
1. `caller.require_auth()`
2. Verify nonce matches
3. Increment nonce
4. Verify `metadata_ipfs_hash` is non-zero (use `env.crypto().sha256()` or compare bytes)
5. Increment `ProjectCount` (start at 1)
6. Create `Project` with `id = count`, `owner = caller`, `status = Pending`
7. Store under `DataKey::Project(project_id)` using a `BytesN<32>` encoding of the u64 ID (use `BytesN::from_array` with the u64 bytes)
8. Append to `OwnerProjects(owner)` list
9. Return the new project ID

**approve_project / reject_project:**
1. `caller.require_auth()`
2. Verify nonce
3. `require_admin` check
4. Load project, verify status is `Pending`
5. Set status to `Approved` or `Rejected`
6. Store updated project

**Status transition rules:**
- `Pending` → `Approved` (approve)
- `Pending` → `Rejected` (reject)  
- `Approved` → `Inactive` (future use, not implemented yet)
- No other transitions allowed

### Helper: project_id_from_u64

Since DataKey::Project takes `BytesN<32>`, encode `u64` as the first 8 bytes of a 32-byte array (big-endian):
```rust
fn project_id_to_bytes(env: &Env, id: u64) -> BytesN<32> {
    let mut arr = [0u8; 32];
    arr[..8].copy_from_slice(&id.to_be_bytes());
    BytesN::from_array(env, &arr)
}
```

Similarly for decoding back to u64 when reading from DataKey.

## Edge Cases

| Case | Expected Behavior |
|------|------------------|
| Register with zero hash | Return `Err(RegistryError::ProjectNotFound)` — use a custom check, hash must have at least one non-zero byte |
| Duplicate registration attempt | Each registration creates a new project (no dedup by hash) — owner can register multiple projects |
| Approve non-existent project | Return `Err(RegistryError::ProjectNotFound)` |
| Approve already-approved project | Return `Err(RegistryError::InvalidStatusTransition)` |
| Non-admin tries to approve | Return `Err(RegistryError::Unauthorized)` |
| Reject already-rejected project | Return `Err(RegistryError::InvalidStatusTransition)` |
| Invalid nonce | Return `Err(RegistryError::InvalidNonce)` |
| Query non-existent project | Return `Err(RegistryError::ProjectNotFound)` |
| Zero-address admin in constructor | Allow it (admin can be set to any address) |

## Verification

```bash
cd contracts && cargo test -p nbbs-project-registry -- --nocapture
```

Expected: 7+ passing tests covering all functions and edge cases.

## Commit Message

```
feat(contract): ProjectRegistry — register, approve, reject, query projects
```
