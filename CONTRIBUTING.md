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
