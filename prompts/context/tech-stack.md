# Tech Stack & Conventions

Load this into your agent context before executing any day prompt.

## Versions

| Tool | Version |
|------|---------|
| Rust | nightly-2024-08-01 (Soroban-compatible) |
| Soroban SDK | 26.0.1 |
| Soroban CLI | 26.0.1 |
| Stellar Horizon | 20.x |
| Node.js | 20 LTS |
| NestJS | 10.4.x |
| Angular | 17.3.x |
| PostgreSQL | 16 |
| Redis | 7 |

## Project Layout

```
nbs-bond-protocol/
├── contracts/                    # Cargo workspace
│   ├── Cargo.toml               # Workspace root
│   ├── shared/                  # Shared types crate
│   ├── bond-issuer/
│   ├── coupon-engine/
│   ├── oracle-consumer/
│   ├── dex-router/
│   ├── project-registry/
│   ├── credit-retirement/
│   └── tests/                   # Integration tests
├── api/                         # NestJS app
│   ├── src/
│   │   ├── bonds/
│   │   ├── projects/
│   │   ├── oracle/
│   │   ├── marketplace/
│   │   ├── auth/
│   │   └── stellar/
│   └── test/
├── frontend/                    # Angular app
│   └── src/app/
│       ├── dashboard/
│       ├── projects/
│       ├── marketplace/
│       ├── bonds/
│       ├── auth/
│       └── shared/
├── oracle/                      # Scripts
├── ipfs/                        # IPFS utilities
├── scripts/                     # Deploy & seed
├── .github/workflows/
└── docker-compose.yml
```

## General Rules

1. Every state-changing Soroban function must accept a `nonce: u64` parameter for replay protection.
2. All arithmetic in Rust contracts must use `checked_add`, `checked_sub`, `checked_mul` — never raw `+`/`-`/`*`.
3. Contract storage keys use the `DataKey` enum pattern with `#[contracttype]`.
4. All API responses use RFC 7807 problem details for errors.
5. Angular components use standalone components (not NgModules), signals for state, and OnPush change detection.
6. Every public function must have a doc comment.
