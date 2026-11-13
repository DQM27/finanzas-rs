import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import type { Mes as DatosMes, Quincena } from '../../nucleo/api';
import { Mes, mesVecino, nombreDelMes } from './mes';

const Q1: Quincena = {
  pago: '2026-11-13',
  presupuestada: true,
  automaticos: { salario: 232225, extras: 103404, llegaron: true },
  ingresos: [{ concepto: 'PAPÁ', monto: 65000, hecho: true }],
  pagos: [
    { concepto: 'MOTO', monto: 104000, hecho: true },
    { concepto: 'COMIDA', monto: 100000, hecho: false },
  ],
  otros_ingresos: 65000,
  total_ingresos: 400629,
  total_pagos: 204000,
  sobra: 196629,
  falta_por_pagar: 100000,
  avance: { hechos: 1, total: 2 },
};
const Q2: Quincena = {
  pago: '2026-11-30',
  presupuestada: false,
  automaticos: { salario: 232225, extras: 22232, llegaron: false },
  ingresos: [],
  pagos: [],
  otros_ingresos: 0,
  total_ingresos: 254457,
  total_pagos: 0,
  sobra: 254457,
  falta_por_pagar: 0,
  avance: { hechos: 0, total: 0 },
};
const NOVIEMBRE: DatosMes = {
  mes: '2026-11',
  quincenas: [Q1, Q2],
  balance: {
    salario: 464450,
    extras: 125636,
    otros_ingresos: 65000,
    total_ingresos: 655086,
    total_pagos: 204000,
    sobra: 451086,
    falta_por_pagar: 100000,
  },
};

describe('ayudantes del mes', () => {
  it('se mueve entre meses y cruza el año', () => {
    expect(mesVecino('2026-11', 1)).toBe('2026-12');
    expect(mesVecino('2026-12', 1)).toBe('2027-01');
    expect(mesVecino('2027-01', -1)).toBe('2026-12');
    expect(mesVecino('2026-11', -11)).toBe('2025-12');
  });

  it('nombra el mes', () => {
    expect(nombreDelMes('2026-11')).toBe('noviembre 2026');
  });
});

describe('Mes', () => {
  let http: HttpTestingController;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Mes],
      providers: [provideHttpClient(), provideHttpClientTesting(), provideRouter([])],
    }).compileComponents();
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  async function montar() {
    const fixture = TestBed.createComponent(Mes);
    await fixture.whenStable();
    http.expectOne('/api/meses/actual').flush(NOVIEMBRE);
    await fixture.whenStable();
    return { fixture, pagina: fixture.nativeElement as HTMLElement };
  }

  it('muestra Q1 y Q2 lado a lado con lo que entra solo y los totales del mes', async () => {
    const { pagina } = await montar();
    expect(pagina.querySelector('.nombre')?.textContent).toContain('noviembre 2026');
    expect(pagina.querySelector('.resumen')?.textContent).toContain('₡655,086');
    expect(pagina.querySelector('.resumen')?.textContent).toContain('₡451,086');

    const [q1, q2] = pagina.querySelectorAll<HTMLElement>('.quincena');
    expect(q1.querySelector('h2')?.textContent).toContain('Q1');
    expect(q1.querySelector('h2')?.textContent).toContain('VIE 13 nov');
    const automaticos = [...q1.querySelectorAll('.automatico')].map((e) => e.textContent);
    expect(automaticos[0]).toContain('₡232,225');
    expect(automaticos[1]).toContain('₡103,404');
    expect(q1.textContent).toContain('PAPÁ');
    expect(q1.querySelector('.editar')?.textContent).toContain('Editar presupuesto');

    expect(q2.textContent).toContain('Todavía sin presupuesto.');
    expect(q2.querySelector('.editar')?.textContent).toContain('Armar presupuesto');
    expect(q2.querySelector('.editar')?.getAttribute('href')).toBe('/quincena?pago=2026-11-30');
  });

  it('marca un pago y vuelve a pedir el mes', async () => {
    const { fixture, pagina } = await montar();
    const comida = [
      ...pagina.querySelectorAll<HTMLButtonElement>('.quincena [role="checkbox"]'),
    ].find((b) => b.getAttribute('aria-label') === 'COMIDA')!;
    expect(comida.getAttribute('aria-checked')).toBe('false');
    comida.click();
    const pedido = http.expectOne('/api/quincenas/2026-11-13/marcas');
    expect(pedido.request.body).toEqual({ lado: 'PAGO', indice: 1, hecho: true });
    pedido.flush(Q1);
    await fixture.whenStable();
    http.expectOne('/api/meses/actual').flush(NOVIEMBRE);
  });

  it('va al mes siguiente y vuelve a este', async () => {
    const { fixture, pagina } = await montar();
    pagina.querySelector<HTMLButtonElement>('[aria-label="Mes siguiente"]')!.click();
    await fixture.whenStable();
    http.expectOne('/api/meses/2026-12').flush({ ...NOVIEMBRE, mes: '2026-12', quincenas: [] });
    await fixture.whenStable();
    expect(pagina.querySelector('.nombre')?.textContent).toContain('diciembre 2026');
    expect(pagina.textContent).toContain('no tiene pagos en este mes');

    pagina.querySelector<HTMLButtonElement>('.hoy')!.click();
    await fixture.whenStable();
    http.expectOne('/api/meses/actual').flush(NOVIEMBRE);
  });
});
