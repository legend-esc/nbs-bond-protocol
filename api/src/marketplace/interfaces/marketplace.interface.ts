export enum OrderStatus {
  Open = 'Open',
  PartiallyFilled = 'PartiallyFilled',
  Filled = 'Filled',
  Cancelled = 'Cancelled',
  Expired = 'Expired',
}

export interface OrderResponse {
  id: number;
  seller: string;
  bondId: number;
  amount: number;
  pricePerToken: number;
  quoteAsset: 'USDC' | 'XLM';
  status: OrderStatus;
  createdAt: string;
}

export interface PriceFeedResponse {
  bondId: number;
  bestPrice: number;
  averagePrice: number;
  totalOrders: number;
  totalVolume: number;
}

export interface PriceLevel {
  price: number;
  amount: number;
  total: number;
}

export interface SlippageResponse {
  bondId: number;
  amount: number;
  averagePrice: number;
  estimatedTotal: number;
  slippagePercent: number;
}
