export enum ReportStatus {
  Pending = 'Pending',
  Verified = 'Verified',
  Challenged = 'Challenged',
  Rejected = 'Rejected',
}

export interface ReportResponse {
  id: number;
  projectId: string;
  periodStart: number;
  periodEnd: number;
  carbonSequestered: number;
  methodology: string;
  ipfsHash: string;
  providerAddress: string;
  status: ReportStatus;
  createdAt: string;
}

export interface ChallengeResponse {
  reportId: number;
  challengerAddress: string;
  reason: string;
  counterEvidenceHash: string;
  resolved: boolean;
  createdAt: string;
}

export interface ProviderResponse {
  providerAddress: string;
  methodology: string;
  name: string;
  active: boolean;
  registeredAt: string;
}
