import { Pipe, PipeTransform } from '@angular/core';

/** El signo menos tipográfico, no el guion. */
const MENOS = '−';

/** Colones enteros, con separador de miles y signo claro. */
export function colones(monto: number): string {
  const signo = monto < 0 ? MENOS : '';
  return `${signo}₡${Math.abs(Math.round(monto)).toLocaleString('en-US')}`;
}

/** Sólo los dígitos de lo que se escribió, como colones enteros. Vacío es `null`. */
export function leerMonto(texto: string): number | null {
  const digitos = texto.replace(/\D/g, '');
  return digitos ? Number(digitos) : null;
}

@Pipe({ name: 'colones' })
export class ColonesPipe implements PipeTransform {
  transform(monto: number | null | undefined): string {
    return typeof monto === 'number' ? colones(monto) : '—';
  }
}

const DIAS = ['DOM', 'LUN', 'MAR', 'MIÉ', 'JUE', 'VIE', 'SÁB'];
const MESES = [
  'ene', 'feb', 'mar', 'abr', 'may', 'jun',
  'jul', 'ago', 'set', 'oct', 'nov', 'dic',
];

/** Una fecha `AAAA-MM-DD` leída como día local, sin zona horaria. */
export function dia(fecha: string): Date {
  const [a, m, d] = fecha.split('-').map(Number);
  return new Date(a, m - 1, d);
}

/** `2026-11-13` → `VIE 13 nov`. */
export function fechaCorta(fecha: string): string {
  const f = dia(fecha);
  return `${DIAS[f.getDay()]} ${f.getDate()} ${MESES[f.getMonth()]}`;
}

@Pipe({ name: 'fechaCorta' })
export class FechaCortaPipe implements PipeTransform {
  transform(fecha: string | null | undefined): string {
    return fecha ? fechaCorta(fecha) : '—';
  }
}

/** Hoy, como `AAAA-MM-DD` en la hora local. */
export function hoy(): string {
  const f = new Date();
  const mes = `${f.getMonth() + 1}`.padStart(2, '0');
  const d = `${f.getDate()}`.padStart(2, '0');
  return `${f.getFullYear()}-${mes}-${d}`;
}

/** Días entre hoy y una fecha, en días completos. */
export function diasHasta(fecha: string): number {
  const ms = dia(fecha).getTime() - dia(hoy()).getTime();
  return Math.round(ms / 86_400_000);
}

/** «es hoy», «falta 1 día», «faltan 5 días» o «hace 2 días». */
export function cuanto(fecha: string): string {
  const d = diasHasta(fecha);
  if (d === 0) return 'es hoy';
  if (d === 1) return 'falta 1 día';
  if (d > 1) return `faltan ${d} días`;
  if (d === -1) return 'fue ayer';
  return `hace ${-d} días`;
}
