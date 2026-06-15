import { Routes } from '@angular/router';

const routes: Routes = [
  { path: '', loadComponent: () => import('./projects-list/projects-list.component').then(m => m.ProjectsListComponent) },
  { path: 'new', loadComponent: () => import('./project-create/project-create.component').then(m => m.ProjectCreateComponent) },
  { path: ':id', loadComponent: () => import('./project-detail/project-detail.component').then(m => m.ProjectDetailComponent) },
];

export default routes;
