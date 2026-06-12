export interface OracleProviderAdapter {
  readonly name: string;
  readonly methodology: string;

  fetchMeasurement(projectId: string): Promise<MeasurementData>;
}

export interface MeasurementData {
  projectId: string;
  periodStart: Date;
  periodEnd: Date;
  carbonSequesteredKg: number;
  confidence: number;
  evidenceHashes: string[];
}
