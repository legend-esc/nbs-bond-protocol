export enum CreditTypeEnum {
  Carbon = 'Carbon',
  Biodiversity = 'Biodiversity',
  Basket = 'Basket',
}

export enum BondStatusEnum {
  Active = 'Active',
  Matured = 'Matured',
  Defaulted = 'Defaulted',
}

export interface BondResponse {
  id: number;
  projectId: string;
  faceValue: number;
  couponSchedule: number[];
  creditType: CreditTypeEnum;
  maturityDate: number;
  totalSupply: number;
  totalSubscribed: number;
  status: BondStatusEnum;
  createdAt: string;
}

export interface SubscriptionResponse {
  bondId: number;
  investorAddress: string;
  amount: number;
  transactionHash: string;
}

export interface HolderListResponse {
  bondId: number;
  holders: Array<{ address: string; balance: number }>;
  total: number;
}

export interface CouponDistributionResponse {
  bondId: number;
  periodIndex: number;
  totalCredits: number;
  holderCount: number;
}
