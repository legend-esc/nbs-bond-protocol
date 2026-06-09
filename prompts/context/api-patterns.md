# NestJS API Patterns — Reference

## Module Structure

```
{bonds,projects,oracle,marketplace,auth}/
├── {entity}.controller.ts    # @Controller()
├── {entity}.service.ts       # @Injectable()
├── {entity}.module.ts        # @Module()
├── dto/
│   ├── create-{entity}.dto
│   └── update-{entity}.dto
└── interfaces/
    └── {entity}.interface.ts
```

## Controller Template

```typescript
@Controller('bonds')
export class BondsController {
  constructor(private readonly bondsService: BondsService) {}

  @Post()
  @UseGuards(JwtAuthGuard, AdminGuard)
  async create(@Body() dto: CreateBondDto): Promise<BondResponse> { ... }

  @Get()
  async findAll(@Query() query: PaginationDto): Promise<PaginatedResponse<BondResponse>> { ... }

  @Get(':id')
  async findOne(@Param('id', ParseUUIDPipe) id: string): Promise<BondResponse> { ... }
}
```

## Service Template

```typescript
@Injectable()
export class BondsService {
  constructor(
    private readonly contractService: ContractService,
    private readonly stellarService: StellarService,
    @InjectRedis() private readonly redis: Redis,
  ) {}

  async create(dto: CreateBondDto): Promise<BondResponse> { ... }
}
```

## DTO Conventions

- Use `class-validator` decorators for validation
- Use `@ApiProperty()` from `@nestjs/swagger` for docs
- Always validate Stellar addresses with a custom `@IsStellarAddress()` decorator

## Error Handling

Use the global exception filter pattern — throw `HttpException` with RFC 7807 body:

```typescript
{
  type: 'https://errors.nbs-bond-protocol.org/invalid-nonce',
  title: 'Invalid Nonce',
  status: 409,
  detail: 'Nonce 5 provided but expected 4',
  instance: '/bonds/subscribe'
}
```

## Dependency Configuration

```json
{
  "@nestjs/common": "^10.4.0",
  "@nestjs/core": "^10.4.0",
  "@nestjs/passport": "^10.0.3",
  "@nestjs/jwt": "^10.2.0",
  "@nestjs/swagger": "^7.3.0",
  "@nestjs/bull": "^10.1.0",
  "@nestjs/schedule": "^4.0.0",
  "class-validator": "^0.14.1",
  "class-transformer": "^0.5.1",
  "passport": "^0.7.0",
  "passport-jwt": "^4.0.1",
  "@stellar/stellar-sdk": "^12.0.0",
  "@redis/client": "^1.5.0",
  "zod": "^3.22.0"
}
```
