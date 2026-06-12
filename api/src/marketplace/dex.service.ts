import { Injectable } from '@nestjs/common';
import { ContractService } from '../stellar/contract.service';
import { StellarService } from '../stellar/stellar.service';
import { ListBondDto } from './dto/list-bond.dto';
import { BuyBondDto } from './dto/buy-bond.dto';
import {
  OrderResponse,
  OrderStatus,
} from './interfaces/marketplace.interface';
import { createClient, RedisClientType } from '@redis/client';
import { nativeToScVal, scValToNative, Address } from '@stellar/stellar-sdk';
import { PaginatedResponse } from '../common/dto/pagination.dto';

const DEX_ROUTER = () => process.env.DEX_ROUTER_ADDRESS || '';

@Injectable()
export class DexService {
  private redis: RedisClientType;

  constructor(
    private readonly contractService: ContractService,
    private readonly stellarService: StellarService,
  ) {
    this.redis = createClient({ url: process.env.REDIS_URL || 'redis://localhost:6379' });
    this.redis.connect().catch(() => {});
  }

  async listOrders(
    bondId?: number,
    status?: string,
    page = 1,
    limit = 20,
  ): Promise<PaginatedResponse<OrderResponse>> {
    const cacheKey = `orders:${bondId || 'all'}:${status || 'all'}:${page}:${limit}`;
    const cached = await this.redis.get(cacheKey);
    if (cached) return JSON.parse(cached);

    const orders: OrderResponse[] = [];
    let total = 0;
    let index = 1;

    while (true) {
      try {
        const orderScVal = await this.contractService.simulateCall({
          contractAddress: DEX_ROUTER(),
          method: 'get_order',
          args: [nativeToScVal(BigInt(index), { type: 'u64' })],
        });
        const data = scValToNative(orderScVal) as any[];
        const orderBondId = Number(data[2]);
        const orderStatus = data[5] as string;

        if (bondId && orderBondId !== bondId) {
          index++;
          continue;
        }
        if (status && orderStatus !== status) {
          index++;
          continue;
        }

        orders.push({
          id: Number(data[0]),
          seller: data[1] as string,
          bondId: orderBondId,
          amount: Number(data[3]),
          pricePerToken: Number(data[4]),
          quoteAsset: data[6] as 'USDC' | 'XLM',
          status: orderStatus as OrderStatus,
          createdAt: new Date(Number(data[7]) * 1000).toISOString(),
        });
        total++;
        index++;
      } catch {
        break;
      }
    }

    const start = (page - 1) * limit;
    const paged = orders.slice(start, start + limit);

    const result = {
      data: paged,
      meta: { page, limit, total, totalPages: Math.ceil(total / limit) || 1 },
    };

    await this.redis.setEx(cacheKey, 30, JSON.stringify(result));
    return result;
  }

  async listBondTokens(dto: ListBondDto, sellerAddress: string): Promise<OrderResponse> {
    const adminSecret = this.getAdminSecret();

    const { result } = await this.contractService.invokeContractMethod(
      DEX_ROUTER(), 'list_bond_tokens', adminSecret,
      [
        Address.fromString(sellerAddress).toScVal(),
        nativeToScVal(BigInt(dto.bondId), { type: 'u64' }),
        nativeToScVal(BigInt(dto.amount), { type: 'i128' }),
        nativeToScVal(BigInt(dto.pricePerToken), { type: 'i128' }),
        nativeToScVal(dto.quoteAsset, { type: 'symbol' }),
        nativeToScVal(BigInt(dto.expiresAfterSeconds || 604800), { type: 'u64' }),
      ],
      dto.nonce,
    );

    const data = scValToNative(result) as any[];

    await this.redis.del(`orders:*`);

    return {
      id: Number(data[0]),
      seller: sellerAddress,
      bondId: dto.bondId,
      amount: dto.amount,
      pricePerToken: dto.pricePerToken,
      quoteAsset: dto.quoteAsset,
      status: OrderStatus.Open,
      createdAt: new Date().toISOString(),
    };
  }

  async buyBondTokens(dto: BuyBondDto, buyerAddress: string): Promise<OrderResponse> {
    const adminSecret = this.getAdminSecret();

    const { result } = await this.contractService.invokeContractMethod(
      DEX_ROUTER(), 'execute_purchase', adminSecret,
      [
        Address.fromString(buyerAddress).toScVal(),
        nativeToScVal(BigInt(dto.orderId), { type: 'u64' }),
        nativeToScVal(BigInt(dto.amount), { type: 'i128' }),
        nativeToScVal(BigInt(dto.maxPrice), { type: 'i128' }),
      ],
      dto.nonce,
    );

    const data = scValToNative(result) as any[];

    await this.redis.del(`orders:*`);

    return {
      id: dto.orderId,
      seller: data[1] as string,
      bondId: Number(data[2]),
      amount: dto.amount,
      pricePerToken: dto.maxPrice,
      quoteAsset: data[6] as 'USDC' | 'XLM',
      status: OrderStatus.Open,
      createdAt: new Date().toISOString(),
    };
  }

  async cancelOrder(orderId: number, callerAddress: string): Promise<void> {
    const adminSecret = this.getAdminSecret();

    await this.contractService.invokeContractMethod(
      DEX_ROUTER(), 'cancel_listing', adminSecret,
      [
        Address.fromString(callerAddress).toScVal(),
        nativeToScVal(BigInt(orderId), { type: 'u64' }),
      ],
      await this.getNonce(callerAddress),
    );

    await this.redis.del(`orders:*`);
  }

  async getOrder(orderId: number): Promise<OrderResponse> {
    const cacheKey = `order:${orderId}`;
    const cached = await this.redis.get(cacheKey);
    if (cached) return JSON.parse(cached);

    const orderScVal = await this.contractService.simulateCall({
      contractAddress: DEX_ROUTER(),
      method: 'get_order',
      args: [nativeToScVal(BigInt(orderId), { type: 'u64' })],
    });
    const data = scValToNative(orderScVal) as any[];

    const order = {
      id: Number(data[0]),
      seller: data[1] as string,
      bondId: Number(data[2]),
      amount: Number(data[3]),
      pricePerToken: Number(data[4]),
      quoteAsset: data[6] as 'USDC' | 'XLM',
      status: data[5] as OrderStatus,
      createdAt: new Date(Number(data[7]) * 1000).toISOString(),
    };

    await this.redis.setEx(cacheKey, 60, JSON.stringify(order));
    return order;
  }

  private getAdminSecret(): string {
    return process.env.ADMIN_SECRET_KEY || '';
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
}
