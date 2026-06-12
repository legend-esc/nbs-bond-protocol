import { Injectable } from '@nestjs/common';
import { OracleProviderAdapter, MeasurementData } from './provider.interface';

@Injectable()
export class VerraProvider implements OracleProviderAdapter {
  readonly name = 'Verra';
  readonly methodology = 'VERRA-VCS';

  async fetchMeasurement(projectId: string): Promise<MeasurementData> {
    return {
      projectId,
      periodStart: new Date('2025-01-01'),
      periodEnd: new Date('2025-03-31'),
      carbonSequesteredKg: 50000,
      confidence: 0.95,
      evidenceHashes: ['QmMockHash123'],
    };
  }
}
