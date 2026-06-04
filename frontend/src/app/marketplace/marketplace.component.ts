import { Component, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';

@Component({
  selector: 'app-marketplace',
  standalone: true,
  imports: [CommonModule, RouterModule],
  template: `<p>marketplace works!</p>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MarketplaceComponent {}
