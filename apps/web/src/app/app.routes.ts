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
