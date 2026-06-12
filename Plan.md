# NbS Bond Protocol — 16-Day Development Sprint

**Goal:** 55% project completion — a robust, contributor-ready foundation with core smart contracts, API backbone, dev environment, and testing infrastructure fully operational.

**Team:** 2–3 developers (1 Rust/Soroban, 1 NestJS/Angular, 1 cross-cutting)
**Theme:** *Foundations first. Contracts before API. Test everything.*

---

## Phase 1 — Project Scaffold & Dev Environment (Days 1–2)

### Day 1 — Monorepo, Toolchain, Shared Types

| Area | Deliverable | Details |
|------|-------------|---------|
| **Toolchain** | Rust workspace + Soroban CLI pinned | `Cargo.toml` workspace at `contracts/` with all 6 contract crates + `shared` crate |
| **Shared types** | `contracts/shared/src/types.rs` | `BondConfig`, `OracleReport`, `CreditType`, `BondId`, `ReportId`, `OrderId`, `CouponResult` |
| **Shared errors** | `contracts/shared/src/errors.rs` | `BondError`, `OracleError`, `DEXError`, `RegistryError`, `CreditError` with `#[contracterror]` derives |
| **API scaffold** | NestJS app with modules | `nest new api` — generate `bonds`, `projects`, `oracle`, `marketplace`, `auth`, `stellar` modules |
| **Frontend scaffold** | Angular workspace | `ng new frontend` — generate `dashboard`, `projects`, `marketplace`, `bonds`, `auth` modules |
| **Commit** | `chore: scaffold monorepo, shared types, and API/frontend generators` | |

### Day 2 — Dev Environment, CI, Docker, Scripts

| Area | Deliverable | Details |
|------|-------------|---------|
| **Docker** | `docker-compose.yml` | API + PostgreSQL 16 + Redis 7 + IPFS node (kubo) |
| **CI** | `.github/workflows/ci.yml` | Rust `cargo test` / `cargo clippy`, API `npm run test`, linting |
| **Env** | `.env.example` + env validation | Stellar testnet defaults, Pinata placeholders, JWT config |
| **Scripts** | `scripts/deploy-testnet.sh` | Deploy all 6 contracts, write addresses to `.env` |
| **Scripts** | `scripts/seed-testnet.ts` | Seed 2 sample projects + 1 bond tranche on testnet |
| **IPFS utils** | `ipfs/upload.ts` | Upload + pin JSON to IPFS via Pinata, return content hash |
| **IPFS schemas** | `ipfs/schemas/` | JSON Schema for `project-prospectus` and `oracle-report` |
| **Commit** | `chore: docker, CI, deploy scripts, and IPFS utilities` | |

**Phase 1 completion: 10%**

---

## Phase 2 — Smart Contracts (Days 3–8)

### Day 3 — `ProjectRegistry` Contract

| Area | Deliverable | Details |
|------|-------------|---------|
| **Contract** | `contracts/project-registry/src/lib.rs` | ~180 lines. Register projects, approve/reject, query by ID/status |
| **Storage** | `Project` struct + `DataKey` enum | Fields: `id`, `owner`, `metadata_ipfs_hash`, `status` (Pending/Approved/Rejected/Inactive), `methodology`, `location` |
| **Functions** | `register_project`, `approve_project`, `reject_project`, `get_project`, `list_projects` | Admin-guarded approval; `require_auth()` on owner for registration |
| **Testing** | `mod test` with 5 unit tests | Happy path registration, duplicate rejection, admin-only approval, query non-existent, status transitions |
| **Commit** | `feat(contract): ProjectRegistry — register, approve, query` | |

### Day 4 — `BondIssuer` Contract

| Area | Deliverable | Details |
|------|-------------|---------|
| **Contract** | `contracts/bond-issuer/src/lib.rs` | ~280 lines. Issue bonds, subscribe, manage supply |
| **Storage** | `BondConfig`, `BondState`, `HolderBalance` | Track total supply, subscribed supply, maturity, coupon schedule |
| **Functions** | `issue_bond`, `subscribe`, `redeem`, `get_bond`, `get_holder_balance`, `total_supply` | Nonce-based replay protection on `subscribe` and `redeem`; check project is Approved via cross-contract call |
| **Edge cases** | Overflow-safe arithmetic, max supply cap, post-maturity rejection | Rust `checked_add`/`checked_sub` on all supply math |
| **Testing** | 8 unit tests | Issue, subscribe (pro-rata), subscribe max, double-subscribe with nonce replay, redeem at maturity, redeem early (reject), non-existent bond, zero-amount subscribe |
| **Commit** | `feat(contract): BondIssuer — issue, subscribe, redeem with nonce` | |

### Day 5 — `CouponEngine` Contract

| Area | Deliverable | Details |
|------|-------------|---------|
| **Contract** | `contracts/coupon-engine/src/lib.rs` | ~200 lines. Read oracle reports, calculate pro-rata, distribute credits |
| **Storage** | `CouponPeriod`, `CreditAllocation` | Period index → total credits, per-holder accrued map |
| **Functions** | `distribute_coupon`, `accrued_credits`, `claim_credits`, `get_period_credits` | Read latest `OracleReport` for the bond's project; calculate `credits_period * (holder_tokens / total_tokens)` |
| **Edge cases** | Zero holders (escrow), partial period, division precision | Use `fixed_point` multiplier (1e7) to avoid truncation |
| **Testing** | 6 unit tests | Distribute to 1 holder, distribute to 3 holders pro-rata, zero holders (no-op), double-distribute (idempotent), query accrued before claim, claim after distribute |
| **Commit** | `feat(contract): CouponEngine — pro-rata credit distribution` | |

### Day 6 — `OracleConsumer` Contract

| Area | Deliverable | Details |
|------|-------------|---------|
| **Contract** | `contracts/oracle-consumer/src/lib.rs` | ~250 lines. Provider registry, report submission, challenge window |
| **Storage** | `OracleProvider`, `Report`, `Challenge` | Provider whitelist with stake; report with status (Pending/Verified/Challenged/Rejected) |
| **Functions** | `register_provider`, `remove_provider`, `submit_report`, `verify_report`, `challenge_report`, `resolve_challenge` | 72-hour challenge window via block timestamp; provider signatures; multi-sig threshold support |
| **Edge cases** | Late challenge rejection, double-submit prevention, dispute freeze on coupons | Challenge freezes associated coupon distributions |
| **Testing** | 7 unit tests | Register provider, submit + verify, submit + challenge + resolve, late challenge (reject), non-whitelisted submit (reject), duplicate submit (reject), multi-sig threshold |
| **Commit** | `feat(contract): OracleConsumer — provider registry, reports, challenge` | |

### Day 7 — `DEXRouter` Contract

| Area | Deliverable | Details |
|------|-------------|---------|
| **Contract** | `contracts/dex-router/src/lib.rs` | ~200 lines. List, buy, cancel orders on Stellar DEX |
| **Storage** | `Order`, `OrderBook` | seller, bond_id, amount, price_per_token, status (Open/Filled/Cancelled) |
| **Functions** | `list_bond_tokens`, `cancel_listing`, `execute_purchase`, `get_order`, `list_orders` | Abstracts Stellar native offers; path payment support via `stellar_asset` helper |
| **Edge cases** | Partial fill, self-buy rejection, expired order, insufficient balance | Check holder balance against `BondIssuer` before listing |
| **Testing** | 6 unit tests | List tokens, execute full purchase, partial fill, cancel listing, self-buy reject, expired order cleanup |
| **Commit** | `feat(contract): DEXRouter — list, buy, cancel on Stellar DEX` | |

### Day 8 — `CreditRetirement` + Contract Integration Tests

| Area | Deliverable | Details |
|------|-------------|---------|
| **Contract** | `contracts/credit-retirement/src/lib.rs` | ~120 lines. Retire credits, mint NFT certificate |
| **Functions** | `retire_credits`, `get_retirement_record`, `get_retirement_certificate` | Burn credits, emit `RetirementEvent`, mint metadata-bound certificate |
| **Integration** | Cross-contract integration tests | Full flow: register project → issue bond → subscribe → submit oracle report → distribute coupon → retire credits |
| **Integration** | `contracts/tests/integration.rs` | ~8 end-to-end scenarios covering happy path and failure modes |
| **Audit prep** | `cargo clippy --all-targets` clean | Zero warnings on all contracts |
| **Commit** | `feat(contract): CreditRetirement + full integration test suite` | |

**Phase 2 completion: 40%**

---

## Phase 3 — Backend API (Days 9–12)

### Day 9 — `StellarService` + `ContractService`

| Area | Deliverable | Details |
|------|-------------|---------|
| **Service** | `api/src/stellar/stellar.service.ts` | Horizon RPC wrapper: `submitTransaction`, `getAccount`, `getBalance`, `streamPayments` |
| **Service** | `api/src/stellar/contract.service.ts` | Soroban contract interaction: `callContract`, `simulateContract`, `sendTransaction` — typed wrappers for all 6 contracts |
| **Validation** | Zod schemas for all contract payloads | Match shared Rust types exactly |
| **Testing** | `stellar.service.spec.ts` + `contract.service.spec.ts` | Mocked Horizon responses; 10+ tests |
| **Commit** | `feat(api): StellarService and typed ContractService` | |

### Day 10 — `BondsModule` + `ProjectsModule`

| Area | Deliverable | Details |
|------|-------------|---------|
| **Controller** | `bonds.controller.ts` | `POST /bonds`, `GET /bonds`, `GET /bonds/:id`, `POST /bonds/:id/subscribe`, `POST /bonds/:id/coupon` |
| **Service** | `bonds.service.ts` | Orchestrate `BondIssuer` contract calls, cache bond state in Redis, emit events |
| **Controller** | `projects.controller.ts` | `POST /projects`, `GET /projects`, `GET /projects/:id`, `POST /projects/:id/documents` |
| **Service** | `projects.service.ts` | Orchestrate `ProjectRegistry` + IPFS upload; validate project metadata |
| **DTOs** | Full `class-validator` DTOs | `CreateBondDto`, `SubscribeDto`, `CreateProjectDto`, `UploadDocumentDto` |
| **Testing** | Controller + service specs | 15+ tests with mocked contract layer |
| **Commit** | `feat(api): Bonds and Projects CRUD endpoints` | |

### Day 11 — `OracleModule` + `MarketplaceModule`

| Area | Deliverable | Details |
|------|-------------|---------|
| **Controller** | `oracle.controller.ts` | `POST /oracle/reports`, `GET /oracle/reports/:projectId`, `POST /oracle/challenge/:reportId` |
| **Service** | `oracle.service.ts` | Poll oracle providers, submit reports, monitor challenges |
| **Scheduler** | `oracle.scheduler.ts` | `@Cron` job every 5min to check for pending oracle data |
| **Providers** | `verra.provider.ts`, `satellite.provider.ts` | Adapter interface + 2 implementations (mock fetch) |
| **Controller** | `marketplace.controller.ts` | `GET /marketplace/orders`, `POST /marketplace/list`, `POST /marketplace/buy`, `GET /marketplace/prices` |
| **Service** | `marketplace/dex.service.ts` | DEX order management via `DEXRouter` contract |
| **Service** | `marketplace/liquidity.service.ts` | Price feed aggregation, slippage calculation |
| **Testing** | Oracle + marketplace specs | 15+ tests with mocked providers and contracts |
| **Commit** | `feat(api): Oracle feeds and marketplace DEX integration` | |

### Day 12 — `AuthModule`, Middleware, Error Handling

| Area | Deliverable | Details |
|------|-------------|---------|
| **Auth** | `auth.controller.ts` + `auth.service.ts` | Wallet-based auth: challenge/response with Stellar keypair |
| **JWT** | `jwt.strategy.ts` | Passport strategy; JWT payload includes `walletAddress` and `kycStatus` |
| **KYC** | `kyc.service.ts` | Integration stub for third-party KYC provider; allow-list caching in Redis |
| **Middleware** | Global exception filter + logging interceptor | Structured error responses (RFC 7807 problem details); request ID tracing |
| **Guards** | `KycGuard`, `AdminGuard`, `ProviderGuard` | Reusable NestJS guards for route protection |
| **Testing** | Auth flow E2E | Challenge → sign → verify → JWT → protected route; 10+ tests |
| **Commit** | `feat(api): Auth, KYC guards, global error handling` | |

**Phase 3 completion: 50%**

---

## Phase 4 — Frontend & Polish (Days 13–16)

### Day 13 — Shared UI Library + Stellar Wallet Connect

| Area | Deliverable | Details |
|------|-------------|---------|
| **Shared** | Angular shared module | Reusable components: `BondCard`, `ProjectCard`, `StatusBadge`, `WalletButton`, `LoadingSpinner` |
| **Wallet** | Stellar wallet connector | `@stellar/freighter` + `@stellar/wallet-sdk` integration; connect/disconnect, sign, network detection |
| **Services** | `stellar.service.ts`, `api.service.ts` | Typed HTTP client (auto-generated from API DTOs); wallet RPC wrapper |
| **Routing** | App routing module + lazy loading | Dashboard, Projects, Marketplace, Bonds, Auth — all lazy-loaded |
| **Commit** | `feat(ui): shared components, wallet connect, routing` | |

### Day 14 — Dashboard + Project Registry Pages

| Area | Deliverable | Details |
|------|-------------|---------|
| **Dashboard** | `dashboard/` | Portfolio view: active bonds, accrued credits, portfolio value (mock data → API) |
| **Projects** | `projects/` | Project list + detail page; IPFS document viewer; oracle history chart |
| **IPFS viewer** | IPFS content resolver | Fetch + render JSON docs from IPFS gateway |
| **State** | NgRx store (or signals) | `BondState`, `ProjectState`, `WalletState` — selectors + effects |
| **Commit** | `feat(ui): dashboard and project registry pages` | |

### Day 15 — Bond Issuance + Marketplace Interface

| Area | Deliverable | Details |
|------|-------------|---------|
| **Bonds** | `bonds/` | Issuance form (issuer role), subscription flow, bond detail with coupon timeline |
| **Marketplace** | `marketplace/` | Order book view, buy/sell interface, price chart, trade history |
| **Forms** | Reactive forms with validation | Aligned to API DTOs; Stellar address validation; amount precision |
| **Commit** | `feat(ui): bond issuance flow and marketplace trading UI` | |

### Day 16 — Final Integration, Documentation, CI Polish

| Area | Deliverable | Details |
|------|-------------|---------|
| **Integration** | End-to-end test script | `scripts/e2e-test.sh` — deploys contracts, seeds data, hits API, verifies frontend renders |
| **Docs** | `CONTRIBUTING.md` update | Dev setup guide, coding conventions, PR template, review checklist |
| **Docs** | `docs/architecture.md` | Updated with final contract interfaces and API routes |
| **Docs** | `docs/oracle-design.md` | Oracle provider spec, data flow, challenge mechanics |
| **CI** | GitHub Actions polish | Add contract deployment to testnet on merge to `main`; add `npm audit` and `cargo audit` |
| **Readme** | Badge updates | CI passing, test coverage, contract count |
| **Commit** | `chore: e2e tests, documentation, and CI finalization` | |

**Phase 4 completion: 55%**

---

## Completion Summary

| Component | Status | What's Done | What's Left (45%) |
|-----------|--------|-------------|-------------------|
| **Smart Contracts** (6) | ✅ 100% | All contracts implemented, unit-tested, integration-tested, clippy-clean | Production audit, multi-sig admin UI, upgrade mechanism |
| **API** (NestJS) | ✅ 80% | All modules, controllers, services, guards, DTOs, error handling | Rate limiting, caching strategy, full E2E coverage, webhook system |
| **Frontend** (Angular) | 🔶 40% | Scaffold, shared lib, wallet connect, dashboard, projects, bonds, marketplace | Mature UX, mobile responsive, real-time subscriptions, i18n |
| **Oracle Adapters** | ✅ 50% | Interface defined, 2 mock providers, scheduler, Verra/satellite/IoT adapter scripts | Real Verra/Gold Standard integration, production monitoring |
| **DevOps** | ✅ 95% | Docker, CI, deploy scripts (testnet+mainnet), seed scripts | Monitoring/alerting, HSM integration |
| **Docs** | 🔶 65% | Architecture, oracle design, credit methodology, governance, contributing guide, SECURITY.md, CODE_OF_CONDUCT.md | API reference auto-generation |
| **Testing** | ✅ Core | Contract unit (38) + integration (8), API unit (50+), E2E script | Fuzz testing, property-based testing, load testing |

---

## Key Decisions & Conventions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Rust edition | 2021 | Soroban stable compatibility |
| Fixed-point precision | 1e7 (7 decimals) | Balances against overflow in i128 range |
| Nonce scheme | `u64` per-address counter, stored in contract data | Simple, deterministic, replay-proof |
| API validation | `class-validator` + Zod | NestJS-native + cross-boundary schema sharing |
| State management | NgRx Signals | Angular 17+ idiomatic; lighter than classic NgRx Store |
| Contract upgrade | Timelock pattern (48hr delay) in each contract | Aligned with security model in README |
| Error handling | RFC 7807 problem details | Standardized, machine-parseable API errors |

---

## How to Contribute After Day 16

1. **Pick an area** from the "What's Left" column above
2. **Check** `contracts/`, `api/`, `frontend/` for existing patterns
3. **Run** the e2e test suite to verify nothing is broken
4. **Open a PR** — CI will run tests + lint + contract deployment dry-run
5. **Tag** reviewers by area: `@core-contracts`, `@api-team`, `@ui-team`

The protocol at 55% has enough structure to be *useful* on testnet — issue bonds, subscribe, receive mock credits, trade on DEX — while leaving ample room for meaningful contributor impact.
