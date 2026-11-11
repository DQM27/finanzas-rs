import { ChangeDetectionStrategy, Component, input, model } from '@angular/core';

export interface Segmento<T extends string> {
  valor: T;
  etiqueta: string;
}

/**
 * Selector segmentado (por ejemplo Q1/Q2). Se comporta como un grupo de
 * opciones: las flechas cambian de segmento.
 */
@Component({
  selector: 'fz-segmentado',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    role: 'radiogroup',
    '[attr.aria-label]': 'etiqueta()',
    '(keydown.arrowright)': 'mover(1, $event)',
    '(keydown.arrowleft)': 'mover(-1, $event)',
  },
  template: `
    @for (s of segmentos(); track s.valor) {
      <button type="button" role="radio" class="segmento"
              [attr.aria-checked]="s.valor === valor()"
              [attr.tabindex]="s.valor === valor() ? 0 : -1"
              (click)="valor.set(s.valor)">
        {{ s.etiqueta }}
      </button>
    }
  `,
  styles: `
    :host {
      display: flex;
      padding: var(--hueco-segmentado);
      border-radius: var(--radio-segmentado);
      background: var(--variante);
    }
    .segmento {
      flex: 1;
      min-height: var(--toque);
      padding: 0 var(--espacio);
      border: 0;
      border-radius: calc(var(--radio-segmentado) - var(--hueco-segmentado));
      background: transparent;
      font: var(--cuerpo-chico);
      font-weight: 600;
      cursor: pointer;
      transition: background var(--rapido) var(--curva);
    }
    .segmento[aria-checked='true'] {
      background: var(--realce);
      box-shadow: var(--sombra-segmento);
    }
  `,
})
export class Segmentado<T extends string> {
  readonly segmentos = input.required<readonly Segmento<T>[]>();
  readonly etiqueta = input.required<string>();
  readonly valor = model.required<T>();

  protected mover(paso: number, evento: Event): void {
    const lista = this.segmentos();
    const actual = lista.findIndex((s) => s.valor === this.valor());
    const siguiente = lista[(actual + paso + lista.length) % lista.length];
    if (!siguiente) return;
    evento.preventDefault();
    this.valor.set(siguiente.valor);
    const host = evento.currentTarget as HTMLElement;
    queueMicrotask(() =>
      host.querySelector<HTMLButtonElement>('[aria-checked="true"]')?.focus(),
    );
  }
}
