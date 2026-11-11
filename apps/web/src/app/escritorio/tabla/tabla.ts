import {
  ChangeDetectionStrategy,
  Component,
  computed,
  input,
  output,
  signal,
  type OnInit,
  type Signal,
} from '@angular/core';
import { AG_GRID_LOCALE_ES } from '@ag-grid-community/locale';
import {
  AllCommunityModule,
  ModuleRegistry,
  colorSchemeVariable,
  themeQuartz,
  type ColDef,
  type ColTypeDefs,
  type ColumnState,
  type GetRowIdParams,
  type GridApi,
  type GridReadyEvent,
  type ModelUpdatedEvent,
  type RowClassParams,
} from 'ag-grid-community';
import { AgGridAngular } from 'ag-grid-angular';
import { Icono } from '../../ui/icono';
import { claveDeColumna, normalizar, textoDeTooltip, tiposDeColumna } from './columnas';
import { guardarDiseno, leerDiseno } from './diseno-guardado';

// Los módulos de AG Grid se registran una sola vez, aquí. Este archivo sólo
// lo cargan las pantallas de computadora: el celular no baja AG Grid.
ModuleRegistry.registerModules([AllCommunityModule]);

/** Alto de fila y de encabezado, en px (AG Grid los pide como número). */
const ALTO_FILA = 36;
const ALTO_ENCABEZADO = 38;

/**
 * La tabla de la versión de computadora, tomada de Limen. Cada pantalla le da
 * sus columnas y sus filas y hereda todo lo demás: AG Grid con los colores de
 * la app, textos en español, una sola fila de herramientas (buscador sin
 * tildes ni mayúsculas, conteo, filtros por columna, anchos, columnas
 * visibles y exportar a CSV), estados de carga y vacío, y el diseño que cada
 * persona le da queda guardado.
 *
 * La tabla no calcula nada: lo que muestra viene de la API.
 *
 * La pantalla puede poner sus controles a la izquierda (`[controles]`) y sus
 * acciones a la derecha (`[acciones]`) de la fila de herramientas.
 */
@Component({
  selector: 'fz-tabla',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [AgGridAngular, Icono],
  host: { '(document:keydown.escape)': 'menuAbierto.set(false)' },
  templateUrl: './tabla.html',
  styleUrl: './tabla.scss',
})
export class Tabla<T> implements OnInit {
  /** Cómo se llama esta tabla («Extras»): la anuncian los lectores de pantalla. */
  readonly etiqueta = input.required<string>();
  readonly columnas = input.required<ColDef<T>[]>();
  readonly filas = input.required<T[]>();
  readonly cargando = input(false);
  readonly marcador = input('Buscar…');
  readonly mensajeVacio = input('Sin registros');
  /** Identificador de esta tabla: con él se guarda su diseño y se nombra el CSV. */
  readonly id = input<string>();
  /** Identificador estable de cada fila; mantiene el estado al refrescar. */
  readonly idFila = input<(fila: T) => string>();
  readonly claseFila = input<(fila: T) => string | undefined>();

  readonly filaDobleClic = output<T>();

  protected readonly textos = AG_GRID_LOCALE_ES;
  /** Quartz de serie con los colores de la app; Día o Noche según el sistema. */
  protected readonly tema = themeQuartz.withPart(colorSchemeVariable).withParams({
    backgroundColor: 'var(--tarjeta)',
    foregroundColor: 'var(--texto)',
    headerBackgroundColor: 'var(--variante)',
    borderColor: 'color-mix(in srgb, var(--borde) 45%, transparent)',
    accentColor: 'var(--acento)',
    chromeBackgroundColor: 'var(--tarjeta)',
    fontFamily: 'var(--fuente)',
    wrapperBorderRadius: 'var(--radio-grupo)',
  });
  protected readonly tiposDeColumna = tiposDeColumna as ColTypeDefs<T>;
  protected readonly altoFila = ALTO_FILA;
  protected readonly altoEncabezado = ALTO_ENCABEZADO;

  protected readonly filtro = signal('');
  protected readonly ocultas = signal<ReadonlySet<string>>(new Set());
  protected readonly filtrosVisibles = signal(true);
  protected readonly menuAbierto = signal(false);
  /** Cuántas filas se ven con los filtros puestos. */
  protected readonly visibles = signal<number | null>(null);

  #api: GridApi<T> | null = null;
  /** Orden, ancho y fijado de las columnas: lo último guardado o lo que haya hoy. */
  #columnasGuardadas: ColumnState[] = [];

  protected readonly columnaPorDefecto: Signal<ColDef<T>> = computed(() => ({
    sortable: true,
    resizable: true,
    minWidth: 90,
    flex: 1,
    filter: true,
    floatingFilter: this.filtrosVisibles(),
    // Los filtros de texto no distinguen tildes ni mayúsculas.
    filterParams: { textFormatter: (texto: string) => normalizar(texto) },
    tooltipValueGetter: textoDeTooltip,
    ...(this.idFila() ? { enableCellChangeFlash: true } : {}),
  }));

  protected readonly columnasConVisibilidad: Signal<ColDef<T>[]> = computed(() =>
    this.columnas().map((columna) => {
      const clave = claveDeColumna(columna);
      return clave ? { ...columna, hide: this.ocultas().has(clave) } : columna;
    }),
  );

  /** Las columnas que se pueden mostrar u ocultar; una con `lockVisible` siempre se ve. */
  protected readonly columnasDelMenu = computed(() =>
    this.columnas().flatMap((columna) => {
      const clave = claveDeColumna(columna);
      return clave && !columna.lockVisible ? [{ clave, titulo: columna.headerName ?? clave }] : [];
    }),
  );

  protected readonly conteo = computed(() => {
    const total = this.filas().length;
    const visibles = this.visibles() ?? total;
    const palabra = total === 1 ? 'registro' : 'registros';
    return visibles === total ? `${total} ${palabra}` : `${visibles} de ${total} ${palabra}`;
  });

  protected readonly obtenerIdFila = computed(() => {
    const id = this.idFila();
    return id ? (p: GetRowIdParams<T>): string => id(p.data) : undefined;
  });

  protected readonly obtenerClaseFila = computed(() => {
    const clase = this.claseFila();
    return clase
      ? (p: RowClassParams<T>): string | undefined => (p.data ? clase(p.data) : undefined)
      : undefined;
  });

  /** El buscador compara sin tildes ni mayúsculas, palabra por palabra. */
  protected readonly coincide = (partes: string[], textoDeFila: string): boolean => {
    const fila = normalizar(textoDeFila);
    return partes.every((parte) => fila.includes(normalizar(parte)));
  };

  /** Lo guardado se lee al crear la tabla: no depende de que la grilla esté lista. */
  ngOnInit(): void {
    const guardado = leerDiseno(this.id());
    if (!guardado) return;
    this.ocultas.set(new Set(guardado.ocultas));
    this.filtrosVisibles.set(guardado.filtrosVisibles);
    this.#columnasGuardadas = guardado.columnas;
  }

  protected escribirFiltro(evento: Event): void {
    this.filtro.set((evento.target as HTMLInputElement).value);
  }

  protected alIniciar(evento: GridReadyEvent<T>): void {
    this.#api = evento.api;
    if (this.#columnasGuardadas.length > 0) {
      evento.api.applyColumnState({ state: this.#columnasGuardadas, applyOrder: true });
    }
  }

  protected alActualizarFilas(evento: ModelUpdatedEvent<T>): void {
    this.visibles.set(evento.api.getDisplayedRowCount());
  }

  protected alCambiarColumnas(terminado: boolean | undefined): void {
    if (terminado !== false) this.#guardar();
  }

  protected alDobleClic(fila: T | undefined): void {
    if (fila) this.filaDobleClic.emit(fila);
  }

  protected alternarFiltros(): void {
    this.filtrosVisibles.update((visibles) => !visibles);
    this.#guardar();
  }

  protected alternarColumna(clave: string): void {
    this.ocultas.update((actuales) => {
      const siguientes = new Set(actuales);
      if (!siguientes.delete(clave)) siguientes.add(clave);
      return siguientes;
    });
    this.#guardar();
  }

  /** Cada columna al ancho de su contenido (sin el reparto proporcional). */
  protected ajustarAnchos(): void {
    const api = this.#api;
    if (!api) return;
    api.applyColumnState({
      state: api.getColumnState().map((columna) => ({ colId: columna.colId, flex: null })),
    });
    api.autoSizeAllColumns();
    this.#guardar();
  }

  /** Vuelve a los anchos con que la pantalla definió sus columnas. */
  protected restablecerAnchos(): void {
    const api = this.#api;
    if (!api) return;
    api.applyColumnState({
      state: this.columnas().flatMap((columna) => {
        const colId = claveDeColumna(columna);
        return colId
          ? [{ colId, flex: columna.flex ?? null, width: columna.flex ? undefined : columna.width }]
          : [];
      }),
    });
    this.#guardar();
  }

  /**
   * Lo que se ve (con filtros y orden) a CSV, con los valores crudos: fechas
   * `AAAA-MM-DD` y colones enteros, para que una hoja de cálculo los entienda.
   */
  protected exportar(): void {
    this.#api?.exportDataAsCsv({
      fileName: `${this.id() ?? 'tabla'}.csv`,
      columnSeparator: ';',
      processCellCallback: (p) => String(p.value ?? ''),
    });
  }

  #guardar(): void {
    if (this.#api) {
      // `hide` no se guarda: `ocultas` es la única fuente de verdad.
      this.#columnasGuardadas = this.#api
        .getColumnState()
        .map(({ hide: _oculta, ...resto }) => resto);
    }
    guardarDiseno(this.id(), {
      ocultas: [...this.ocultas()],
      columnas: this.#columnasGuardadas,
      filtrosVisibles: this.filtrosVisibles(),
    });
  }
}
