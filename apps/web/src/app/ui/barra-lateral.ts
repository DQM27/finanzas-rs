import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { Icono, NombreIcono } from './icono';

export interface EnlaceLateral {
  ruta: string;
  etiqueta: string;
  icono: NombreIcono;
}

export interface GrupoLateral {
  titulo?: string;
  enlaces: readonly EnlaceLateral[];
}

/**
 * Barra lateral fija de la versión de computadora (docs/diseno.md §0). Reemplaza
 * a la barra de secciones del celular.
 */
@Component({
  selector: 'fz-barra-lateral',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [Icono, RouterLink, RouterLinkActive],
  template: `
    <p class="marca">
      <span class="logo" aria-hidden="true">₡</span>
      Finanzas
    </p>
    <nav aria-label="Secciones">
      @for (g of grupos(); track $index) {
        @if (g.titulo) {
          <p class="titulo">{{ g.titulo }}</p>
        }
        @for (e of g.enlaces; track e.ruta) {
          <a class="enlace" [routerLink]="e.ruta" routerLinkActive="elegido"
             [routerLinkActiveOptions]="{ exact: e.ruta === '/' }" ariaCurrentWhenActive="page">
            <fz-icono [nombre]="e.icono" />
            {{ e.etiqueta }}
          </a>
        }
      }
    </nav>
  `,
  styles: `
    :host {
      position: fixed;
      inset: 0 auto 0 0;
      display: flex;
      flex-direction: column;
      gap: var(--espacio);
      width: var(--ancho-lateral);
      padding: var(--margen) var(--espacio);
      overflow-y: auto;
      background: var(--tarjeta);
    }
    .marca {
      display: flex;
      align-items: center;
      gap: var(--espacio);
      margin: 0 0 var(--espacio);
      padding-inline: var(--chico);
      font: var(--titulo-medio);
    }
    .logo {
      display: inline-grid;
      place-items: center;
      width: var(--icono-renglon);
      height: var(--icono-renglon);
      border-radius: var(--radio-icono);
      background: var(--relleno);
      color: var(--sobre);
      font: var(--titulo-medio);
    }
    nav {
      display: grid;
      gap: var(--hueco-segmentado);
    }
    .titulo {
      margin: var(--espacio) 0 var(--chico);
      padding-inline: var(--espacio);
      font: var(--etiqueta);
      color: var(--texto-2);
      text-transform: uppercase;
    }
    .enlace {
      display: flex;
      align-items: center;
      gap: var(--espacio);
      min-height: var(--toque);
      padding: 0 var(--espacio);
      border-radius: var(--radio-icono);
      color: var(--texto);
      font: var(--cuerpo);
      font-size: var(--margen);
      text-decoration: none;
    }
    .enlace:hover {
      background: var(--pildora);
    }
    .enlace.elegido {
      background: var(--pildora);
      color: var(--acento);
      font-weight: 600;
    }
    fz-icono {
      font-size: var(--icono-barra);
    }
  `,
})
export class BarraLateral {
  readonly grupos = input.required<readonly GrupoLateral[]>();
}
