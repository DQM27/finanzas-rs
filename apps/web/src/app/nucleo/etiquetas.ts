import type { Categoria } from './api';

/** Cómo se lee cada categoría en pantalla. */
export const CATEGORIAS: Record<Categoria, string> = {
  COMIDA: 'Comida',
  CASA: 'Casa',
  MOTO: 'Moto',
  SALUD: 'Salud',
  FAMILIA: 'Familia',
  ROPA: 'Ropa',
  TRAMITES: 'Trámites',
  GUSTO: 'Gusto',
  OTRO: 'Otro',
};

/** «es hoy», «falta 1 día» o «faltan N días», con los días que manda la API. */
export function faltan(dias: number): string {
  if (dias <= 0) return 'es hoy';
  if (dias === 1) return 'falta 1 día';
  return `faltan ${dias} días`;
}
