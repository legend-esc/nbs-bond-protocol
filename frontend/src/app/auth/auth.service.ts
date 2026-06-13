import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import { WalletService } from './wallet.service';

@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly http = inject(HttpClient);
  private readonly walletService = inject(WalletService);

  readonly token = signal<string | null>(localStorage.getItem('nbs_token'));
  readonly isAuthenticated = computed(() => this.token() !== null);

  async login(): Promise<void> {
    const address = this.walletService.address();
    if (!address) throw new Error('Wallet not connected');

    const { challenge } = await firstValueFrom(
      this.http.post<{ challenge: string }>('/api/auth/challenge', { address }),
    );

    const signedChallenge = await this.walletService.signChallenge(challenge);

    const { accessToken } = await firstValueFrom(
      this.http.post<{ accessToken: string }>('/api/auth/verify', {
        address,
        signedChallenge,
        originalChallenge: challenge,
      }),
    );

    localStorage.setItem('nbs_token', accessToken);
    this.token.set(accessToken);
  }

  logout(): void {
    localStorage.removeItem('nbs_token');
    this.token.set(null);
    this.walletService.disconnect();
  }
}
