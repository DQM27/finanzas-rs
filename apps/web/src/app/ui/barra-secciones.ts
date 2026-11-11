import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { Icono, NombreIcono } from './icono';

export interface Seccion {
  ruta: string;
  etiqueta: string;
  icono: NombreIcono;
  /** Distintivo gris (por ejemplo, pagos pendientes). Nunca rojo. */
  distintivo?: number;
}

/**
 * Barra de secciones de vidrio, flotando sobre el borde inferior. Es la
 * navegación del celular; en la computadora la reemplaza la barra lateral.
 */
@Component({
  selector: 'fz-barra-secciones',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [Icono, RouterLink, RouterLinkActive],
  template: `
    <nav aria-label="Secciones">
      @for (s of secciones(); track s.ruta) {
        <a class="pestana" [routerLink]="s.ruta" routerLinkActive="elegida"
           [routerLinkActiveOptions]="{ exact: s.ruta === '/' }"
           ariaCurrentWhenActive="page" (click)="vibrar()">
          <span class="icono">
            <fz-icono [nombre]="s.icono" />
            @if (s.distintivo) {
              <span class="distintivo cifras">{{ s.distintivo }}</span>
            }
          </span>
          <span class="rotulo">{{ s.etiqueta }}</span>
        </a>
      }
    </nav>
  `,
  styles: `
    :host {
      position: fixed;
      z-index: 10;
      right: var(--margen);
      bottom: calc(env(safe-area-inset-bottom) + var(--espacio));
      left: var(--margen);
      max-width: var(--ancho-maximo);
      margin: 0 auto;
    }
    nav {
      display: flex;
      height: var(--alto-barra);
      padding: var(--hueco-barra);
      border: var(--linea) solid var(--vidrio);
      border-radius: var(--radio-barra);
      background: color-mix(in srgb, var(--tarjeta) var(--velo), transparent);
      backdrop-filter: blur(var(--desenfoque)) saturate(1.6);
      -webkit-backdrop-filter: blur(var(--desenfoque)) saturate(1.6);
    }
    .pestana {
      display: flex;
      flex: 1;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: var(--hueco-segmentado);
      border-radius: calc(var(--radio-barra) - var(--hueco-barra));
      color: var(--texto-2);
      text-decoration: none;
      transition:
        background var(--rapido) var(--curva),
        color var(--rapido) var(--curva);
    }
    .pestana.elegida {
      background: var(--pildora);
      color: var(--acento);
    }
    .icono {
      position: relative;
      display: inline-flex;
      font-size: var(--icono-barra);
    }
    .distintivo {
      position: absolute;
      top: calc(var(--chico) * -1);
      right: calc(var(--espacio) * -1);
      min-width: var(--margen);
      padding: 0 var(--chico);
      border-radius: var(--margen);
      background: var(--texto-2);
      color: var(--tarjeta);
      font: var(--etiqueta-chica);
      text-align: center;
    }
    .rotulo {
      font: var(--etiqueta-chica);
    }
  `,
})
export class BarraSecciones {
  readonly secciones = input.required<readonly Seccion[]>();

  /** Respuesta háptica donde el sistema la ofrece (Android). */
  protected vibrar(): void {
    navigator.vibrate?.(10);
  }
}
