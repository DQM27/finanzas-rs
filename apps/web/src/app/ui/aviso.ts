import { ChangeDetectionStrategy, Component, Injectable, inject, signal } from '@angular/core';

export type TipoAviso = 'exito' | 'error';

export interface AvisoVisible {
  texto: string;
  tipo: TipoAviso;
  /** Cambia en cada aviso, aunque el texto se repita, para reiniciar la animación. */
  vez: number;
}

/** Cuánto dura un aviso en pantalla (`--duracion-aviso`). */
export const DURACION_AVISO_MS = 2400;

/**
 * Los avisos de la app. Uno a la vez: uno nuevo reemplaza al anterior.
 * Lo muestra `fz-aviso`, que va una sola vez en la raíz.
 */
@Injectable({ providedIn: 'root' })
export class Avisos {
  readonly #actual = signal<AvisoVisible | null>(null);
  readonly actual = this.#actual.asReadonly();
  #reloj: ReturnType<typeof setTimeout> | undefined;
  #vez = 0;

  exito(texto: string): void {
    this.#mostrar(texto, 'exito');
  }

  error(texto: string): void {
    this.#mostrar(texto, 'error');
  }

  cerrar(): void {
    clearTimeout(this.#reloj);
    this.#actual.set(null);
  }

  #mostrar(texto: string, tipo: TipoAviso): void {
    clearTimeout(this.#reloj);
    this.#actual.set({ texto, tipo, vez: ++this.#vez });
    this.#reloj = setTimeout(() => this.#actual.set(null), DURACION_AVISO_MS);
  }
}

/** Baja desde arriba, bajo el área segura. Tocarlo lo cierra. */
@Component({
  selector: 'fz-aviso',
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="zona" aria-live="polite">
      @if (avisos.actual(); as aviso) {
        @for (a of [aviso]; track a.vez) {
          <div class="aviso" [attr.data-tipo]="a.tipo"
               [attr.role]="a.tipo === 'error' ? 'alert' : 'status'"
               (click)="avisos.cerrar()">
            {{ a.texto }}
          </div>
        }
      }
    </div>
  `,
  styles: `
    .zona {
      position: fixed;
      z-index: 20;
      top: calc(env(safe-area-inset-top) + var(--espacio));
      right: var(--margen);
      left: var(--margen);
      max-width: var(--ancho-maximo);
      margin: 0 auto;
      pointer-events: none;
    }
    .aviso {
      padding: var(--espacio) var(--margen);
      border-radius: var(--radio-control);
      color: var(--sobre);
      font: var(--cuerpo);
      font-weight: 600;
      pointer-events: auto;
      animation: bajar var(--lento) var(--curva);
    }
    .aviso[data-tipo='exito'] {
      background: var(--relleno-exito);
    }
    .aviso[data-tipo='error'] {
      background: var(--relleno-error);
    }
    @keyframes bajar {
      from {
        transform: translateY(-120%);
        opacity: 0;
      }
    }
  `,
})
export class Aviso {
  protected readonly avisos = inject(Avisos);
}
