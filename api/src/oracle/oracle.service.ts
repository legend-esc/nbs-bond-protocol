import { Injectable, BadRequestException, NotFoundException } from '@nestjs/common';
import { ContractService } from '../stellar/contract.service';
import { IpfsService } from '../projects/ipfs.service';
import { SubmitReportDto } from './dto/submit-report.dto';
import { ChallengeDto } from './dto/challenge.dto';
import { RegisterProviderDto } from './dto/register-provider.dto';
import {
  ReportResponse,
  ChallengeResponse,
  ProviderResponse,
  ReportStatus,
} from './interfaces/oracle.interface';
import { createClient, RedisClientType } from '@redis/client';
import { nativeToScVal, scValToNative, Address } from '@stellar/stellar-sdk';
import { StellarService } from '../stellar/stellar.service';

const ORACLE_CONSUMER = () => process.env.ORACLE_CONSUMER_ADDRESS || '';

@Injectable()
export class OracleService {
  private redis: RedisClientType;

  constructor(
    private readonly contractService: ContractService,
    private readonly ipfsService: IpfsService,
    private readonly stellarService: StellarService,
  ) {
    this.redis = createClient({ url: process.env.REDIS_URL || 'redis://localhost:6379' });
    this.redis.connect().catch(() => {});
  }

  async submitReport(dto: SubmitReportDto, providerAddress: string): Promise<ReportResponse> {
    const ipfsResult = await this.ipfsService.uploadJson({
      projectId: dto.projectId,
      periodStart: dto.periodStart,
      periodEnd: dto.periodEnd,
      carbonSequestered: dto.carbonSequestered,
      methodology: dto.methodology,
      evidenceHash: dto.evidenceHash,
      providerAddress,
      timestamp: new Date().toISOString(),
    });

    const adminSecret = this.getAdminSecret();

    const { result } = await this.contractService.invokeContractMethod(
      ORACLE_CONSUMER(), 'submit_report', adminSecret,
      [
        Address.fromString(providerAddress).toScVal(),
        nativeToScVal(BigInt(dto.periodStart), { type: 'u64' }),
        nativeToScVal(BigInt(dto.periodEnd), { type: 'u64' }),
        nativeToScVal(BigInt(dto.carbonSequestered), { type: 'i128' }),
        nativeToScVal(dto.methodology, { type: 'symbol' }),
        nativeToScVal(ipfsResult.hash, { type: 'string' }),
      ],
      dto.nonce,
    );

    const reportId = Number(scValToNative(result));

    return {
      id: reportId,
      projectId: dto.projectId,
      periodStart: dto.periodStart,
      periodEnd: dto.periodEnd,
      carbonSequestered: dto.carbonSequestered,
      methodology: dto.methodology,
      ipfsHash: ipfsResult.hash,
      providerAddress,
      status: ReportStatus.Pending,
      createdAt: new Date().toISOString(),
    };
  }

  async getProjectReports(projectId: string): Promise<ReportResponse[]> {
    const cacheKey = `reports:${projectId}`;
    const cached = await this.redis.get(cacheKey);
    if (cached) return JSON.parse(cached);

    const reports: ReportResponse[] = [];
    let index = 1;

    while (true) {
      try {
        const reportScVal = await this.contractService.simulateCall({
          contractAddress: ORACLE_CONSUMER(),
          method: 'get_report',
          args: [nativeToScVal(BigInt(index), { type: 'u64' })],
        });
        const data = scValToNative(reportScVal) as any[];
        const reportProjectId = Buffer.from(data[0] as Uint8Array).toString('hex');

        if (reportProjectId === projectId) {
          reports.push({
            id: index,
            projectId: reportProjectId,
            periodStart: Number(data[1]),
            periodEnd: Number(data[2]),
            carbonSequestered: Number(data[3]),
            methodology: data[4] as string,
            ipfsHash: data[5] as string,
            providerAddress: data[6] as string,
            status: data[7] as ReportStatus,
            createdAt: new Date(Number(data[8]) * 1000).toISOString(),
          });
        }
        index++;
      } catch {
        break;
      }
    }

    await this.redis.setEx(cacheKey, 60, JSON.stringify(reports));
    return reports;
  }

  async challengeReport(reportId: number, dto: ChallengeDto, challengerAddress: string): Promise<ChallengeResponse> {
    const adminSecret = this.getAdminSecret();

    await this.contractService.invokeContractMethod(
      ORACLE_CONSUMER(), 'challenge_report', adminSecret,
      [
        nativeToScVal(BigInt(reportId), { type: 'u64' }),
        Address.fromString(challengerAddress).toScVal(),
        nativeToScVal(dto.counterEvidenceHash, { type: 'string' }),
        nativeToScVal(dto.reason, { type: 'string' }),
      ],
      dto.nonce,
    );

    return {
      reportId,
      challengerAddress,
      reason: dto.reason,
      counterEvidenceHash: dto.counterEvidenceHash,
      resolved: false,
      createdAt: new Date().toISOString(),
    };
  }

  async registerProvider(dto: RegisterProviderDto): Promise<ProviderResponse> {
    const adminSecret = this.getAdminSecret();
    const adminAddress = this.stellarService.getKeypairFromSecret(adminSecret).publicKey();

    await this.contractService.invokeContractMethod(
      ORACLE_CONSUMER(), 'register_provider', adminSecret,
      [
        Address.fromString(dto.providerAddress).toScVal(),
        nativeToScVal(dto.methodology, { type: 'symbol' }),
      ],
      await this.getNonce(adminAddress),
    );

    return {
      providerAddress: dto.providerAddress,
      methodology: dto.methodology,
      name: `Oracle ${dto.providerAddress.slice(0, 6)}`,
      active: true,
      registeredAt: new Date().toISOString(),
    };
  }

  async listProviders(): Promise<ProviderResponse[]> {
    const cacheKey = 'oracle:providers';
    const cached = await this.redis.get(cacheKey);
    if (cached) return JSON.parse(cached);

    const providers: ProviderResponse[] = [];
    let index = 1;

    while (true) {
      try {
        const providerScVal = await this.contractService.simulateCall({
          contractAddress: ORACLE_CONSUMER(),
          method: 'get_provider',
          args: [nativeToScVal(BigInt(index), { type: 'u64' })],
        });
        const data = scValToNative(providerScVal) as any[];
        providers.push({
          providerAddress: data[0] as string,
          methodology: data[1] as string,
          name: data[2] as string,
          active: data[3] as boolean,
          registeredAt: new Date(Number(data[4]) * 1000).toISOString(),
        });
        index++;
      } catch {
        break;
      }
    }

    await this.redis.setEx(cacheKey, 120, JSON.stringify(providers));
    return providers;
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
