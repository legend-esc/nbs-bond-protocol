import { Component, inject, OnInit, ChangeDetectionStrategy, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Router } from '@angular/router';
import { ApiService } from '../../shared/services/api.service';
import { BondCardComponent } from '../../shared/components/bond-card/bond-card.component';
import { LoadingSpinnerComponent } from '../../shared/components/loading-spinner/loading-spinner.component';
import { Bond } from '../../shared/interfaces/bond.interface';

@Component({
  selector: 'app-bonds-list',
  standalone: true,
  imports: [CommonModule, RouterModule, BondCardComponent, LoadingSpinnerComponent],
  template: `
    <div class="bonds-page">
      <div class="page-header">
        <h1 class="page-title">Bonds</h1>
      </div>

      @if (error()) {
        <div class="error-banner">{{ error() }}</div>
      }

      @if (loading()) {
        <div class="loading-section"><app-loading-spinner size="lg" /></div>
      } @else if (bonds().length === 0) {
        <div class="empty-section">
          <p>No bonds available yet.</p>
        </div>
      } @else {
        <div class="card-grid">
          @for (bond of bonds(); track bond.id) {
            <app-bond-card [bond]="bond" (subscribe)="onSubscribe(bond.id)" />
          }
        </div>

        <div class="pagination">
          <button class="btn btn-outline" [disabled]="page() <= 1" (click)="prevPage()">Previous</button>
          <span class="page-info">Page {{ page() }} of {{ totalPages() }}</span>
          <button class="btn btn-outline" [disabled]="page() >= totalPages()" (click)="nextPage()">Next</button>
        </div>
      }
    </div>
  `,
  styles: [`
    .bonds-page { max-width: 1200px; }
    .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
    .page-title { font-size: 1.5rem; font-weight: 700; }
    .error-banner { background: #fef2f2; color: #ef4444; padding: 12px 16px; border-radius: 8px; margin-bottom: 16px; font-size: 0.875rem; }
    .loading-section { display: flex; justify-content: center; padding: 48px 0; }
    .empty-section { text-align: center; padding: 48px 0; color: #6b7280; }
    .card-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 16px; }
    .pagination { display: flex; justify-content: center; align-items: center; gap: 16px; margin-top: 32px; }
    .page-info { font-size: 0.875rem; color: #6b7280; }
    .btn { padding: 8px 16px; border-radius: 8px; font-size: 0.875rem; font-weight: 500; cursor: pointer; border: none; text-decoration: none; display: inline-block; }
    .btn-outline { background: #fff; color: #1a1a2e; border: 1px solid #d1d5db; }
    .btn-outline:hover:not(:disabled) { background: #f0f2f5; }
    .btn-outline:disabled { opacity: 0.5; cursor: not-allowed; }
  `],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BondsListComponent implements OnInit {
  private readonly apiService = inject(ApiService);
  private readonly router = inject(Router);

  readonly bonds = signal<Bond[]>([]);
  readonly loading = signal(true);
  readonly error = signal('');
  readonly page = signal(1);
  readonly totalPages = signal(1);
  private readonly limit = 12;

  ngOnInit(): void {
    this.loadBonds();
  }

  private loadBonds(): void {
    this.loading.set(true);
    this.error.set('');
    this.apiService.getBonds(this.page(), this.limit).subscribe({
      next: (res) => {
        this.bonds.set(res.data);
        this.totalPages.set(res.meta.totalPages);
        this.loading.set(false);
      },
      error: () => {
        this.error.set('Failed to load bonds');
        this.loading.set(false);
      },
    });
  }

  onSubscribe(bondId: number): void {
    this.router.navigate(['/bonds', bondId]);
  }

  prevPage(): void {
    if (this.page() > 1) {
      this.page.update(p => p - 1);
      this.loadBonds();
    }
  }

  nextPage(): void {
    if (this.page() < this.totalPages()) {
      this.page.update(p => p + 1);
      this.loadBonds();
    }
  }
}
