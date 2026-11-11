import { ChangeDetectionStrategy, Component, computed, inject, signal } from '@angular/core';
import { toObservable } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import type { ColDef } from 'ag-grid-community';
import { BehaviorSubject, combineLatest } from 'rxjs';
import type { Categoria, Gasto, Origen } from '../nucleo/api';
import { recargable, valor } from '../nucleo/carga';
import { CATEGORIAS, ORIGENES } from '../nucleo/etiquetas';
import { FallaApi, Finanzas } from '../nucleo/finanzas';
import { colones, fechaCorta, hoy, leerMonto } from '../nucleo/formato';
import { Avisos } from '../ui/aviso';
import { CeldaDeshacer, ParametrosDeshacer } from './tabla/celda-deshacer';
import { Tabla } from './tabla/tabla';

/** El primer y el último día del mes de una fecha `AAAA-MM-DD`. */
export function mesDe(fecha: string): { desde: string; hasta: string } {
  const [a, m] = fecha.split('-').map(Number);
  const ultimo = new Date(a, m, 0).getDate();
  const mes = String(m).padStart(2, '0');
  return { desde: `${a}-${mes}-01`, hasta: `${a}-${mes}-${String(ultimo).padStart(2, '0')}` };
}

/**
 * Gastos puntuales en la computadora: la grilla de un rango de fechas (por
 * defecto, el mes en curso) y una fila para anotar otro.
 */
@Component({
  selector: 'fz-gastos',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [FormsModule, Tabla],
  template: `
    <main class="pantalla-ancha" data-seccion="gasto">
      <h1>Gastos</h1>

      <form class="alta" (ngSubmit)="anotar()" aria-label="Anotar un gasto">
        <label>Fecha <input type="date" name="fecha" [(ngModel)]="fecha" /></label>
        <label>Concepto
          <input class="ancho" name="concepto" maxlength="80" autocomplete="off" placeholder="Requerido"
                 [(ngModel)]="concepto" />
        </label>
        <label>Monto
          <input class="cifras" name="monto" inputmode="numeric" autocomplete="off" placeholder="Requerido"
                 [(ngModel)]="monto" />
        </label>
        <label>Categoría
          <select name="categoria" [(ngModel)]="categoria">
            @for (c of categorias; track c[0]) {
              <option [value]="c[0]">{{ c[1] }}</option>
            }
          </select>
        </label>
        <label>Salió de
          <select name="origen" [(ngModel)]="origen">
            @for (o of origenes; track o[0]) {
              <option [value]="o[0]">{{ o[1] }}</option>
            }
          </select>
        </label>
        <button type="submit" class="guardar" [disabled]="guardando()">
          {{ guardando() ? 'Guardando…' : 'Anotar gasto' }}
        </button>
        @if (falla(); as mensaje) {
          <p class="falla" role="alert">{{ mensaje }}</p>
        }
      </form>

      <fz-tabla etiqueta="Gastos" id="gastos" [columnas]="columnas" [filas]="filas()"
                [cargando]="cargando()" [idFila]="idFila" mensajeVacio="No hay gastos en estas fechas"
                marcador="Buscar por concepto, categoría…">
        <div controles class="rango-controles">
          <label>Desde <input type="date" [value]="desde()" (change)="desde.set($any($event.target).value)" /></label>
          <label>Hasta <input type="date" [value]="hasta()" (change)="hasta.set($any($event.target).value)" /></label>
        </div>
      </fz-tabla>
    </main>
  `,
  styleUrl: './alta.scss',
  styles: `
    .rango-controles {
      display: flex;
      gap: var(--espacio);
    }
    .rango-controles label {
      grid-auto-flow: column;
      align-items: center;
    }
  `,
})
export class Gastos {
  readonly #finanzas = inject(Finanzas);
  readonly #avisos = inject(Avisos);
  readonly #recarga = new BehaviorSubject<void>(undefined);

  protected readonly desde = signal(mesDe(hoy()).desde);
  protected readonly hasta = signal(mesDe(hoy()).hasta);

  protected readonly gastos = recargable(
    combineLatest([toObservable(this.desde), toObservable(this.hasta), this.#recarga]),
    ([desde, hasta]) => this.#finanzas.gastos(desde, hasta),
  );
  protected readonly filas = computed(() => valor(this.gastos()) ?? []);
  protected readonly cargando = computed(() => this.gastos().estado === 'cargando');
  protected readonly categorias = Object.entries(CATEGORIAS) as [Categoria, string][];
  protected readonly origenes = Object.entries(ORIGENES) as [Origen, string][];
  protected readonly idFila = (g: Gasto) => g.id;

  protected fecha = hoy();
  protected concepto = '';
  protected monto = '';
  protected categoria: Categoria = 'COMIDA';
  protected origen: Origen = 'CUENTA_GASTOS';
  protected readonly guardando = signal(false);
  protected readonly falla = signal<string | null>(null);

  protected readonly columnas: ColDef<Gasto>[] = [
    { field: 'fecha', headerName: 'Fecha', type: 'fecha', sort: 'desc' },
    { field: 'concepto', headerName: 'Concepto', flex: 2 },
    {
      field: 'categoria',
      headerName: 'Categoría',
      valueFormatter: (p) => CATEGORIAS[p.value as Categoria] ?? '',
    },
    {
      field: 'origen',
      headerName: 'Salió de',
      valueFormatter: (p) => ORIGENES[p.value as Origen] ?? '',
    },
    { field: 'monto', headerName: 'Monto', type: 'colones' },
    {
      colId: 'acciones',
      headerName: '',
      width: 60,
      flex: 0,
      sortable: false,
      filter: false,
      resizable: false,
      lockVisible: true,
      cellRenderer: CeldaDeshacer,
      cellRendererParams: {
        deshacer: (g) => this.deshacer(g),
        nombre: (g) => g.concepto,
      } satisfies ParametrosDeshacer<Gasto>,
    },
  ];

  protected anotar(): void {
    if (this.guardando()) return;
    this.guardando.set(true);
    this.falla.set(null);
    this.#finanzas
      .anotarGasto({
        fecha: this.fecha,
        concepto: this.concepto,
        monto: leerMonto(this.monto) ?? 0,
        categoria: this.categoria,
        origen: this.origen,
      })
      .subscribe({
        next: (r) => {
          this.guardando.set(false);
          this.concepto = '';
          this.monto = '';
          this.#avisos.exito(`Gasto anotado · quedan ${colones(r.caja.queda)} en la caja chica`);
          this.#recarga.next();
        },
        error: (e: FallaApi) => {
          this.guardando.set(false);
          this.falla.set(e.message);
        },
      });
  }

  protected deshacer(g: Gasto): void {
    if (!confirm(`¿Deshacer «${g.concepto}» del ${fechaCorta(g.fecha)} (${colones(g.monto)})?`)) return;
    this.#finanzas.borrarGasto(g.id).subscribe({
      next: () => {
        this.#avisos.exito('Gasto deshecho');
        this.#recarga.next();
      },
      error: (falla: FallaApi) => this.#avisos.error(falla.message),
    });
  }
}
