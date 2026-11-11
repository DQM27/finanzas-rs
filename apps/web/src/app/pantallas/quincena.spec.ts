import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { ActivatedRoute, convertToParamMap } from '@angular/router';
import { Avisos } from '../ui/aviso';
import { Quincena, lineasDe } from './quincena';

const corte = (pago: string) => ({ inicio: pago, fin: pago, reportar_antes: pago, pago });
const CORTES = {
  en_curso: corte('2026-11-30'),
  ultimo_pago: corte('2026-11-13'),
  proximo_pago: corte('2026-11-30'),
  dias_para_pago: 15,
};
const AUTOMATICOS = { salario: 232225, extras: 0, llegaron: true };
const PRESUPUESTO = {
  pago: '2026-11-13',
  presupuestada: true,
  automaticos: AUTOMATICOS,
  ingresos: [],
  pagos: [
    { concepto: 'ALQUILER', monto: 150000, hecho: true },
    { concepto: 'LUZ', monto: 15000, hecho: false },
  ],
  otros_ingresos: 0,
  total_ingresos: 232225,
  total_pagos: 165000,
  sobra: 67225,
  falta_por_pagar: 15000,
  avance: { hechos: 1, total: 2 },
};

describe('lineasDe', () => {
  it('deja fuera las filas vacías y lee los montos como colones enteros', () => {
    expect(
      lineasDe([
        { concepto: 'Luz', monto: '15.000' },
        { concepto: '  ', monto: '' },
        { concepto: 'Agua', monto: '' },
      ]),
    ).toEqual([
      { concepto: 'Luz', monto: 15000 },
      { concepto: 'Agua', monto: 0 },
    ]);
  });
});

describe('Quincena', () => {
  let http: HttpTestingController;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Quincena],
      providers: [provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  async function montar() {
    const fixture = TestBed.createComponent(Quincena);
    await fixture.whenStable();
    const pagina = fixture.nativeElement as HTMLElement;
    http.expectOne('/api/cortes/hoy').flush(CORTES);
    await fixture.whenStable();
    return { fixture, pagina };
  }

  it('muestra la quincena en curso y marca un pago con lo que devuelve la API', async () => {
    const { fixture, pagina } = await montar();
    http.expectOne('/api/quincenas/2026-11-13').flush(PRESUPUESTO);
    await fixture.whenStable();

    expect(pagina.querySelector('.resumen .cifra')?.textContent).toContain('₡67,225');
    expect(pagina.textContent).toContain('Pagos · 1 de 2');
    const [, luz] = pagina.querySelectorAll<HTMLButtonElement>('fz-grupo [role="checkbox"]');
    expect(luz.getAttribute('aria-label')).toBe('LUZ');
    expect(luz.getAttribute('aria-checked')).toBe('false');
    expect(pagina.querySelector('.pendiente')?.textContent).toContain('₡15,000');
    const salario = pagina.querySelector('.automatico');
    expect(salario?.textContent).toContain('Salario');
    expect(salario?.textContent).toContain('Llegó el VIE 13 nov');
    expect(salario?.textContent).toContain('₡232,225');
    expect(salario?.querySelector('[role="checkbox"]')).toBeNull(); // no se marca

    luz.click();
    const pedido = http.expectOne('/api/quincenas/2026-11-13/marcas');
    expect(pedido.request.method).toBe('PUT');
    expect(pedido.request.body).toEqual({ lado: 'PAGO', indice: 1, hecho: true });
    pedido.flush({
      ...PRESUPUESTO,
      pagos: PRESUPUESTO.pagos.map((p) => ({ ...p, hecho: true })),
      falta_por_pagar: 0,
      avance: { hechos: 2, total: 2 },
    });
    await fixture.whenStable();

    expect(pagina.textContent).toContain('Pagos · 2 de 2');
    expect(
      pagina.querySelectorAll('fz-grupo [role="checkbox"]')[1].getAttribute('aria-checked'),
    ).toBe('true');
    expect(pagina.querySelector('.pendiente')).toBeNull();
  });

  it('sin presupuesto abre el editor sin el salario (entra solo) y guarda', async () => {
    const { fixture, pagina } = await montar();
    http.expectOne('/api/quincenas/2026-11-13').flush({
      ...PRESUPUESTO,
      presupuestada: false,
      pagos: [],
      total_pagos: 0,
      sobra: 232225,
      falta_por_pagar: 0,
      avance: { hechos: 0, total: 0 },
    });
    await fixture.whenStable();

    const campos = () => pagina.querySelectorAll<HTMLInputElement>('form input');
    expect(campos().length).toBe(2); // sólo la fila de pagos: otros ingresos arranca vacío
    expect(pagina.textContent).toContain('El salario y las extras entran solos.');
    expect(pagina.querySelector('.secundario')).toBeNull(); // nada que cancelar

    // La primera fila de pagos viene vacía: se llena.
    const escribir = async (campo: HTMLInputElement, texto: string) => {
      campo.value = texto;
      campo.dispatchEvent(new Event('input'));
      await fixture.whenStable();
    };
    await escribir(campos()[0], 'Luz');
    await escribir(campos()[1], '15000');
    pagina.querySelector<HTMLButtonElement>('button[type="submit"]')!.click();
    await fixture.whenStable();

    const pedido = http.expectOne('/api/quincenas/2026-11-13');
    expect(pedido.request.method).toBe('PUT');
    expect(pedido.request.body).toEqual({
      ingresos: [],
      pagos: [{ concepto: 'Luz', monto: 15000 }],
    });
    pedido.flush(PRESUPUESTO);
    await fixture.whenStable();

    expect(TestBed.inject(Avisos).actual()?.texto).toBe('Presupuesto guardado');
    expect(pagina.querySelector('form')).toBeNull();
    expect(pagina.querySelector('.resumen')).not.toBeNull();
  });

  it('deja ver la próxima quincena para planearla', async () => {
    const { fixture, pagina } = await montar();
    http.expectOne('/api/quincenas/2026-11-13').flush(PRESUPUESTO);
    await fixture.whenStable();

    const [, proxima] = pagina.querySelectorAll<HTMLButtonElement>('fz-segmentado [role="radio"]');
    expect(proxima.textContent).toContain('La del LUN 30 nov');
    proxima.click();
    await fixture.whenStable();
    http.expectOne('/api/quincenas/2026-11-30').flush({ ...PRESUPUESTO, pago: '2026-11-30' });
    await fixture.whenStable();
    expect(pagina.querySelector('.resumen .clave')?.textContent).toContain('LUN 30 nov');
  });

  it('abre el pago que pide la ruta (desde Mes)', async () => {
    TestBed.resetTestingModule();
    await TestBed.configureTestingModule({
      imports: [Quincena],
      providers: [
        provideHttpClient(),
        provideHttpClientTesting(),
        {
          provide: ActivatedRoute,
          useValue: { snapshot: { queryParamMap: convertToParamMap({ pago: '2026-12-15' }) } },
        },
      ],
    }).compileComponents();
    http = TestBed.inject(HttpTestingController);
    const { fixture, pagina } = await montar();
    http.expectOne('/api/quincenas/2026-12-15').flush({ ...PRESUPUESTO, pago: '2026-12-15' });
    await fixture.whenStable();
    const opciones = [...pagina.querySelectorAll('fz-segmentado [role="radio"]')].map((b) => b.textContent?.trim());
    expect(opciones).toEqual(['Esta quincena', 'La del LUN 30 nov', 'La del MAR 15 dic']);
    expect(pagina.querySelector('.resumen .clave')?.textContent).toContain('MAR 15 dic');
  });
});
