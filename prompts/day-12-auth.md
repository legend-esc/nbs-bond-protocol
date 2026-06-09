# Day 12 — Auth Module, Guards, Global Error Handling

Load context: `prompts/context/tech-stack.md`, `prompts/context/api-patterns.md`

## Goal

Implement wallet-based authentication, JWT strategy, KYC integration stub, authorization guards, and global error handling middleware.

## Files to Create

### Auth Module

#### `api/src/auth/auth.module.ts`

```typescript
@Module({
  imports: [JwtModule.register({
    secret: process.env.JWT_SECRET || 'dev-secret-change-in-production',
    signOptions: { expiresIn: process.env.JWT_EXPIRY || '7d' },
  })],
  controllers: [AuthController],
  providers: [AuthService, JwtStrategy, KycService],
  exports: [AuthService],
})
export class AuthModule {}
```

#### `api/src/auth/auth.controller.ts`

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/auth/challenge` | None | Request a sign-in challenge |
| `POST` | `/auth/verify` | None | Verify signed challenge, return JWT |
| `POST` | `/auth/refresh` | JWT | Refresh an expiring token |
| `GET` | `/auth/profile` | JWT | Get current user profile |

**Flow:**
1. Client calls `POST /auth/challenge` with their wallet address
2. Server returns a random challenge string + nonce
3. Client signs the challenge with their Stellar keypair
4. Client calls `POST /auth/verify` with `{ address, signedChallenge, originalChallenge }`
5. Server verifies the signature using Stellar `Keypair.verify()`
6. Server issues a JWT with `{ sub: address, kycStatus }` in payload

#### `api/src/auth/auth.service.ts`

```typescript
@Injectable()
export class AuthService {
  constructor(
    private readonly jwtService: JwtService,
    private readonly kycService: KycService,
    @InjectRedis() private readonly redis: Redis,
  ) {}

  /// Generate a challenge for wallet authentication
  async generateChallenge(address: string): Promise<ChallengeResponse>

  /// Verify a signed challenge and issue JWT
  async verifySignature(dto: VerifySignatureDto): Promise<AuthTokenResponse>

  /// Refresh an existing JWT
  async refreshToken(token: string): Promise<AuthTokenResponse>

  /// Get user profile from JWT payload
  async getProfile(userId: string): Promise<UserProfileResponse>
}
```

**Key logic:**

```typescript
async generateChallenge(address: string): Promise<ChallengeResponse> {
  if (!this.stellarService.isValidPublicKey(address)) {
    throw new BadRequestException('Invalid Stellar address');
  }

  const nonce = crypto.randomBytes(32).toString('hex');
  const challenge = `NbS Bond Protocol sign-in\nAddress: ${address}\nNonce: ${nonce}\nTimestamp: ${Date.now()}`;

  // Store challenge in Redis with 5min TTL
  await this.redis.set(`challenge:${address}`, challenge, { EX: 300 });

  return { challenge, nonce };
}

async verifySignature(dto: VerifySignatureDto): Promise<AuthTokenResponse> {
  const storedChallenge = await this.redis.get(`challenge:${dto.address}`);
  if (!storedChallenge || storedChallenge !== dto.originalChallenge) {
    throw new UnauthorizedException('Challenge not found or expired');
  }

  // Verify Stellar signature
  const keypair = Keypair.fromPublicKey(dto.address);
  const isValid = keypair.verify(
    Buffer.from(dto.originalChallenge),
    Buffer.from(dto.signedChallenge, 'hex'),
  );

  if (!isValid) {
    throw new UnauthorizedException('Invalid signature');
  }

  // Delete used challenge
  await this.redis.del(`challenge:${dto.address}`);

  // Check KYC status
  const kycStatus = await this.kycService.getStatus(dto.address);

  // Issue JWT
  const payload = { sub: dto.address, kycStatus };
  const accessToken = this.jwtService.sign(payload);

  return { accessToken, tokenType: 'Bearer', expiresIn: '7d' };
}
```

#### `api/src/auth/jwt.strategy.ts`

```typescript
@Injectable()
export class JwtStrategy extends PassportStrategy(Strategy) {
  constructor() {
    super({
      jwtFromRequest: ExtractJwt.fromAuthHeaderAsBearerToken(),
      ignoreExpiration: false,
      secretOrKey: process.env.JWT_SECRET || 'dev-secret-change-in-production',
    });
  }

  async validate(payload: { sub: string; kycStatus: string }): Promise<AuthenticatedUser> {
    return {
      walletAddress: payload.sub,
      kycStatus: payload.kycStatus as KycStatus,
    };
  }
}
```

#### `api/src/auth/kyc.service.ts`

```typescript
@Injectable()
export class KycService {
  constructor(@InjectRedis() private readonly redis: Redis) {}

  /// Get KYC status for a wallet address
  async getStatus(address: string): Promise<KycStatus> {
    const cached = await this.redis.get(`kyc:${address}`);
    if (cached) return cached as KycStatus;
    return KycStatus.PENDING;
  }

  /// Update KYC status (called by webhook from KYC provider)
  async updateStatus(address: string, status: KycStatus): Promise<void> {
    await this.redis.set(`kyc:${address}`, status);
  }

  /// Check if KYC is required and complete for a tranche
  async isEligible(address: string, requiredStatus: KycStatus): Promise<boolean> {
    const actual = await this.getStatus(address);
    return this.compareStatus(actual, requiredStatus);
  }

  private compareStatus(actual: KycStatus, required: KycStatus): boolean {
    const order = [KycStatus.NONE, KycStatus.PENDING, KycStatus.VERIFIED, KycStatus.ACCREDITED];
    return order.indexOf(actual) >= order.indexOf(required);
  }
}

export enum KycStatus {
  NONE = 'none',
  PENDING = 'pending',
  VERIFIED = 'verified',
  ACCREDITED = 'accredited',
}
```

### Guards

#### `api/src/common/guards/jwt-auth.guard.ts`

```typescript
@Injectable()
export class JwtAuthGuard extends AuthGuard('jwt') {
  handleRequest(err: any, user: any, info: any) {
    if (err || !user) {
      throw err || new UnauthorizedException('Invalid or expired token');
    }
    return user;
  }
}
```

#### `api/src/common/guards/admin.guard.ts`

```typescript
@Injectable()
export class AdminGuard implements CanActivate {
  constructor(private readonly stellarService: StellarService) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest();
    const adminKey = process.env.STELLAR_PUBLIC_KEY;
    return request.user?.walletAddress === adminKey;
  }
}
```

#### `api/src/common/guards/kyc.guard.ts`

```typescript
@Injectable()
export class KycGuard implements CanActivate {
  constructor(private readonly kycService: KycService) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest();
    const user = request.user as AuthenticatedUser;

    if (!user) throw new UnauthorizedException('Authentication required');

    const eligible = await this.kycService.isEligible(user.walletAddress, KycStatus.VERIFIED);
    if (!eligible) {
      throw new ForbiddenException('KYC verification required');
    }

    return true;
  }
}
```

#### `api/src/common/guards/provider.guard.ts`

```typescript
@Injectable()
export class ProviderGuard implements CanActivate {
  constructor(private readonly contractService: ContractService) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest();
    const user = request.user as AuthenticatedUser;
    // Check if user's address is a registered oracle provider via contract call
    // Simplified: check env whitelist
    const providers = (process.env.ORACLE_PROVIDER_WHITELIST || '').split(',');
    return providers.includes(user?.walletAddress);
  }
}
```

### Global Error Handling

#### `api/src/common/filters/rfc7807-exception.filter.ts`

Implement an exception filter that catches all `HttpException` and formats errors as RFC 7807 problem details:

```typescript
@Catch()
export class Rfc7807ExceptionFilter implements ExceptionFilter {
  catch(exception: unknown, host: ArgumentsHost) {
    const ctx = host.switchToHttp();
    const response = ctx.getResponse<Response>();
    const request = ctx.getRequest<Request>();

    let status = 500;
    let title = 'Internal Server Error';
    let detail = 'An unexpected error occurred';

    if (exception instanceof HttpException) {
      status = exception.getStatus();
      const exResponse = exception.getResponse();

      if (typeof exResponse === 'object' && exResponse !== null) {
        const resp = exResponse as Record<string, any>;
        title = resp.message || exception.message;
        detail = resp.detail || (typeof resp.message === 'string' ? resp.message : JSON.stringify(resp.message));
      } else {
        detail = String(exResponse);
      }
    }

    response.status(status).json({
      type: `https://errors.nbs-bond-protocol.org/${status}`,
      title,
      status,
      detail,
      instance: request.url,
      timestamp: new Date().toISOString(),
    });
  }
}
```

#### `api/src/common/interceptors/request-logging.interceptor.ts`

```typescript
@Injectable()
export class RequestLoggingInterceptor implements NestInterceptor {
  constructor(private readonly logger: Logger) {}

  intercept(context: ExecutionContext, next: CallHandler): Observable<any> {
    const request = context.switchToHttp().getRequest();
    const requestId = crypto.randomUUID();
    request.requestId = requestId;

    const start = Date.now();
    this.logger.log(`[${requestId}] ${request.method} ${request.url}`);

    return next.handle().pipe(
      tap({
        next: () => this.logger.log(`[${requestId}] ${Date.now() - start}ms`),
        error: (err) => this.logger.error(`[${requestId}] ${err.message}`, err.stack),
      }),
    );
  }
}
```

#### `api/src/common/interfaces/authenticated-request.interface.ts`

```typescript
export interface AuthenticatedUser {
  walletAddress: string;
  kycStatus: KycStatus;
}

export interface AuthenticatedRequest extends Request {
  user: AuthenticatedUser;
  requestId: string;
}
```

### Module Registration

Update `api/src/app.module.ts` to include the error filter and logging interceptor globally:

```typescript
@Module({
  imports: [BondsModule, ProjectsModule, OracleModule, MarketplaceModule, AuthModule, StellarModule],
  providers: [
    { provide: APP_FILTER, useClass: Rfc7807ExceptionFilter },
    { provide: APP_INTERCEPTOR, useClass: RequestLoggingInterceptor },
  ],
})
export class AppModule {}
```

## Verification

```bash
cd api && npm run test -- --testPathPattern='auth'
cd api && npm run test:e2e
```

Expected: 
- 10+ auth tests: challenge generation, signature verification, JWT issuance, refresh, expired challenge, invalid signature, malformed address
- All guards tested with mock contexts
- Global filter formats errors correctly

## Commit Message

```
feat(api): Stellar wallet auth, KYC stubs, guards, and global error handling
```
