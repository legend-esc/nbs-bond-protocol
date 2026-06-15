import { Routes } from '@angular/router';

const routes: Routes = [
  { path: '', loadComponent: () => import('./marketplace-list/marketplace-list.component').then(m => m.MarketplaceListComponent) },
  { path: 'sell', loadComponent: () => import('./marketplace-sell/marketplace-sell.component').then(m => m.MarketplaceSellComponent) },
];

export default routes;
