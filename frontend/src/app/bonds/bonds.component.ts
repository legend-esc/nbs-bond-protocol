import { Component, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';

@Component({
  selector: 'app-bonds',
  standalone: true,
  imports: [CommonModule, RouterModule],
  template: `<p>bonds works!</p>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BondsComponent {}
