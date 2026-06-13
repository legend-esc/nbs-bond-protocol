import { Component, inject, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { WalletService } from '../../../auth/wallet.service';

@Component({
  selector: 'app-wallet-button',
  standalone: true,
  imports: [CommonModule],
  template: `
    <button *ngIf="!walletService.isConnected()" (click)="connect()" class="wallet-btn">
      Connect Wallet
    </button>
    <button *ngIf="walletService.isConnected()" class="wallet-btn connected">
      {{ walletService.address()?.slice(0, 6) }}...{{ walletService.address()?.slice(-4) }}
    </button>
  `,
  styles: [`
    .wallet-btn { padding: 8px 16px; border: 1px solid #1a1a2e; border-radius: 8px; background: transparent; color: #1a1a2e; cursor: pointer; font-size: 0.875rem; font-weight: 500; }
    .wallet-btn:hover { background: #1a1a2e; color: #fff; }
    .wallet-btn.connected { background: #1a1a2e; color: #fff; font-family: monospace; }
  `],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WalletButtonComponent {
  readonly walletService = inject(WalletService);

  async connect(): Promise<void> {
    await this.walletService.connect();
  }
}
