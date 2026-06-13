import { Component, input, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';

const SIZE_MAP: Record<string, string> = {
  sm: '16px',
  md: '32px',
  lg: '48px',
};

@Component({
  selector: 'app-loading-spinner',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="spinner" [style.width]="sizePx()" [style.height]="sizePx()"></div>
  `,
  styles: [`
    .spinner { border: 3px solid #e5e7eb; border-top-color: #1a1a2e; border-radius: 50%; animation: spin 0.6s linear infinite; display: inline-block; }
    @keyframes spin { to { transform: rotate(360deg); } }
  `],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class LoadingSpinnerComponent {
  readonly size = input<'sm' | 'md' | 'lg'>('md');

  sizePx(): string {
    return SIZE_MAP[this.size()] || SIZE_MAP['md'];
  }
}
