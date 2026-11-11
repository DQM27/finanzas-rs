import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import type { Perfil } from '../nucleo/api';
import { leerPorcentaje, porcentaje } from '../nucleo/formato';
import { Nivel } from '../nucleo/nivel';
import { Avisos } from '../ui/aviso';
import { Configuracion, FACTOR_ILEGIBLE, borradorDe, pedidoDe } from './configuracion';

const PERFIL: Perfil = {
  salario_base: 293000,
  factor_neto: 8617,
  rebajos: 20253,
  salario_neto: 232225,
  tarifas: { nocturna: 25800, diurna: 19800, especial: 40000, feriado: 19500 },
  tope_caja_chica: 5000,
  cortes: [
    { inicio: '2099-01-02', fin: '2099-01-16', reportar_antes: '2099-01-20', pago: '2099-01-30' },
  ],
  cubre_hasta: '2099-01-30',
};

describe('porcentajes', () => {
  it('muestra diezmilésimas como porcentaje y lee lo escrito sin decimales flotantes', () => {
    expect(porcentaje(8617)).toBe('86,17 %');
    expect(porcentaje(10000)).toBe('100,00 %');
    expect(porcentaje(905)).toBe('9,05 %');
    expect(leerPorcentaje('86,17')).toBe(8617);
    expect(leerPorcentaje('86.17 %')).toBe(8617);
    expect(leerPorcentaje('86,5')).toBe(8650);
    expect(leerPorcentaje('86')).toBe(8600);
    expect(leerPorcentaje('86,175')).toBeNull();
    expect(leerPorcentaje('mucho')).toBeNull();
  });

  it('el borrador vuelve al mismo pedido', () => {
    const { salario_neto: _, cubre_hasta: __, ...pedido } = PERFIL;
    expect(pedidoDe(borradorDe(PERFIL))).toEqual(pedido);
  });
});

describe('Configuracion', () => {
  let http: HttpTestingController;

  async function montar(computadora: boolean) {
    await TestBed.configureTestingModule({
      imports: [Configuracion],
      providers: [
        provideHttpClient(),
        provideHttpClientTesting(),
        { provide: Nivel, useValue: { computadora: signal(computadora) } },
      ],
    }).compileComponents();
    http = TestBed.inject(HttpTestingController);
    const fixture = TestBed.createComponent(Configuracion);
    await fixture.whenStable();
    http.expectOne('/api/perfil').flush(PERFIL);
    await fixture.whenStable();
    const pagina = fixture.nativeElement as HTMLElement;
    return { fixture, pagina };
  }

  afterEach(() => http.verify());

  it('en el celular es de consulta', async () => {
    const { pagina } = await montar(false);
    expect(pagina.textContent).toContain('86,17 %');
    expect(pagina.textContent).toContain('₡232,225');
    expect(pagina.textContent).toContain('Pago VIE 30 ene');
    expect(pagina.textContent).toContain('abrí la app en la computadora');
    expect(pagina.querySelector('button[fz-boton]')).toBeNull();
  });

  it('en la computadora edita y guarda', async () => {
    const { fixture, pagina } = await montar(true);
    pagina.querySelector<HTMLButtonElement>('button[fz-boton]')!.click();
    await fixture.whenStable();

    const tope = pagina.querySelector<HTMLInputElement>('input[name="tope_caja_chica"]')!;
    tope.value = '6000';
    tope.dispatchEvent(new Event('input'));
    await fixture.whenStable();
    pagina.querySelector<HTMLButtonElement>('button[type="submit"]')!.click();
    await fixture.whenStable();

    const pedido = http.expectOne('/api/perfil');
    expect(pedido.request.method).toBe('PUT');
    expect(pedido.request.body).toMatchObject({ factor_neto: 8617, tope_caja_chica: 6000 });
    pedido.flush({ ...PERFIL, tope_caja_chica: 6000 });
    await fixture.whenStable();

    expect(TestBed.inject(Avisos).actual()?.texto).toBe('Configuración guardada');
    expect(pagina.querySelector('form')).toBeNull();
    expect(pagina.textContent).toContain('₡6,000');
  });

  it('un factor ilegible no se manda; un rechazo del dominio se muestra', async () => {
    const { fixture, pagina } = await montar(true);
    pagina.querySelector<HTMLButtonElement>('button[fz-boton]')!.click();
    await fixture.whenStable();
    const escribir = async (nombre: string, texto: string) => {
      const campo = pagina.querySelector<HTMLInputElement>(`input[name="${nombre}"]`)!;
      campo.value = texto;
      campo.dispatchEvent(new Event('input'));
      await fixture.whenStable();
    };
    const guardar = async () => {
      pagina.querySelector<HTMLButtonElement>('button[type="submit"]')!.click();
      await fixture.whenStable();
    };

    await escribir('factor_neto', 'mucho');
    await guardar();
    http.expectNone('/api/perfil');
    expect(pagina.querySelector('[role="alert"]')?.textContent).toContain(FACTOR_ILEGIBLE);

    await escribir('factor_neto', '86,17');
    await escribir('salario_base', '');
    await guardar();
    http
      .expectOne('/api/perfil')
      .flush(
        { codigo: 'DATOS_INVALIDOS', mensaje: 'el salario base debe ser mayor que cero' },
        { status: 422, statusText: 'Unprocessable Entity' },
      );
    await fixture.whenStable();
    expect(pagina.querySelector('[role="alert"]')?.textContent).toContain('mayor que cero');
    expect(pagina.querySelector('form')).not.toBeNull();
  });
});
