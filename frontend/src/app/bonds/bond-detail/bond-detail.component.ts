import { Component, inject, OnInit, ChangeDetectionStrategy, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, ActivatedRoute } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { ApiService } from '../../shared/services/api.service';
import { StatusBadgeComponent } from '../../shared/components/status-badge/status-badge.component';
import { LoadingSpinnerComponent } from '../../shared/components/loading-spinner/loading-spinner.component';
import { Bond } from '../../shared/interfaces/bond.interface';

@Component({
  selector: 'app-bond-detail',
  standalone: true,
  imports: [CommonModule, RouterModule, FormsModule, StatusBadgeComponent, LoadingSpinnerComponent],
  template: `
    <div class="detail-page">
      <a class="back-link" routerLink="/bonds">← Back to Bonds</a>

      @if (bond(); as b) {
        <div class="detail-grid">
          <div class="detail-card main">
            <div class="detail-header">
              <h1 class="detail-title">Bond #{{ b.id }}</h1>
              <app-status-badge [status]="b.status" variant="bond" />
            </div>

            <div class="detail-body">
              <div class="detail-field">
                <span class="field-label">Project ID</span>
                <span class="field-value mono">{{ b.projectId }}</span>
              </div>
              <div class="detail-field">
                <span class="field-label">Face Value</span>
                <span class="field-value">{{ b.faceValue | number }}</span>
              </div>
              <div class="detail-field">
                <span class="field-label">Credit Type</span>
                <span class="field-value">{{ b.creditType }}</span>
              </div>
              <div class="detail-field">
                <span class="field-label">Maturity Date</span>
                <span class="field-value">{{ b.maturityDate | date }}</span>
              </div>
              <div class="detail-field">
                <span class="field-label">Total Supply</span>
                <span class="field-value">{{ b.totalSupply | number }}</span>
              </div>
              <div class="detail-field">
                <span class="field-label">Created</span>
                <span class="field-value">{{ b.createdAt | date }}</span>
              </div>
            </div>

            <div class="coupon-section">
              <h3 class="section-title">Coupon Schedule ({{ b.couponSchedule.length }} payments)</h3>
              <ul class="coupon-list">
                @for (ts of b.couponSchedule; track ts; let i = $index) {
                  <li class="coupon-item">
                    <span class="coupon-index">Period {{ i + 1 }}</span>
                    <span class="coupon-date">{{ ts | date }}</span>
                  </li>
                }
              </ul>
            </div>
          </div>

          <div class="detail-card sidebar">
            <h3 class="section-title">Subscription Progress</h3>
            <div class="progress-bar">
              <div class="progress-fill" [style.width.%]="subscribeProgress()"></div>
            </div>
            <div class="progress-text">
              {{ b.totalSubscribed | number }} / {{ b.totalSupply | number }}
              ({{ subscribeProgress() }}%)
            </div>

            <div class="subscribe-section">
              <h3 class="section-title">Subscribe</h3>
              @if (b.status !== 'Active') {
                <p class="status-notice">This bond is {{ b.status }} and is not accepting new subscriptions.</p>
              } @else {
                <div class="subscribe-form">
                  <label class="form-label" for="amount">Amount</label>
                  <input
                    id="amount"
                    type="number"
                    class="form-input"
                    [(ngModel)]="subscribeAmount"
                    placeholder="Enter amount"
                    min="1"
                  />
                  <button
                    class="btn btn-primary subscribe-btn"
                    [disabled]="!subscribeAmount || subscribeAmount < 1 || subscribeSubmitting()"
                    (click)="onSubscribe()"
                  >
                    {{ subscribeSubmitting() ? 'Subscribing...' : 'Subscribe' }}
                  </button>
                  @if (subscribeSuccess()) {
                    <div class="success-msg">Subscribed! Tx: {{ subscribeTx() }}</div>
                  }
                  @if (subscribeError()) {
                    <div class="error-msg">{{ subscribeError() }}</div>
                  }
                </div>
              }
            </div>

            <div class="marketplace-link">
              <a class="btn btn-outline" [routerLink]="['/marketplace']" [queryParams]="{ bondId: b.id }">
                View on Marketplace
              </a>
            </div>
          </div>
        </div>
      } @else if (loading()) {
        <div class="loading-section"><app-loading-spinner size="lg" /></div>
      } @else if (error()) {
        <div class="error-card">{{ error() }}</div>
      }
    </div>
  `,
  styles: [`
    .detail-page { max-width: 1200px; }
    .back-link { display: inline-block; margin-bottom: 24px; color: #3b82f6; text-decoration: none; font-size: 0.875rem; }
    .back-link:hover { text-decoration: underline; }
    .loading-section { display: flex; justify-content: center; padding: 48px 0; }
    .error-card { background: #fef2f2; color: #ef4444; padding: 24px; border-radius: 12px; text-align: center; }
    .detail-grid { display: grid; grid-template-columns: 1fr 360px; gap: 24px; }
    .detail-card { background: #fff; border-radius: 12px; padding: 32px; box-shadow: 0 1px 4px rgba(0,0,0,0.08); }
    .detail-card.sidebar { padding: 24px; }
    .detail-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
    .detail-title { font-size: 1.5rem; font-weight: 700; }
    .detail-body { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
    .detail-field { display: flex; flex-direction: column; }
    .field-label { font-size: 0.75rem; color: #6b7280; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 4px; }
    .field-value { font-size: 0.9375rem; color: #1a1a2e; }
    .field-value.mono { font-family: monospace; font-size: 0.8125rem; word-break: break-all; }
    .section-title { font-size: 1rem; font-weight: 600; margin-bottom: 12px; }
    .coupon-section { margin-top: 24px; padding-top: 20px; border-top: 1px solid #e5e7eb; }
    .coupon-list { list-style: none; padding: 0; display: flex; flex-direction: column; gap: 8px; }
    .coupon-item { display: flex; justify-content: space-between; padding: 8px 12px; background: #f9fafb; border-radius: 6px; font-size: 0.8125rem; }
    .coupon-index { color: #6b7280; }
    .coupon-date { font-weight: 500; }
    .progress-bar { height: 8px; background: #e5e7eb; border-radius: 4px; overflow: hidden; margin-bottom: 8px; }
    .progress-fill { height: 100%; background: #22c55e; border-radius: 4px; transition: width 0.3s; }
    .progress-text { font-size: 0.8125rem; color: #6b7280; margin-bottom: 20px; }
    .subscribe-section { margin-top: 20px; padding-top: 20px; border-top: 1px solid #e5e7eb; }
    .subscribe-form { display: flex; flex-direction: column; gap: 12px; }
    .form-label { font-size: 0.8125rem; font-weight: 600; color: #1a1a2e; }
    .form-input { padding: 10px 12px; border: 1px solid #d1d5db; border-radius: 8px; font-size: 0.875rem; outline: none; }
    .form-input:focus { border-color: #3b82f6; box-shadow: 0 0 0 2px rgba(59,130,246,0.15); }
    .status-notice { font-size: 0.8125rem; color: #6b7280; padding: 8px 0; }
    .btn { padding: 10px 20px; border-radius: 8px; font-size: 0.875rem; font-weight: 500; cursor: pointer; border: none; text-decoration: none; display: inline-block; text-align: center; }
    .btn-primary { background: #1a1a2e; color: #fff; }
    .btn-primary:hover:not(:disabled) { background: #2a2a4e; }
    .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
    .btn-outline { background: #fff; color: #1a1a2e; border: 1px solid #d1d5db; width: 100%; }
    .btn-outline:hover { background: #f0f2f5; }
    .subscribe-btn { width: 100%; }
    .success-msg { font-size: 0.8125rem; color: #22c55e; word-break: break-all; padding: 8px; background: #f0fdf4; border-radius: 6px; }
    .error-msg { font-size: 0.8125rem; color: #ef4444; padding: 8px; background: #fef2f2; border-radius: 6px; }
    .marketplace-link { margin-top: 20px; }
  `],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BondDetailComponent implements OnInit {
  private readonly route = inject(ActivatedRoute);
  private readonly apiService = inject(ApiService);

  readonly bond = signal<Bond | null>(null);
  readonly loading = signal(true);
  readonly error = signal('');
  readonly subscribeSubmitting = signal(false);
  readonly subscribeSuccess = signal(false);
  readonly subscribeTx = signal('');
  readonly subscribeError = signal('');

  subscribeAmount = 0;

  subscribeProgress(): number {
    const b = this.bond();
    if (!b || b.totalSupply === 0) return 0;
    return Math.round((b.totalSubscribed / b.totalSupply) * 100);
  }

  ngOnInit(): void {
    const id = Number(this.route.snapshot.paramMap.get('id'));
    if (!id) {
      this.error.set('Invalid bond ID');
      this.loading.set(false);
      return;
    }
    this.apiService.getBond(id).subscribe({
      next: (bond) => {
        this.bond.set(bond);
        this.loading.set(false);
      },
      error: (err) => {
        this.error.set(err.status === 404 ? 'Bond not found' : 'Failed to load bond');
        this.loading.set(false);
      },
    });
  }

  onSubscribe(): void {
    const b = this.bond();
    if (!b || !this.subscribeAmount || this.subscribeAmount < 1) return;
    this.subscribeSubmitting.set(true);
    this.subscribeSuccess.set(false);
    this.subscribeError.set('');

    this.apiService.subscribeToBond(b.id, this.subscribeAmount, Date.now()).subscribe({
      next: (res) => {
        this.subscribeSuccess.set(true);
        this.subscribeTx.set(res.transactionHash);
        this.subscribeSubmitting.set(false);
        this.apiService.getBond(b.id).subscribe({
          next: (updated) => this.bond.set(updated),
        });
      },
      error: (err) => {
        this.subscribeError.set(err.error?.detail || err.message || 'Subscription failed');
        this.subscribeSubmitting.set(false);
      },
    });
  }
}
