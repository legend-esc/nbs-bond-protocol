# Day 2 — Dev Environment, CI, Docker, Deploy Scripts

Load context: `prompts/context/tech-stack.md`

## Goal

Create the development environment: Docker Compose for local services, GitHub Actions CI, environment config, deployment scripts, and IPFS utilities.

## Files to Create

### `docker-compose.yml`

```yaml
services:
  api:
    build:
      context: ./api
      dockerfile: Dockerfile
    ports: ["3000:3000"]
    environment:
      STELLAR_NETWORK: testnet
      STELLAR_HORIZON_URL: https://horizon-testnet.stellar.org
      DATABASE_URL: postgresql://nbs:nbs@postgres:5432/nbs_bond
      REDIS_URL: redis://redis:6379
    depends_on: [postgres, redis]
    volumes: [./api:/app, /app/node_modules]

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: nbs
      POSTGRES_PASSWORD: nbs
      POSTGRES_DB: nbs_bond
    ports: ["5432:5432"]
    volumes: [pgdata:/var/lib/postgresql/data]

  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]

  ipfs:
    image: ipfs/kubo:latest
    ports: ["5001:5001", "8080:8080"]
    volumes: [ipfsdata:/data/ipfs]

volumes:
  pgdata:
  ipfsdata:
```

### `api/Dockerfile`

```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./
EXPOSE 3000
CMD ["node", "dist/main"]
```

### `.env.example`

Mirror the example from the project README — all sections:
- `STELLAR_NETWORK`, `STELLAR_HORIZON_URL`, `STELLAR_SECRET_KEY`, `STELLAR_PUBLIC_KEY`
- `CONTRACT_BOND_ISSUER` through `CONTRACT_CREDIT_RETIREMENT` — commented placeholders
- `IPFS_API_URL`, `IPFS_API_KEY`, `IPFS_SECRET_API_KEY`, `IPFS_GATEWAY` — placeholders
- `ORACLE_PROVIDER_URL`, `ORACLE_API_KEY`, `ORACLE_POLLING_INTERVAL_MS`
- `JWT_SECRET`, `JWT_EXPIRY`, `KYC_PROVIDER_URL`, `KYC_API_KEY`
- `DATABASE_URL`, `REDIS_URL`
- `PORT`, `NODE_ENV`, `LOG_LEVEL`

### `.github/workflows/ci.yml`

Trigger: push to `main`, pull requests to `main`.

Jobs:

**contracts**:
```yaml
- name: Checkout
  uses: actions/checkout@v4
- name: Install Rust
  uses: dtolnay/rust-toolchain@stable
  with: { toolchain: stable, components: clippy }
- name: Build
  run: cargo build --release
  working-directory: contracts
- name: Test
  run: cargo test
  working-directory: contracts
- name: Clippy
  run: cargo clippy --all-targets -- -D warnings
  working-directory: contracts
```

**api**:
```yaml
- name: Setup Node
  uses: actions/setup-node@v4
  with: { node-version: 20 }
- name: Install
  run: npm ci
  working-directory: api
- name: Lint
  run: npm run lint
  working-directory: api
- name: Test
  run: npm run test
  working-directory: api
- name: Build
  run: npm run build
  working-directory: api
```

**frontend**:
```yaml
- name: Setup Node
  uses: actions/setup-node@v4
  with: { node-version: 20 }
- name: Install
  run: npm ci
  working-directory: frontend
- name: Lint
  run: npm run lint
  working-directory: frontend
- name: Build
  run: npm run build
  working-directory: frontend
```

### `scripts/deploy-testnet.sh`

A bash script that:

1. Sources `.env` if available
2. For each contract in order (shared → project-registry → bond-issuer → coupon-engine → oracle-consumer → dex-router → credit-retirement):
   - Run `soroban contract build --package nbbs-{name}`
   - Run `soroban contract deploy --wasm target/wasm32-unknown-unknown/release/nbbs_{name}.wasm --network testnet`
   - Capture the deployed contract address (C...)
   - Initialize the contract: `soroban contract invoke --id {address} --fn __constructor --arg {admin_address} --network testnet`
   - Write address to `.env` as `CONTRACT_{NAME}=C...`
3. Output summary of all deployed addresses

Use `sed -i` to update the `.env` file with each address. Handle errors with `set -e`.

### `scripts/seed-testnet.ts`

TypeScript script using `@stellar/stellar-sdk` that:

1. Connects to testnet via Horizon
2. Registers 2 sample NbS projects via `ProjectRegistry`
3. Issues 1 bond tranche via `BondIssuer` backed by first project
4. Subscribes a test investor to the bond

Use placeholder addresses and hardcoded keys for now — the contracts don't exist yet but the script should be structurally correct.

### `scripts/migrate.ts`

A placeholder TypeScript script that logs "NbS Bond Protocol — migration runner" and exits. Will be filled with actual migration logic later.

### `scripts/deploy-mainnet.sh`

Same structure as `deploy-testnet.sh` but:
- Uses `--network mainnet` instead of testnet
- Sources a separate `.env.mainnet` file with production keys
- Prints a prominent warning before each action and requires `read -p "Continue? (y/N)"` confirmation
- Logs all output to `deploy-mainnet-$(date +%Y%m%d-%H%M%S).log`
- At the end, outputs: `⚠️  Verify contract addresses on Stellar Expert before using in production`

### `.github/workflows/deploy.yml`

```yaml
name: Deploy

on:
  workflow_dispatch:
    inputs:
      network:
        description: 'Target network'
        required: true
        default: 'testnet'
        type: choice
        options: [testnet, mainnet]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Soroban CLI
        run: cargo install --locked soroban-cli --version 26.0.1
      - name: Deploy contracts
        run: |
          if [ "${{ github.event.inputs.network }}" = "mainnet" ]; then
            ./scripts/deploy-mainnet.sh
          else
            ./scripts/deploy-testnet.sh
          fi
        env:
          STELLAR_SECRET_KEY: ${{ secrets.STELLAR_${{ github.event.inputs.network == 'mainnet' && 'MAINNET' || 'TESTNET' }}_SECRET }}
```

### `oracle/` — Standalone Oracle Adapter Scripts

Create three scripts in the root `oracle/` directory. These run independently from the NestJS API (e.g. as cron jobs or GitHub Actions) and feed data into the protocol.

#### `oracle/verra-adapter.ts`

```typescript
import axios from 'axios';

interface VerraProject {
  id: string;
  name: string;
  registry: string;
  methodology: string;
  status: string;
  credits_issued: number;
  credits_retired: number;
  last_update: string;
}

const VVC_REGISTRY_URL = 'https://registry.verra.org/api/v1/projects';

export async function fetchVerraProject(projectId: string): Promise<VerraProject> {
  const { data } = await axios.get(`${VVC_REGISTRY_URL}/${projectId}`);
  return {
    id: data.id,
    name: data.name,
    registry: 'VERRA-VCS',
    methodology: data.methodology,
    status: data.status,
    credits_issued: parseInt(data.creditsIssued, 10),
    credits_retired: parseInt(data.creditsRetired, 10),
    last_update: data.lastModifiedDate,
  };
}

export async function verifyVerraReport(projectId: string, periodStart: string, periodEnd: string): Promise<boolean> {
  // In production, fetch and validate monitoring reports from Verra registry
  return true;
}
```

#### `oracle/satellite-processor.ts`

```typescript
import axios from 'axios';

interface SatelliteImagery {
  sceneId: string;
  captureDate: string;
  cloudCover: number;
  ndvi: number;
  ndwi: number;
  source: 'sentinel-2' | 'landsat-8' | 'planet';
}

export async function fetchNdviData(bbox: string, startDate: string, endDate: string): Promise<SatelliteImagery[]> {
  // Query Sentinel Hub / Planet API for NDVI data within bounding box
  return [];
}

export function calculateBiomassChange(ndviBaseline: number, ndviCurrent: number): number {
  const fractionalCover = (ndviCurrent - ndviBaseline) / (1 - ndviBaseline);
  return Math.max(0, fractionalCover);
}

export function estimateCarbonSequestration(areaHa: number, ndviChange: number): number {
  // Simplified IPCC Tier 1: biomass carbon = area × NDVI-derived factor × conversion
  const defaultBiomassFactor = 3.67; // tC/ha per unit NDVI
  return areaHa * ndviChange * defaultBiomassFactor;
}
```

#### `oracle/iot-aggregator.ts`

```typescript
interface IoTSensorReading {
  deviceId: string;
  timestamp: string;
  metrics: {
    soilMoisture: number | null;
    temperature: number | null;
    humidity: number | null;
    co2: number | null;
  };
}

export function aggregateSensorReadings(readings: IoTSensorReading[]): {
  avgSoilMoisture: number;
  avgTemperature: number;
  avgHumidity: number;
  sampleCount: number;
  deviceCount: number;
} {
  const valid = readings.filter(r => r.metrics.soilMoisture !== null);
  return {
    avgSoilMoisture: valid.reduce((s, r) => s + r.metrics.soilMoisture!, 0) / valid.length,
    avgTemperature: valid.reduce((s, r) => s + r.metrics.temperature!, 0) / valid.length,
    avgHumidity: valid.reduce((s, r) => s + r.metrics.humidity!, 0) / valid.length,
    sampleCount: readings.length,
    deviceCount: new Set(readings.map(r => r.deviceId)).size,
  };
}

export function validateIotReading(reading: IoTSensorReading): boolean {
  const m = reading.metrics;
  if (m.soilMoisture !== null && (m.soilMoisture < 0 || m.soilMoisture > 100)) return false;
  if (m.temperature !== null && (m.temperature < -50 || m.temperature > 60)) return false;
  if (m.humidity !== null && (m.humidity < 0 || m.humidity > 100)) return false;
  return true;
}
```

### `ipfs/upload.ts`

```typescript
import axios from 'axios';

interface IpfsConfig {
  apiUrl: string;
  apiKey: string;
  secretKey: string;
  gateway: string;
}

interface IpfsUploadResult {
  hash: string;
  gatewayUrl: string;
  pinSize: number;
  timestamp: string;
}

export async function uploadToIpfs(
  data: Record<string, unknown>,
  config: IpfsConfig,
): Promise<IpfsUploadResult> {
  const response = await axios.post(
    `${config.apiUrl}/pinning/pinJSONToIPFS`,
    { pinataContent: data, pinataMetadata: { name: `nbs-${Date.now()}` } },
    {
      headers: {
        pinata_api_key: config.apiKey,
        pinata_secret_api_key: config.secretKey,
      },
    },
  );

  return {
    hash: response.data.IpfsHash,
    gatewayUrl: `${config.gateway}${response.data.IpfsHash}`,
    pinSize: response.data.PinSize,
    timestamp: new Date().toISOString(),
  };
}
```

### `ipfs/schemas/project-prospectus.schema.json`

JSON Schema for project documents:
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["projectName", "methodology", "location", "totalAreaHa", "carbonSequestrationEstimate"],
  "properties": {
    "projectName": { "type": "string", "minLength": 1 },
    "methodology": { "type": "string", "enum": ["VERRA-VCS", "GOLD-STANDARD", "ACR", "CAR"] },
    "location": {
      "type": "object",
      "required": ["country", "coordinates"],
      "properties": {
        "country": { "type": "string" },
        "coordinates": { "type": "object", "properties": { "lat": { "type": "number" }, "lng": { "type": "number" } } }
      }
    },
    "totalAreaHa": { "type": "number", "minimum": 1 },
    "carbonSequestrationEstimate": { "type": "number", "minimum": 0 },
    "biodiversityCorridor": { "type": "boolean" },
    "blueCarbon": { "type": "boolean" },
    "projectDeveloper": { "type": "string" },
    "description": { "type": "string" },
    "startDate": { "type": "string", "format": "date" },
    "documents": { "type": "array", "items": { "type": "string" } }
  }
}
```

### `ipfs/schemas/oracle-report.schema.json`

JSON Schema for oracle measurement reports:
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["projectId", "periodStart", "periodEnd", "carbonSequesteredKg", "methodology", "provider"],
  "properties": {
    "projectId": { "type": "string" },
    "periodStart": { "type": "string", "format": "date" },
    "periodEnd": { "type": "string", "format": "date" },
    "carbonSequesteredKg": { "type": "number", "minimum": 0 },
    "methodology": { "type": "string" },
    "provider": { "type": "string" },
    "satelliteImageHash": { "type": "string" },
    "auditorReportHash": { "type": "string" },
    "sensorDataHash": { "type": "string" },
    "confidence": { "type": "number", "minimum": 0, "maximum": 1 }
  }
}
```

## Verification

```bash
# Docker config valid
docker compose config --quiet && echo "OK"

# CI config valid (YAML)
node -e "console.log(require('js-yaml').load(require('fs').readFileSync('.github/workflows/ci.yml','utf8')))"

# Scripts are executable
chmod +x scripts/deploy-testnet.sh scripts/deploy-mainnet.sh
shellcheck scripts/deploy-testnet.sh 2>/dev/null || echo "shellcheck not installed, skipping"

# Oracle adapter scripts parse correctly
npx tsc --noEmit oracle/verra-adapter.ts oracle/satellite-processor.ts oracle/iot-aggregator.ts"

# IPFS module compiles
cd ipfs && npm init -y && npm install axios typescript @types/node && npx tsc --noEmit upload.ts
```

## Commit Message

```
chore: docker, CI pipelines, deploy scripts, and IPFS utilities
```
