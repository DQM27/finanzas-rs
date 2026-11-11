import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { ActivatedRoute } from '@angular/router';
import { map } from 'rxjs';
import { Grupo } from '../ui/grupo';
import { Renglon } from '../ui/renglon';

/** Lugar de una pantalla que todavía no existe. */
@Component({
  selector: 'fz-pronto',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [Grupo, Renglon],
  template: `
    <main class="pantalla">
      <h1>{{ titulo() }}</h1>
      <fz-grupo nota="Esta pantalla viene en las próximas partes.">
        <fz-renglon>En construcción</fz-renglon>
      </fz-grupo>
    </main>
  `,
})
export class Pronto {
  protected readonly titulo = toSignal(
    inject(ActivatedRoute).data.pipe(map((d) => String(d['titulo'] ?? ''))),
    { initialValue: '' },
  );
}
