import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import type { Caja } from '../nucleo/api';
import { FallaApi, Finanzas } from '../nucleo/finanzas';
import { fechaCorta } from '../nucleo/formato';
import { Esqueleto } from '../ui/esqueleto';
import { Tarjeta } from '../ui/tarjeta';

/**
 * Inicio, todavía provisional: la caja chica. La pantalla completa (caja con
 * su barra, quincena actual, próximos cortes) viene en la parte siguiente.
 */
@Component({
  selector: 'fz-inicio',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [Tarjeta, Esqueleto],
  template: `
    <main class="pantalla">
      <h1>Finanzas</h1>
      @if (caja(); as c) {
        <fz-tarjeta clave="Caja chica" seccion="gasto" [monto]="c.queda" [nota]="periodo(c)" />
      } @else if (falla(); as mensaje) {
        <fz-tarjeta clave="Caja chica" [tenida]="true" tono="error">
          <p class="falla" role="alert">{{ mensaje }}</p>
        </fz-tarjeta>
      } @else {
        <fz-esqueleto alto="var(--alto-renglon)" />
      }
    </main>
  `,
  styles: `
    .falla {
      margin: var(--chico) 0 0;
      color: var(--error);
    }
  `,
})
export class Inicio {
  readonly #finanzas = inject(Finanzas);
  protected readonly caja = signal<Caja | null>(null);
  protected readonly falla = signal<string | null>(null);

  constructor() {
    this.#finanzas.caja().subscribe({
      next: (caja) => this.caja.set(caja),
      error: (e: FallaApi) => this.falla.set(e.message),
    });
  }

  protected periodo(c: Caja): string {
    const exceso = c.exceso > 0 ? ' · te pasaste' : '';
    return `${fechaCorta(c.desde)} – ${fechaCorta(c.hasta)}${exceso}`;
  }
}
