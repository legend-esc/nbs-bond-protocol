import { Component, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';

@Component({
  selector: 'app-auth',
  standalone: true,
  imports: [CommonModule, RouterModule],
  template: `<p>auth works!</p>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AuthComponent {}
