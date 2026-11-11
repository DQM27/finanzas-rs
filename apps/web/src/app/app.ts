import { Component, inject, signal } from '@angular/core';
import type { Caja } from './nucleo/api';
import { FallaApi, Finanzas } from './nucleo/finanzas';
import { ColonesPipe, FechaCortaPipe } from './nucleo/formato';

/**
 * Pantalla provisional: comprueba de punta a punta que la página sale del
 * Worker y que la sesión de Access llega a la API. La reemplaza Inicio.
 */
@Component({
  imports: [ColonesPipe, FechaCortaPipe],
  selector: 'app-root',
  styleUrl: './app.scss',
  templateUrl: './app.html',
})
export class App {
  readonly #finanzas = inject(Finanzas);
  protected readonly caja = signal<Caja | null>(null);
  protected readonly falla = signal<string | null>(null);

  constructor() {
    this.#finanzas.caja().subscribe({
      next: (caja) => this.caja.set(caja),
      error: (e: FallaApi) => this.falla.set(e.message),
    });
  }
}
