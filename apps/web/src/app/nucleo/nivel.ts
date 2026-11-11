import { Injectable, signal } from '@angular/core';

/** Desde este ancho se usa la versión de computadora (docs/diseno.md §0). */
export const ANCHO_COMPUTADORA = '(min-width: 900px)';

/**
 * En qué nivel está la app: celular o computadora. Decide el ancho de la
 * ventana, no el dispositivo, y es el único lugar que lo pregunta.
 */
@Injectable({ providedIn: 'root' })
export class Nivel {
  readonly #consulta = typeof matchMedia === 'function' ? matchMedia(ANCHO_COMPUTADORA) : null;
  readonly #computadora = signal(this.#consulta?.matches ?? false);
  /** `true` en la versión de computadora. */
  readonly computadora = this.#computadora.asReadonly();

  constructor() {
    this.#consulta?.addEventListener('change', (e) => this.#computadora.set(e.matches));
  }
}
