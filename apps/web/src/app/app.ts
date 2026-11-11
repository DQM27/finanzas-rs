import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { Nivel } from './nucleo/nivel';
import { Aviso } from './ui/aviso';
import { BarraLateral, GrupoLateral } from './ui/barra-lateral';
import { BarraSecciones, Seccion } from './ui/barra-secciones';

/**
 * El marco de la app: la pantalla de turno, la navegación del nivel en que
 * se está (barra de secciones en el celular, barra lateral en la computadora)
 * y los avisos.
 */
@Component({
  selector: 'app-root',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [RouterOutlet, BarraLateral, BarraSecciones, Aviso],
  host: { '[class.escritorio]': 'nivel.computadora()' },
  template: `
    @if (nivel.computadora()) {
      <fz-barra-lateral [grupos]="lateral" />
      <div class="contenido">
        <router-outlet />
      </div>
    } @else {
      <router-outlet />
      <fz-barra-secciones [secciones]="secciones" />
    }
    <fz-aviso />
  `,
})
export class App {
  protected readonly nivel = inject(Nivel);

  protected readonly secciones: readonly Seccion[] = [
    { ruta: '/', etiqueta: 'Inicio', icono: 'inicio' },
    { ruta: '/quincena', etiqueta: 'Quincena', icono: 'quincena' },
    { ruta: '/anotar', etiqueta: 'Anotar', icono: 'anotar' },
    { ruta: '/mas', etiqueta: 'Más', icono: 'mas' },
  ];

  protected readonly lateral: readonly GrupoLateral[] = [
    {
      enlaces: [
        { ruta: '/', etiqueta: 'Inicio', icono: 'inicio' },
        { ruta: '/mes', etiqueta: 'Mes', icono: 'mes' },
        { ruta: '/quincena', etiqueta: 'Quincena', icono: 'quincena' },
        { ruta: '/anotar', etiqueta: 'Anotar', icono: 'anotar' },
      ],
    },
    {
      titulo: 'Registros',
      enlaces: [
        { ruta: '/extras', etiqueta: 'Extras', icono: 'extras' },
        { ruta: '/gastos', etiqueta: 'Gastos', icono: 'gastos' },
      ],
    },
    {
      titulo: 'Ajustes',
      enlaces: [{ ruta: '/configuracion', etiqueta: 'Configuración', icono: 'ajustes' }],
    },
  ];
}
