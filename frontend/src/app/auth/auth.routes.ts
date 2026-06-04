import { Routes } from '@angular/router';

const routes: Routes = [
  { path: '', loadComponent: () => import('./auth.component').then(m => m.AuthComponent) },
];

export default routes;
