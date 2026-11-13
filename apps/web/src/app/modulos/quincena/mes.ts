import { ChangeDetectionStrategy, Component, computed, inject, signal } from '@angular/core';
import { toObservable } from '@angular/core/rxjs-interop';
import { RouterLink } from '@angular/router';
import { BehaviorSubject, combineLatest } from 'rxjs';
import type { Lado, Linea, Quincena } from '../../nucleo/api';
import { recargable, valor } from '../../nucleo/carga';
import { FallaApi, Finanzas } from '../../nucleo/finanzas';
import { ColonesPipe, FechaCortaPipe } from '../../nucleo/formato';
import { Avisos } from '../../ui/aviso';
import { Casilla } from '../../ui/casilla';
import { Esqueleto } from '../../ui/esqueleto';
import { Grupo } from '../../ui/grupo';
import { Icono } from '../../ui/icono';
import { Renglon } from '../../ui/renglon';
import { Tarjeta } from '../../ui/tarjeta';

/** El mes `AAAA-MM` que queda `pasos` meses antes (negativo) o después. */
export function mesVecino(mes: string, pasos: number): string {
  const [a, m] = mes.split('-').map(Number);
  const indice = a * 12 + (m - 1) + pasos;
  const anio = Math.floor(indice / 12);
  return `${anio}-${String((indice % 12) + 1).padStart(2, '0')}`;
}

/** «noviembre 2026». */
export function nombreDelMes(mes: string): string {
  const [a, m] = mes.split('-').map(Number);
  return new Intl.DateTimeFormat('es-CR', { month: 'long', year: 'numeric' })
    .format(new Date(a, m - 1, 1))
    .replace(' de ', ' ');
}

/**
 * El mes en la computadora, como la hoja: Q1 y Q2 lado a lado. En cada una
 * entran solos el salario y las extras de ese pago (Q7); los otros ingresos
 * y los pagos se marcan aquí y se arman en Quincena. Todos los totales los
 * calcula el dominio (Q1, Q2, Q8).
 */
@Component({
  selector: 'fz-mes',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    Casilla,
    ColonesPipe,
    Esqueleto,
    FechaCortaPipe,
    Grupo,
    Icono,
    Renglon,
    RouterLink,
    Tarjeta,
  ],
  templateUrl: './mes.html',
  styleUrl: './mes.scss',
})
export class Mes {
  readonly #finanzas = inject(Finanzas);
  readonly #avisos = inject(Avisos);
  readonly #recarga = new BehaviorSubject<void>(undefined);

  /** El mes pedido: `actual` hasta que se navega. */
  protected readonly pedido = signal<string>('actual');
  protected readonly mes = recargable(
    combineLatest([toObservable(this.pedido), this.#recarga]),
    ([mes]) => this.#finanzas.mes(mes),
  );
  /** El mes que se está viendo (`AAAA-MM`), cuando ya llegó. */
  protected readonly visto = computed(() => valor(this.mes())?.mes ?? null);
  protected readonly titulo = computed(() => {
    const m = this.visto();
    return m ? nombreDelMes(m) : '';
  });
  /** La línea que se está marcando, para no tocarla dos veces. */
  protected readonly marcando = signal<string | null>(null);

  protected mover(pasos: number): void {
    const m = this.visto();
    if (m) this.pedido.set(mesVecino(m, pasos));
  }

  protected marcar(q: Quincena, lado: Lado, indice: number, linea: Linea): void {
    const clave = `${q.pago}-${lado}-${indice}`;
    if (this.marcando()) return;
    this.marcando.set(clave);
    this.#finanzas.marcar(q.pago, { lado, indice, hecho: !linea.hecho }).subscribe({
      next: () => {
        this.marcando.set(null);
        this.#recarga.next();
      },
      error: (e: FallaApi) => {
        this.marcando.set(null);
        this.#avisos.error(e.message);
      },
    });
  }
}
