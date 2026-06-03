# 🌍 NbS Bond Protocol
### *Tokenized Nature-based Solution Bonds on Stellar*

> **Redefining green finance** — where bond interest is paid in carbon and biodiversity credits, every tranche backs a living ecosystem, and DeFi unlocks liquidity for the planet.

---

## 📌 Table of Contents

- [Overview](#-overview)
- [Why This Exists](#-why-this-exists)
- [How It Works](#-how-it-works)
- [Key Features](#-key-features)
- [Architecture](#-architecture)
- [Tech Stack](#-tech-stack)
- [Smart Contract Design](#-smart-contract-design)
- [Oracle & Project Performance](#-oracle--project-performance)
- [Secondary Market](#-secondary-market)
- [Relationship to CarbonChain](#-relationship-to-carbonchain)
- [Getting Started](#-getting-started)
- [Project Structure](#-project-structure)
- [Roadmap](#-roadmap)
- [Contributing](#-contributing)
- [License](#-license)

---

## 🌿 Overview

**NbS Bond Protocol** is a blockchain-native green finance instrument built on the [Stellar](https://stellar.org) network. It issues bonds where **coupon payments are denominated in carbon credits and biodiversity credits** rather than fiat currency — directly linking investor returns to the ecological performance of real-world nature-based projects.

Each bond tranche is backed by a specific, verifiable reforestation, blue carbon (mangrove, seagrass, wetland), or biodiversity restoration project. Smart contracts automate the entire bond lifecycle, while on-chain oracles provide tamper-resistant proof of carbon stock growth.

> This is not a carbon offset product. It is a **financial primitive** — a programmable, tradeable, yield-bearing instrument whose yield is the planet healing.

---

## 🔍 Why This Exists

The global carbon market is fragmented, opaque, and largely inaccessible to retail and institutional DeFi participants. At the same time, billions in green bonds are issued annually with little traceability or performance accountability.

**NbS Bond Protocol solves three fundamental problems:**

| Problem | Our Solution |
|---|---|
| Carbon credit integrity is hard to verify | On-chain oracle feeds real project data |
| Green bonds lack coupon-to-impact traceability | Each tranche is 1:1 backed by a specific project |
| Carbon credits are illiquid for bondholders | Stellar DEX enables secondary trading of bond tokens and credit coupons |

---

## ⚙️ How It Works

```
  ┌─────────────────────────────────────────────────────────────────┐
  │                      NbS BOND LIFECYCLE                         │
  │                                                                 │
  │  1. ISSUANCE         2. BACKING           3. PERFORMANCE        │
  │  Bond tranche   →    Linked to a      →   Oracle monitors       │
  │  tokenized on        reforestation /       carbon stock         │
  │  Stellar             blue carbon           growth on-chain      │
  │                      project                                    │
  │                                                                 │
  │  4. COUPON           5. DISTRIBUTION       6. SECONDARY MKT     │
  │  Carbon/biodiv  →    Smart contract    →   Trade bond tokens    │
  │  credits minted      distributes to        & credit coupons     │
  │  per period          bondholders           on Stellar DEX       │
  └─────────────────────────────────────────────────────────────────┘
```

### Step-by-Step

1. **A project developer** registers a nature-based project (reforestation, mangrove restoration, etc.) with verified geospatial and ecological baseline data stored on IPFS.
2. **A bond issuer** creates a tokenized bond tranche, specifying face value, maturity, and coupon schedule — denominated in carbon/biodiversity credits.
3. **Investors** purchase bond tokens. Each token represents a fractional claim on the bond tranche.
4. **Oracles** continuously feed verified carbon stock growth data on-chain. At each coupon date, the smart contract calculates earned credits.
5. **Coupons** are distributed automatically to bondholder wallets in the form of carbon or biodiversity credit tokens.
6. **At maturity**, the principal is returned and remaining credits are settled. Investors may trade their bond tokens or coupon credits at any time on the Stellar DEX.

---

## ✨ Key Features

### 🔗 Smart Contract Bond Lifecycle
The entire bond lifecycle — issuance, coupon accrual, distribution, maturity, and redemption — is handled by Soroban smart contracts written in Rust. No intermediary, no manual settlement, no counterparty risk.

### 🌐 Oracle-Driven Performance Verification
An oracle network feeds real-world project data (carbon stock measurements, satellite imagery hashes, third-party audit reports) on-chain. Credit coupon amounts are dynamically calculated based on **actual ecological performance**, not projections.

### 💱 Secondary Market Liquidity
Bond tokens and individual credit coupons are fully tradeable on the **Stellar Decentralized Exchange (DEX)**. Investors can exit positions, speculate on project performance, or accumulate credits from multiple projects — all without leaving the protocol.

### 🌱 Fractional Green Exposure
Tokenization allows **fractional ownership** of bond tranches, opening nature-based finance to retail investors who couldn't access traditional green bond minimums (often $100,000+).

### 📄 IPFS-Backed Project Documentation
All project prospectuses, audit reports, satellite data, and legal documentation are stored on **IPFS** with content hashes anchored on-chain — ensuring permanent, tamper-proof records.

### 🔐 Nonce-Based Replay Protection
All sensitive contract interactions are protected by nonce-based replay protection, preventing double-spend and replay attacks across the protocol.

---

## 🏗️ Architecture

```
┌──────────────────────────────────────────────────────────┐
│                     FRONTEND (Angular)                   │
│         Investor Dashboard │ Project Registry │ DEX UI   │
└─────────────────────────┬────────────────────────────────┘
                          │ HTTP / WebSocket
┌─────────────────────────▼────────────────────────────────┐
│                   API LAYER (NestJS)                     │
│    Bond Management │ Oracle Integration │ Auth / KYC     │
└──────┬───────────────────────────────────────┬───────────┘
       │ Stellar SDK                            │ IPFS API
┌──────▼──────────────────┐        ┌───────────▼───────────┐
│   STELLAR BLOCKCHAIN    │        │      IPFS STORAGE      │
│                         │        │                        │
│  ┌───────────────────┐  │        │  Project Documents     │
│  │  Soroban Contracts│  │        │  Audit Reports         │
│  │  - BondIssuer     │  │        │  Satellite Data        │
│  │  - CouponEngine   │  │        │  Legal Prospectus      │
│  │  - OracleConsumer │  │        └────────────────────────┘
│  │  - DEX Router     │  │
│  └───────────────────┘  │        ┌────────────────────────┐
│                         │        │    ORACLE NETWORK      │
│  Stellar DEX            │◄───────│  Carbon Stock Feeds    │
│  (Secondary Market)     │        │  Biodiversity Indices  │
└─────────────────────────┘        │  Satellite Hashes      │
                                   └────────────────────────┘
```

---

## 🧰 Tech Stack

| Layer | Technology | Purpose |
|---|---|---|
| **Smart Contracts** | Soroban (Rust) | Bond lifecycle, credit distribution, replay protection |
| **Backend API** | NestJS (TypeScript) | Business logic, oracle integration, KYC/auth |
| **Frontend** | Angular | Investor dashboard, project registry, DEX interface |
| **Off-chain Storage** | IPFS | Project documents, audit trails, satellite data |
| **Marketplace** | Stellar DEX | Secondary trading of bond tokens & credit coupons |
| **Security** | Nonce-based replay protection | Prevent double-spend and replay attacks |
| **Blockchain** | Stellar Network | Settlement, tokenization, decentralized exchange |

---

## 📜 Smart Contract Design

The protocol is composed of four primary Soroban contracts:

### `BondIssuer`
Handles bond creation, tranche configuration, and investor token minting.

```rust
// Example: Issuing a bond tranche
pub fn issue_bond(
    env: Env,
    issuer: Address,
    project_id: BytesN<32>,  // IPFS hash of project docs
    face_value: i128,
    coupon_schedule: Vec<u64>,
    credit_type: CreditType,  // Carbon | Biodiversity
) -> BondId { ... }
```

### `CouponEngine`
Calculates and distributes credit coupons to bondholders based on oracle-reported performance data.

```rust
pub fn distribute_coupon(
    env: Env,
    bond_id: BondId,
    period: u32,
    oracle_report: OracleReport,  // Verified carbon stock measurement
) -> CouponResult { ... }
```

### `OracleConsumer`
Validates and ingests signed oracle reports, anchoring real-world ecological data on-chain.

### `DEXRouter`
Manages bond token and coupon credit listings, order routing, and settlement on the Stellar DEX.

---

## 🛰️ Oracle & Project Performance

The integrity of this protocol depends on the quality of its ecological data. NbS Bond Protocol uses a multi-source oracle architecture:

- **Primary**: Verified carbon stock measurements from accredited third-party auditors (e.g., Verra, Gold Standard)
- **Secondary**: Satellite imagery processed off-chain (NDVI, biomass proxies), with result hashes submitted on-chain
- **Tertiary**: IoT sensor data from project sites (where available)

Oracle reports are signed by approved data providers and validated by the `OracleConsumer` contract before influencing any coupon calculations. Disputed reports trigger a **challenge window** during which stakeholders may submit counter-evidence.

---

## 🔄 Secondary Market

Bond tokens and credit coupon tokens are standard **Stellar Assets**, making them natively compatible with the Stellar DEX. Investors can:

- **Sell** bond tokens before maturity at market price
- **Trade** credit coupon tokens for XLM, USDC, or other assets
- **Speculate** on project performance by accumulating credits from outperforming projects
- **Provide liquidity** to bond token / USDC pairs

The Stellar DEX's built-in path payment and CLMM (concentrated liquidity market maker) features ensure efficient price discovery and deep liquidity without a centralized order book.

---

## 🔗 Relationship to CarbonChain

NbS Bond Protocol is a **financial primitive layered on top of CarbonChain's credit infrastructure**. It reuses CarbonChain's:

- Credit issuance and registry contracts
- Oracle infrastructure for carbon stock verification
- IPFS document anchoring pattern
- Stellar DEX integration modules
- NestJS API patterns and authentication middleware

**The key distinction**: CarbonChain manages raw carbon credits. NbS Bond Protocol wraps those credits into a structured **bond instrument** — giving investors fixed-income mechanics (face value, coupon schedule, maturity) while preserving the ecological integrity of the underlying credits.

Think of it as: `CarbonChain` = credit infrastructure. `NbS Bond Protocol` = DeFi bank built on top of it.

---

## 🚀 Getting Started

### Prerequisites

- Node.js `v18+`
- Rust + `soroban-cli`
- Stellar testnet account (funded via [Friendbot](https://friendbot.stellar.org))
- IPFS node (or Pinata API key)

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/nbs-bond-protocol.git
cd nbs-bond-protocol

# Install API dependencies
cd api && npm install

# Install frontend dependencies
cd ../frontend && npm install

# Build Soroban contracts
cd ../contracts && cargo build --release
```

### Configuration

```bash
cp api/.env.example api/.env
# Fill in:
# STELLAR_NETWORK=testnet
# STELLAR_SECRET_KEY=S...
# IPFS_API_URL=https://api.pinata.cloud
# IPFS_API_KEY=your_key
# ORACLE_PROVIDER_URL=https://...
```

### Deploy Contracts

```bash
cd contracts

# Deploy to Stellar testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/bond_issuer.wasm \
  --source your-account \
  --network testnet

# Repeat for CouponEngine, OracleConsumer, DEXRouter
```

### Run Locally

```bash
# Start API
cd api && npm run start:dev

# Start frontend
cd frontend && ng serve

# Open http://localhost:4200
```

---

## 📁 Project Structure

```
nbs-bond-protocol/
├── contracts/                  # Soroban smart contracts (Rust)
│   ├── bond-issuer/
│   ├── coupon-engine/
│   ├── oracle-consumer/
│   └── dex-router/
├── api/                        # NestJS backend
│   ├── src/
│   │   ├── bonds/             # Bond issuance & management
│   │   ├── oracle/            # Oracle feed integration
│   │   ├── projects/          # NbS project registry
│   │   ├── marketplace/       # DEX routing & liquidity
│   │   └── auth/              # KYC & authentication
│   └── test/
├── frontend/                   # Angular application
│   ├── src/
│   │   ├── app/
│   │   │   ├── dashboard/     # Investor portfolio view
│   │   │   ├── projects/      # Project registry & detail
│   │   │   ├── marketplace/   # DEX trading interface
│   │   │   └── bonds/         # Bond issuance (issuer role)
│   └── e2e/
├── ipfs/                       # IPFS upload scripts & schemas
├── oracle/                     # Oracle adapter scripts
├── docs/                       # Protocol documentation
└── scripts/                    # Deployment & migration scripts
```

---

## 🗺️ Roadmap

| Phase | Milestone | Status |
|---|---|---|
| **Phase 1** | Core smart contracts (BondIssuer, CouponEngine) | 🔨 In Progress |
| **Phase 2** | Oracle integration & testnet deployment | 📋 Planned |
| **Phase 3** | Angular frontend MVP | 📋 Planned |
| **Phase 4** | Stellar DEX secondary market integration | 📋 Planned |
| **Phase 5** | Mainnet launch with pilot project | 📋 Planned |
| **Phase 6** | Biodiversity credit coupon support | 🔭 Future |
| **Phase 7** | Multi-chain bridge (for credit portability) | 🔭 Future |

---

## 🤝 Contributing

We welcome contributions from smart contract engineers, climate scientists, financial modelers, and frontend developers.

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/oracle-adapter`
3. Commit with conventional commits: `git commit -m "feat(oracle): add Verra feed adapter"`
4. Push and open a Pull Request

Please read [CONTRIBUTING.md](./CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) before submitting.

---

## 📄 License

This project is licensed under the **MIT License** — see [LICENSE](./LICENSE) for details.

---

<div align="center">

**Built for the planet. Powered by Stellar.**

*NbS Bond Protocol — where financial returns and ecological restoration are the same thing.*

</div>
