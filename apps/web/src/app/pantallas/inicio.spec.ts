import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { Avisos } from '../ui/aviso';
import { Inicio } from './inicio';

const CAJA = { desde: '2026-11-13', hasta: '2026-11-29', queda: 4000, exceso: 0, nivel: 'BAJA' };
const CORTE = {
  inicio: '2026-11-02',
  fin: '2026-11-13',
  reportar_antes: '2026-11-17',
  pago: '2026-11-30',
};
const QUINCENA = {
  pago: '2026-11-30',
  presupuestada: true,
  automaticos: { salario: 232225, extras: 0, llegaron: true },
  ingresos: [],
  pagos: [],
  otros_ingresos: 0,
  total_ingresos: 232225,
  total_pagos: 250000,
  sobra: -17775,
  falta_por_pagar: 120000,
  avance: { hechos: 3, total: 9 },
};
const GASTOS = [
  { id: 'a', fecha: '2026-11-13', concepto: 'ALMUERZO', monto: 3500, categoria: 'COMIDA', origen: 'CUENTA_GASTOS' },
  { id: 'b', fecha: '2026-11-14', concepto: 'PASES', monto: 1500, categoria: 'TRAMITES', origen: 'CUENTA_GASTOS' },
];

const NO_ENCONTRADO = { status: 404, statusText: 'Not Found' };

describe('Inicio', () => {
  let http: HttpTestingController;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Inicio],
      providers: [provideHttpClient(), provideHttpClientTesting(), provideRouter([])],
    }).compileComponents();
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  async function montar() {
    const fixture = TestBed.createComponent(Inicio);
    await fixture.whenStable();
    return { fixture, pagina: fixture.nativeElement as HTMLElement };
  }

  function tarjeta(pagina: HTMLElement, clave: string): HTMLElement {
    const t = [...pagina.querySelectorAll<HTMLElement>('fz-tarjeta')].find((e) =>
      e.querySelector('.clave')?.textContent?.startsWith(clave),
    );
    if (!t) throw new Error(`no hay tarjeta «${clave}»`);
    return t;
  }

  it('arma el día con lo que manda la API', async () => {
    const { fixture, pagina } = await montar();
    expect(pagina.querySelectorAll('fz-esqueleto').length).toBeGreaterThan(0);

    http.expectOne('/api/caja-chica').flush(CAJA);
    http.expectOne('/api/perfil').flush({ tope_caja_chica: 5000 });
    http.expectOne('/api/cortes/hoy').flush({
      en_curso: CORTE,
      proximo_pago: CORTE,
      dias_para_pago: 15,
    });
    http.expectOne('/api/quincenas/actual').flush(QUINCENA);
    http.expectOne('/api/fondo').flush({ saldo: 415000, por_reponer: 0, cuota_mensual: 0 });
    await fixture.whenStable();
    http.expectOne('/api/gastos/2026-11-13/2026-11-29').flush(GASTOS);
    await fixture.whenStable();

    const caja = tarjeta(pagina, 'Caja chica');
    expect(caja.querySelector('.cifra')?.textContent).toContain('₡4,000');
    // El color lo decide el dominio (BAJA), aunque la barra vaya al 80 %.
    expect(caja.querySelector('.barra')?.getAttribute('data-estado')).toBe('baja');
    expect(caja.querySelector('[role="progressbar"]')?.getAttribute('aria-valuenow')).toBe('80');

    const quincena = tarjeta(pagina, 'Quincena del LUN 30 nov');
    expect(quincena.classList).toContain('tenida');
    expect(quincena.querySelector('.cifra')?.textContent).toContain('−₡17,775');
    expect(quincena.textContent).toContain('Pagos · 3 de 9');
    expect(quincena.textContent).toContain('faltan ₡120,000');
    expect(quincena.closest('a')?.getAttribute('href')).toBe('/quincena');

    const pago = tarjeta(pagina, 'Próximo pago');
    expect(pago.querySelector('.fecha')?.textContent).toContain('LUN 30 nov');
    expect(pago.textContent).toContain('faltan 15 días');
    expect(pago.textContent).toContain('antes del MAR 17 nov');

    expect(tarjeta(pagina, 'Fondo de emergencia').textContent).toContain('Nada por reponer');

    const gastos = [...pagina.querySelectorAll('fz-grupo fz-renglon .linea-1')].map((e) =>
      e.textContent?.trim(),
    );
    expect(gastos).toEqual(['PASES', 'ALMUERZO']);
    expect(pagina.querySelector('fz-grupo')?.textContent).toContain('Trámites');
    expect(pagina.querySelectorAll('fz-esqueleto').length).toBe(0);
  });

  it('deshace un gasto con confirmación y vuelve a pedir la caja', async () => {
    const { fixture, pagina } = await montar();
    http.expectOne('/api/caja-chica').flush(CAJA);
    http.expectOne('/api/perfil').flush({ tope_caja_chica: 5000 });
    http.expectOne('/api/cortes/hoy').flush({ en_curso: CORTE, proximo_pago: CORTE, dias_para_pago: 15 });
    http.expectOne('/api/quincenas/actual').flush(QUINCENA);
    http.expectOne('/api/fondo').flush({ saldo: 0, por_reponer: 0, cuota_mensual: 0 });
    await fixture.whenStable();
    http.expectOne('/api/gastos/2026-11-13/2026-11-29').flush(GASTOS);
    await fixture.whenStable();

    const confirmar = vi.spyOn(window, 'confirm').mockReturnValueOnce(false);
    pagina.querySelector<HTMLButtonElement>('[aria-label="Deshacer PASES"]')!.click();
    expect(confirmar).toHaveBeenCalledWith('¿Deshacer «PASES» de ₡1,500?');
    http.expectNone('/api/gastos/b');

    confirmar.mockReturnValueOnce(true);
    pagina.querySelector<HTMLButtonElement>('[aria-label="Deshacer PASES"]')!.click();
    http.expectOne({ method: 'DELETE', url: '/api/gastos/b' }).flush(null, { status: 204, statusText: 'No Content' });
    http.expectOne('/api/caja-chica').flush({ ...CAJA, queda: 5000, nivel: 'HOLGADA' });
    await fixture.whenStable();
    http.expectOne('/api/gastos/2026-11-13/2026-11-29').flush([GASTOS[0]]);
    await fixture.whenStable();

    expect(TestBed.inject(Avisos).actual()?.texto).toBe('Gasto deshecho');
    expect(tarjeta(pagina, 'Caja chica').querySelector('.cifra')?.textContent).toContain('₡5,000');
    const gastos = [...pagina.querySelectorAll('fz-grupo .linea-1')].map((e) => e.textContent?.trim());
    expect(gastos).toEqual(['ALMUERZO']);
    confirmar.mockRestore();
  });

  it('sin presupuesto, invita a armarlo; lo demás se ve igual', async () => {
    const { fixture, pagina } = await montar();
    http.expectOne('/api/caja-chica').flush(CAJA);
    http.expectOne('/api/perfil').flush({ tope_caja_chica: 5000 });
    http.expectOne('/api/cortes/hoy').flush({ en_curso: null, proximo_pago: null, dias_para_pago: null });
    http.expectOne('/api/quincenas/actual').flush({
      ...QUINCENA,
      presupuestada: false,
      automaticos: { salario: 232225, extras: 22232, llegaron: true },
      total_ingresos: 254457,
      total_pagos: 0,
      sobra: 254457,
      falta_por_pagar: 0,
      avance: { hechos: 0, total: 0 },
    });
    http.expectOne('/api/fondo').flush({ saldo: 0, por_reponer: 0, cuota_mensual: 0 });
    await fixture.whenStable();
    http.expectOne('/api/gastos/2026-11-13/2026-11-29').flush([]);
    await fixture.whenStable();

    const quincena = tarjeta(pagina, 'Quincena del LUN 30 nov');
    expect(quincena.textContent).toContain('tocá para armarlo');
    expect(quincena.querySelector('.cifra')?.textContent).toContain('₡254,457');
    expect(tarjeta(pagina, 'Próximo pago').textContent).toContain('ya no cubre');
    expect(tarjeta(pagina, 'Caja chica').querySelector('.cifra')?.textContent).toContain('₡4,000');
    expect(pagina.querySelector('fz-grupo')?.textContent).toContain('Todavía no anotaste gastos');
  });

  it('si la caja chica falla, lo dice ahí y las demás tarjetas siguen', async () => {
    const { fixture, pagina } = await montar();
    http
      .expectOne('/api/caja-chica')
      .flush({ codigo: 'FUERA_DE_CALENDARIO', mensaje: 'hoy no cae en ningún periodo' }, NO_ENCONTRADO);
    http.expectOne('/api/perfil').flush({ tope_caja_chica: 5000 });
    http.expectOne('/api/cortes/hoy').flush({ en_curso: CORTE, proximo_pago: CORTE, dias_para_pago: 1 });
    http.expectOne('/api/quincenas/actual').flush(QUINCENA);
    http.expectOne('/api/fondo').flush({ saldo: 100000, por_reponer: 30000, cuota_mensual: 10000 });
    await fixture.whenStable();

    expect(tarjeta(pagina, 'Caja chica').querySelector('[role="alert"]')?.textContent).toContain(
      'ningún periodo',
    );
    expect(pagina.querySelector('fz-grupo')).toBeNull();
    expect(tarjeta(pagina, 'Próximo pago').textContent).toContain('falta 1 día');
    expect(tarjeta(pagina, 'Fondo de emergencia').textContent).toContain(
      'Por reponer ₡30,000 · ₡10,000 por mes',
    );
  });
});
