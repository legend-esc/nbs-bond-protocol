import axios from 'axios';

interface SatelliteImagery {
  sceneId: string;
  captureDate: string;
  cloudCover: number;
  ndvi: number;
  ndwi: number;
  source: 'sentinel-2' | 'landsat-8' | 'planet';
}

export async function fetchNdviData(bbox: string, startDate: string, endDate: string): Promise<SatelliteImagery[]> {
  // Query Sentinel Hub / Planet API for NDVI data within bounding box
  return [];
}

export function calculateBiomassChange(ndviBaseline: number, ndviCurrent: number): number {
  const fractionalCover = (ndviCurrent - ndviBaseline) / (1 - ndviBaseline);
  return Math.max(0, fractionalCover);
}

export function estimateCarbonSequestration(areaHa: number, ndviChange: number): number {
  // Simplified IPCC Tier 1: biomass carbon = area × NDVI-derived factor × conversion
  const defaultBiomassFactor = 3.67; // tC/ha per unit NDVI
  return areaHa * ndviChange * defaultBiomassFactor;
}
