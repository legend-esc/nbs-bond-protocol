import axios from 'axios';

interface VerraProject {
  id: string;
  name: string;
  registry: string;
  methodology: string;
  status: string;
  credits_issued: number;
  credits_retired: number;
  last_update: string;
}

const VVC_REGISTRY_URL = 'https://registry.verra.org/api/v1/projects';

export async function fetchVerraProject(projectId: string): Promise<VerraProject> {
  const { data } = await axios.get(`${VVC_REGISTRY_URL}/${projectId}`);
  return {
    id: data.id,
    name: data.name,
    registry: 'VERRA-VCS',
    methodology: data.methodology,
    status: data.status,
    credits_issued: parseInt(data.creditsIssued, 10),
    credits_retired: parseInt(data.creditsRetired, 10),
    last_update: data.lastModifiedDate,
  };
}

export async function verifyVerraReport(projectId: string, periodStart: string, periodEnd: string): Promise<boolean> {
  // In production, fetch and validate monitoring reports from Verra registry
  return true;
}
