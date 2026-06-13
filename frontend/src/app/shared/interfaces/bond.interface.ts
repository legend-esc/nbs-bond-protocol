export interface Bond {
  id: number;
  projectId: string;
  faceValue: number;
  couponSchedule: number[];
  creditType: 'Carbon' | 'Biodiversity' | 'Basket';
  maturityDate: number;
  totalSupply: number;
  totalSubscribed: number;
  status: 'Active' | 'Matured' | 'Defaulted';
  createdAt: string;
}

export interface Project {
  id: number;
  name: string;
  status: 'Pending' | 'Approved' | 'Rejected' | 'Inactive';
  methodology: string;
  country: string;
  metadataIpfsHash: string;
  ownerAddress: string;
  totalAreaHa: number;
  carbonSequestrationEstimate: number;
  createdAt: string;
}

export interface Order {
  id: number;
  seller: string;
  bondId: number;
  amount: number;
  pricePerToken: number;
  quoteAsset: 'USDC' | 'XLM';
  status: 'Open' | 'PartiallyFilled' | 'Filled' | 'Cancelled' | 'Expired';
  createdAt: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  meta: { page: number; limit: number; total: number; totalPages: number };
}

export interface SubscriptionResponse {
  bondId: number;
  subscriber: string;
  amount: number;
  transactionHash: string;
}

export interface CreateProjectDto {
  name: string;
  methodology: string;
  country: string;
  totalAreaHa: number;
  carbonSequestrationEstimate: number;
  metadataIpfsHash?: string;
  nonce: number;
}

export interface ListBondDto {
  bondId: number;
  amount: number;
  pricePerToken: number;
  quoteAsset: 'USDC' | 'XLM';
  nonce: number;
}

export interface BuyBondDto {
  orderId: number;
  amount: number;
  maxPrice: number;
  nonce: number;
}
