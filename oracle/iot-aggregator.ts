interface IoTSensorReading {
  deviceId: string;
  timestamp: string;
  metrics: {
    soilMoisture: number | null;
    temperature: number | null;
    humidity: number | null;
    co2: number | null;
  };
}

export function aggregateSensorReadings(readings: IoTSensorReading[]): {
  avgSoilMoisture: number;
  avgTemperature: number;
  avgHumidity: number;
  sampleCount: number;
  deviceCount: number;
} {
  const valid = readings.filter(r => r.metrics.soilMoisture !== null);
  return {
    avgSoilMoisture: valid.reduce((s, r) => s + r.metrics.soilMoisture!, 0) / valid.length,
    avgTemperature: valid.reduce((s, r) => s + r.metrics.temperature!, 0) / valid.length,
    avgHumidity: valid.reduce((s, r) => s + r.metrics.humidity!, 0) / valid.length,
    sampleCount: readings.length,
    deviceCount: new Set(readings.map(r => r.deviceId)).size,
  };
}

export function validateIotReading(reading: IoTSensorReading): boolean {
  const m = reading.metrics;
  if (m.soilMoisture !== null && (m.soilMoisture < 0 || m.soilMoisture > 100)) return false;
  if (m.temperature !== null && (m.temperature < -50 || m.temperature > 60)) return false;
  if (m.humidity !== null && (m.humidity < 0 || m.humidity > 100)) return false;
  return true;
}
