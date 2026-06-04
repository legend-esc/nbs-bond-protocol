import { Routes } from '@angular/router';

const routes: Routes = [
  { path: '', loadComponent: () => import('./projects.component').then(m => m.ProjectsComponent) },
];

export default routes;
