import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    title: 'Finanzas',
    loadComponent: () => import('./modulos/inicio/inicio').then((m) => m.Inicio),
  },
  {
    path: 'quincena',
    title: 'Quincena · Finanzas',
    loadComponent: () => import('./modulos/quincena/quincena').then((m) => m.Quincena),
  },
  {
    path: 'anotar',
    title: 'Anotar · Finanzas',
    loadComponent: () => import('./modulos/anotar/anotar').then((m) => m.Anotar),
  },
  {
    path: 'mas',
    title: 'Más · Finanzas',
    loadComponent: () => import('./modulos/mas/mas').then((m) => m.Mas),
  },
  {
    path: 'mes',
    title: 'Mes · Finanzas',
    loadComponent: () => import('./modulos/quincena/mes').then((m) => m.Mes),
  },
  {
    path: 'extras',
    title: 'Extras · Finanzas',
    loadComponent: () => import('./modulos/extras/extras').then((m) => m.Extras),
  },
  {
    path: 'gastos',
    title: 'Gastos · Finanzas',
    loadComponent: () => import('./modulos/gastos/gastos').then((m) => m.Gastos),
  },
  {
    path: 'configuracion',
    title: 'Configuración · Finanzas',
    loadComponent: () => import('./modulos/perfil/configuracion').then((m) => m.Configuracion),
  },
  {
    path: 'muestrario',
    title: 'Muestrario · Finanzas',
    loadComponent: () => import('./ui/muestrario').then((m) => m.Muestrario),
  },
  { path: '**', redirectTo: '' },
];
