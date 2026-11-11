import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { hoy, leerMonto } from '../nucleo/formato';
import { Avisos } from '../ui/aviso';
import { Anotar } from './anotar';

describe('leerMonto', () => {
  it('se queda con los dígitos, como colones enteros', () => {
    expect(leerMonto('3500')).toBe(3500);
    expect(leerMonto('₡3,500')).toBe(3500);
    expect(leerMonto(' 3.500 ')).toBe(3500);
    expect(leerMonto('')).toBeNull();
    expect(leerMonto('abc')).toBeNull();
  });
});

describe('Anotar', () => {
  let http: HttpTestingController;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Anotar],
      providers: [provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  async function montar() {
    const fixture = TestBed.createComponent(Anotar);
    await fixture.whenStable();
    const pagina = fixture.nativeElement as HTMLElement;
    const escribir = async (nombre: string, texto: string) => {
      const campo = pagina.querySelector<HTMLInputElement | HTMLSelectElement>(`[name="${nombre}"]`)!;
      campo.value = texto;
      campo.dispatchEvent(new Event(campo instanceof HTMLSelectElement ? 'change' : 'input'));
      await fixture.whenStable();
    };
    const guardar = async () => {
      pagina.querySelector<HTMLButtonElement>('button[type="submit"]')!.click();
      await fixture.whenStable();
    };
    return { fixture, pagina, escribir, guardar };
  }

  it('guarda un gasto, avisa cuánto queda en la caja y limpia concepto y monto', async () => {
    const { fixture, pagina, escribir, guardar } = await montar();
    await escribir('concepto', 'almuerzo');
    await escribir('monto', '3.500');
    await escribir('categoria', 'COMIDA');
    await guardar();

    const pedido = http.expectOne('/api/gastos');
    expect(pedido.request.method).toBe('POST');
    expect(pedido.request.body).toEqual({
      fecha: hoy(),
      concepto: 'almuerzo',
      monto: 3500,
      categoria: 'COMIDA',
      origen: 'CUENTA_GASTOS',
    });
    expect(pagina.querySelector('button[type="submit"]')?.textContent).toContain('Guardando…');

    pedido.flush(
      {
        gasto: { id: 'g', fecha: hoy(), concepto: 'ALMUERZO', monto: 3500, categoria: 'COMIDA', origen: 'CUENTA_GASTOS' },
        caja: { desde: hoy(), hasta: hoy(), queda: 1500, exceso: 0, nivel: 'BAJA' },
      },
      { status: 201, statusText: 'Created' },
    );
    await fixture.whenStable();

    expect(TestBed.inject(Avisos).actual()?.texto).toBe(
      'Gasto guardado · quedan ₡1,500 en la caja chica',
    );
    expect(pagina.querySelector<HTMLInputElement>('[name="concepto"]')?.value).toBe('');
    expect(pagina.querySelector<HTMLInputElement>('[name="monto"]')?.value).toBe('');
  });

  it('guarda una extra con la tarifa y avisa el neto y el día de pago', async () => {
    const { fixture, pagina, escribir, guardar } = await montar();
    pagina.querySelectorAll<HTMLButtonElement>('[role="radio"]')[1].click();
    await fixture.whenStable();
    await escribir('tipo', 'NOCTURNA');
    await guardar();

    const pedido = http.expectOne('/api/extras');
    expect(pedido.request.body).toEqual({ fecha: hoy(), tipo: 'NOCTURNA', monto: null });
    pedido.flush(
      { id: 'e', fecha: hoy(), tipo: 'NOCTURNA', bruto: 25800, neto: 22232, fecha_pago: '2026-11-30' },
      { status: 201, statusText: 'Created' },
    );
    await fixture.whenStable();

    expect(TestBed.inject(Avisos).actual()?.texto).toBe(
      'Extra guardada · ₡22,232 neto, se paga el LUN 30 nov',
    );
  });

  it('si la API rechaza, deja su mensaje debajo del formulario', async () => {
    const { fixture, pagina, escribir, guardar } = await montar();
    pagina.querySelectorAll<HTMLButtonElement>('[role="radio"]')[1].click();
    await fixture.whenStable();
    await escribir('tipo', 'OTRA');
    await guardar();

    http
      .expectOne('/api/extras')
      .flush(
        { codigo: 'DATOS_INVALIDOS', mensaje: 'una extra Otra necesita el monto' },
        { status: 422, statusText: 'Unprocessable Entity' },
      );
    await fixture.whenStable();

    expect(pagina.querySelector('[role="alert"]')?.textContent).toContain('necesita el monto');
    expect(TestBed.inject(Avisos).actual()).toMatchObject({ tipo: 'error' });
    expect(pagina.querySelector<HTMLButtonElement>('button[type="submit"]')?.disabled).toBe(false);
  });
});
