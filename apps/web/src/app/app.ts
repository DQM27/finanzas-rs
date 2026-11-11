import { ChangeDetectionStrategy, Component } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { Aviso } from './ui/aviso';
import { BarraSecciones, Seccion } from './ui/barra-secciones';

/** El marco de la app: la pantalla de turno, la barra de secciones y los avisos. */
@Component({
  selector: 'app-root',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [RouterOutlet, BarraSecciones, Aviso],
  template: `
    <router-outlet />
    <fz-barra-secciones [secciones]="secciones" />
    <fz-aviso />
  `,
})
export class App {
  protected readonly secciones: readonly Seccion[] = [
    { ruta: '/', etiqueta: 'Inicio', icono: 'inicio' },
    { ruta: '/quincena', etiqueta: 'Quincena', icono: 'quincena' },
    { ruta: '/anotar', etiqueta: 'Anotar', icono: 'anotar' },
    { ruta: '/mas', etiqueta: 'Más', icono: 'mas' },
  ];
}
