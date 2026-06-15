import { Component, inject, OnInit, ChangeDetectionStrategy, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Router, ActivatedRoute } from '@angular/router';
import { FormBuilder, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
import { ApiService } from '../../shared/services/api.service';
import { Bond } from '../../shared/interfaces/bond.interface';

@Component({
  selector: 'app-marketplace-sell',
  standalone: true,
  imports: [CommonModule, RouterModule, ReactiveFormsModule],
  template: `
    <div class="sell-page">
      <a class="back-link" routerLink="/marketplace">← Back to Marketplace</a>
      <h1 class="page-title">List Tokens for Sale</h1>

      @if (error()) {
        <div class="error-banner">{{ error() }}</div>
      }

      <form class="sell-form" [formGroup]="form" (ngSubmit)="onSubmit()">
        <div class="form-group">
          <label class="form-label" for="bondId">Bond</label>
          <select id="bondId" class="form-select" formControlName="bondId">
            <option [ngValue]="null" disabled>Select a bond</option>
            @for (bond of bonds(); track bond.id) {
              <option [ngValue]="bond.id">Bond #{{ bond.id }} — {{ bond.creditType }}</option>
            }
          </select>
          @if (form.get('bondId')?.invalid && form.get('bondId')?.touched) {
            <span class="form-error">Select a bond</span>
          }
        </div>

        <div class="form-row">
          <div class="form-group">
            <label class="form-label" for="amount">Amount</label>
            <input id="amount" type="number" class="form-input" formControlName="amount" placeholder="100" />
            @if (form.get('amount')?.invalid && form.get('amount')?.touched) {
              <span class="form-error">Enter a positive amount</span>
            }
          </div>
          <div class="form-group">
            <label class="form-label" for="pricePerToken">Price per Token</label>
            <input id="pricePerToken" type="number" class="form-input" formControlName="pricePerToken" placeholder="10.50" step="0.01" />
            @if (form.get('pricePerToken')?.invalid && form.get('pricePerToken')?.touched) {
              <span class="form-error">Enter a positive price</span>
            }
          </div>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label class="form-label" for="quoteAsset">Quote Asset</label>
            <select id="quoteAsset" class="form-select" formControlName="quoteAsset">
              <option value="USDC">USDC</option>
              <option value="XLM">XLM</option>
            </select>
          </div>
          <div class="form-group">
            <label class="form-label" for="expiresAfterSeconds">Expires After (seconds)</label>
            <input id="expiresAfterSeconds" type="number" class="form-input" formControlName="expiresAfterSeconds" placeholder="604800 (7 days)" />
          </div>
        </div>

        <input type="hidden" formControlName="nonce" />

        <div class="form-actions">
          <a class="btn btn-outline" routerLink="/marketplace">Cancel</a>
          <button type="submit" class="btn btn-primary" [disabled]="form.invalid || submitting()">
            {{ submitting() ? 'Listing...' : 'List for Sale' }}
          </button>
        </div>
      </form>
    </div>
  `,
  styles: [`
    .sell-page { max-width: 640px; }
    .back-link { display: inline-block; margin-bottom: 16px; color: #3b82f6; text-decoration: none; font-size: 0.875rem; }
    .back-link:hover { text-decoration: underline; }
    .page-title { font-size: 1.5rem; font-weight: 700; margin-bottom: 24px; }
    .error-banner { background: #fef2f2; color: #ef4444; padding: 12px 16px; border-radius: 8px; margin-bottom: 16px; font-size: 0.875rem; }
    .sell-form { background: #fff; border-radius: 12px; padding: 32px; box-shadow: 0 1px 4px rgba(0,0,0,0.08); }
    .form-group { display: flex; flex-direction: column; margin-bottom: 20px; flex: 1; }
    .form-label { font-size: 0.8125rem; font-weight: 600; color: #1a1a2e; margin-bottom: 6px; }
    .form-input, .form-select { padding: 10px 12px; border: 1px solid #d1d5db; border-radius: 8px; font-size: 0.875rem; outline: none; transition: border-color 0.15s; background: #fff; }
    .form-input:focus, .form-select:focus { border-color: #3b82f6; box-shadow: 0 0 0 2px rgba(59,130,246,0.15); }
    .form-error { font-size: 0.75rem; color: #ef4444; margin-top: 4px; }
    .form-row { display: flex; gap: 16px; }
    .form-actions { display: flex; gap: 12px; justify-content: flex-end; margin-top: 24px; padding-top: 16px; border-top: 1px solid #e5e7eb; }
    .btn { padding: 10px 20px; border-radius: 8px; font-size: 0.875rem; font-weight: 500; cursor: pointer; border: none; text-decoration: none; display: inline-block; }
    .btn-primary { background: #1a1a2e; color: #fff; }
    .btn-primary:hover:not(:disabled) { background: #2a2a4e; }
    .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
    .btn-outline { background: #fff; color: #1a1a2e; border: 1px solid #d1d5db; }
    .btn-outline:hover { background: #f0f2f5; }
  `],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MarketplaceSellComponent implements OnInit {
  private readonly fb = inject(FormBuilder);
  private readonly apiService = inject(ApiService);
  private readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);

  readonly bonds = signal<Bond[]>([]);
  readonly submitting = signal(false);
  readonly error = signal('');

  form: FormGroup = this.fb.group({
    bondId: [null, Validators.required],
    amount: [null, [Validators.required, Validators.min(1)]],
    pricePerToken: [null, [Validators.required, Validators.min(0.01)]],
    quoteAsset: ['USDC', Validators.required],
    expiresAfterSeconds: [604800],
    nonce: [Date.now()],
  });

  ngOnInit(): void {
    const bondIdParam = this.route.snapshot.queryParamMap.get('bondId');
    if (bondIdParam) {
      this.form.patchValue({ bondId: Number(bondIdParam) });
    }
    this.apiService.getBonds(1, 100).subscribe({
      next: (res) => this.bonds.set(res.data),
    });
  }

  onSubmit(): void {
    if (this.form.invalid) return;
    this.submitting.set(true);
    this.error.set('');

    this.form.patchValue({ nonce: Date.now() });
    const formValue = { ...this.form.value };
    if (!formValue.expiresAfterSeconds) delete formValue.expiresAfterSeconds;

    this.apiService.listBondTokens(formValue).subscribe({
      next: () => {
        this.router.navigate(['/marketplace']);
      },
      error: (err) => {
        this.error.set(err.error?.detail || err.message || 'Failed to list tokens');
        this.submitting.set(false);
      },
    });
  }
}
