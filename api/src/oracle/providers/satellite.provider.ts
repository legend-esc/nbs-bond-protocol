import { Injectable } from '@nestjs/common';
import { OracleProviderAdapter, MeasurementData } from './provider.interface';

@Injectable()
export class SatelliteProvider implements OracleProviderAdapter {
  readonly name = 'SatelliteProcessor';
  readonly methodology = 'REMOTE-SENSING';

  async fetchMeasurement(projectId: string): Promise<MeasurementData> {
    return {
      projectId,
      periodStart: new Date('2025-01-01'),
      periodEnd: new Date('2025-03-31'),
      carbonSequesteredKg: 42000,
      confidence: 0.88,
      evidenceHashes: ['QmSatMockHash456'],
    };
  }
}
