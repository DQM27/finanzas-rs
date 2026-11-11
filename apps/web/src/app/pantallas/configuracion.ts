import { ChangeDetectionStrategy, Component, computed, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import type { Corte, Perfil, PerfilPedido } from '../nucleo/api';
import { Carga } from '../nucleo/carga';
import { FallaApi, Finanzas } from '../nucleo/finanzas';
import {
  ColonesPipe, FechaCortaPipe, hoy, leerMonto, leerPorcentaje, porcentaje,
} from '../nucleo/formato';
import { Nivel } from '../nucleo/nivel';
import { Avisos } from '../ui/aviso';
import { Boton } from '../ui/boton';
import { Esqueleto } from '../ui/esqueleto';
import { Grupo } from '../ui/grupo';
import { Icono } from '../ui/icono';
import { Renglon } from '../ui/renglon';
import { Tarjeta } from '../ui/tarjeta';

/** Cuántos cortes se ven antes de «Ver todos». */
const CORTES_A_LA_VISTA = 6;

/** El perfil mientras se escribe: todo como texto, tal cual lo tecleó la persona. */
export interface Borrador {
  salario_base: string;
  factor_neto: string;
  rebajos: string;
  nocturna: string;
  diurna: string;
  especial: string;
  feriado: string;
  tope_caja_chica: string;
  cortes: Corte[];
}

export function borradorDe(p: Perfil): Borrador {
  return {
    salario_base: String(p.salario_base),
    factor_neto: porcentaje(p.factor_neto).replace(' %', ''),
    rebajos: String(p.rebajos),
    nocturna: String(p.tarifas.nocturna),
    diurna: String(p.tarifas.diurna),
    especial: String(p.tarifas.especial),
    feriado: String(p.tarifas.feriado),
    tope_caja_chica: String(p.tope_caja_chica),
    cortes: p.cortes.map((c) => ({ ...c })),
  };
}

/** Qué decir cuando el factor neto no se puede leer como porcentaje. */
export const FACTOR_ILEGIBLE = 'El factor neto tiene que ser un porcentaje, por ejemplo 86,17.';

/**
 * Lo que se manda a `PUT /perfil`, o `null` si el factor neto no se puede
 * leer. Un monto vacío va como 0: el dominio lo rechaza con su propio
 * mensaje, y aquí no se repite ninguna regla.
 */
export function pedidoDe(b: Borrador): PerfilPedido | null {
  const factor = leerPorcentaje(b.factor_neto);
  if (factor === null) return null;
  const monto = (t: string) => leerMonto(t) ?? 0;
  return {
    salario_base: monto(b.salario_base),
    factor_neto: factor,
    rebajos: monto(b.rebajos),
    tarifas: {
      nocturna: monto(b.nocturna),
      diurna: monto(b.diurna),
      especial: monto(b.especial),
      feriado: monto(b.feriado),
    },
    tope_caja_chica: monto(b.tope_caja_chica),
    cortes: b.cortes,
  };
}

type CampoMonto = Exclude<keyof Borrador, 'cortes'>;

/**
 * Configuración: salario, tarifas, caja chica y calendario de cortes. En el
 * celular es de consulta; en la computadora se puede editar
 * (docs/diseno.md §0). El salario neto y hasta cuándo cubre el calendario
 * los calcula el dominio.
 */
@Component({
  selector: 'fz-configuracion',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    Boton, ColonesPipe, Esqueleto, FechaCortaPipe, FormsModule, Grupo, Icono, Renglon, Tarjeta,
  ],
  templateUrl: './configuracion.html',
  styleUrl: './configuracion.scss',
})
export class Configuracion {
  readonly #finanzas = inject(Finanzas);
  readonly #avisos = inject(Avisos);
  protected readonly nivel = inject(Nivel);

  protected readonly perfil = signal<Carga<Perfil>>({ estado: 'cargando' });
  protected readonly borrador = signal<Borrador | null>(null);
  protected readonly guardando = signal(false);
  protected readonly falla = signal<string | null>(null);
  protected readonly todos = signal(false);
  protected readonly porcentaje = porcentaje;

  protected readonly salario: readonly [CampoMonto, string][] = [
    ['salario_base', 'Salario base'],
    ['rebajos', 'Rebajos fijos'],
  ];
  protected readonly tarifas: readonly [CampoMonto, string][] = [
    ['nocturna', 'Nocturna'],
    ['diurna', 'Diurna'],
    ['especial', 'Especial'],
    ['feriado', 'Feriado'],
  ];

  /** Los cortes que se ven: de hoy en adelante, o todos. */
  protected readonly cortesVisibles = computed<Corte[]>(() => {
    const p = this.perfil();
    if (p.estado !== 'listo') return [];
    if (this.todos()) return p.valor.cortes;
    const desde = hoy();
    return p.valor.cortes.filter((c) => c.pago >= desde).slice(0, CORTES_A_LA_VISTA);
  });

  constructor() {
    this.#finanzas.perfil().subscribe({
      next: (p) => this.perfil.set({ estado: 'listo', valor: p }),
      error: (e: FallaApi) =>
        this.perfil.set({ estado: 'falla', codigo: e.codigo, mensaje: e.message }),
    });
  }

  protected editar(p: Perfil): void {
    this.borrador.set(borradorDe(p));
    this.falla.set(null);
  }

  protected cancelar(): void {
    this.borrador.set(null);
    this.falla.set(null);
  }

  protected cambiar(campo: CampoMonto, texto: string): void {
    this.borrador.update((b) => (b ? { ...b, [campo]: texto } : b));
  }

  protected cambiarCorte(indice: number, campo: keyof Corte, fecha: string): void {
    this.borrador.update((b) =>
      b
        ? { ...b, cortes: b.cortes.map((c, i) => (i === indice ? { ...c, [campo]: fecha } : c)) }
        : b,
    );
  }

  protected agregarCorte(): void {
    this.borrador.update((b) =>
      b ? { ...b, cortes: [...b.cortes, { inicio: '', fin: '', reportar_antes: '', pago: '' }] } : b,
    );
  }

  protected quitarCorte(indice: number): void {
    this.borrador.update((b) =>
      b ? { ...b, cortes: b.cortes.filter((_, i) => i !== indice) } : b,
    );
  }

  protected guardar(): void {
    const b = this.borrador();
    if (!b || this.guardando()) return;
    const pedido = pedidoDe(b);
    if (!pedido) {
      this.falla.set(FACTOR_ILEGIBLE);
      return;
    }
    this.guardando.set(true);
    this.falla.set(null);
    this.#finanzas.guardarPerfil(pedido).subscribe({
      next: (p) => {
        this.guardando.set(false);
        this.borrador.set(null);
        this.perfil.set({ estado: 'listo', valor: p });
        this.#avisos.exito('Configuración guardada');
      },
      error: (e: FallaApi) => {
        this.guardando.set(false);
        this.falla.set(e.message);
        this.#avisos.error('No se guardó');
      },
    });
  }
}
