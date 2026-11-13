import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  signal,
} from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';
import type { Lado, Linea, Quincena as DatosQuincena } from '../../nucleo/api';
import { Carga, cargar, valor } from '../../nucleo/carga';
import { FallaApi, Finanzas } from '../../nucleo/finanzas';
import { ColonesPipe, FechaCortaPipe, fechaCorta, leerMonto } from '../../nucleo/formato';
import { Avisos } from '../../ui/aviso';
import { Boton } from '../../ui/boton';
import { Casilla } from '../../ui/casilla';
import { Esqueleto } from '../../ui/esqueleto';
import { Grupo } from '../../ui/grupo';
import { Icono } from '../../ui/icono';
import { Renglon } from '../../ui/renglon';
import { Segmentado, Segmento } from '../../ui/segmentado';
import { Tarjeta } from '../../ui/tarjeta';

/** Una línea del presupuesto mientras se escribe. */
export interface Fila {
  concepto: string;
  monto: string;
}

type Lista = 'ingresos' | 'pagos';

/** Las filas que se van a guardar: sin las que quedaron vacías. */
export function lineasDe(filas: readonly Fila[]): { concepto: string; monto: number }[] {
  return filas
    .filter((f) => f.concepto.trim() !== '' || f.monto.trim() !== '')
    .map((f) => ({ concepto: f.concepto, monto: leerMonto(f.monto) ?? 0 }));
}

/**
 * La quincena: lo que sobra, los pagos por marcar y los ingresos, y el
 * editor del presupuesto. El salario y las extras del pago entran solos
 * (Q7); en el editor sólo se escriben los otros ingresos. Los totales, lo
 * que sobra y el avance vienen siempre de la API (Q1, Q2); al marcar o
 * guardar se muestra lo que ella devuelve. Rehacer el presupuesto no borra
 * lo marcado (Q4).
 */
@Component({
  selector: 'fz-quincena',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    Boton,
    Casilla,
    ColonesPipe,
    Esqueleto,
    FechaCortaPipe,
    FormsModule,
    Grupo,
    Icono,
    Renglon,
    Segmentado,
    Tarjeta,
  ],
  templateUrl: './quincena.html',
  styleUrl: './quincena.scss',
})
export class Quincena {
  readonly #finanzas = inject(Finanzas);
  readonly #avisos = inject(Avisos);

  protected readonly cortes = cargar(this.#finanzas.cortesHoy());
  /** Un pago pedido en la ruta (`/quincena?pago=AAAA-MM-DD`), por ejemplo desde Mes. */
  readonly #pedido =
    inject(ActivatedRoute, { optional: true })?.snapshot.queryParamMap.get('pago') ?? null;

  /** Esta quincena y, si ya se sabe y es otra, la próxima. */
  protected readonly opciones = computed<Segmento<string>[]>(() => {
    const c = valor(this.cortes());
    const opciones: Segmento<string>[] = [];
    if (c?.ultimo_pago) opciones.push({ valor: c.ultimo_pago.pago, etiqueta: 'Esta quincena' });
    if (c?.proximo_pago && c.proximo_pago.pago !== c.ultimo_pago?.pago) {
      opciones.push({
        valor: c.proximo_pago.pago,
        etiqueta: `La del ${fechaCorta(c.proximo_pago.pago)}`,
      });
    }
    const pedido = this.#pedido;
    if (c && pedido && !opciones.some((o) => o.valor === pedido)) {
      opciones.push({ valor: pedido, etiqueta: `La del ${fechaCorta(pedido)}` });
    }
    return opciones;
  });

  protected readonly listas: readonly Lista[] = ['ingresos', 'pagos'];

  /** El día de pago que se está viendo. */
  protected readonly pago = signal<string | null>(null);
  protected readonly quincena = signal<Carga<DatosQuincena>>({ estado: 'cargando' });

  protected readonly editando = signal(false);
  protected readonly ingresos = signal<Fila[]>([]);
  protected readonly pagos = signal<Fila[]>([]);
  protected readonly guardando = signal(false);
  protected readonly falla = signal<string | null>(null);
  /** La línea que se está marcando, para no tocarla dos veces. */
  protected readonly marcando = signal<string | null>(null);

  protected readonly sinPresupuesto = computed(() => {
    const q = this.quincena();
    return q.estado === 'listo' && !q.valor.presupuestada;
  });

  constructor() {
    // Al llegar el calendario, se abre la pedida o la quincena en curso.
    effect(() => {
      const primera = this.opciones()[0];
      if (primera && this.pago() === null) this.abrir(this.#pedido ?? primera.valor);
    });
    // Sin presupuesto, se arranca el editor. El salario y las extras no se
    // escriben: entran solos (Q7).
    effect(() => {
      if (this.sinPresupuesto() && !this.editando()) this.#empezar([], []);
    });
  }

  protected abrir(pago: string): void {
    this.pago.set(pago);
    this.editando.set(false);
    this.falla.set(null);
    this.quincena.set({ estado: 'cargando' });
    this.#finanzas.quincena(pago).subscribe({
      next: (q) => this.quincena.set({ estado: 'listo', valor: q }),
      error: (e: FallaApi) =>
        this.quincena.set({ estado: 'falla', codigo: e.codigo, mensaje: e.message }),
    });
  }

  protected editar(q: DatosQuincena): void {
    this.#empezar(q.ingresos, q.pagos);
  }

  protected cancelar(): void {
    this.editando.set(false);
    this.falla.set(null);
  }

  protected cambiar(lista: Lista, indice: number, campo: keyof Fila, texto: string): void {
    this.#lista(lista).update((filas) =>
      filas.map((f, i) => (i === indice ? { ...f, [campo]: texto } : f)),
    );
  }

  protected agregar(lista: Lista): void {
    this.#lista(lista).update((filas) => [...filas, { concepto: '', monto: '' }]);
  }

  protected quitar(lista: Lista, indice: number): void {
    this.#lista(lista).update((filas) => filas.filter((_, i) => i !== indice));
  }

  protected guardar(): void {
    const pago = this.pago();
    if (!pago || this.guardando()) return;
    this.guardando.set(true);
    this.falla.set(null);
    this.#finanzas
      .guardarPresupuesto(pago, {
        ingresos: lineasDe(this.ingresos()),
        pagos: lineasDe(this.pagos()),
      })
      .subscribe({
        next: (q) => {
          this.guardando.set(false);
          this.editando.set(false);
          this.quincena.set({ estado: 'listo', valor: q });
          this.#avisos.exito('Presupuesto guardado');
        },
        error: (e: FallaApi) => {
          this.guardando.set(false);
          this.falla.set(e.message);
          this.#avisos.error('No se guardó');
        },
      });
  }

  protected marcar(lado: Lado, indice: number, linea: Linea): void {
    const pago = this.pago();
    const clave = `${lado}-${indice}`;
    if (!pago || this.marcando()) return;
    this.marcando.set(clave);
    this.#finanzas.marcar(pago, { lado, indice, hecho: !linea.hecho }).subscribe({
      next: (q) => {
        this.marcando.set(null);
        this.quincena.set({ estado: 'listo', valor: q });
      },
      error: (e: FallaApi) => {
        this.marcando.set(null);
        this.#avisos.error(e.message);
      },
    });
  }

  #empezar(ingresos: readonly Linea[], pagos: readonly Linea[]): void {
    const filas = (lineas: readonly Linea[]): Fila[] =>
      lineas.map((l) => ({ concepto: l.concepto, monto: String(l.monto) }));
    this.ingresos.set(filas(ingresos));
    this.pagos.set(pagos.length ? filas(pagos) : [{ concepto: '', monto: '' }]);
    this.falla.set(null);
    this.editando.set(true);
  }

  #lista(lista: Lista) {
    return lista === 'ingresos' ? this.ingresos : this.pagos;
  }
}
