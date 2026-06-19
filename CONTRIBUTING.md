# Contributing to NbS Bond Protocol

Thank you for your interest in contributing! We welcome contributions from smart contract engineers, climate scientists, financial modelers, oracle architects, and frontend developers.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Code Conventions](#code-conventions)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Security Disclosures](#security-disclosures)
- [Getting Help](#getting-help)

## Code of Conduct

This project is governed by the [Code of Conduct](CODE_OF_CONDUCT.md). All participants are expected to uphold its principles.

## Getting Started

### Prerequisites

- **Node.js** `v18+` and **npm** `v9+`
- **Rust** (stable toolchain) and **Cargo**
- **Soroban CLI** — `cargo install --locked soroban-cli`
- A funded **Stellar testnet account**

### Local Setup

```bash
# Clone the repository
git clone https://github.com/your-org/nbs-bond-protocol.git
cd nbs-bond-protocol

# Copy environment template
cp .env.example api/.env
# Edit api/.env with testnet values

# Install dependencies
cd api && npm install && cd ..
cd frontend && npm install && cd ..

# Build and test contracts
cd contracts && cargo build --release && cargo test && cd ..
```

## Project Structure

```
nbs-bond-protocol/
├── contracts/            # Soroban smart contracts (Rust)
│   ├── bond-issuer/
│   ├── coupon-engine/
│   ├── oracle-consumer/
│   ├── dex-router/
│   ├── project-registry/
│   ├── credit-retirement/
│   └── shared/           # Shared types and errors
├── api/                  # NestJS backend
│   └── src/
│       ├── bonds/
│       ├── oracle/
│       ├── projects/
│       ├── marketplace/
│       ├── auth/
│       └── stellar/
├── frontend/             # Angular application
├── oracle/               # Oracle adapter scripts
├── ipfs/                 # IPFS utilities & schemas
├── scripts/              # Deployment & migration scripts
├── docs/                 # Documentation
└── .github/              # CI, issue templates, PR template
```

## Development Workflow

1. **Fork** the repository
2. **Create a feature branch:** `git checkout -b feat/your-feature-name`
3. **Make changes** following the code conventions below
4. **Write tests** for new functionality
5. **Run the test suite** locally
6. **Commit** using conventional commits (see below)
7. **Push** and open a **Pull Request** against `main`

### Branch Naming

- `feat/description` — New features
- `fix/description` — Bug fixes
- `docs/description` — Documentation changes
- `chore/description` — Maintenance, CI, refactoring
- `security/description` — Security fixes

### Commit Messages

Use [conventional commits](https://www.conventionalcommits.org/):

```
feat(contract): add nonce-based replay protection to BondIssuer
fix(oracle): correct challenge window calculation
docs(readme): expand oracle security section
chore(ci): add cargo-audit to workflow
```

## Code Conventions

### Rust / Soroban Contracts

- Run `cargo clippy --all-targets -- -D warnings` before committing
- Run `cargo fmt` to format code
- Every public function must have at least one unit test
- Use `checked_add` / `checked_sub` for arithmetic
- Follow existing patterns in `contracts/shared/src/types.rs`

### TypeScript / NestJS API

- Run `npm run lint` before committing
- Use `class-validator` DTOs for request validation
- Services should be injectable and testable
- Controllers should be thin — delegate logic to services

### Angular Frontend

- Run `npm run lint` before committing
- Use Angular reactive forms for user input
- Follow the existing component structure in `frontend/src/app/shared/`

## Testing

### Smart Contracts

```bash
cd contracts
cargo test                    # All contracts
cargo test -p bond-issuer     # Single contract
cargo test -- --nocapture     # With output
```

### API

```bash
cd api
npm run test             # Unit tests
npm run test:e2e         # Integration tests (requires testnet)
npm run test:cov         # Coverage report
```

### Frontend

```bash
cd frontend
ng test                  # Unit tests
ng e2e                   # E2E tests
```

### Full Suite

```bash
cd contracts && cargo test && cd ..
cd api && npm run test && cd ..
cd frontend && ng test --watch=false --browsers=ChromeHeadless && cd ..
```

## Pull Request Process

1. Ensure all CI checks pass (tests, lint, build)
2. Update documentation if adding or changing functionality
3. Add tests for any new code
4. Link related issues in the PR description
5. Request review from the relevant team:
   - `@core-contracts` for Rust/Soroban changes
   - `@api-team` for NestJS changes
   - `@ui-team` for Angular changes

### PR Checklist

- [ ] `cargo test` passes
- [ ] `cargo clippy` is clean
- [ ] `npm run test` passes (API + Frontend)
- [ ] `npm run build` succeeds (API + Frontend)
- [ ] New code is tested
- [ ] New types / endpoints are documented
- [ ] PR description explains the change and motivation

## Issue Reporting

### Bug Reports

Use the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.md). Include:
- Clear description of the issue
- Steps to reproduce
- Expected vs actual behavior
- Environment details (Rust version, Soroban SDK, network)

### Feature Requests

Use the [Feature Request template](.github/ISSUE_TEMPLATE/feature_request.md). Include:
- What problem does this solve?
- What should be built?
- Alternatives considered

## Security Disclosures

**Do not open public GitHub issues for security vulnerabilities.** Instead, email **security@nbs-bond-protocol.org** with details. See [SECURITY.md](SECURITY.md) for our disclosure policy and bug bounty program.

## Getting Help

- Check existing [issues](https://github.com/your-org/nbs-bond-protocol/issues) and [discussions](https://github.com/your-org/nbs-bond-protocol/discussions)
- Review [docs/](./docs/) for architecture and design details
- Open a [discussion](https://github.com/your-org/nbs-bond-protocol/discussions) for questions

---

Thank you for helping make NbS Bond Protocol better!
