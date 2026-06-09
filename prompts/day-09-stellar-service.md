# Day 9 — `StellarService` + `ContractService`

Load context: `prompts/context/tech-stack.md`, `prompts/context/api-patterns.md`

## Goal

Implement the two core API services that wrap Stellar Horizon RPC and Soroban contract interactions. These are the foundation for all other API modules.

## Files to Create

### `api/src/stellar/stellar.module.ts`

```typescript
@Module({
  providers: [StellarService, ContractService],
  exports: [StellarService, ContractService],
  global: true,
})
export class StellarModule {}
```

### `api/src/stellar/stellar.service.ts`

Wraps `@stellar/stellar-sdk` Horizon interactions:

```typescript
@Injectable()
export class StellarService {
  private horizon: Horizon.Server;
  private networkPassphrase: string;

  constructor() {
    this.horizon = new Horizon.Server(
      process.env.STELLAR_HORIZON_URL || 'https://horizon-testnet.stellar.org',
    );
    this.networkPassphrase = process.env.STELLAR_NETWORK === 'mainnet'
      ? Networks.PUBLIC
      : Networks.TESTNET;
  }
}
```

**Methods:**

```typescript
/// Get account details by public key
async getAccount(publicKey: string): Promise<AccountResponse>

/// Get native (XLM) balance for an account
async getBalance(publicKey: string): Promise<string>

/// Get all balances (native + assets) for an account
async getBalances(publicKey: string): Promise<BalanceLine[]>

/// Submit a raw transaction to Horizon
async submitTransaction(
  txEnvelope: string,  // base64-encoded XDR
): Promise<SubmitTransactionResponse>

/// Stream payment operations for an account
streamPayments(
  publicKey: string,
  onPayment: (payment: PaymentOperation) => void,
  cursor?: string,
): Promise<void>

/// Generate a keypair from a secret key (for admin operations)
getKeypairFromSecret(secretKey: string): Keypair

/// Generate a random keypair (for test wallets)
generateKeypair(): { publicKey: string; secretKey: string }

/// Verify that a string is a valid Stellar public key (G...)
isValidPublicKey(address: string): boolean

/// Get the network passphrase
getNetworkPassphrase(): string

/// Check if account exists on network
async accountExists(publicKey: string): Promise<boolean>
```

**Implementation notes:**
- Use `@stellar/stellar-sdk` v12 — `Horizon.Server`, `Keypair`, `Networks`
- All methods should be async where they make network calls
- Handle Horizon errors gracefully — wrap in `RpcException` with descriptive messages
- Cache account lookups in a local Map with 30-second TTL (simple in-memory cache, no Redis for this service)
- Add a `isValidPublicKey` method using `StrKey.isValidEd25519PublicKey`

### `api/src/stellar/contract.service.ts`

Typed wrapper for invoking Soroban contract functions:

```typescript
@Injectable()
export class ContractService {
  constructor(private readonly stellarService: StellarService) {}
}
```

**Types:**

```typescript
interface ContractCallOptions {
  contractAddress: string;     // C... address
  method: string;              // e.g., "get_project"
  args: ScVal[];               // Soroban-formatted arguments
  sourceSecretKey?: string;    // For state-changing calls
}

interface ContractCallResult {
  result: ScVal;
  transactionHash?: string;
  successful: boolean;
}
```

**Methods:**

```typescript
/// Simulate a contract call (read-only, no fee)
async simulateCall(options: ContractCallOptions): Promise<ScVal>

/// Execute a state-changing contract call
async sendTransaction(options: ContractCallOptions): Promise<ContractCallResult>

/// Encode a native JS value to ScVal (handle Address, u64, i128, BytesN, Symbol, Vec)
encodeArg(value: unknown, type: ScValType): ScVal

/// Decode ScVal to native JS value
decodeArg(scval: ScVal): unknown

/// Convenience: call a contract function that takes (env, caller, ...args, nonce) pattern
async invokeContractMethod(
  contractAddress: string,
  method: string,
  callerSecretKey: string,
  args: unknown[],
  nonce: number,
): Promise<ContractCallResult>
```

**ScVal encoding helpers:**
- `Address.fromString(address)` → wrap in `Address.scVal()`
- `i128` → `scvI128()` or using `nativeToScVal`
- `u64` → `scvU64()` or `nativeToScVal`
- `BytesN<32>` → hex string to `Buffer` to `xdr.ScVal.scvBytes()`
- `Symbol` → `nativeToScVal(symbol, { type: 'symbol' })`
- `Vec` → `nativeToScVal(array, { type: 'vec' })`

Use `xdr.ScVal` and related types from `@stellar/stellar-sdk`.

**Error handling:**
- Parse Soroban error responses from Horizon
- Map contract errors (e.g., `ContractError(1)`) to human-readable messages using the error enums from shared types
- Throw `BadRequestException` with the decoded error message

### `api/src/stellar/interfaces/stellar.interface.ts`

```typescript
export interface StellarAccount {
  publicKey: string;
  balances: BalanceLine[];
  sequenceNumber: string;
}

export interface ContractDeployment {
  contractId: string;
  wasmHash: string;
  deployTxHash: string;
}

export interface ContractCallError {
  code: number;
  message: string;
  contractAddress: string;
  method: string;
}
```

## Verification

```bash
cd api && npm run test -- --testPathPattern='stellar'
```

Expected: 10+ tests covering:
- `StellarService` — account lookup (mocked), balance query, key validation, keypair generation
- `ContractService` — ScVal encoding for each type, ScVal decoding, error mapping, simulate call (mocked)

## Commit Message

```
feat(api): StellarService and ContractService with typed Soroban interaction
```
