# Day 1 — Monorepo Scaffold, Shared Types, API & Frontend Generators

Load context: `prompts/context/tech-stack.md`, `prompts/context/soroban-patterns.md`, `prompts/context/api-patterns.md`, `prompts/context/frontend-patterns.md`

## Goal

Create the Rust workspace with all contract crates + shared types, scaffold the NestJS API, and scaffold the Angular frontend.

## Files to Create

### Rust Workspace

#### `contracts/Cargo.toml`
Workspace root referencing all 6 contract crates + `shared` + `tests`:
- `shared`
- `bond-issuer`
- `coupon-engine`
- `oracle-consumer`
- `dex-router`
- `project-registry`
- `credit-retirement`
- `tests`

Set `resolver = "2"`, `version = "0.1.0"`, `edition = "2021"` in `[workspace.package]`.

#### `contracts/shared/Cargo.toml`
```toml
[package]
name = "nbbs-shared"
version.workspace = true
edition.workspace = true

[dependencies]
soroban-sdk = "20.4.0"

[features]
testutils = ["soroban-sdk/testutils"]
```

#### `contracts/shared/src/lib.rs`
```rust
#![no_std]
mod types;
mod errors;
pub use types::*;
pub use errors::*;
```

#### `contracts/shared/src/types.rs`
All these types with `#[derive(Clone)]` `#[contracttype]`:

```rust
#[derive(Clone, Copy, PartialEq)]
#[contracttype]
pub enum CreditType {
    Carbon,
    Biodiversity,
    Basket,
}

#[derive(Clone)]
#[contracttype]
pub struct BondConfig {
    pub project_id: BytesN<32>,       // IPFS content hash
    pub face_value: i128,              // in stroops
    pub coupon_schedule: Vec<u64>,     // unix timestamps
    pub credit_type: CreditType,
    pub maturity_date: u64,
    pub total_supply: i128,
}

pub type BondId = u64;
pub type ReportId = u64;
pub type OrderId = u64;

#[derive(Clone)]
#[contracttype]
pub struct OracleReport {
    pub project_id: BytesN<32>,
    pub period_start: u64,
    pub period_end: u64,
    pub carbon_sequestered: i128,     // kg CO2e
    pub methodology: Symbol,
    pub provider_signature: BytesN<64>,
    pub ipfs_evidence_hash: BytesN<32>,
}

#[derive(Clone, Copy, PartialEq)]
#[contracttype]
pub enum BondStatus {
    Active,
    Matured,
    Defaulted,
}

#[derive(Clone, Copy, PartialEq)]
#[contracttype]
pub enum ProjectStatus {
    Pending,
    Approved,
    Rejected,
    Inactive,
}

#[derive(Clone, Copy, PartialEq)]
#[contracttype]
pub enum ReportStatus {
    Pending,
    Verified,
    Challenged,
    Rejected,
}
```

#### `contracts/shared/src/errors.rs`
Each error enum uses `#[contracterror]`:

```rust
#[derive(Clone, Debug, PartialEq)]
#[contracterror]
pub enum BondError {
    NotInitialized = 1,
    Unauthorized = 2,
    InvalidNonce = 3,
    BondNotFound = 4,
    BondAlreadyMatured = 5,
    InsufficientSupply = 6,
    ZeroAmount = 7,
    ProjectNotApproved = 8,
    Overflow = 9,
}

#[derive(Clone, Debug, PartialEq)]
#[contracterror]
pub enum OracleError {
    NotInitialized = 1,
    Unauthorized = 2,
    InvalidNonce = 3,
    ProviderNotFound = 4,
    ProviderAlreadyExists = 5,
    ReportNotFound = 6,
    ReportAlreadyVerified = 7,
    ChallengeWindowExpired = 8,
    InsufficientStake = 9,
    InvalidSignature = 10,
}

#[derive(Clone, Debug, PartialEq)]
#[contracterror]
pub enum DEXError {
    NotInitialized = 1,
    Unauthorized = 2,
    InvalidNonce = 3,
    OrderNotFound = 4,
    OrderAlreadyFilled = 5,
    InsufficientBalance = 6,
    SelfBuyNotAllowed = 7,
    OrderExpired = 8,
}

#[derive(Clone, Debug, PartialEq)]
#[contracterror]
pub enum RegistryError {
    NotInitialized = 1,
    Unauthorized = 2,
    ProjectNotFound = 3,
    ProjectAlreadyExists = 4,
    InvalidStatusTransition = 5,
}

#[derive(Clone, Debug, PartialEq)]
#[contracterror]
pub enum CreditError {
    NotInitialized = 1,
    Unauthorized = 2,
    InsufficientCredits = 3,
    AlreadyRetired = 4,
    InvalidNonce = 5,
}
```

#### Individual Contract Crates

For each of `bond-issuer`, `coupon-engine`, `oracle-consumer`, `dex-router`, `project-registry`, `credit-retirement`:

Create `contracts/{name}/Cargo.toml` using the template from `soroban-patterns.md` with:
- package name: `nbbs-{name}`
- dependency on `nbbs-shared = { path = "../shared" }`
- dependency on `soroban-sdk = "20.4.0"`

Create `contracts/{name}/src/lib.rs` with:
```rust
#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env};

#[contract]
pub struct ContractName;

#[contractimpl]
impl ContractName {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }
}
```

Use uppercase CamelCase for `ContractName` matching the crate purpose (e.g., `BondIssuer`, `CouponEngine`).

#### `contracts/tests/Cargo.toml`
```toml
[package]
name = "nbbs-tests"
version.workspace = true
edition.workspace = true

[dependencies]
soroban-sdk = "20.4.0"
nbbs-bond-issuer = { path = "../bond-issuer" }
nbbs-coupon-engine = { path = "../coupon-engine" }
nbbs-oracle-consumer = { path = "../oracle-consumer" }
nbbs-dex-router = { path = "../dex-router" }
nbbs-project-registry = { path = "../project-registry" }
nbbs-credit-retirement = { path = "../credit-retirement" }
nbbs-shared = { path = "../shared" }
```

Create `contracts/tests/src/lib.rs` with a placeholder `#[test] fn placeholder() {}`.

### NestJS API

Run `nest new api --skip-install --skip-git` equivalent — manually create:

#### `api/package.json`
Use the dependencies from `api-patterns.md` plus:
- `@nestjs/cli` as devDependency
- `typescript`, `ts-node`, `ts-loader`
- `jest`, `@types/jest`, `supertest`, `@types/supertest`
- `@nestjs/testing`
- Scripts: `start:dev`, `build`, `test`, `test:e2e`

#### `api/tsconfig.json`
```json
{
  "compilerOptions": {
    "module": "commonjs",
    "declaration": true,
    "removeComments": true,
    "emitDecoratorMetadata": true,
    "experimentalDecorators": true,
    "allowSyntheticDefaultImports": true,
    "target": "ES2021",
    "sourceMap": true,
    "outDir": "./dist",
    "baseUrl": "./",
    "incremental": true,
    "skipLibCheck": true,
    "strictNullChecks": true,
    "noImplicitAny": true,
    "strictBindCallApply": true,
    "forceConsistentCasingInFileNames": true,
    "noFallthroughCasesInSwitch": true,
    "paths": { "@/*": ["src/*"] }
  }
}
```

#### `api/src/main.ts`
Standard NestJS bootstrap on `process.env.PORT || 3000`, with `app.enableCors()`, `app.setGlobalPrefix('api')`.

#### `api/src/app.module.ts`
```typescript
@Module({
  imports: [
    BondsModule,
    ProjectsModule,
    OracleModule,
    MarketplaceModule,
    AuthModule,
    StellarModule,
  ],
})
export class AppModule {}
```

Create each module file as a stub:
- `api/src/bonds/bonds.module.ts` — `@Module({ controllers: [], providers: [] })`
- `api/src/projects/projects.module.ts`
- `api/src/oracle/oracle.module.ts`
- `api/src/marketplace/marketplace.module.ts`
- `api/src/auth/auth.module.ts`
- `api/src/stellar/stellar.module.ts`

#### `api/nest-cli.json`
```json
{ "$schema": "https://json.schemastore.org/nest-cli", "collection": "@nestjs/schematics", "sourceRoot": "src", "compilerOptions": { "deleteOutDir": true } }
```

#### `api/test/jest-e2e.json`
```json
{
  "moduleFileExtensions": ["js", "json", "ts"],
  "rootDir": "..",
  "testEnvironment": "node",
  "testRegex": ".e2e-spec.ts$",
  "transform": { "^.+\\.ts$": "ts-jest" },
  "moduleNameMapper": { "^@/(.*)$": "<rootDir>/src/$1" }
}
```

### Angular Frontend

Run `ng new frontend --routing --style=css --skip-install` equivalent — manually create:

#### `frontend/package.json`
```json
{
  "name": "nbs-bond-frontend",
  "dependencies": {
    "@angular/core": "^17.3.0",
    "@angular/common": "^17.3.0",
    "@angular/router": "^17.3.0",
    "@angular/forms": "^17.3.0",
    "@angular/platform-browser": "^17.3.0",
    "@angular/platform-browser-dynamic": "^17.3.0",
    "@stellar/freighter": "^2.0.0",
    "rxjs": "^7.8.0",
    "zone.js": "^0.14.0"
  },
  "devDependencies": {
    "@angular/cli": "^17.3.0",
    "@angular/compiler-cli": "^17.3.0",
    "@angular/build": "^17.3.0",
    "typescript": "~5.4.0",
    "jasmine-core": "~5.1.0",
    "karma": "~6.4.0",
    "karma-chrome-launcher": "~3.2.0",
    "karma-jasmine": "~5.1.0"
  }
}
```

#### `frontend/angular.json`
Standard Angular 17 workspace config with projects:`nbs-bond-frontend`, root `.`, src dir `src/`, main `src/main.ts`, index `src/index.html`.

#### `frontend/src/main.ts`
```typescript
import { bootstrapApplication } from '@angular/platform-browser';
import { appConfig } from './app/app.config';
import { AppComponent } from './app/app.component';

bootstrapApplication(AppComponent, appConfig).catch((err) => console.error(err));
```

#### `frontend/src/index.html`
Standard HTML5 boilerplate.

#### `frontend/src/app/app.config.ts`
Application config with `provideRouter(routes)`.

#### `frontend/src/app/app.component.ts`
Standalone component with `<router-outlet>`.

#### `frontend/src/app/app.routes.ts`
```typescript
export const routes: Routes = [
  { path: '', loadComponent: () => import('./dashboard/dashboard.component').then(m => m.DashboardComponent) },
  { path: 'projects', loadChildren: () => import('./projects/projects.routes') },
  { path: 'marketplace', loadChildren: () => import('./marketplace/marketplace.routes') },
  { path: 'bonds', loadChildren: () => import('./bonds/bonds.routes') },
  { path: 'auth', loadChildren: () => import('./auth/auth.routes') },
];
```

#### Generate stub components for each module:
- `frontend/src/app/dashboard/dashboard.component.ts` — `<p>dashboard works!</p>`
- `frontend/src/app/projects/projects.component.ts`
- `frontend/src/app/marketplace/marketplace.component.ts`
- `frontend/src/app/bonds/bonds.component.ts`
- `frontend/src/app/auth/auth.component.ts`

Each must be a standalone component with `imports: [CommonModule, RouterModule]`.

### Root `LICENSE`

```text
MIT License

Copyright (c) 2026 NbS Bond Protocol

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Verification

```bash
# Rust — compiles and tests pass
cd contracts && cargo check --all-targets && cargo test

# API — compiles
cd api && npm install && npm run build

# Frontend — compiles
cd frontend && npm install && npm run build
```

## Commit Message

```
chore: scaffold monorepo workspace, shared types, API and frontend generators
```
