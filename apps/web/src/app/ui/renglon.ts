import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { Icono } from './icono';

/**
 * Un renglón de `fz-grupo`.
 *
 * - Con `etiqueta`: renglón de campo. La etiqueta va en su columna y el
 *   contenido (un campo sin marco o un valor) a la derecha.
 * - Sin `etiqueta`: renglón libre. El contenido ocupa todo el ancho y lo que
 *   lleve `slot="valor"` va a la derecha.
 * - `[slot=icono]` pone el ícono de sección a la izquierda; el divisor se corre.
 * - `interactivo`: se puede tocar (con teclado también) y marca la presión.
 * - `chevron`: es un selector; el valor va en `acento` con `›`.
 */
@Component({
  selector: 'fz-renglon',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [Icono],
  host: {
    '[class.interactivo]': 'interactivo() || chevron()',
    '[class.con-icono]': 'conIcono()',
    '[attr.role]': "interactivo() || chevron() ? 'button' : null",
    '[attr.tabindex]': 'interactivo() || chevron() ? 0 : null',
    '(keydown.enter)': 'tocar($event)',
    '(keydown.space)': 'tocar($event)',
  },
  template: `
    <ng-content select="[slot=icono]" />
    @if (etiqueta()) {
      <span class="etiqueta">{{ etiqueta() }}</span>
    }
    <div class="contenido" [class.a-la-derecha]="!!etiqueta()">
      <ng-content />
    </div>
    <div class="valor" [class.selector]="chevron()">
      <ng-content select="[slot=valor]" />
      @if (chevron()) {
        <fz-icono nombre="chevron" class="chevron" />
      }
    </div>
  `,
  styles: `
    :host {
      position: relative;
      display: flex;
      align-items: center;
      gap: var(--espacio);
      min-height: var(--alto-renglon);
      padding: var(--chico) var(--margen);
      outline-offset: calc(var(--chico) * -1);
    }
    :host:not(:first-child)::before {
      content: '';
      position: absolute;
      top: 0;
      right: 0;
      left: var(--sangria-divisor);
      border-top: var(--divisor) solid color-mix(in srgb, var(--borde) 45%, transparent);
    }
    :host(.con-icono):not(:first-child)::before {
      left: var(--sangria-divisor-icono);
    }
    :host(.interactivo) {
      cursor: pointer;
      transition: background var(--rapido) var(--curva);
    }
    :host(.interactivo:active) {
      background: var(--pildora);
    }
    .etiqueta {
      flex: none;
      width: var(--columna-etiqueta);
    }
    .contenido {
      flex: 1;
      min-width: 0;
    }
    .contenido.a-la-derecha {
      text-align: right;
    }
    .valor {
      display: flex;
      align-items: center;
      gap: var(--chico);
    }
    .valor:empty {
      display: none;
    }
    .valor.selector {
      color: var(--acento);
    }
    .chevron {
      color: var(--texto-2);
    }
  `,
})
export class Renglon {
  readonly etiqueta = input<string>();
  readonly interactivo = input(false);
  readonly chevron = input(false);
  /** El renglón lleva ícono de sección: el divisor arranca después. */
  readonly conIcono = input(false);

  /** Enter o espacio sobre un renglón tocable hacen lo mismo que un toque. */
  protected tocar(evento: Event): void {
    if (!(this.interactivo() || this.chevron())) return;
    if (evento.target !== evento.currentTarget) return;
    evento.preventDefault();
    (evento.currentTarget as HTMLElement).click();
  }
}
