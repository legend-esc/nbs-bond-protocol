# Day 10 — Bonds & Projects API Modules

Load context: `prompts/context/tech-stack.md`, `prompts/context/api-patterns.md`

## Goal

Implement the Bonds and Projects CRUD API endpoints that orchestrate the `BondIssuer` and `ProjectRegistry` smart contracts.

## Files to Create

### Bonds Module

#### `api/src/bonds/bonds.module.ts`
```typescript
@Module({
  controllers: [BondsController],
  providers: [BondsService],
})
export class BondsModule {}
```

#### `api/src/bonds/bonds.controller.ts`

**Endpoints:**

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/bonds` | Admin | Issue a new bond tranche |
| `GET` | `/bonds` | Public | List active bond tranches (paginated) |
| `GET` | `/bonds/:id` | Public | Get bond detail |
| `POST` | `/bonds/:id/subscribe` | Investor (KYC) | Subscribe to a bond |
| `GET` | `/bonds/:id/holders` | Public | List holders for a bond |
| `POST` | `/bonds/:id/coupon` | Admin | Trigger coupon distribution |
| `POST` | `/bonds/:id/mature` | Admin | Mark bond as matured |

**Decorators:**
```typescript
@Controller('bonds')
export class BondsController {
  constructor(private readonly bondsService: BondsService) {}

  @Post()
  @UseGuards(JwtAuthGuard, AdminGuard)
  @HttpCode(HttpStatus.CREATED)
  async create(@Body() dto: CreateBondDto): Promise<BondResponse>

  @Get()
  async findAll(@Query() query: PaginationDto): Promise<PaginatedResponse<BondResponse>>

  @Get(':id')
  async findOne(@Param('id', ParseIntPipe) id: number): Promise<BondResponse>

  @Post(':id/subscribe')
  @UseGuards(JwtAuthGuard, KycGuard)
  @HttpCode(HttpStatus.OK)
  async subscribe(
    @Param('id', ParseIntPipe) id: number,
    @Body() dto: SubscribeDto,
    @Req() req: AuthenticatedRequest,
  ): Promise<SubscriptionResponse>

  @Get(':id/holders')
  async getHolders(@Param('id', ParseIntPipe) id: number): Promise<HolderListResponse>

  @Post(':id/coupon')
  @UseGuards(JwtAuthGuard, AdminGuard)
  @HttpCode(HttpStatus.OK)
  async distributeCoupon(
    @Param('id', ParseIntPipe) id: number,
    @Body() dto: DistributeCouponDto,
  ): Promise<CouponDistributionResponse>

  @Post(':id/mature')
  @UseGuards(JwtAuthGuard, AdminGuard)
  @HttpCode(HttpStatus.OK)
  async mature(@Param('id', ParseIntPipe) id: number): Promise<BondResponse>
}
```

#### `api/src/bonds/bonds.service.ts`

```typescript
@Injectable()
export class BondsService {
  constructor(
    private readonly contractService: ContractService,
    @InjectRedis() private readonly redis: Redis,
  ) {}

  async create(dto: CreateBondDto): Promise<BondResponse>
  async findAll(page: number, limit: number): Promise<PaginatedResponse<BondResponse>>
  async findOne(id: number): Promise<BondResponse>
  async subscribe(id: number, investorAddress: string, amount: number): Promise<SubscriptionResponse>
  async getHolders(id: number): Promise<HolderListResponse>
  async distributeCoupon(id: number, report: OracleReportDto): Promise<CouponDistributionResponse>
  async mature(id: number): Promise<BondResponse>
}
```

**Key logic:**
- `create`: Build `BondConfig` from DTO, call `BondIssuer.issue_bond()` via `ContractService`, cache result in Redis with 5min TTL, return response
- `findAll`: Check Redis cache first, fall back to listing from contract, paginate in-memory
- `findOne`: Cache-first, then call `BondIssuer.get_bond()` + `get_bond_state()`
- `subscribe`: Call `BondIssuer.subscribe()` with caller's nonce, update Redis cache
- `getHolders`: Iterate through known addresses via Redis set, call `get_holder_balance` for each
- `distributeCoupon`: Call `CouponEngine.distribute_coupon()` with OracleReport
- `mature`: Call `BondIssuer.mature_bond()`, update Redis

#### `api/src/bonds/dto/`

**`create-bond.dto.ts`**:
```typescript
export class CreateBondDto {
  @IsString() @IsNotEmpty()
  projectId: string;              // hex-encoded BytesN<32>

  @IsNumber() @IsPositive()
  faceValue: number;

  @IsArray() @IsNumber({}, { each: true })
  couponSchedule: number[];       // unix timestamps

  @IsEnum(CreditTypeEnum)
  creditType: CreditTypeEnum;

  @IsNumber() @IsPositive()
  maturityDate: number;

  @IsNumber() @IsPositive()
  totalSupply: number;
}
```

**`subscribe.dto.ts`**:
```typescript
export class SubscribeDto {
  @IsNumber() @IsPositive()
  amount: number;

  @IsNumber()
  nonce: number;

  @IsString() @IsStellarAddress()
  investorAddress: string;
}
```

**`distribute-coupon.dto.ts`**:
```typescript
export class DistributeCouponDto {
  @IsNumber() @IsPositive()
  periodIndex: number;

  @ValidateNested()
  @Type(() => OracleReportDto)
  report: OracleReportDto;
}
```

#### `api/src/bonds/interfaces/bond.interface.ts`

```typescript
interface BondResponse {
  id: number;
  projectId: string;
  faceValue: number;
  couponSchedule: number[];
  creditType: CreditTypeEnum;
  maturityDate: number;
  totalSupply: number;
  totalSubscribed: number;
  status: BondStatusEnum;
  createdAt: string;
}

interface SubscriptionResponse {
  bondId: number;
  investorAddress: string;
  amount: number;
  transactionHash: string;
}

interface HolderListResponse {
  bondId: number;
  holders: Array<{ address: string; balance: number }>;
  total: number;
}

interface CouponDistributionResponse {
  bondId: number;
  periodIndex: number;
  totalCredits: number;
  holderCount: number;
}
```

### Projects Module

#### `api/src/projects/projects.module.ts`
```typescript
@Module({
  controllers: [ProjectsController],
  providers: [ProjectsService, IpfsService],
})
export class ProjectsModule {}
```

#### `api/src/projects/projects.controller.ts`

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/projects` | Any (signed) | Register a new project |
| `GET` | `/projects` | Public | List all projects |
| `GET` | `/projects/:id` | Public | Get project detail |
| `POST` | `/projects/:id/approve` | Admin | Approve a project |
| `POST` | `/projects/:id/reject` | Admin | Reject a project |
| `POST` | `/projects/:id/documents` | Owner | Upload docs to IPFS |

#### `api/src/projects/projects.service.ts`

```typescript
@Injectable()
export class ProjectsService {
  constructor(
    private readonly contractService: ContractService,
    private readonly ipfsService: IpfsService,
    @InjectRedis() private readonly redis: Redis,
  ) {}

  async register(dto: CreateProjectDto, ownerAddress: string): Promise<ProjectResponse>
  async findAll(page: number, limit: number): Promise<PaginatedResponse<ProjectResponse>>
  async findOne(id: number): Promise<ProjectResponse>
  async approve(id: number): Promise<ProjectResponse>
  async reject(id: number): Promise<ProjectResponse>
  async uploadDocuments(id: number, files: Express.Multer.File[]): Promise<DocumentUploadResponse>
}
```

**Key logic:**
- `register`: Upload project metadata to IPFS, get hash → call `ProjectRegistry.register_project()` with the hash
- `approve`/`reject`: Call admin functions on ProjectRegistry contract
- `uploadDocuments`: Pin files to IPFS via `IpfsService`, store document hashes in Redis (key: `project:{id}:documents`)

#### `api/src/projects/ipfs.service.ts`

```typescript
@Injectable()
export class IpfsService {
  constructor() {
    // Configure from env: IPFS_API_URL, IPFS_API_KEY, etc.
  }

  async uploadJson(data: Record<string, unknown>): Promise<IpfsUploadResult>
  async uploadFile(buffer: Buffer, filename: string): Promise<IpfsUploadResult>
  async getContent(hash: string): Promise<Record<string, unknown>>
  pin(hash: string): Promise<void>
}
```

Wraps the lower-level `ipfs/upload.ts` functions.

#### `api/src/projects/dto/`

**`create-project.dto.ts`**:
```typescript
export class CreateProjectDto {
  @IsString() @IsNotEmpty()
  name: string;

  @IsString() @IsNotEmpty()
  methodology: string;

  @IsString() @IsNotEmpty()
  country: string;

  @IsObject()
  @ValidateNested()
  @Type(() => LocationDto)
  location: LocationDto;

  @IsNumber() @IsPositive()
  totalAreaHa: number;

  @IsNumber() @IsPositive()
  carbonSequestrationEstimate: number;

  @IsOptional() @IsBoolean()
  blueCarbon?: boolean;

  @IsOptional() @IsBoolean()
  biodiversityCorridor?: boolean;

  @IsOptional() @IsString()
  description?: string;

  @IsNumber()
  nonce: number;
}

class LocationDto {
  @IsNumber()
  lat: number;

  @IsNumber()
  lng: number;
}
```

#### `api/src/projects/interfaces/project.interface.ts`

```typescript
interface ProjectResponse {
  id: number;
  name: string;
  status: ProjectStatusEnum;
  methodology: string;
  country: string;
  metadataIpfsHash: string;
  ownerAddress: string;
  totalAreaHa: number;
  carbonSequestrationEstimate: number;
  createdAt: string;
}

interface DocumentUploadResponse {
  projectId: number;
  documentHashes: string[];
  gatewayUrls: string[];
}
```

### Shared Utilities

#### `api/src/common/decorators/is-stellar-address.decorator.ts`

```typescript
import { registerDecorator, ValidationOptions } from 'class-validator';

export function IsStellarAddress(validationOptions?: ValidationOptions) {
  return function (object: object, propertyName: string) {
    registerDecorator({
      name: 'isStellarAddress',
      target: object.constructor,
      propertyName,
      options: validationOptions,
      validator: {
        validate(value: string) {
          return typeof value === 'string' && /^G[A-Z0-9]{55}$/.test(value);
        },
        defaultMessage: () => 'Invalid Stellar address (must start with G and be 56 chars)',
      },
    });
  };
}
```

#### `api/src/common/dto/pagination.dto.ts`

```typescript
export class PaginationDto {
  @IsOptional() @IsNumber() @Min(1) @Type(() => Number)
  page?: number = 1;

  @IsOptional() @IsNumber() @Min(1) @Max(100) @Type(() => Number)
  limit?: number = 20;
}

export class PaginatedResponse<T> {
  data: T[];
  meta: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  };
}
```

## Verification

```bash
cd api && npm run test -- --testPathPattern='(bonds|projects)'
```

Expected: 15+ tests — full CRUD for both modules, validation, auth guard behavior (mocked).

## Commit Message

```
feat(api): Bonds and Projects CRUD modules with contract orchestration
```
