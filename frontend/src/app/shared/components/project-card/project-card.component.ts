import { Component, input, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Project } from '../../interfaces/bond.interface';
import { StatusBadgeComponent } from '../status-badge/status-badge.component';

const COUNTRY_FLAGS: Record<string, string> = {
  US: '🇺🇸', UK: '🇬🇧', FR: '🇫🇷', DE: '🇩🇪', BR: '🇧🇷', IN: '🇮🇳',
  CN: '🇨🇳', JP: '🇯🇵', KE: '🇰🇪', CO: '🇨🇴', ID: '🇮🇩', MY: '🇲🇾',
  AU: '🇦🇺', CA: '🇨🇦', ZA: '🇿🇦', NG: '🇳🇬',
};

@Component({
  selector: 'app-project-card',
  standalone: true,
  imports: [CommonModule, StatusBadgeComponent],
  template: `
    <div class="project-card">
      <div class="project-header">
        <span class="project-name">{{ project().name }}</span>
        <app-status-badge [status]="project().status" variant="project" />
      </div>
      <div class="project-body">
        <div class="project-field">
          <span class="label">Methodology</span>
          <span class="value methodology">{{ project().methodology }}</span>
        </div>
        <div class="project-field">
          <span class="label">Country</span>
          <span class="value">{{ flag() }} {{ project().country }}</span>
        </div>
        <div class="project-field">
          <span class="label">Area</span>
          <span class="value">{{ project().totalAreaHa | number }} ha</span>
        </div>
        <div class="project-field">
          <span class="label">Carbon Estimate</span>
          <span class="value">{{ project().carbonSequestrationEstimate | number }} tCO₂e</span>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .project-card { background: #fff; border-radius: 12px; padding: 16px; box-shadow: 0 1px 4px rgba(0,0,0,0.08); }
    .project-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
    .project-name { font-weight: 600; font-size: 1rem; }
    .project-body { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
    .project-field { display: flex; flex-direction: column; }
    .label { font-size: 0.75rem; color: #6b7280; text-transform: uppercase; letter-spacing: 0.5px; }
    .value { font-size: 0.875rem; color: #1a1a2e; font-weight: 500; }
    .value.methodology { font-family: monospace; font-size: 0.8rem; }
  `],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ProjectCardComponent {
  readonly project = input.required<Project>();

  flag(): string {
    return COUNTRY_FLAGS[this.project().country] || '🌍';
  }
}
