import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { Icono } from './icono';

/**
 * Casilla para marcar un pago o un ingreso como hecho. El botón es el área
 * tocable (44 px) y adentro va el círculo de 24 px; el margen negativo lo
 * deja ocupar el lugar de un ícono de renglón (30 px).
 *
 * Uso: `<button fz-casilla slot="icono" type="button" [hecho]="…"
 * [attr.aria-label]="…" (click)="…"></button>`.
 */
@Component({
  selector: 'button[fz-casilla]',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [Icono],
  host: {
    role: 'checkbox',
    '[attr.aria-checked]': 'hecho()',
  },
  template: `
    <span class="circulo">
      @if (hecho()) {
        <fz-icono nombre="check" />
      }
    </span>
  `,
  styles: `
    :host {
      display: inline-grid;
      flex: none;
      place-items: center;
      width: var(--toque);
      height: var(--toque);
      margin: calc((var(--toque) - var(--icono-renglon)) / -2);
      padding: 0;
      border: 0;
      background: transparent;
      cursor: pointer;
    }
    .circulo {
      display: inline-grid;
      place-items: center;
      width: var(--marca);
      height: var(--marca);
      border: var(--linea) solid var(--borde);
      border-radius: 50%;
      color: var(--sobre);
      font-size: var(--margen);
      transition: background var(--rapido) var(--curva);
    }
    :host([aria-checked='true']) .circulo {
      border-color: var(--exito);
      background: var(--exito);
    }
    :host(:disabled) {
      opacity: 0.5;
      cursor: default;
    }
  `,
})
export class Casilla {
  readonly hecho = input(false);
}
