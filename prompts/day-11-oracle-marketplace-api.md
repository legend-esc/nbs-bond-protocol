# Day 11 — Oracle & Marketplace API Modules

Load context: `prompts/context/tech-stack.md`, `prompts/context/api-patterns.md`

## Goal

Implement the Oracle feeds and Marketplace DEX integration API modules.

## Files to Create

### Oracle Module

#### `api/src/oracle/oracle.module.ts`

```typescript
@Module({
  controllers: [OracleController],
  providers: [OracleService, VerraProvider, SatelliteProvider],
})
export class OracleModule {}
```

#### `api/src/oracle/oracle.controller.ts`

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/oracle/reports` | Provider | Submit a measurement report |
| `GET` | `/oracle/reports/:projectId` | Public | Get oracle history for a project |
| `POST` | `/oracle/challenge/:reportId` | Any | Challenge a submitted report |
| `POST` | `/oracle/providers` | Admin | Register a new oracle provider |
| `GET` | `/oracle/providers` | Public | List registered providers |

#### `api/src/oracle/oracle.service.ts`

```typescript
@Injectable()
export class OracleService {
  constructor(
    private readonly contractService: ContractService,
    private readonly ipfsService: IpfsService,
  ) {}

  async submitReport(dto: SubmitReportDto, providerAddress: string): Promise<ReportResponse>
  async getProjectReports(projectId: string): Promise<ReportResponse[]>
  async challengeReport(reportId: number, dto: ChallengeDto, challengerAddress: string): Promise<ChallengeResponse>
  async registerProvider(dto: RegisterProviderDto): Promise<ProviderResponse>
  async listProviders(): Promise<ProviderResponse[]>
}
```

**Key logic:**
- `submitReport`: Upload report data to IPFS → call `OracleConsumer.submit_report()` with the IPFS hash, return report ID
- `challengeReport`: Call `OracleConsumer.challenge_report()` with counter-evidence hash
- `registerProvider`: Admin calls `OracleConsumer.register_provider()`

#### `api/src/oracle/oracle.scheduler.ts`

```typescript
@Injectable()
export class OracleScheduler {
  constructor(
    private readonly oracleService: OracleService,
    private readonly verraProvider: VerraProvider,
    private readonly satelliteProvider: SatelliteProvider,
  ) {}

  @Cron('0 */5 * * * *')  // Every 5 minutes
  async pollOracleData(): Promise<void> {
    // 1. Fetch pending oracle data from providers
    // 2. If new data available, auto-submit report
    // 3. Log results
  }
}
```

The scheduler is a stub that logs "Oracle poll cycle" — real provider integration comes later.

#### `api/src/oracle/providers/provider.interface.ts`

```typescript
export interface OracleProviderAdapter {
  readonly name: string;
  readonly methodology: string;

  /// Fetch latest measurement data for a project
  fetchMeasurement(projectId: string): Promise<MeasurementData>;
}

export interface MeasurementData {
  projectId: string;
  periodStart: Date;
  periodEnd: Date;
  carbonSequesteredKg: number;
  confidence: number;
  evidenceHashes: string[];
}
```

#### `api/src/oracle/providers/verra.provider.ts`

```typescript
@Injectable()
export class VerraProvider implements OracleProviderAdapter {
  readonly name = 'Verra';
  readonly methodology = 'VERRA-VCS';

  async fetchMeasurement(projectId: string): Promise<MeasurementData> {
    // Stub: return mock data
    // Real implementation would call Verra API
    return {
      projectId,
      periodStart: new Date('2025-01-01'),
      periodEnd: new Date('2025-03-31'),
      carbonSequesteredKg: 50000,
      confidence: 0.95,
      evidenceHashes: ['QmMockHash123'],
    };
  }
}
```

#### `api/src/oracle/providers/satellite.provider.ts`

Same pattern as VerraProvider with:
```typescript
readonly name = 'SatelliteProcessor';
readonly methodology = 'REMOTE-SENSING';
```

Returns mock NDVI-based biomass estimation data.

#### `api/src/oracle/dto/`

**`submit-report.dto.ts`**:
```typescript
export class SubmitReportDto {
  @IsString() @IsNotEmpty()
  projectId: string;

  @IsNumber() @IsPositive()
  periodStart: number;

  @IsNumber() @IsPositive()
  periodEnd: number;

  @IsNumber() @Min(0)
  carbonSequestered: number;

  @IsString() @IsNotEmpty()
  methodology: string;

  @IsString() @IsOptional()
  evidenceHash?: string;

  @IsNumber()
  nonce: number;
}
```

**`challenge.dto.ts`**:
```typescript
export class ChallengeDto {
  @IsString() @IsNotEmpty()
  counterEvidenceHash: string;

  @IsString() @IsNotEmpty()
  reason: string;

  @IsNumber()
  nonce: number;
}
```

**`register-provider.dto.ts`**:
```typescript
export class RegisterProviderDto {
  @IsString() @IsStellarAddress()
  providerAddress: string;

  @IsString() @IsNotEmpty()
  methodology: string;
}
```

### Marketplace Module

#### `api/src/marketplace/marketplace.module.ts`

```typescript
@Module({
  controllers: [MarketplaceController],
  providers: [DexService, LiquidityService],
})
export class MarketplaceModule {}
```

#### `api/src/marketplace/marketplace.controller.ts`

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/marketplace/orders` | Public | List open DEX orders |
| `POST` | `/marketplace/list` | Any (signed) | List bond tokens for sale |
| `POST` | `/marketplace/buy` | Any (signed) | Purchase bond tokens |
| `DELETE` | `/marketplace/orders/:id` | Seller | Cancel an order |
| `GET` | `/marketplace/orders/:id` | Public | Get order details |
| `GET` | `/marketplace/prices` | Public | Current price feed |

#### `api/src/marketplace/dex.service.ts`

```typescript
@Injectable()
export class DexService {
  constructor(
    private readonly contractService: ContractService,
    @InjectRedis() private readonly redis: Redis,
  ) {}

  async listOrders(bondId?: number, status?: string, page?: number, limit?: number): Promise<PaginatedResponse<OrderResponse>>
  async listBondTokens(dto: ListBondDto, sellerAddress: string): Promise<OrderResponse>
  async buyBondTokens(dto: BuyBondDto, buyerAddress: string): Promise<OrderResponse>
  async cancelOrder(orderId: number, callerAddress: string): Promise<void>
  async getOrder(orderId: number): Promise<OrderResponse>
}
```

**Key logic:**
- `listBondTokens`: Call `DEXRouter.list_bond_tokens()` → cache in Redis
- `buyBondTokens`: Call `DEXRouter.execute_purchase()` with buyer's address and max price
- `cancelOrder`: Call `DEXRouter.cancel_listing()`, only seller can cancel
- `listOrders`: Cache-first, filter by bond_id and status, support pagination

#### `api/src/marketplace/liquidity.service.ts`

```typescript
@Injectable()
export class LiquidityService {
  constructor(
    private readonly dexService: DexService,
    @InjectRedis() private readonly redis: Redis,
  ) {}

  async getPriceFeed(bondId?: number): Promise<PriceFeedResponse>
  async getBestPrice(bondId: number, side: 'buy' | 'sell'): Promise<PriceLevel>
  async calculateSlippage(bondId: number, amount: number): Promise<SlippageResponse>
}
```

**Key logic:**
- `getPriceFeed`: For each bond, calculate weighted average price from open orders, cache with 30s TTL
- `getBestPrice`: Find the lowest sell price or highest buy price (for now, only sell orders exist)
- `calculateSlippage`: Given an amount, walk the order book to compute average execution price

#### `api/src/marketplace/dto/`

**`list-bond.dto.ts`**:
```typescript
export class ListBondDto {
  @IsNumber() @IsPositive()
  bondId: number;

  @IsNumber() @IsPositive()
  amount: number;

  @IsNumber() @IsPositive()
  pricePerToken: number;  // in USDC stroops

  @IsString()
  quoteAsset: 'USDC' | 'XLM';

  @IsNumber() @IsOptional()
  expiresAfterSeconds?: number = 604800;  // default 7 days

  @IsNumber()
  nonce: number;
}
```

**`buy-bond.dto.ts`**:
```typescript
export class BuyBondDto {
  @IsNumber() @IsPositive()
  orderId: number;

  @IsNumber() @IsPositive()
  amount: number;

  @IsNumber() @IsPositive()
  maxPrice: number;

  @IsNumber()
  nonce: number;
}
```

## Verification

```bash
cd api && npm run test -- --testPathPattern='(oracle|marketplace)'
```

Expected: 15+ tests — report submission, provider registry, challenge flow, order listing, buy/sell, price aggregation.

## Commit Message

```
feat(api): Oracle feeds with mock providers and Marketplace DEX integration
```
