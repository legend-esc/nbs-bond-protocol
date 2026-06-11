import { Injectable, BadRequestException } from '@nestjs/common';
import {
  rpc,
  TransactionBuilder,
  Keypair,
  nativeToScVal,
  scValToNative,
  Address,
  Contract,
  Account,
  BASE_FEE,
  xdr,
} from '@stellar/stellar-sdk';
import { StellarService } from './stellar.service';

export interface ContractCallOptions {
  contractAddress: string;
  method: string;
  args: xdr.ScVal[];
  sourceSecretKey?: string;
}

export interface ContractCallResult {
  result: xdr.ScVal;
  transactionHash?: string;
  successful: boolean;
}

@Injectable()
export class ContractService {
  private sorobanRpc: rpc.Server;

  constructor(private readonly stellarService: StellarService) {
    this.sorobanRpc = new rpc.Server(
      process.env.SOROBAN_RPC_URL || 'http://localhost:8000/soroban/rpc',
    );
  }

  async simulateCall(options: ContractCallOptions): Promise<xdr.ScVal> {
    try {
      const { contractAddress, method, args } = options;

      const keypair = options.sourceSecretKey
        ? Keypair.fromSecret(options.sourceSecretKey)
        : Keypair.random();

      const account = new Account(keypair.publicKey(), '0');
      const contract = new Contract(contractAddress);

      const transaction = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: this.stellarService.getNetworkPassphrase(),
      })
        .addOperation(contract.call(method, ...args))
        .setTimeout(30)
        .build();

      const simulation = await this.sorobanRpc.simulateTransaction(transaction);

      if (rpc.Api.isSimulationError(simulation)) {
        throw new BadRequestException(
          `Contract simulation failed: ${simulation.error}`,
        );
      }

      if (!simulation.result) {
        throw new BadRequestException(
          'Simulation returned no result',
        );
      }

      return simulation.result.retval;
    } catch (error) {
      if (error instanceof BadRequestException) {
        throw error;
      }
      throw new BadRequestException(
        `Failed to simulate contract call: ${error.message}`,
      );
    }
  }

  async sendTransaction(options: ContractCallOptions): Promise<ContractCallResult> {
    try {
      const { contractAddress, method, args, sourceSecretKey } = options;

      if (!sourceSecretKey) {
        throw new BadRequestException(
          'sourceSecretKey is required for state-changing transactions',
        );
      }

      const keypair = Keypair.fromSecret(sourceSecretKey);
      const contract = new Contract(contractAddress);

      const horizonAccount = await this.stellarService.getAccount(keypair.publicKey());
      const account = new Account(keypair.publicKey(), horizonAccount.sequence);

      const transaction = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: this.stellarService.getNetworkPassphrase(),
      })
        .addOperation(contract.call(method, ...args))
        .setTimeout(30)
        .build();

      const simulation = await this.sorobanRpc.simulateTransaction(transaction);

      if (rpc.Api.isSimulationError(simulation)) {
        throw new BadRequestException(
          `Transaction simulation failed: ${simulation.error}`,
        );
      }

      const preparedTransaction = await this.sorobanRpc.prepareTransaction(transaction);

      preparedTransaction.sign(keypair);

      const response = await this.sorobanRpc.sendTransaction(preparedTransaction);

      if (response.status === 'ERROR') {
        const errorMessage = this.decodeContractError(contractAddress, method);
        throw new BadRequestException(errorMessage);
      }

      return {
        result: simulation.result?.retval ?? xdr.ScVal.scvVoid(),
        transactionHash: response.hash,
        successful: true,
      };
    } catch (error) {
      if (error instanceof BadRequestException) {
        throw error;
      }
      throw new BadRequestException(
        `Failed to submit contract transaction: ${error.message}`,
      );
    }
  }

  encodeArg(value: unknown, type: string): xdr.ScVal {
    switch (type) {
      case 'address': {
        return Address.fromString(value as string).toScVal();
      }
      case 'i128': {
        return nativeToScVal(BigInt(value as number | bigint | string), { type: 'i128' });
      }
      case 'u64': {
        return nativeToScVal(BigInt(value as number | bigint | string), { type: 'u64' });
      }
      case 'bytes': {
        const buf = Buffer.from(value as string, 'hex');
        return xdr.ScVal.scvBytes(buf);
      }
      case 'symbol': {
        return nativeToScVal(value as string, { type: 'symbol' });
      }
      case 'string': {
        return nativeToScVal(value as string, { type: 'string' });
      }
      case 'bool': {
        return xdr.ScVal.scvBool(value as boolean);
      }
      case 'u32': {
        return xdr.ScVal.scvU32(value as number);
      }
      case 'i32': {
        return xdr.ScVal.scvI32(value as number);
      }
      case 'void': {
        return xdr.ScVal.scvVoid();
      }
      case 'vec': {
        return xdr.ScVal.scvVec(value as xdr.ScVal[]);
      }
      case 'map': {
        return xdr.ScVal.scvMap(value as xdr.ScMapEntry[]);
      }
      default:
        throw new BadRequestException(`Unsupported ScVal type: ${type}`);
    }
  }

  decodeArg(scval: xdr.ScVal): unknown {
    return scValToNative(scval);
  }

  async invokeContractMethod(
    contractAddress: string,
    method: string,
    callerSecretKey: string,
    args: unknown[],
    nonce: number,
  ): Promise<ContractCallResult> {
    const encodedArgs = args.map((arg) => {
      if (arg instanceof xdr.ScVal) {
        return arg;
      }
      return nativeToScVal(arg);
    });

    const nonceScVal = nativeToScVal(BigInt(nonce), { type: 'u64' });
    const allArgs = [...encodedArgs, nonceScVal];

    return this.sendTransaction({
      contractAddress,
      method,
      args: allArgs,
      sourceSecretKey: callerSecretKey,
    });
  }

  private decodeContractError(
    contractAddress: string,
    method: string,
  ): string {
    return `Contract error on ${contractAddress}.${method}`;
  }

  getSorobanRpc(): rpc.Server {
    return this.sorobanRpc;
  }
}
