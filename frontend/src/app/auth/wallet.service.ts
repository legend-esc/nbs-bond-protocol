import { Injectable, signal } from '@angular/core';
import { isConnected, getAddress, signTransaction } from '@stellar/freighter-api';

@Injectable({ providedIn: 'root' })
export class WalletService {
  readonly address = signal<string | null>(null);
  readonly isConnected = signal(false);
  readonly isConnecting = signal(false);

  async connect(): Promise<void> {
    this.isConnecting.set(true);
    try {
      const connected = await isConnected();
      if (!connected.isConnected) {
        throw new Error('Freighter not detected');
      }
      const { address } = await getAddress();
      this.address.set(address);
      this.isConnected.set(true);
    } finally {
      this.isConnecting.set(false);
    }
  }

  async signChallenge(challenge: string): Promise<string> {
    const { signedTxXdr } = await signTransaction(challenge, {
      networkPassphrase: 'Test SDF Network ; September 2015',
    });
    return signedTxXdr;
  }

  disconnect(): void {
    this.address.set(null);
    this.isConnected.set(false);
  }
}
