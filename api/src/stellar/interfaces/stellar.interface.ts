import { Horizon } from '@stellar/stellar-sdk';

export interface StellarAccount {
  publicKey: string;
  balances: Horizon.HorizonApi.BalanceLine[];
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
