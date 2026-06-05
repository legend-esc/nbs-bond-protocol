import axios from 'axios';

interface IpfsConfig {
  apiUrl: string;
  apiKey: string;
  secretKey: string;
  gateway: string;
}

interface IpfsUploadResult {
  hash: string;
  gatewayUrl: string;
  pinSize: number;
  timestamp: string;
}

export async function uploadToIpfs(
  data: Record<string, unknown>,
  config: IpfsConfig,
): Promise<IpfsUploadResult> {
  const response = await axios.post(
    `${config.apiUrl}/pinning/pinJSONToIPFS`,
    { pinataContent: data, pinataMetadata: { name: `nbs-${Date.now()}` } },
    {
      headers: {
        pinata_api_key: config.apiKey,
        pinata_secret_api_key: config.secretKey,
      },
    },
  );

  return {
    hash: response.data.IpfsHash,
    gatewayUrl: `${config.gateway}${response.data.IpfsHash}`,
    pinSize: response.data.PinSize,
    timestamp: new Date().toISOString(),
  };
}
