import { Injectable } from '@nestjs/common';
import { DexService } from './dex.service';
import {
  PriceFeedResponse,
  PriceLevel,
  SlippageResponse,
  OrderStatus,
} from './interfaces/marketplace.interface';
import { createClient, RedisClientType } from '@redis/client';

@Injectable()
export class LiquidityService {
  private redis: RedisClientType;

  constructor(
    private readonly dexService: DexService,
  ) {
    this.redis = createClient({ url: process.env.REDIS_URL || 'redis://localhost:6379' });
    this.redis.connect().catch(() => {});
  }

  async getPriceFeed(bondId?: number): Promise<PriceFeedResponse[]> {
    const cacheKey = `pricefeed:${bondId || 'all'}`;
    const cached = await this.redis.get(cacheKey);
    if (cached) return JSON.parse(cached);

    const ordersResult = await this.dexService.listOrders(bondId, 'Open', 1, 100);
    const openOrders = ordersResult.data;

    const grouped = new Map<number, { prices: number[]; amounts: number[]; totalVolume: number }>();

    for (const order of openOrders) {
      if (order.status !== OrderStatus.Open) continue;

      const group = grouped.get(order.bondId) || { prices: [], amounts: [], totalVolume: 0 };
      group.prices.push(order.pricePerToken);
      group.amounts.push(order.amount);
      group.totalVolume += order.amount * order.pricePerToken;
      grouped.set(order.bondId, group);
    }

    const feeds: PriceFeedResponse[] = [];

    for (const [id, group] of grouped) {
      const bestPrice = Math.min(...group.prices);
      const averagePrice = group.prices.reduce((a, b) => a + b, 0) / group.prices.length;

      feeds.push({
        bondId: id,
        bestPrice,
        averagePrice,
        totalOrders: group.prices.length,
        totalVolume: group.totalVolume,
      });
    }

    await this.redis.setEx(cacheKey, 30, JSON.stringify(feeds));
    return feeds;
  }

  async getBestPrice(bondId: number, side: 'buy' | 'sell'): Promise<PriceLevel> {
    const ordersResult = await this.dexService.listOrders(bondId, 'Open', 1, 100);
    const openOrders = ordersResult.data;

    const sorted = [...openOrders].sort((a, b) => a.pricePerToken - b.pricePerToken);

    if (sorted.length === 0) {
      return { price: 0, amount: 0, total: 0 };
    }

    const best = sorted[0];

    return {
      price: best.pricePerToken,
      amount: best.amount,
      total: best.pricePerToken * best.amount,
    };
  }

  async calculateSlippage(bondId: number, amount: number): Promise<SlippageResponse> {
    const ordersResult = await this.dexService.listOrders(bondId, 'Open', 1, 100);
    const openOrders = ordersResult.data;

    const sorted = [...openOrders].sort((a, b) => a.pricePerToken - b.pricePerToken);

    let remaining = amount;
    let totalCost = 0;
    let totalAmount = 0;

    for (const order of sorted) {
      if (remaining <= 0) break;
      const take = Math.min(remaining, order.amount);
      totalCost += take * order.pricePerToken;
      totalAmount += take;
      remaining -= take;
    }

    const averagePrice = totalAmount > 0 ? totalCost / totalAmount : 0;
    const idealCost = amount > 0 ? amount * (sorted[0]?.pricePerToken || 0) : 0;
    const slippagePercent = idealCost > 0 ? ((totalCost - idealCost) / idealCost) * 100 : 0;

    return {
      bondId,
      amount,
      averagePrice,
      estimatedTotal: totalCost,
      slippagePercent: Math.max(0, slippagePercent),
    };
  }
}
