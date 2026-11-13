import type { ColDef, ITooltipParams, ValueFormatterParams } from 'ag-grid-community';
import { colones, fechaCorta } from '../../nucleo/formato';

/** Sin tildes y en minúscula: así se compara al buscar y al filtrar. */
export function normalizar(texto: string): string {
  return texto.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();
}

/**
 * Compara la fecha que eligió la persona con la de la celda, que viaja como
 * `AAAA-MM-DD`. Devuelve negativo, cero o positivo, como pide AG Grid.
 */
export function compararFechaYMD(elegida: Date, valorDeCelda: string | null): number {
  if (!valorDeCelda) return -1;
  const [anio, mes, dia] = valorDeCelda.slice(0, 10).split('-').map(Number);
  if (!anio || !mes || !dia) return -1;
  const celda = new Date(anio, mes - 1, dia);
  const elegido = new Date(elegida.getFullYear(), elegida.getMonth(), elegida.getDate());
  return celda.getTime() - elegido.getTime();
}

/**
 * Tipos de columna de la app. Una columna con `type: 'fecha'` muestra
 * `VIE 13 nov` y filtra por fechas; una con `type: 'colones'` muestra
 * `₡25,800`, va a la derecha en cifras tabulares y filtra por número. La
 * tabla nunca suma ni calcula dinero: sólo lo muestra.
 */
export const tiposDeColumna: Record<string, ColDef> = {
  fecha: {
    filter: 'agDateColumnFilter',
    filterParams: { comparator: compararFechaYMD },
    valueFormatter: (p: ValueFormatterParams) => (p.value ? fechaCorta(String(p.value)) : ''),
    width: 130,
    flex: 0,
  },
  colones: {
    filter: 'agNumberColumnFilter',
    valueFormatter: (p: ValueFormatterParams) =>
      typeof p.value === 'number' ? colones(p.value) : '',
    cellClass: 'cifras a-la-derecha',
    headerClass: 'a-la-derecha',
    width: 140,
    flex: 0,
  },
};

/** Texto completo al pasar el mouse, sólo para celdas de texto o número. */
export function textoDeTooltip(parametros: ITooltipParams): string | undefined {
  const valor: unknown = parametros.valueFormatted ?? parametros.value;
  return typeof valor === 'string' || typeof valor === 'number' ? String(valor) : undefined;
}

/** La clave de una columna: su `colId` o, si no lo tiene, su `field`. */
export function claveDeColumna<T>(columna: ColDef<T>): string | undefined {
  return columna.colId ?? (typeof columna.field === 'string' ? columna.field : undefined);
}
