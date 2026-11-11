/** Los tipos que devuelve la API (crates/api/src/dto.rs). */

export type Categoria =
  | 'COMIDA' | 'CASA' | 'MOTO' | 'SALUD' | 'FAMILIA' | 'ROPA' | 'TRAMITES' | 'GUSTO' | 'OTRO';
export type Origen = 'CUENTA_GASTOS' | 'PLANILLA' | 'SOBRA_DEL_MES' | 'GUSTO' | 'FONDO';
export type TipoExtra = 'NOCTURNA' | 'DIURNA' | 'ESPECIAL' | 'FERIADO' | 'OTRA';
export type Nivel = 'VITAL' | 'TRABAJO' | 'DESCUIDO' | 'FALTA';
export type NivelCaja = 'HOLGADA' | 'BAJA' | 'AGOTADA';
export type Lado = 'INGRESO' | 'PAGO';

export interface Caja {
  desde: string;
  hasta: string;
  queda: number;
  exceso: number;
  nivel: NivelCaja;
}

export interface Gasto {
  id: string;
  fecha: string;
  concepto: string;
  monto: number;
  categoria: Categoria;
  origen: Origen;
}

export interface GastoRegistrado {
  gasto: Gasto;
  caja: Caja;
}

export interface Extra {
  id: string;
  fecha: string;
  tipo: TipoExtra;
  bruto: number;
  neto: number;
  fecha_pago: string | null;
}

export interface Fondo {
  saldo: number;
  por_reponer: number;
  cuota_mensual: number;
}

export interface Plan {
  nivel: Nivel;
  sacado: number;
  a_reponer: number;
  meses: number;
  cuota_mensual: number;
}

export interface Retiro {
  plan: Plan;
  fondo: Fondo;
}

export interface MovimientoFondo {
  id: string;
  fecha: string;
  concepto: string;
  monto: number;
  tipo: 'APORTE' | 'RETIRO' | 'REPOSICION';
  plan: Plan | null;
}

export interface Linea {
  concepto: string;
  monto: number;
  hecho: boolean;
}

export interface Quincena {
  pago: string;
  ingresos: Linea[];
  pagos: Linea[];
  total_ingresos: number;
  total_pagos: number;
  sobra: number;
  falta_por_pagar: number;
  avance: { hechos: number; total: number };
}

export interface Tarifas {
  nocturna: number;
  diurna: number;
  especial: number;
  feriado: number;
}

export interface Corte {
  inicio: string;
  fin: string;
  reportar_antes: string;
  pago: string;
}

export interface Perfil {
  salario_base: number;
  factor_neto: number;
  rebajos: number;
  salario_neto: number;
  tarifas: Tarifas;
  tope_caja_chica: number;
  cortes: Corte[];
  cubre_hasta: string | null;
}

/** `GET /cortes/hoy`: el calendario visto desde hoy. */
export interface CortesHoy {
  /** El corte en que cae hoy: lo trabajado hoy se reporta y se paga según él. */
  en_curso: Corte | null;
  /** El próximo pago, hoy incluido. */
  proximo_pago: Corte | null;
  /** Días hasta ese pago (0 si es hoy). */
  dias_para_pago: number | null;
}

/** El cuerpo de `PUT /perfil`: el perfil sin lo que calcula el dominio. */
export type PerfilPedido = Omit<Perfil, 'salario_neto' | 'cubre_hasta'>;

/** Un error de la API: el código decide, el mensaje se muestra. */
export interface ErrorApi {
  codigo: string;
  mensaje: string;
}
