import { ChangeDetectionStrategy, Component } from '@angular/core';
import type { ICellRendererAngularComp } from 'ag-grid-angular';
import type { ICellRendererParams } from 'ag-grid-community';
import { Icono } from '../../ui/icono';

/** Lo que la columna le pasa a la celda (`cellRendererParams`). */
export interface ParametrosDeshacer<T> {
  /** Qué hacer con la fila; quien la usa pide la confirmación. */
  deshacer: (fila: T) => void;
  /** Cómo nombrar la fila para el lector de pantalla. */
  nombre: (fila: T) => string;
}

/** Botón de deshacer al final de una fila. */
@Component({
  selector: 'fz-celda-deshacer',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [Icono],
  template: `
    <button type="button" [attr.aria-label]="'Deshacer ' + nombre" title="Deshacer" (click)="tocar()">
      <fz-icono nombre="deshacer" />
    </button>
  `,
  styles: `
    button {
      display: inline-grid;
      place-items: center;
      width: var(--alto-herramienta);
      height: calc(var(--alto-herramienta) - var(--chico));
      padding: 0;
      border: 0;
      border-radius: var(--radio-icono);
      background: transparent;
      color: var(--texto-2);
      font-size: var(--margen);
      cursor: pointer;
    }
    button:hover {
      background: var(--pildora);
      color: var(--error);
    }
  `,
})
export class CeldaDeshacer<T> implements ICellRendererAngularComp {
  protected nombre = '';
  #fila: T | undefined;
  #params: ParametrosDeshacer<T> | undefined;

  agInit(p: ICellRendererParams<T> & ParametrosDeshacer<T>): void {
    this.#fila = p.data;
    this.#params = p;
    this.nombre = p.data ? p.nombre(p.data) : '';
  }

  refresh(): boolean {
    return false;
  }

  protected tocar(): void {
    if (this.#fila !== undefined) this.#params?.deshacer(this.#fila);
  }
}
