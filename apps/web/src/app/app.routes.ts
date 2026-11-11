import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    title: 'Finanzas',
    loadComponent: () => import('./pantallas/inicio').then((m) => m.Inicio),
  },
  {
    path: 'quincena',
    title: 'Quincena · Finanzas',
    loadComponent: () => import('./pantallas/quincena').then((m) => m.Quincena),
  },
  {
    path: 'anotar',
    title: 'Anotar · Finanzas',
    loadComponent: () => import('./pantallas/anotar').then((m) => m.Anotar),
  },
  {
    path: 'mas',
    title: 'Más · Finanzas',
    loadComponent: () => import('./pantallas/mas').then((m) => m.Mas),
  },
  {
    path: 'mes',
    title: 'Mes · Finanzas',
    loadComponent: () => import('./escritorio/mes').then((m) => m.Mes),
  },
  {
    path: 'extras',
    title: 'Extras · Finanzas',
    loadComponent: () => import('./escritorio/extras').then((m) => m.Extras),
  },
  {
    path: 'gastos',
    title: 'Gastos · Finanzas',
    loadComponent: () => import('./escritorio/gastos').then((m) => m.Gastos),
  },
  {
    path: 'configuracion',
    title: 'Configuración · Finanzas',
    loadComponent: () => import('./pantallas/configuracion').then((m) => m.Configuracion),
  },
  {
    path: 'muestrario',
    title: 'Muestrario · Finanzas',
    loadComponent: () => import('./pantallas/muestrario').then((m) => m.Muestrario),
  },
  { path: '**', redirectTo: '' },
];
