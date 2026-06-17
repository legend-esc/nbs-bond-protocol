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

### CouponEngine
```rust
// Public functions
pub fn distribute_coupon(...)
pub fn accrued_credits(...)
pub fn get_coupon_history(...)
pub fn get_coupon_status(...)
```

### OracleConsumer
```rust
// Public functions
pub fn register_provider(...)
pub fn submit_report(...)
pub fn challenge_report(...)
pub fn resolve_challenge(...)
pub fn get_report(...)
pub fn get_provider(...)
```

### DEXRouter
```rust
// Public functions
pub fn list_bond_tokens(...)
pub fn execute_purchase(...)
pub fn cancel_listing(...)
pub fn get_order(...)
pub fn get_orders_by_seller(...)
```

### ProjectRegistry
```rust
// Public functions
pub fn register_project(...)
pub fn approve_project(...)
pub fn reject_project(...)
pub fn get_project(...)
pub fn get_all_projects(...)
```

### CreditRetirement
```rust
// Public functions
pub fn retire_credits(...)
pub fn get_retirement_record(...)
pub fn get_retired_balance(...)
```

## Storage Layout

| Contract | DataKey | Value Type | Description |
|----------|---------|------------|-------------|
| BondIssuer | Bond(bond_id) | BondConfig | Bond configuration |
| BondIssuer | HolderBalance(bond_id, holder) | i128 | Token balance |
| BondIssuer | BondState(bond_id) | BondState | Current bond state |
| CouponEngine | Coupon(bond_id, period) | CouponData | Coupon distribution |
| CouponEngine | Accrued(bond_id, holder) | i128 | Accrued credits |
| OracleConsumer | Report(report_id) | OracleReport | Measurement report |
| OracleConsumer | Provider(addr) | ProviderInfo | Oracle provider |
| DEXRouter | Order(order_id) | OrderData | Marketplace order |
| ProjectRegistry | Project(project_id) | ProjectInfo | Project record |

## Cross-Contract Calls

```
ProjectRegistry ──► BondIssuer (verify project exists)
BondIssuer ──► CouponEngine (distribute coupons)
CouponEngine ──► OracleConsumer (read verified reports)
DEXRouter ──► BondIssuer (verify bond token ownership)
CreditRetirement ──► CouponEngine (verify credit ownership)
```

## API Layer

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /bonds | Issue a new bond tranche |
| GET | /bonds | List active bond tranches |
| GET | /bonds/:id | Get bond details |
| POST | /bonds/:id/subscribe | Subscribe to bond |
| GET | /bonds/:id/holders | List token holders |
| POST | /bonds/:id/coupon | Trigger coupon distribution |
| POST | /projects | Register project |
| GET | /projects | List projects |
| GET | /projects/:id | Get project details |
| POST | /projects/:id/documents | Upload IPFS docs |
| GET | /marketplace/orders | List open orders |
| POST | /marketplace/list | List tokens for sale |
| POST | /marketplace/buy | Purchase tokens |
| GET | /marketplace/prices | Current prices |
| POST | /oracle/reports | Submit oracle report |
| GET | /oracle/reports/:projectId | Get project oracle history |
| POST | /oracle/challenge/:reportId | Challenge a report |

## Frontend

### Component Tree
```
AppComponent
├── WalletButtonComponent
├── DashboardComponent
│   ├── BondCardComponent
│   └── ProjectCardComponent
├── ProjectsListComponent
│   └── ProjectCardComponent
├── ProjectDetailComponent
│   └── StatusBadgeComponent
├── ProjectCreateComponent
├── BondsListComponent
│   ├── BondCardComponent
│   └── StatusBadgeComponent
├── BondDetailComponent
│   ├── StatusBadgeComponent
│   └── LoadingSpinnerComponent
├── IssueBondComponent
├── MarketplaceListComponent
│   ├── StatusBadgeComponent
│   └── LoadingSpinnerComponent
├── MarketplaceSellComponent
└── AuthComponent
```

### Route Map
```
/ → redirect to /dashboard
/dashboard → DashboardComponent
/projects → ProjectsListComponent
/projects/new → ProjectCreateComponent
/projects/:id → ProjectDetailComponent
/bonds → BondsListComponent
/bonds/issue → IssueBondComponent
/bonds/:id → BondDetailComponent
/marketplace → MarketplaceListComponent
/marketplace/sell → MarketplaceSellComponent
/auth → AuthComponent
```
