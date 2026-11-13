import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { hoy } from '../../nucleo/formato';
import { Gastos, mesDe } from './gastos';

describe('el mes de una fecha', () => {
  it('va del primero al último día', () => {
    expect(mesDe('2026-11-11')).toEqual({ desde: '2026-11-01', hasta: '2026-11-30' });
    expect(mesDe('2028-02-03')).toEqual({ desde: '2028-02-01', hasta: '2028-02-29' });
  });
});

describe('pantalla Gastos', () => {
  let http: HttpTestingController;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  it('pide el mes en curso y vuelve a pedir al cambiar el rango', async () => {
    const fixture = TestBed.createComponent(Gastos);
    await fixture.whenStable();
    const { desde, hasta } = mesDe(hoy());
    http.expectOne(`/api/gastos/${desde}/${hasta}`).flush([
      {
        id: 'g',
        fecha: desde,
        concepto: 'PASAJES',
        monto: 1000,
        categoria: 'TRAMITES',
        origen: 'PLANILLA',
      },
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
