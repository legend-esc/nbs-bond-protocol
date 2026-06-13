import { Component, input, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';

const STATUS_COLORS: Record<string, string> = {
  Active: '#22c55e',
  Pending: '#eab308',
  Matured: '#3b82f6',
  Defaulted: '#ef4444',
  Rejected: '#ef4444',
  Verified: '#22c55e',
  Approved: '#22c55e',
  Inactive: '#6b7280',
  Open: '#22c55e',
  Filled: '#3b82f6',
  Cancelled: '#6b7280',
  Expired: '#ef4444',
};

@Component({
  selector: 'app-status-badge',
  standalone: true,
  imports: [CommonModule],
  template: `
    <span class="status-badge" [style.background]="color()" [style.color]="'#fff'">
      {{ status() }}
    </span>
  `,
  styles: [`
    .status-badge { display: inline-block; padding: 2px 10px; border-radius: 12px; font-size: 0.75rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; }
  `],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class StatusBadgeComponent {
  readonly status = input.required<string>();
  readonly variant = input<'bond' | 'project' | 'report'>('bond');

  color(): string {
    return STATUS_COLORS[this.status()] || '#6b7280';
  }
}
