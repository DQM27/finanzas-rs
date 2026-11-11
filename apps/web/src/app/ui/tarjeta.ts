import { ChangeDetectionStrategy, Component, computed, input } from '@angular/core';
import { ColonesPipe } from '../nucleo/formato';

/** El color de una barra de progreso, según cuánto queda. */
export type EstadoBarra = 'sana' | 'baja' | 'agotada' | 'acento';

/** Arriba del 30 % es sana, al 30 % o menos baja, en 0 agotada. */
export function estadoBarra(proporcion: number): Exclude<EstadoBarra, 'acento'> {
  if (proporcion <= 0) return 'agotada';
  if (proporcion <= 0.3) return 'baja';
  return 'sana';
}

/**
 * Tarjeta de resumen: clave, cifra y, opcional, una barra con una nota.
 *
 * - `monto` se muestra con la tubería `colones`; un negativo va en `error`
 *   además de llevar el signo `−`.
 * - `proporcion` (0 a 1) dibuja la barra. `crece` la pinta en `acento`: para
 *   el fondo de emergencia, crecer no es alarma.
 * - `tenida` con `tono` (`error`, `exito` o una sección) la vuelve una tarjeta
 *   teñida: fondo suave del tono y una franja a la izquierda.
 */
@Component({
  selector: 'fz-tarjeta',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [ColonesPipe],
  host: {
    '[class.tenida]': 'tenida()',
    '[attr.data-tono]': 'tono() ?? null',
    '[attr.data-seccion]': 'seccion() ?? null',
  },
  template: `
    <p class="clave">{{ clave() }}</p>
    @if (monto() !== undefined) {
      <p class="cifra cifras" [class.negativo]="(monto() ?? 0) < 0">{{ monto() | colones }}</p>
    }
    <ng-content />
    @if (proporcion() !== undefined) {
      <div class="pista" role="progressbar" aria-valuemin="0" aria-valuemax="100"
           [attr.aria-valuenow]="porcentaje()" [attr.aria-label]="clave()">
        <div class="barra" [attr.data-estado]="color()" [style.width.%]="porcentaje()"></div>
      </div>
    }
    @if (nota()) {
      <p class="nota">{{ nota() }}</p>
    }
  `,
  styles: `
    :host {
      display: block;
      padding: var(--margen);
      border-radius: var(--radio-tarjeta);
      background: var(--tarjeta);
    }
    :host(.tenida) {
      --tono: var(--seccion, var(--acento));
      border-left: var(--franja) solid var(--tono);
      background: color-mix(in srgb, var(--tono) 12%, var(--tarjeta));
    }
    :host([data-tono='error']) {
      --tono: var(--error);
    }
    :host([data-tono='exito']) {
      --tono: var(--exito);
    }
    p {
      margin: 0;
    }
    .clave {
      font: var(--etiqueta);
      color: var(--texto-2);
      text-transform: uppercase;
    }
    .cifra {
      margin-top: var(--chico);
      font: var(--cifra);
    }
    .cifra.negativo {
      color: var(--error);
    }
    .pista {
      overflow: hidden;
      height: var(--alto-progreso);
      margin-top: var(--espacio);
      border-radius: var(--alto-progreso);
      background: var(--variante);
    }
    .barra {
      height: 100%;
      border-radius: inherit;
      transition: width var(--lento) var(--curva);
    }
    .barra[data-estado='sana'] {
      background: var(--exito);
    }
    .barra[data-estado='baja'] {
      background: var(--advertencia);
    }
    .barra[data-estado='agotada'] {
      background: var(--error);
    }
    .barra[data-estado='acento'] {
      background: var(--acento);
    }
    .nota {
      margin-top: var(--chico);
      font: var(--cuerpo-chico);
      color: var(--texto-2);
    }
  `,
})
export class Tarjeta {
  readonly clave = input.required<string>();
  readonly monto = input<number | null>();
  /** De 0 a 1. Sin valor no hay barra. */
  readonly proporcion = input<number>();
  /** La barra va en `acento` sea cual sea el valor. */
  readonly crece = input(false);
  /**
   * El color de la barra ya decidido. Cuando el dominio da el nivel (como el
   * de la caja chica), manda él y la tarjeta no calcula nada.
   */
  readonly estado = input<EstadoBarra>();
  readonly nota = input<string>();
  readonly tenida = input(false);
  readonly tono = input<'error' | 'exito'>();
  readonly seccion = input<string>();

  protected readonly porcentaje = computed(() =>
    Math.round(Math.min(1, Math.max(0, this.proporcion() ?? 0)) * 100),
  );
  protected readonly color = computed<EstadoBarra>(
    () => this.estado() ?? (this.crece() ? 'acento' : estadoBarra(this.proporcion() ?? 0)),
  );
}
