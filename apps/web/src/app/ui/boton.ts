import { ChangeDetectionStrategy, Component, input } from '@angular/core';

/**
 * Botón principal: ancho completo, sobre el relleno de la sección
 * (`data-seccion` en un antecesor) o el relleno general. Mientras
 * `guardando`, queda deshabilitado y dice «Guardando…».
 *
 * Uso: `<button fz-boton type="submit" [guardando]="…">Guardar</button>`.
 */
@Component({
  selector: 'button[fz-boton]',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    '[disabled]': 'guardando() || deshabilitado()',
    '[attr.aria-busy]': 'guardando()',
  },
  template: `
    @if (guardando()) {
      Guardando…
    } @else {
      <ng-content />
    }
  `,
  styles: `
    :host {
      display: block;
      width: 100%;
      min-height: var(--alto-boton);
      padding: 0 var(--margen);
      border: 0;
      border-radius: var(--radio-grupo);
      background: var(--seccion-relleno, var(--relleno));
      color: var(--sobre);
      font: var(--titulo-medio);
      cursor: pointer;
      transition: opacity var(--rapido) var(--curva);
    }
    :host(:active) {
      opacity: 0.85;
    }
    :host(:disabled) {
      opacity: 0.5;
      cursor: default;
    }
  `,
})
export class Boton {
  readonly guardando = input(false);
  readonly deshabilitado = input(false);
}
