import { ChangeDetectionStrategy, Component, input } from '@angular/core';

/**
 * Grupo de formulario al estilo de Ajustes de iOS: encabezado afuera, los
 * renglones (`fz-renglon`) adentro y una nota al pie opcional.
 */
@Component({
  selector: 'fz-grupo',
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (titulo()) {
      <h2 class="titulo">{{ titulo() }}</h2>
    }
    <div class="grupo" role="group" [attr.aria-label]="titulo() || null">
      <ng-content />
    </div>
    @if (nota()) {
      <p class="nota" [class.error]="error()" [attr.role]="error() ? 'alert' : null">
        {{ nota() }}
      </p>
    }
  `,
  styles: `
    :host {
      display: block;
      margin-bottom: calc(var(--espacio) * 2);
    }
    .titulo {
      margin: 0 0 var(--chico);
      padding-inline: var(--margen);
      font: var(--etiqueta);
      // Un formulario de sección lo pinta de su tono con --titulo-grupo.
      color: var(--titulo-grupo, var(--texto-2));
      text-transform: uppercase;
    }
    .grupo {
      overflow: hidden;
      border-radius: var(--radio-grupo);
      background: var(--tarjeta);
    }
    .nota {
      margin: var(--chico) 0 0;
      padding-inline: var(--margen);
      font: var(--cuerpo-chico);
      color: var(--texto-2);
    }
    .nota.error {
      color: var(--error);
      font-weight: 600;
    }
  `,
})
export class Grupo {
  readonly titulo = input<string>();
  readonly nota = input<string>();
  /** La nota es un error de validación. */
  readonly error = input(false);
}
