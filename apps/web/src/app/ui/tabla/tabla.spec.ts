import { compararFechaYMD, normalizar } from './columnas';
import { guardarDiseno, leerDiseno } from './diseno-guardado';

describe('ayudantes de la tabla', () => {
  it('normaliza sin tildes ni mayúsculas', () => {
    expect(normalizar('TRÁMITES Ñandú')).toBe('tramites nandu');
  });

  it('compara una fecha elegida con la de la celda', () => {
    const elegida = new Date(2026, 10, 13);
    expect(compararFechaYMD(elegida, '2026-11-13')).toBe(0);
    expect(compararFechaYMD(elegida, '2026-11-14')).toBeGreaterThan(0);
    expect(compararFechaYMD(elegida, '2026-11-12')).toBeLessThan(0);
    expect(compararFechaYMD(elegida, null)).toBe(-1);
  });

  it('guarda y lee el diseño, y descarta lo que no sirve', () => {
    guardarDiseno('prueba', { ocultas: ['neto'], columnas: [], filtrosVisibles: false });
    expect(leerDiseno('prueba')).toEqual({
      ocultas: ['neto'],
      columnas: [],
      filtrosVisibles: false,
    });
    localStorage.setItem('finanzas.tabla.prueba', '{malo');
    expect(leerDiseno('prueba')).toBeNull();
    expect(leerDiseno(undefined)).toBeNull();
  });
});
