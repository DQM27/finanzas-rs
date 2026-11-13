import { ChangeDetectionStrategy, Component, computed, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import type { ColDef } from 'ag-grid-community';
import { BehaviorSubject } from 'rxjs';
import type { Extra, TipoExtra } from '../../nucleo/api';
import { recargable, valor } from '../../nucleo/carga';
import { TIPOS_EXTRA } from '../../nucleo/etiquetas';
import { FallaApi, Finanzas } from '../../nucleo/finanzas';
import { colones, fechaCorta, hoy, leerMonto } from '../../nucleo/formato';
import { Avisos } from '../../ui/aviso';
import { CeldaDeshacer, ParametrosDeshacer } from '../../ui/tabla/celda-deshacer';
import { Tabla } from '../../ui/tabla/tabla';

/**
 * Extras en la computadora: la grilla de todas, con su neto y su fecha de
 * pago (los calcula el dominio, E1–E3), y una fila para anotar otra.
 */
@Component({
  selector: 'fz-extras',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [FormsModule, Tabla],
  template: `
    <main class="pantalla-ancha" data-seccion="extra">
      <h1>Extras</h1>

      <form class="alta" (ngSubmit)="anotar()" aria-label="Anotar una extra">
        <label>Fecha <input type="date" name="fecha" [(ngModel)]="fecha" /></label>
        <label
          >Tipo
          <select name="tipo" [(ngModel)]="tipo">
            @for (t of tipos; track t[0]) {
              <option [value]="t[0]">{{ t[1] }}</option>
            }
          </select>
        </label>
        <label
          >Monto bruto
          <input
            class="cifras"
            name="monto"
            inputmode="numeric"
            autocomplete="off"
            [placeholder]="tipo === 'OTRA' ? 'Requerido' : 'Según tarifa'"
            [(ngModel)]="monto"
          />
        </label>
        <button type="submit" class="guardar" [disabled]="guardando()">
          {{ guardando() ? 'Guardando…' : 'Anotar extra' }}
        </button>
        @if (falla(); as mensaje) {
          <p class="falla" role="alert">{{ mensaje }}</p>
        }
      </form>

      <fz-tabla
        etiqueta="Extras"
        id="extras"
        [columnas]="columnas"
        [filas]="filas()"
        [cargando]="cargando()"
        [idFila]="idFila"
        mensajeVacio="Todavía no hay extras"
        marcador="Buscar por tipo o fecha…"
      />
    </main>
  `,
  styleUrl: '../../ui/tabla/alta.scss',
})
export class Extras {
  readonly #finanzas = inject(Finanzas);
  readonly #avisos = inject(Avisos);
  readonly #recarga = new BehaviorSubject<void>(undefined);

  protected readonly extras = recargable(this.#recarga, () => this.#finanzas.extras());
  protected readonly filas = computed(() => valor(this.extras()) ?? []);
  protected readonly cargando = computed(() => this.extras().estado === 'cargando');
  protected readonly tipos = Object.entries(TIPOS_EXTRA) as [TipoExtra, string][];
  protected readonly idFila = (e: Extra) => e.id;

  protected fecha = hoy();
  protected tipo: TipoExtra = 'NOCTURNA';
  protected monto = '';
  protected readonly guardando = signal(false);
  protected readonly falla = signal<string | null>(null);

  protected readonly columnas: ColDef<Extra>[] = [
    { field: 'fecha', headerName: 'Trabajada', type: 'fecha', sort: 'desc' },
    {
      field: 'tipo',
      headerName: 'Tipo',
      valueFormatter: (p) => TIPOS_EXTRA[p.value as TipoExtra] ?? '',
    },
    { field: 'bruto', headerName: 'Bruto', type: 'colones' },
    { field: 'neto', headerName: 'Neto', type: 'colones' },
    { field: 'fecha_pago', headerName: 'Se paga', type: 'fecha' },
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
        deshacer: (e) => this.deshacer(e),
        nombre: (e) => `la extra del ${fechaCorta(e.fecha)}`,
      } satisfies ParametrosDeshacer<Extra>,
    },
  ];

  protected anotar(): void {
    if (this.guardando()) return;
    this.guardando.set(true);
    this.falla.set(null);
    this.#finanzas
      .anotarExtra({ fecha: this.fecha, tipo: this.tipo, monto: leerMonto(this.monto) })
      .subscribe({
        next: (e) => {
          this.guardando.set(false);
          this.monto = '';
          const pago = e.fecha_pago ? `, se paga el ${fechaCorta(e.fecha_pago)}` : '';
          this.#avisos.exito(`Extra anotada · ${colones(e.neto)} neto${pago}`);
          this.#recarga.next();
        },
        error: (e: FallaApi) => {
          this.guardando.set(false);
          this.falla.set(e.message);
        },
      });
  }

  protected deshacer(e: Extra): void {
    if (!confirm(`¿Deshacer la extra del ${fechaCorta(e.fecha)} (${colones(e.bruto)} bruto)?`))
      return;
    this.#finanzas.borrarExtra(e.id).subscribe({
      next: () => {
        this.#avisos.exito('Extra deshecha');
        this.#recarga.next();
      },
      error: (falla: FallaApi) => this.#avisos.error(falla.message),
    });
  }
}
