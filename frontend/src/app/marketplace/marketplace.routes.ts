import { Routes } from '@angular/router';

const routes: Routes = [
  { path: '', loadComponent: () => import('./marketplace.component').then(m => m.MarketplaceComponent) },
];

export default routes;
