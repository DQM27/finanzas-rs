import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ColonesPipe } from '../nucleo/formato';
import { Avisos } from '../ui/aviso';
import { Boton } from '../ui/boton';
import { Esqueleto } from '../ui/esqueleto';
import { Grupo } from '../ui/grupo';
import { Renglon } from '../ui/renglon';
import { Segmentado, Segmento } from '../ui/segmentado';
import { Tarjeta } from '../ui/tarjeta';

type Mitad = 'Q1' | 'Q2';

/**
 * Todos los componentes con datos de ejemplo, para revisarlos en el celular
 * en los dos temas. No toca la API.
 */
@Component({
  selector: 'fz-muestrario',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [Boton, ColonesPipe, Esqueleto, FormsModule, Grupo, Renglon, Segmentado, Tarjeta],
  template: `
    <main class="pantalla">
      <h1>Muestrario</h1>

      <h2 class="seccion-titulo">Tarjetas de resumen</h2>
      <div class="pila">
        <fz-tarjeta clave="Caja chica" [monto]="3200" [proporcion]="0.64" nota="Sana: más del 30 %" />
        <fz-tarjeta clave="Caja chica" [monto]="1200" [proporcion]="0.24" nota="Baja: 30 % o menos" />
        <fz-tarjeta clave="Caja chica" [monto]="0" [proporcion]="0" nota="Agotada" />
        <fz-tarjeta clave="Fondo de emergencia" [monto]="415000" [proporcion]="0.45" [crece]="true"
                    nota="Crecer no es alarma: va en acento" />
        <fz-tarjeta clave="Cierre de quincena" [monto]="-18500" [tenida]="true" tono="error"
                    nota="Tarjeta teñida: la quincena cierra en negativo" />
        <fz-tarjeta clave="Extras" seccion="extra" [tenida]="true" [monto]="51600"
                    nota="Teñida con el tono de una sección" />
      </div>

      <h2 class="seccion-titulo">Selector segmentado</h2>
      <fz-segmentado etiqueta="Quincena" [segmentos]="mitades" [(valor)]="mitad" />
      <p class="nota-libre">Elegida: {{ mitad() }}</p>

      <h2 class="seccion-titulo">Grupo de formulario</h2>
      <fz-grupo titulo="Nuevo gasto" nota="El monto va en colones enteros.">
        <fz-renglon etiqueta="Concepto">
          <input class="campo" placeholder="Requerido" [(ngModel)]="concepto" />
        </fz-renglon>
        <fz-renglon etiqueta="Monto">
          <input class="campo cifras" inputmode="numeric" placeholder="Requerido" />
        </fz-renglon>
        <fz-renglon etiqueta="Categoría" [chevron]="true" (click)="avisos.exito('Abriría el selector')">
          <span slot="valor">Comida</span>
        </fz-renglon>
        <fz-renglon etiqueta="Nota">
          <input class="campo" placeholder="Opcional" />
        </fz-renglon>
      </fz-grupo>

      <fz-grupo titulo="Con error" nota="El monto tiene que ser mayor que cero." [error]="true">
        <fz-renglon etiqueta="Monto">
          <input class="campo cifras" value="0" />
        </fz-renglon>
      </fz-grupo>

      <h2 class="seccion-titulo">Renglones de movimiento</h2>
      <fz-grupo titulo="Hoy">
        @for (m of movimientos; track m.concepto) {
          <fz-renglon [conIcono]="true" [attr.data-seccion]="m.seccion">
            <span slot="icono" class="icono-seccion" aria-hidden="true">{{ m.letra }}</span>
            <span class="linea-1">{{ m.concepto }}</span>
            <span class="linea-2">{{ m.detalle }}</span>
            <span slot="valor" class="cifras" [class.negativo]="m.monto < 0">{{ m.monto | colones }}</span>
          </fz-renglon>
        }
      </fz-grupo>

      <h2 class="seccion-titulo">Botón principal</h2>
      <div class="pila" data-seccion="gasto">
        <button fz-boton type="button" (click)="guardar()" [guardando]="guardando()">Guardar gasto</button>
      </div>
      <div class="pila" data-seccion="extra">
        <button fz-boton type="button">Guardar extra</button>
      </div>

      <h2 class="seccion-titulo">Avisos</h2>
      <div class="pila">
        <button fz-boton type="button" (click)="avisos.exito('Gasto guardado')">Aviso correcto</button>
        <div data-seccion="insta">
          <button fz-boton type="button" (click)="avisos.error('No hay conexión. Revisá el internet.')">
            Aviso de error
          </button>
        </div>
      </div>

      <h2 class="seccion-titulo">Carga</h2>
      <div class="pila">
        <fz-esqueleto alto="var(--alto-boton)" />
        <fz-esqueleto />
      </div>
    </main>
  `,
  styles: `
    .seccion-titulo {
      margin: calc(var(--espacio) * 2) 0 var(--espacio);
      font: var(--titulo);
    }
    .pila {
      display: grid;
      gap: var(--espacio);
      margin-bottom: var(--espacio);
    }
    .nota-libre {
      margin: var(--chico) 0 0;
      font: var(--cuerpo-chico);
      color: var(--texto-2);
    }
    .negativo {
      color: var(--error);
    }
  `,
})
export class Muestrario {
  protected readonly avisos = inject(Avisos);
  protected readonly mitades: readonly Segmento<Mitad>[] = [
    { valor: 'Q1', etiqueta: 'Q1' },
    { valor: 'Q2', etiqueta: 'Q2' },
  ];
  protected readonly mitad = signal<Mitad>('Q1');
  protected readonly concepto = signal('');
  protected readonly guardando = signal(false);
  protected readonly movimientos = [
    { seccion: 'gasto', letra: 'G', concepto: 'Almuerzo', detalle: 'Comida · VIE 13 nov', monto: -3500 },
    { seccion: 'quincena', letra: 'Q', concepto: 'Gasolina', detalle: 'Moto · VIE 13 nov', monto: -8000 },
    { seccion: 'extra', letra: 'E', concepto: 'Extra nocturna', detalle: 'Se paga el 30 nov', monto: 25800 },
  ];

  /** Simula un guardado para ver el estado «Guardando…». */
  protected guardar(): void {
    this.guardando.set(true);
    setTimeout(() => {
      this.guardando.set(false);
      this.avisos.exito('Gasto guardado');
    }, 1200);
  }
}
