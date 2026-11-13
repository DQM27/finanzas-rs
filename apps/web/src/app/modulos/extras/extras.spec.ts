import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { hoy } from '../../nucleo/formato';
import { Avisos } from '../../ui/aviso';
import { Extras } from './extras';

describe('pantalla Extras', () => {
  let http: HttpTestingController;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  it('lista, anota y vuelve a pedir la lista', async () => {
    const fixture = TestBed.createComponent(Extras);
    await fixture.whenStable();
    const pagina = fixture.nativeElement as HTMLElement;
    http.expectOne('/api/extras').flush([
      {
        id: 'a',
        fecha: '2026-11-05',
        tipo: 'ESPECIAL',
        bruto: 40000,
        neto: 34468,
        fecha_pago: '2026-11-30',
      },
    ]);
    await fixture.whenStable();
    expect(pagina.querySelector('.conteo')?.textContent).toContain('1 registro');

    pagina.querySelector<HTMLButtonElement>('button[type="submit"]')!.click();
    await fixture.whenStable();
    const pedido = http.expectOne({ method: 'POST', url: '/api/extras' });
    expect(pedido.request.body).toEqual({ fecha: hoy(), tipo: 'NOCTURNA', monto: null });
    pedido.flush({
      id: 'b',
      fecha: hoy(),
      tipo: 'NOCTURNA',
      bruto: 25800,
      neto: 22232,
      fecha_pago: null,
    });
    await fixture.whenStable();
    expect(TestBed.inject(Avisos).actual()?.texto).toBe('Extra anotada · ₡22,232 neto');
    http.expectOne({ method: 'GET', url: '/api/extras' }).flush([]);
  });
});
