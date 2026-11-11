import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { App } from './app';

describe('App', () => {
  let http: HttpTestingController;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [App],
      providers: [provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  it('muestra la caja chica que manda la API', async () => {
    const fixture = TestBed.createComponent(App);
    http.expectOne('/api/caja-chica').flush({
      desde: '2026-10-30',
      hasta: '2026-11-12',
      queda: 5000,
      exceso: 0,
      nivel: 'HOLGADA',
    });
    await fixture.whenStable();
    const pagina = fixture.nativeElement as HTMLElement;
    expect(pagina.querySelector('h1')?.textContent).toContain('Finanzas');
    expect(pagina.querySelector('.cifra')?.textContent).toContain('₡5,000');
    expect(pagina.querySelector('.detalle')?.textContent).toContain('VIE 30 oct');
    expect(pagina.querySelector('.exceso')).toBeNull();
  });

  it('muestra el mensaje de la API cuando falla', async () => {
    const fixture = TestBed.createComponent(App);
    http
      .expectOne('/api/caja-chica')
      .flush(
        { codigo: 'PERFIL_NO_ENCONTRADO', mensaje: 'el usuario no tiene perfil configurado' },
        { status: 404, statusText: 'Not Found' },
      );
    await fixture.whenStable();
    const pagina = fixture.nativeElement as HTMLElement;
    expect(pagina.querySelector('[role="alert"]')?.textContent).toContain(
      'no tiene perfil configurado',
    );
  });
});
