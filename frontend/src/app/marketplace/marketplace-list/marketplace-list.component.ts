import { Component, inject, OnInit, ChangeDetectionStrategy, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, ActivatedRoute, Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { ApiService } from '../../shared/services/api.service';
import { StatusBadgeComponent } from '../../shared/components/status-badge/status-badge.component';
import { LoadingSpinnerComponent } from '../../shared/components/loading-spinner/loading-spinner.component';
import { Order, Bond } from '../../shared/interfaces/bond.interface';

@Component({
  selector: 'app-marketplace-list',
  standalone: true,
  imports: [CommonModule, RouterModule, FormsModule, StatusBadgeComponent, LoadingSpinnerComponent],
  template: `
    <div class="marketplace-page">
      <div class="page-header">
        <h1 class="page-title">Marketplace</h1>
        <a class="btn btn-primary" [routerLink]="['/marketplace/sell']" [queryParams]="{ bondId: filterBondId() }">
          List Tokens for Sale
        </a>
      </div>

      @if (error()) {
        <div class="error-banner">{{ error() }}</div>
      }

      <div class="filters">
        <label class="filter-label">
          Bond Filter
          <select class="filter-select" [ngModel]="filterBondId()" (ngModelChange)="onFilterChange($event)">
            <option [ngValue]="null">All Bonds</option>
            @for (bond of bonds(); track bond.id) {
              <option [ngValue]="bond.id">Bond #{{ bond.id }}</option>
            }
          </select>
        </label>
      </div>

      @if (loading()) {
        <div class="loading-section"><app-loading-spinner size="lg" /></div>
      } @else if (orders().length === 0) {
        <div class="empty-section">
          <p>No active orders. List your bond tokens for sale.</p>
        </div>
      } @else {
        <div class="orders-table-wrapper">
          <table class="orders-table">
            <thead>
              <tr>
                <th>ID</th>
                <th>Bond</th>
                <th>Seller</th>
                <th>Amount</th>
                <th>Price</th>
                <th>Asset</th>
                <th>Status</th>
                <th>Created</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              @for (order of orders(); track order.id) {
                <tr>
                  <td>{{ order.id }}</td>
                  <td>{{ order.bondId }}</td>
                  <td class="mono">{{ order.seller.slice(0, 8) }}...</td>
                  <td>{{ order.amount }}</td>
                  <td>{{ order.pricePerToken }}</td>
                  <td>{{ order.quoteAsset }}</td>
                  <td><app-status-badge [status]="order.status" variant="bond" /></td>
                  <td>{{ order.createdAt | date }}</td>
                  <td>
                    @if (order.status === 'Open' || order.status === 'PartiallyFilled') {
                      @if (buyOrderId() === order.id) {
                        <div class="buy-form">
                          <input type="number" class="buy-input" placeholder="Amount" [(ngModel)]="buyAmount" min="1" />
                          <input type="number" class="buy-input" placeholder="Max price" [(ngModel)]="buyMaxPrice" min="0.01" />
                          <div class="buy-actions">
                            <button class="btn btn-sm btn-primary" (click)="onBuy(order)" [disabled]="buySubmitting()">Confirm</button>
                            <button class="btn btn-sm btn-outline" (click)="cancelBuy()">Cancel</button>
                          </div>
                          @if (buyError()) {
                            <div class="error-msg">{{ buyError() }}</div>
                          }
                        </div>
                      } @else {
                        <button class="btn btn-sm btn-primary" (click)="openBuy(order)">Buy</button>
                      }
                    }
                  </td>
                </tr>
              }
            </tbody>
          </table>
        </div>
      }
    </div>
  `,
  styles: [`
    .marketplace-page { max-width: 1200px; }
    .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
    .page-title { font-size: 1.5rem; font-weight: 700; }
    .error-banner { background: #fef2f2; color: #ef4444; padding: 12px 16px; border-radius: 8px; margin-bottom: 16px; font-size: 0.875rem; }
    .filters { margin-bottom: 20px; }
    .filter-label { font-size: 0.8125rem; font-weight: 600; color: #1a1a2e; display: flex; align-items: center; gap: 8px; }
    .filter-select { padding: 8px 12px; border: 1px solid #d1d5db; border-radius: 8px; font-size: 0.875rem; outline: none; background: #fff; }
    .filter-select:focus { border-color: #3b82f6; }
    .loading-section { display: flex; justify-content: center; padding: 48px 0; }
    .empty-section { text-align: center; padding: 48px 0; color: #6b7280; }
    .orders-table-wrapper { overflow-x: auto; background: #fff; border-radius: 12px; box-shadow: 0 1px 4px rgba(0,0,0,0.08); }
    .orders-table { width: 100%; border-collapse: collapse; font-size: 0.875rem; }
    .orders-table th { text-align: left; padding: 12px 16px; font-weight: 600; color: #6b7280; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.5px; border-bottom: 1px solid #e5e7eb; background: #f9fafb; }
    .orders-table td { padding: 12px 16px; border-bottom: 1px solid #f0f2f5; }
    .orders-table tr:last-child td { border-bottom: none; }
    .mono { font-family: monospace; font-size: 0.8125rem; }
    .btn { padding: 8px 16px; border-radius: 8px; font-size: 0.875rem; font-weight: 500; cursor: pointer; border: none; text-decoration: none; display: inline-block; }
    .btn-sm { padding: 6px 12px; font-size: 0.8125rem; }
    .btn-primary { background: #1a1a2e; color: #fff; }
    .btn-primary:hover:not(:disabled) { background: #2a2a4e; }
    .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
    .btn-outline { background: #fff; color: #1a1a2e; border: 1px solid #d1d5db; }
    .btn-outline:hover { background: #f0f2f5; }
    .buy-form { display: flex; flex-direction: column; gap: 6px; min-width: 160px; }
    .buy-input { padding: 6px 8px; border: 1px solid #d1d5db; border-radius: 6px; font-size: 0.8125rem; outline: none; width: 100%; }
    .buy-input:focus { border-color: #3b82f6; }
    .buy-actions { display: flex; gap: 4px; }
    .error-msg { font-size: 0.75rem; color: #ef4444; }
  `],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MarketplaceListComponent implements OnInit {
  private readonly apiService = inject(ApiService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);

  readonly orders = signal<Order[]>([]);
  readonly bonds = signal<Bond[]>([]);
  readonly loading = signal(true);
  readonly error = signal('');
  readonly filterBondId = signal<number | null>(null);

  readonly buyOrderId = signal<number | null>(null);
  readonly buySubmitting = signal(false);
  readonly buyError = signal('');
  buyAmount = 0;
  buyMaxPrice = 0;

  ngOnInit(): void {
    const bondIdParam = this.route.snapshot.queryParamMap.get('bondId');
    if (bondIdParam) {
      this.filterBondId.set(Number(bondIdParam));
    }
    this.loadBonds();
    this.loadOrders();
  }

  private loadBonds(): void {
    this.apiService.getBonds(1, 100).subscribe({
      next: (res) => this.bonds.set(res.data),
    });
  }

  private loadOrders(): void {
    this.loading.set(true);
    this.error.set('');
    this.apiService.getOrders(this.filterBondId() ?? undefined).subscribe({
      next: (orders) => {
        this.orders.set(orders);
        this.loading.set(false);
      },
      error: () => {
        this.error.set('Failed to load orders');
        this.loading.set(false);
      },
    });
  }

  onFilterChange(bondId: number | null): void {
    this.filterBondId.set(bondId);
    this.router.navigate([], {
      relativeTo: this.route,
      queryParams: bondId ? { bondId } : {},
      queryParamsHandling: 'merge',
    });
    this.loadOrders();
  }

  openBuy(order: Order): void {
    this.buyOrderId.set(order.id);
    this.buyAmount = 0;
    this.buyMaxPrice = 0;
    this.buyError.set('');
  }

  cancelBuy(): void {
    this.buyOrderId.set(null);
  }

  onBuy(order: Order): void {
    if (!this.buyAmount || this.buyAmount < 1 || !this.buyMaxPrice || this.buyMaxPrice <= 0) return;
    this.buySubmitting.set(true);
    this.buyError.set('');

    this.apiService.buyBondTokens({
      orderId: order.id,
      amount: this.buyAmount,
      maxPrice: this.buyMaxPrice,
      nonce: Date.now(),
    }).subscribe({
      next: () => {
        this.buyOrderId.set(null);
        this.buySubmitting.set(false);
        this.loadOrders();
      },
      error: (err) => {
        this.buyError.set(err.error?.detail || err.message || 'Buy failed');
        this.buySubmitting.set(false);
      },
    });
  }
}
