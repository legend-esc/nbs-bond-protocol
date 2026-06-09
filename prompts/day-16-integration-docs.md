# Day 16 — E2E Testing, Documentation, CI Polish

Load context: `prompts/context/tech-stack.md`

## Goal

Write the end-to-end test script, update documentation, and polish the CI/CD pipeline. This is the "ship it" day that makes the 55% milestone tangible.

## Files to Create / Update

### E2E Test Script

#### `scripts/e2e-test.sh`

A bash script that validates the full stack works end-to-end:

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "=== NbS Bond Protocol — E2E Smoke Test ==="

# 1. Check toolchain
echo "[1/6] Checking toolchain..."
cargo --version && soroban --version && node --version && npm --version

# 2. Build all contracts
echo "[2/6] Building contracts..."
cd contracts
cargo build --release
cd ..

# 3. Run contract unit + integration tests
echo "[3/6] Running contract tests..."
cd contracts
cargo test -- --nocapture
cd ..

# 4. Build API
echo "[4/6] Building API..."
cd api
npm ci --silent
npm run build
cd ..

# 5. Build Frontend
echo "[5/6] Building Frontend..."
cd frontend
npm ci --silent
npm run build
cd ..

# 6. Summary
echo "[6/6] === ALL CHECKS PASSED ==="
echo "  Contracts: $(find contracts -name '*.wasm' | wc -l) wasm files"
echo "  API: $(find api/dist -name '*.js' | wc -l) compiled modules"
echo "  Frontend: $(find frontend/dist -name '*.js' | wc -l) compiled bundles"
echo "  Tests: $(grep -r '#\[test\]' contracts --include='*.rs' | wc -l) Rust tests"
```

### Documentation

#### `CONTRIBUTING.md` — Full Rewrite

```markdown
# Contributing to NbS Bond Protocol

## Quick Start
1. Install prerequisites (Rust, Node 20, Soroban CLI)
2. `cp .env.example api/.env` and fill in testnet values
3. `cd contracts && cargo test` — verify contracts compile
4. `cd api && npm install && npm run start:dev` — start API
5. `cd frontend && npm install && ng serve` — start UI

## Code Conventions
- **Rust:** `cargo clippy --all-targets -- -D warnings` before committing
- **TypeScript:** `npm run lint` — ESLint with NestJS/Angular presets
- **Tests:** Every public function must have at least one test
- **Commits:** Conventional commits (`feat:`, `fix:`, `chore:`, `docs:`)

## PR Checklist
- [ ] `cargo test` passes
- [ ] `cargo clippy` is clean
- [ ] `npm run test` passes (API + Frontend)
- [ ] `npm run build` succeeds (API + Frontend)
- [ ] New types are documented
- [ ] New endpoints have DTO validation
- [ ] PR description explains the change and motivation

## Architecture
See `docs/architecture.md` for contract interfaces and API routes.
```

#### `docs/architecture.md` — New

Write based on the README architecture section but updated with:
- Final contract interfaces (all 6 contracts' public function signatures in Rust doc format)
- API route table (from day 10-12 implementation)
- Data flow diagrams (ASCII art from README, adapted)
- Key storage layouts (DataKey enums for each contract)
- Cross-contract call graph (BondIssuer→CouponEngine, DEXRouter→BondIssuer, etc.)

Structure:
```markdown
# Architecture

## Smart Contracts
### BondIssuer
```rust
// Public functions
pub fn issue_bond(...)
pub fn subscribe(...)
pub fn redeem(...)
pub fn mature_bond(...)
pub fn get_bond(...)
pub fn get_bond_state(...)
pub fn get_holder_balance(...)
```
... repeat for each contract

## Storage Layout
Table per contract: DataKey → Value Type → Description

## Cross-Contract Calls
ASCII arrow diagram showing which contracts call which

## API Layer
Full route table from API controllers

## Frontend
Component tree and route map
```

#### `docs/oracle-design.md` — New

Document the oracle architecture:
```markdown
# Oracle Design

## Architecture
Multi-source, multi-layer: Auditors + Satellite + IoT → OracleConsumer contract

## Provider Lifecycle
Register → Whitelisted → Submit Reports → Challenge Window → Verify/Reject

## Report Format
```
{
  project_id: BytesN<32>,
  period_start: u64,
  period_end: u64,
  carbon_sequestered: i128,
  methodology: Symbol,
  provider_signature: BytesN<64>,
  ipfs_evidence_hash: BytesN<32>,
}
```

## Challenge Mechanism
- 72-hour window from submission
- Any address can challenge with counter-evidence (IPFS hash)
- Admin resolves via on-chain vote

## Security Model
- Provider whitelist (admin-managed)
- Stake requirement (future)
- Multi-sig for high-value reports
```

#### `docs/credit-methodology.md` — New

```markdown
# Credit Methodology

## Carbon Credit Calculation

credits_per_period = (carbon_sequestered_kg / 1000) * credit_conversion_factor

Where `credit_conversion_factor` is set at bond issuance per methodology:
- VERRA-VCS: 1.0 (standard)
- GOLD-STANDARD: 1.0
- ACR: 0.95 (conservative)
- CAR: 1.05 (includes buffer pool)

## Biodiversity Credit Calculation

Biodiversity credits are calculated using project-specific metrics:
- Habitat hectares restored
- Species Abundance Index (SAI) improvement
- Biodiversity Unit (UK BNG methodology)

## Oracle Data Sources
- Accredited Auditors: annual baseline verification
- Satellite Imagery: monthly NDVI/biomass proxy
- IoT Sensors: continuous soil carbon/moisture
- Community Monitors: quarterly species surveys
```

#### `docs/governance.md` — New

```markdown
# Governance

## Multi-Stakeholder Committee
- Project Developers
- Bond Issuers
- Oracle Providers
- Protocol Maintainers
- Token Holders

## Governance Actions (3-of-5 Multi-sig)
- Add/remove oracle providers
- Update credit conversion factors
- Deploy contract upgrades (48h timelock)
- Modify KYC requirements
- Adjust dispute resolution parameters
```

#### `SECURITY.md` — New

```markdown
# Security Disclosure

## Reporting a Vulnerability
Email security@nbs-bond-protocol.org with:
- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- (Optional) Suggested fix

## Bug Bounty
Critical smart contract vulnerabilities: up to $50,000 USDC
Response SLA: 48h acknowledgement, 7 days triage

## Scope
- Soroban smart contracts (contracts/)
- NestJS API (api/)
- Oracle adapter scripts (oracle/)
```

#### `CODE_OF_CONDUCT.md` — New

```markdown
# Code of Conduct

## Our Pledge
We pledge to make participation in NbS Bond Protocol a harassment-free experience for everyone.

## Standards
- Use welcoming and inclusive language
- Be respectful of differing viewpoints
- Accept constructive criticism gracefully
- Focus on what is best for the ecosystem

## Enforcement
Project maintainers are responsible for clarifying standards and will take appropriate action.
```

#### `.github/ISSUE_TEMPLATE/bug_report.md` — New

```markdown
---
name: Bug Report
about: Report a bug in the protocol
title: '[BUG] '
labels: bug
assignees: ''
---

**Describe the bug**
A clear description of the issue.

**To Reproduce**
1. Deploy contracts with '...'
2. Call function '...'
3. See error

**Expected behavior**
What should happen.

**Environment**
- Rust version:
- Soroban SDK version:
- Network: testnet/mainnet
```

#### `.github/ISSUE_TEMPLATE/feature_request.md` — New

```markdown
---
name: Feature Request
about: Suggest an improvement
title: '[FEATURE] '
labels: enhancement
assignees: ''
---

**Problem**
What problem does this solve?

**Solution**
What should be built?

**Alternatives considered**
What else could work?
```

### CI Improvements

#### `.github/workflows/ci.yml` — Update

Add to existing CI:
- **Testnet deployment job** (manual trigger via `workflow_dispatch`):
  ```yaml
  deploy-testnet:
    if: github.ref == 'refs/heads/main' && github.event_name == 'workflow_dispatch'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Deploy contracts
        run: ./scripts/deploy-testnet.sh
        env:
          STELLAR_SECRET_KEY: ${{ secrets.STELLAR_TESTNET_SECRET }}
  ```
- **Security audit**: Add `cargo audit` and `npm audit` steps
- **Code coverage**: Add `cargo tarpaulin` (contracts) and `npm run test:cov` (API) steps

### README Badge Update

Ensure `README.md` badge section includes:
- CI status (from `.github/workflows/ci.yml`)
- Test coverage (placeholder)
- License (MIT — already there)
- Contract count (6)

### Docker Compose Polish

Update `docker-compose.yml` to add healthchecks:

```yaml
services:
  postgres:
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U nbs"]
      interval: 5s
      timeout: 5s
      retries: 5

  api:
    depends_on:
      postgres: { condition: service_healthy }
      redis: { condition: service_started }
```

## Verification

```bash
# E2E smoke test
cd scripts && bash e2e-test.sh

# All contracts pass
cd contracts && cargo test && cargo clippy --all-targets -- -D warnings

# API builds and tests pass
cd api && npm run build && npm run test

# Frontend builds
cd frontend && npm run build

# Docker compose config is valid
docker compose config --quiet && echo "Docker OK"
```

## Commit Message

```
chore: e2e test suite, architecture docs, CI polish, and Docker healthchecks
```
