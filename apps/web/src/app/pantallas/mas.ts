import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { Router } from '@angular/router';
import { Grupo } from '../ui/grupo';
import { Renglon } from '../ui/renglon';

/**
 * «Más»: lo que no entra en la barra. Aquí van a quedar, en modo consulta,
 * las pantallas de la computadora (proyección, deudas, fondo, calendario).
 */
@Component({
  selector: 'fz-mas',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [Grupo, Renglon],
  template: `
    <main class="pantalla">
      <h1>Más</h1>
      <fz-grupo titulo="Desarrollo">
        <fz-renglon [chevron]="true" (click)="ir('/muestrario')">Muestrario de componentes</fz-renglon>
      </fz-grupo>
    </main>
  `,
})
export class Mas {
  readonly #router = inject(Router);

  protected ir(ruta: string): void {
    void this.#router.navigateByUrl(ruta);
  }
}
