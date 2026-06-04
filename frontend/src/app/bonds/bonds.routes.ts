import { Routes } from '@angular/router';

const routes: Routes = [
  { path: '', loadComponent: () => import('./bonds.component').then(m => m.BondsComponent) },
];

export default routes;
