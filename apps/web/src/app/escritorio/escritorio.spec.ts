import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { App } from '../app';
import { Nivel } from '../nucleo/nivel';
import { hoy } from '../nucleo/formato';
import { Avisos } from '../ui/aviso';
import { Extras } from './extras';
import { Gastos, mesDe } from './gastos';
import { compararFechaYMD, normalizar } from './tabla/columnas';
import { guardarDiseno, leerDiseno } from './tabla/diseno-guardado';

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
    expect(leerDiseno('prueba')).toEqual({ ocultas: ['neto'], columnas: [], filtrosVisibles: false });
    localStorage.setItem('finanzas.tabla.prueba', '{malo');
    expect(leerDiseno('prueba')).toBeNull();
    expect(leerDiseno(undefined)).toBeNull();
  });

  it('el mes de una fecha', () => {
    expect(mesDe('2026-11-11')).toEqual({ desde: '2026-11-01', hasta: '2026-11-30' });
    expect(mesDe('2028-02-03')).toEqual({ desde: '2028-02-01', hasta: '2028-02-29' });
  });
});

describe('marco según el nivel', () => {
  async function marco(computadora: boolean) {
    await TestBed.configureTestingModule({
      imports: [App],
      providers: [provideRouter([]), { provide: Nivel, useValue: { computadora: signal(computadora) } }],
    }).compileComponents();
    const fixture = TestBed.createComponent(App);
    await fixture.whenStable();
    return fixture.nativeElement as HTMLElement;
  }

  it('en la computadora, barra lateral con las secciones de escritorio', async () => {
    const raiz = await marco(true);
    expect(raiz.querySelector('fz-barra-lateral')).not.toBeNull();
    expect(raiz.querySelector('fz-barra-secciones')).toBeNull();
    const enlaces = [...raiz.querySelectorAll('fz-barra-lateral a')].map((a) => a.textContent?.trim());
    expect(enlaces).toEqual(['Inicio', 'Quincena', 'Anotar', 'Extras', 'Gastos', 'Configuración']);
  });

  it('en el celular, la barra de secciones de siempre', async () => {
    const raiz = await marco(false);
    expect(raiz.querySelector('fz-barra-lateral')).toBeNull();
    expect(raiz.querySelector('fz-barra-secciones')).not.toBeNull();
  });
});

describe('pantallas de tabla', () => {
  let http: HttpTestingController;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  it('Extras lista, anota y vuelve a pedir la lista', async () => {
    const fixture = TestBed.createComponent(Extras);
    await fixture.whenStable();
    const pagina = fixture.nativeElement as HTMLElement;
    http.expectOne('/api/extras').flush([
      { id: 'a', fecha: '2026-11-05', tipo: 'ESPECIAL', bruto: 40000, neto: 34468, fecha_pago: '2026-11-30' },
    ]);
    await fixture.whenStable();
    expect(pagina.querySelector('.conteo')?.textContent).toContain('1 registro');

    pagina.querySelector<HTMLButtonElement>('button[type="submit"]')!.click();
    await fixture.whenStable();
    const pedido = http.expectOne({ method: 'POST', url: '/api/extras' });
    expect(pedido.request.body).toEqual({ fecha: hoy(), tipo: 'NOCTURNA', monto: null });
    pedido.flush({ id: 'b', fecha: hoy(), tipo: 'NOCTURNA', bruto: 25800, neto: 22232, fecha_pago: null });
    await fixture.whenStable();
    expect(TestBed.inject(Avisos).actual()?.texto).toBe('Extra anotada · ₡22,232 neto');
    http.expectOne({ method: 'GET', url: '/api/extras' }).flush([]);
  });

  it('Gastos pide el mes en curso y vuelve a pedir al cambiar el rango', async () => {
    const fixture = TestBed.createComponent(Gastos);
    await fixture.whenStable();
    const { desde, hasta } = mesDe(hoy());
    http.expectOne(`/api/gastos/${desde}/${hasta}`).flush([
      { id: 'g', fecha: desde, concepto: 'PASAJES', monto: 1000, categoria: 'TRAMITES', origen: 'PLANILLA' },
    ]);
    await fixture.whenStable();
    const pagina = fixture.nativeElement as HTMLElement;
    expect(pagina.querySelector('.conteo')?.textContent).toContain('1 registro');

    const [campoDesde] = pagina.querySelectorAll<HTMLInputElement>('.rango-controles input');
    campoDesde.value = '2026-01-01';
    campoDesde.dispatchEvent(new Event('change'));
    await fixture.whenStable();
    http.expectOne(`/api/gastos/2026-01-01/${hasta}`).flush([]);
    await fixture.whenStable();
    expect(pagina.querySelector('.conteo')?.textContent).toContain('0 registros');
  });
});
