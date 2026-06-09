# Day 6 — `OracleConsumer` Contract

Load context: `prompts/context/tech-stack.md`, `prompts/context/soroban-patterns.md`

## Goal

Implement the `OracleConsumer` contract that validates and ingests signed oracle reports from approved providers, with a challenge window for dispute resolution.

## File

`contracts/oracle-consumer/src/lib.rs` — replace the stub.

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
    /// Provider(address) -> OracleProvider
    Provider(Address),
    /// Provider list (Vec<Address>)
    ProviderList,
    /// Report(report_id: u64) -> Report
    Report(u64),
    /// Report count
    ReportCount,
    /// ProjectReports(project_id: BytesN<32>) -> Vec<u64>
    ProjectReports(BytesN<32>),
    /// Challenge(report_id: u64) -> Challenge
    Challenge(u64),
    /// Required signature threshold for high-value reports
    SignatureThreshold,
    /// Challenge window duration in seconds (default: 72 hours = 259200)
    ChallengeWindow,
    /// Nonce(address)
    Nonce(Address),
}
```

### Structs

```rust
#[derive(Clone)]
#[contracttype]
pub struct OracleProvider {
    pub address: Address,
    pub methodology: Symbol,
    pub stake: i128,
    pub active: bool,
    pub registered_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct Report {
    pub id: u64,
    pub provider: Address,
    pub project_id: BytesN<32>,
    pub period_start: u64,
    pub period_end: u64,
    pub carbon_sequestered: i128,
    pub methodology: Symbol,
    pub ipfs_evidence_hash: BytesN<32>,
    pub status: ReportStatus,
    pub submitted_at: u64,
    pub verified_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct Challenge {
    pub report_id: u64,
    pub challenger: Address,
    pub counter_evidence_hash: BytesN<32>,
    pub submitted_at: u64,
    pub resolved: bool,
    pub resolution: Option<ReportStatus>,
}
```

### Functions

```rust
#[contractimpl]
impl OracleConsumer {
    /// Initialize with admin address.
    pub fn __constructor(env: Env, admin: Address)

    /// Register a new oracle provider. Admin only.
    /// Provider must not already be registered.
    pub fn register_provider(
        env: Env,
        caller: Address,
        provider: Address,
        methodology: Symbol,
        nonce: u64,
    ) -> Result<(), OracleError>

    /// Remove an oracle provider. Admin only.
    /// Sets provider.active = false.
    pub fn remove_provider(
        env: Env,
        caller: Address,
        provider: Address,
        nonce: u64,
    ) -> Result<(), OracleError>

    /// Submit a signed measurement report.
    /// Provider must be registered and active.
    /// Returns the report ID.
    pub fn submit_report(
        env: Env,
        provider: Address,
        project_id: BytesN<32>,
        period_start: u64,
        period_end: u64,
        carbon_sequestered: i128,
        methodology: Symbol,
        ipfs_evidence_hash: BytesN<32>,
        nonce: u64,
    ) -> Result<u64, OracleError>

    /// Verify a submitted report. Transitions from Pending -> Verified.
    /// Can be called by the same provider or admin.
    /// Only callable within the verification window (before challenge period expires).
    pub fn verify_report(
        env: Env,
        caller: Address,
        report_id: u64,
        nonce: u64,
    ) -> Result<(), OracleError>

    /// Challenge a submitted report. Opens a dispute.
    /// Any address can challenge within the challenge window (72h from submission).
    pub fn challenge_report(
        env: Env,
        challenger: Address,
        report_id: u64,
        counter_evidence_hash: BytesN<32>,
        nonce: u64,
    ) -> Result<(), OracleError>

    /// Resolve a challenge. Admin only.
    /// Sets final report status to Verified or Rejected.
    pub fn resolve_challenge(
        env: Env,
        caller: Address,
        report_id: u64,
        resolution: ReportStatus,
        nonce: u64,
    ) -> Result<(), OracleError>

    /// Get provider details.
    pub fn get_provider(env: Env, provider: Address) -> Result<OracleProvider, OracleError>

    /// Get report details.
    pub fn get_report(env: Env, report_id: u64) -> Result<Report, OracleError>

    /// Get all registered provider addresses.
    pub fn list_providers(env: Env) -> Vec<Address>

    /// Get report IDs for a project.
    pub fn get_project_reports(env: Env, project_id: BytesN<32>) -> Vec<u64>

    /// Get challenge for a report.
    pub fn get_challenge(env: Env, report_id: u64) -> Result<Challenge, OracleError>

    /// Set signature threshold for multi-sig validation. Admin only.
    pub fn set_signature_threshold(
        env: Env,
        caller: Address,
        threshold: u32,
        nonce: u64,
    ) -> Result<(), OracleError>
}
```

### Business Logic

**register_provider:**
1. Admin-only, nonce check
2. Provider must not already exist (check `DataKey::Provider(provider)`)
3. Store `OracleProvider` with `active: true`, `stake: 0`, `registered_at: env.ledger().timestamp()`
4. Append to `ProviderList`

**submit_report:**
1. `provider.require_auth()`, nonce check
2. Load provider — must exist and be active
3. Validate `period_end > period_start`, `carbon_sequestered >= 0`
4. Increment `ReportCount`, assign report_id
5. Store `Report` with `status: Pending`, `submitted_at: now`
6. Append to `ProjectReports(project_id)`
7. Return report_id

**verify_report:**
1. `caller.require_auth()`, nonce check
2. Load report — must exist and be `Pending`
3. Check that the challenge window hasn't been triggered (no challenge exists, or skip if no challenge — simplified: anyone can verify if report is pending and no challenge exists)
4. Set `status = Verified`, `verified_at = now`
5. Emit event

**challenge_report:**
1. `challenger.require_auth()`, nonce check
2. Load report — must exist and not be `Verified` or `Rejected`
3. Check challenge window: `now - report.submitted_at <= 259200` (72 hours)
4. Must not already have an unresolved challenge
5. Store `Challenge` with `resolved: false`
6. Set `report.status = Challenged`

**resolve_challenge:**
1. Admin-only, nonce check
2. Load challenge — must exist and not be resolved
3. Set `challenge.resolved = true`, `challenge.resolution = resolution`
4. Set `report.status = resolution` (either `Verified` or `Rejected`)

## Constants

```rust
pub const CHALLENGE_WINDOW_SECONDS: u64 = 259200; // 72 hours
```

## Edge Cases

| Case | Expected Behavior |
|------|------------------|
| Register same provider twice | `ProviderAlreadyExists` |
| Submit report from non-registered address | `ProviderNotFound` |
| Submit report from inactive provider | `Unauthorized` |
| Verify already-verified report | `ReportAlreadyVerified` |
| Challenge after window expired | `ChallengeWindowExpired` |
| Challenge already-challenged report | Return `OracleError` — Challenge already exists (use a generic error) |
| Verify a challenged report (before resolution) | Return error — report is in Challenged state |
| Resolve already-resolved challenge | No-op — return Ok but don't change anything |
| Query non-existent report | `ReportNotFound` |
| Query non-existent provider | `ProviderNotFound` |

## Verification

```bash
cd contracts && cargo test -p nbbs-oracle-consumer -- --nocapture
```

Expected: 9+ tests — register provider, submit + verify, submit + challenge + resolve, late challenge, non-whitelisted submit, duplicate provider, double-verify, challenge on verified report, inactive provider.

## Commit Message

```
feat(contract): OracleConsumer — provider registry, report submission, challenge window
```
