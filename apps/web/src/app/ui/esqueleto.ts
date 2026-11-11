import { ChangeDetectionStrategy, Component, input } from '@angular/core';

/**
 * Bloque de carga con pulso. Se usa en lugar del contenido que todavía no
 * llegó; nunca un spinner a pantalla completa.
 */
@Component({
  selector: 'fz-esqueleto',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    'aria-hidden': 'true',
    '[style.height]': 'alto()',
  },
  template: '',
  styles: `
    :host {
      display: block;
      height: var(--alto-renglon);
      border-radius: var(--radio-grupo);
      background: var(--variante);
      animation: pulso var(--pulso) var(--curva) infinite;
    }
    @keyframes pulso {
      50% {
        opacity: 0.5;
      }
    }
  `,
})
export class Esqueleto {
  /** Un token de alto, por ejemplo `var(--alto-boton)`. */
  readonly alto = input<string>();
}
