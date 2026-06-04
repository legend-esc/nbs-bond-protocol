import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: '', loadComponent: () => import('./dashboard/dashboard.component').then(m => m.DashboardComponent) },
  { path: 'projects', loadChildren: () => import('./projects/projects.routes') },
  { path: 'marketplace', loadChildren: () => import('./marketplace/marketplace.routes') },
  { path: 'bonds', loadChildren: () => import('./bonds/bonds.routes') },
  { path: 'auth', loadChildren: () => import('./auth/auth.routes') },
];
