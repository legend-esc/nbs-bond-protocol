import { Injectable, NotFoundException } from '@nestjs/common';
import { ContractService, ContractCallOptions } from '../stellar/contract.service';
import { StellarService } from '../stellar/stellar.service';
import { xdr, nativeToScVal, scValToNative, Address } from '@stellar/stellar-sdk';
import { createClient, RedisClientType } from '@redis/client';
import { CreateBondDto } from './dto/create-bond.dto';
import { SubscribeDto } from './dto/subscribe.dto';
import { DistributeCouponDto } from './dto/distribute-coupon.dto';
import {
  BondResponse,
  SubscriptionResponse,
  HolderListResponse,
  CouponDistributionResponse,
  BondStatusEnum,
  CreditTypeEnum,
} from './interfaces/bond.interface';

const BOND_ISSUER = () => process.env.BOND_ISSUER_ADDRESS || '';
const COUPON_ENGINE = () => process.env.COUPON_ENGINE_ADDRESS || '';

@Injectable()
export class BondsService {
  private redis: RedisClientType;

  constructor(
    private readonly contractService: ContractService,
    private readonly stellarService: StellarService,
  ) {
    this.redis = createClient({ url: process.env.REDIS_URL || 'redis://localhost:6379' });
    this.redis.connect().catch(() => {});
  }

  async create(dto: CreateBondDto): Promise<BondResponse> {
    const adminSecret = this.getAdminSecret();
    const adminAddress = this.stellarService.getKeypairFromSecret(adminSecret).publicKey();
    const nonce = await this.getNonce(adminAddress);

    const configScVal = this.encodeBondConfig(dto);

    const { result } = await this.contractService.invokeContractMethod(
      BOND_ISSUER(), 'issue_bond', adminSecret,
      [Address.fromString(adminAddress).toScVal(), configScVal],
      nonce,
    );

    const bondId = Number(scValToNative(result));
    const bond = await this.buildBondResponse(bondId);
    await this.redis.setEx(`bond:${bondId}`, 300, JSON.stringify(bond));
    return bond;
  }

  async findAll(page = 1, limit = 20) {
    const cacheKey = `bonds:${page}:${limit}`;
    const cached = await this.redis.get(cacheKey);
    if (cached) return JSON.parse(cached);

    const bonds: BondResponse[] = [];
    let total = 0;

    try {
      const countScVal = await this.contractService.simulateCall({
        contractAddress: BOND_ISSUER(), method: 'total_subscribed', args: [],
      });
      total = Number(scValToNative(countScVal));
    } catch {}

    const start = (page - 1) * limit;
    const end = Math.min(start + limit, total);

    for (let id = 1; id <= total; id++) {
      if (id > start && id <= end) {
        try {
          bonds.push(await this.buildBondResponse(id));
        } catch {}
      }
    }

    const result = {
      data: bonds,
      meta: { page, limit, total, totalPages: Math.ceil(total / limit) || 1 },
    };

    await this.redis.setEx(cacheKey, 60, JSON.stringify(result));
    return result;
  }

  async findOne(id: number): Promise<BondResponse> {
    const cached = await this.redis.get(`bond:${id}`);
    if (cached) return JSON.parse(cached);

    const bond = await this.buildBondResponse(id);
    await this.redis.setEx(`bond:${id}`, 300, JSON.stringify(bond));
    return bond;
  }

  async subscribe(id: number, dto: SubscribeDto): Promise<SubscriptionResponse> {
    const investorSecret = process.env.INVESTOR_SECRET_KEY || '';
    const { result, transactionHash } = await this.contractService.invokeContractMethod(
      BOND_ISSUER(), 'subscribe', investorSecret,
      [
        Address.fromString(dto.investorAddress).toScVal(),
        nativeToScVal(BigInt(id), { type: 'u64' }),
        nativeToScVal(BigInt(dto.amount), { type: 'i128' }),
      ],
      dto.nonce,
    );

    await this.redis.del(`bond:${id}`);
    await this.redis.sAdd(`bond:${id}:holders`, dto.investorAddress);

    return { bondId: id, investorAddress: dto.investorAddress, amount: dto.amount, transactionHash: transactionHash || '' };
  }

  async getHolders(id: number): Promise<HolderListResponse> {
    const holderAddresses = await this.redis.sMembers(`bond:${id}:holders`);
    const holders = [];

    for (const address of holderAddresses) {
      try {
        const balanceScVal = await this.contractService.simulateCall({
          contractAddress: BOND_ISSUER(), method: 'get_holder_balance',
          args: [nativeToScVal(BigInt(id), { type: 'u64' }), Address.fromString(address).toScVal()],
        });
        const balance = Number(scValToNative(balanceScVal));
        if (balance > 0) holders.push({ address, balance });
      } catch {}
    }

    return { bondId: id, holders, total: holders.length };
  }

  async distributeCoupon(id: number, dto: DistributeCouponDto): Promise<CouponDistributionResponse> {
    const adminSecret = this.getAdminSecret();
    const adminAddress = this.stellarService.getKeypairFromSecret(adminSecret).publicKey();
    const nonce = await this.getNonce(adminAddress);

    const holderAddresses = await this.redis.sMembers(`bond:${id}:holders`);

    const { result } = await this.contractService.invokeContractMethod(
      COUPON_ENGINE(), 'distribute_coupon', adminSecret,
      [
        nativeToScVal(BigInt(id), { type: 'u64' }),
        nativeToScVal(dto.periodIndex, { type: 'u32' }),
        xdr.ScVal.scvVec(holderAddresses.map((h) => Address.fromString(h).toScVal())),
        this.encodeOracleReport(dto.report),
      ],
      nonce,
    );

    const parsed = scValToNative(result) as any[];
    return {
      bondId: id,
      periodIndex: dto.periodIndex,
      totalCredits: Number(parsed?.[2] ?? 0),
      holderCount: Number(parsed?.[3] ?? 0),
    };
  }

  async mature(id: number): Promise<BondResponse> {
    const adminSecret = this.getAdminSecret();
    const adminAddress = this.stellarService.getKeypairFromSecret(adminSecret).publicKey();
    const nonce = await this.getNonce(adminAddress);

    await this.contractService.invokeContractMethod(
      BOND_ISSUER(), 'mature_bond', adminSecret,
      [Address.fromString(adminAddress).toScVal(), nativeToScVal(BigInt(id), { type: 'u64' })],
      nonce,
    );

    await this.redis.del(`bond:${id}`);
    return this.buildBondResponse(id);
  }

  private encodeBondConfig(dto: CreateBondDto): xdr.ScVal {
    return xdr.ScVal.scvVec([
      xdr.ScVal.scvBytes(Buffer.from(dto.projectId, 'hex')),
      nativeToScVal(BigInt(dto.faceValue), { type: 'i128' }),
      xdr.ScVal.scvVec(dto.couponSchedule.map((ts) => nativeToScVal(BigInt(ts), { type: 'u64' }))),
      nativeToScVal(dto.creditType, { type: 'symbol' }),
      nativeToScVal(BigInt(dto.maturityDate), { type: 'u64' }),
      nativeToScVal(BigInt(dto.totalSupply), { type: 'i128' }),
    ]);
  }

  private encodeOracleReport(report: any): xdr.ScVal {
    return xdr.ScVal.scvVec([
      xdr.ScVal.scvBytes(Buffer.from(report.projectId, 'hex')),
      nativeToScVal(BigInt(report.periodStart), { type: 'u64' }),
      nativeToScVal(BigInt(report.periodEnd), { type: 'u64' }),
      nativeToScVal(BigInt(report.carbonSequestered), { type: 'i128' }),
      nativeToScVal(report.methodology, { type: 'symbol' }),
      xdr.ScVal.scvBytes(Buffer.from(report.providerSignature, 'hex')),
      xdr.ScVal.scvBytes(Buffer.from(report.ipfsEvidenceHash, 'hex')),
    ]);
  }

  private async buildBondResponse(id: number): Promise<BondResponse> {
    const configScVal = await this.contractService.simulateCall({
      contractAddress: BOND_ISSUER(), method: 'get_bond',
      args: [nativeToScVal(BigInt(id), { type: 'u64' })],
    });
    const config = scValToNative(configScVal) as any[];

    const stateScVal = await this.contractService.simulateCall({
      contractAddress: BOND_ISSUER(), method: 'get_bond_state',
      args: [nativeToScVal(BigInt(id), { type: 'u64' })],
    });
    const state = scValToNative(stateScVal) as any[];

    return {
      id,
      projectId: Buffer.from(config[0] as Uint8Array).toString('hex'),
      faceValue: Number(config[1]),
      couponSchedule: (config[2] as any[]).map((v: bigint) => Number(v)),
      creditType: config[3] as CreditTypeEnum,
      maturityDate: Number(config[4]),
      totalSupply: Number(config[5]),
      totalSubscribed: Number(state[0]),
      status: state[1] as BondStatusEnum,
      createdAt: new Date(Number(state[2]) * 1000).toISOString(),
    };
  }

  private async getNonce(address: string): Promise<number> {
    try {
      const key = `nonce:${address}`;
      const stored = await this.redis.get(key);
      const next = (stored ? parseInt(stored, 10) : 0) + 1;
      await this.redis.set(key, next.toString());
      return next;
    } catch {
      return Math.floor(Date.now() / 1000);
    }
  }

  private getAdminSecret(): string {
    return process.env.ADMIN_SECRET_KEY || '';
  }
}
