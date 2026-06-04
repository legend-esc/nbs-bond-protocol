import { Component, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';

@Component({
  selector: 'app-projects',
  standalone: true,
  imports: [CommonModule, RouterModule],
  template: `<p>projects works!</p>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ProjectsComponent {}
