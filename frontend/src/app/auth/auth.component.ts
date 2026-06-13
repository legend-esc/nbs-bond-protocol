import { Component, inject, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router } from '@angular/router';
import { WalletService } from './wallet.service';
import { AuthService } from './auth.service';

@Component({
  selector: 'app-auth',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="auth-page">
      <div class="auth-card">
        <h1>NbS Bond Protocol</h1>
        <p class="subtitle">Sign in with your Stellar wallet</p>

        <ng-container *ngIf="!walletService.isConnected()">
          <button class="btn btn-primary" (click)="walletService.connect()" [disabled]="walletService.isConnecting()">
            {{ walletService.isConnecting() ? 'Connecting...' : 'Connect Wallet' }}
          </button>
        </ng-container>

        <ng-container *ngIf="walletService.isConnected()">
          <div class="wallet-address">
            Connected: {{ walletService.address()?.slice(0, 6) }}...{{ walletService.address()?.slice(-4) }}
          </div>
          <button class="btn btn-primary" (click)="signIn()">
            Sign In with Stellar
          </button>
        </ng-container>

        <p *ngIf="error" class="error">{{ error }}</p>
      </div>
    </div>
  `,
  styles: [`
    .auth-page { display: flex; align-items: center; justify-content: center; min-height: 100vh; }
    .auth-card { background: #fff; border-radius: 12px; padding: 40px; text-align: center; box-shadow: 0 2px 8px rgba(0,0,0,0.1); max-width: 400px; width: 100%; }
    h1 { font-size: 1.5rem; margin-bottom: 8px; }
    .subtitle { color: #666; margin-bottom: 24px; }
    .wallet-address { background: #f0f2f5; padding: 8px 16px; border-radius: 8px; margin-bottom: 16px; font-family: monospace; font-size: 0.875rem; }
    .btn { padding: 12px 24px; border: none; border-radius: 8px; font-size: 1rem; cursor: pointer; width: 100%; }
    .btn-primary { background: #1a1a2e; color: #fff; }
    .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
    .error { color: #ef4444; margin-top: 12px; font-size: 0.875rem; }
  `],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AuthComponent {
  private readonly router = inject(Router);
  readonly walletService = inject(WalletService);
  readonly authService = inject(AuthService);

  error = '';

  async signIn(): Promise<void> {
    this.error = '';
    try {
      await this.authService.login();
      await this.router.navigate(['/dashboard']);
    } catch (e: any) {
      this.error = e.message || 'Sign in failed';
    }
  }
}
