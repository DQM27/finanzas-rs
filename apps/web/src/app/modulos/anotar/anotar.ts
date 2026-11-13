import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Observable } from 'rxjs';
import type { Categoria, Origen, TipoExtra } from '../../nucleo/api';
import { CATEGORIAS, ORIGENES, TIPOS_EXTRA } from '../../nucleo/etiquetas';
import { FallaApi, Finanzas } from '../../nucleo/finanzas';
import { colones, fechaCorta, hoy, leerMonto } from '../../nucleo/formato';
import { Avisos } from '../../ui/aviso';
import { Boton } from '../../ui/boton';
import { Grupo } from '../../ui/grupo';
import { Icono } from '../../ui/icono';
import { Renglon } from '../../ui/renglon';
import { Segmentado, Segmento } from '../../ui/segmentado';

type Que = 'gasto' | 'extra';

/**
 * Anotar un gasto o una extra. Lo que se escribe va tal cual a la API: el
 * dominio valida (monto mayor que cero, concepto, tipo «Otra» con monto) y
 * su mensaje se muestra debajo del formulario.
 */
@Component({
  selector: 'fz-anotar',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [Boton, FormsModule, Grupo, Icono, Renglon, Segmentado],
  templateUrl: './anotar.html',
  styleUrl: './anotar.scss',
})
export class Anotar {
  readonly #finanzas = inject(Finanzas);
  readonly #avisos = inject(Avisos);

  protected readonly opciones: readonly Segmento<Que>[] = [
    { valor: 'gasto', etiqueta: 'Gasto' },
    { valor: 'extra', etiqueta: 'Extra' },
  ];
  protected readonly que = signal<Que>('gasto');
  protected readonly guardando = signal(false);
  protected readonly falla = signal<string | null>(null);

  protected readonly categorias = Object.entries(CATEGORIAS) as [Categoria, string][];
  protected readonly origenes = Object.entries(ORIGENES) as [Origen, string][];
  protected readonly tipos = Object.entries(TIPOS_EXTRA) as [TipoExtra, string][];

  // Gasto.
  protected readonly fechaGasto = signal(hoy());
  protected readonly concepto = signal('');
  protected readonly montoGasto = signal('');
  protected readonly categoria = signal<Categoria>('COMIDA');
  protected readonly origen = signal<Origen>('CUENTA_GASTOS');

  // Extra.
  protected readonly fechaExtra = signal(hoy());
  protected readonly tipo = signal<TipoExtra>('NOCTURNA');
  protected readonly montoExtra = signal('');

  protected cambiar(que: Que): void {
    this.que.set(que);
    this.falla.set(null);
  }

  protected guardarGasto(): void {
    this.#guardar(
      this.#finanzas.anotarGasto({
        fecha: this.fechaGasto(),
        concepto: this.concepto(),
        monto: leerMonto(this.montoGasto()) ?? 0,
        categoria: this.categoria(),
        origen: this.origen(),
      }),
      (r) => {
        this.concepto.set('');
        this.montoGasto.set('');
        return `Gasto guardado · quedan ${colones(r.caja.queda)} en la caja chica`;
      },
    );
  }

  protected guardarExtra(): void {
    this.#guardar(
      this.#finanzas.anotarExtra({
        fecha: this.fechaExtra(),
        tipo: this.tipo(),
        monto: leerMonto(this.montoExtra()),
      }),
      (r) => {
        this.montoExtra.set('');
        const pago = r.fecha_pago ? `, se paga el ${fechaCorta(r.fecha_pago)}` : '';
        return `Extra guardada · ${colones(r.neto)} neto${pago}`;
      },
    );
  }

  /** Manda el pedido; si sale bien avisa con `listo`, si no deja el mensaje en la nota. */
  #guardar<T>(pedido: Observable<T>, listo: (r: T) => string): void {
    if (this.guardando()) return;
    this.guardando.set(true);
    this.falla.set(null);
    pedido.subscribe({
      next: (r) => {
        this.guardando.set(false);
        this.#avisos.exito(listo(r));
      },
      error: (e: FallaApi) => {
        this.guardando.set(false);
        this.falla.set(e.message);
        this.#avisos.error('No se guardó');
      },
    });
  }
}
