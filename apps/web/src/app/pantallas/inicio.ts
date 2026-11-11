import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { RouterLink } from '@angular/router';
import { shareReplay, switchMap } from 'rxjs';
import type { Caja, Gasto, NivelCaja, Quincena } from '../nucleo/api';
import { cargar, valor } from '../nucleo/carga';
import { CATEGORIAS, faltan } from '../nucleo/etiquetas';
import { Finanzas } from '../nucleo/finanzas';
import { ColonesPipe, FechaCortaPipe, colones, fechaCorta, hoy } from '../nucleo/formato';
import { Esqueleto } from '../ui/esqueleto';
import { Grupo } from '../ui/grupo';
import { Renglon } from '../ui/renglon';
import { EstadoBarra, Tarjeta } from '../ui/tarjeta';

/** El nivel que decide el dominio, en el color de la barra. */
const COLOR_CAJA: Record<NivelCaja, EstadoBarra> = {
  HOLGADA: 'sana',
  BAJA: 'baja',
  AGOTADA: 'agotada',
};

/**
 * Inicio: lo del día. Caja chica, quincena actual, próximo pago, fondo de
 * emergencia y los gastos del periodo. Cada tarjeta carga y falla por su
 * cuenta: si una no llega, las demás se ven igual.
 *
 * No calcula nada de dinero: montos, niveles y fechas vienen de la API.
 */
@Component({
  selector: 'fz-inicio',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [ColonesPipe, Esqueleto, FechaCortaPipe, Grupo, Renglon, RouterLink, Tarjeta],
  templateUrl: './inicio.html',
  styleUrl: './inicio.scss',
})
export class Inicio {
  readonly #finanzas = inject(Finanzas);
  readonly #caja$ = this.#finanzas.caja().pipe(shareReplay(1));

  protected readonly hoy = fechaCorta(hoy());
  protected readonly caja = cargar(this.#caja$);
  protected readonly perfil = cargar(this.#finanzas.perfil());
  protected readonly cortes = cargar(this.#finanzas.cortesHoy());
  protected readonly quincena = cargar(this.#finanzas.quincena('actual'));
  protected readonly fondo = cargar(this.#finanzas.fondo());
  protected readonly gastos = cargar(
    this.#caja$.pipe(switchMap((c) => this.#finanzas.gastos(c.desde, c.hasta))),
  );

  protected readonly categorias = CATEGORIAS;
  protected readonly faltan = faltan;

  /** Lo que queda contra el tope, sólo para el largo de la barra. */
  protected readonly proporcionCaja = computed(() => {
    const caja = valor(this.caja());
    const tope = valor(this.perfil())?.tope_caja_chica;
    return caja && tope ? caja.queda / tope : undefined;
  });

  /** Los gastos del periodo, el más reciente arriba. */
  protected readonly gastosRecientes = computed<Gasto[]>(() =>
    [...(valor(this.gastos()) ?? [])].reverse(),
  );

  protected colorCaja(c: Caja): EstadoBarra {
    return COLOR_CAJA[c.nivel];
  }

  protected notaCaja(c: Caja): string {
    const periodo = `${fechaCorta(c.desde)} – ${fechaCorta(c.hasta)}`;
    return c.exceso > 0 ? `Te pasaste ${colones(c.exceso)} · ${periodo}` : periodo;
  }

  /** Pagos hechos contra el total, sólo para el largo de la barra. */
  protected avance(q: Quincena): number | undefined {
    return q.avance.total ? q.avance.hechos / q.avance.total : undefined;
  }
}
