import { ChangeDetectionStrategy, Component, computed, input } from '@angular/core';

/** Trazos de 24 × 24, en línea, que toman el color del texto. */
const TRAZOS = {
  inicio: 'M3 10.5 12 3l9 7.5V20a1 1 0 0 1-1 1h-5v-6h-6v6H4a1 1 0 0 1-1-1z',
  quincena: 'M4 6a1 1 0 0 1 1-1h14a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1zM4 10h16M8 3v4M16 3v4',
  mes: 'M4 6a1 1 0 0 1 1-1h14a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1zM4 10h16M12 10v10',
  anotar: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM12 8v8M8 12h8',
  mas: 'M5 12h.01M12 12h.01M19 12h.01',
  chevron: 'm9 6 6 6-6 6',
  check: 'm5 12.5 4.5 4.5L19 7.5',
  quitar: 'M6 6l12 12M18 6 6 18',
  buscar: 'M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zM20 20l-4-4',
  filtro: 'M4 5h16l-6 8v6l-4-2v-4z',
  anchos: 'M4 12h16M7 9l-3 3 3 3M17 9l3 3-3 3',
  restablecer: 'M4 12a8 8 0 1 0 2.3-5.7M4 4v4h4',
  columnas: 'M4 5h16v14H4zM9.5 5v14M14.5 5v14',
  exportar: 'M12 4v11M7.5 10.5 12 15l4.5-4.5M5 19h14',
  extras: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM12 7v5l3 2',
  gastos: 'M6 3h12v18l-3-2-3 2-3-2-3 2zM9 8h6M9 12h6',
  fondo: 'M12 3 5 6v6c0 4.5 3 7.5 7 9 4-1.5 7-4.5 7-9V6z',
  ajustes: 'M4 7h10M18 7h2M4 17h4M12 17h8M14 5v4M8 15v4',
  deshacer:'M9 14 4 9l5-5M4 9h10.5a5.5 5.5 0 0 1 0 11H11',
} as const;

export type NombreIcono = keyof typeof TRAZOS;

/**
 * Un ícono del sistema. Es decorativo: quien lo usa sin texto al lado pone el
 * `aria-label` en el control que lo contiene.
 */
@Component({
  selector: 'fz-icono',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { 'aria-hidden': 'true' },
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
         stroke-linecap="round" stroke-linejoin="round">
      <path [attr.d]="trazo()" />
    </svg>
  `,
  styles: `
    :host {
      display: inline-flex;
      width: 1em;
      height: 1em;
    }
    svg {
      width: 100%;
      height: 100%;
    }
  `,
})
export class Icono {
  readonly nombre = input.required<NombreIcono>();
  protected readonly trazo = computed(() => TRAZOS[this.nombre()]);
}
