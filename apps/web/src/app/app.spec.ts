import { signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { App } from './app';
import { Nivel } from './nucleo/nivel';

describe('App', () => {
  it('pone la barra de secciones con las cuatro pestañas', async () => {
    await TestBed.configureTestingModule({
      imports: [App],
      providers: [provideRouter([])],
    }).compileComponents();
    const fixture = TestBed.createComponent(App);
    await fixture.whenStable();
    const pagina = fixture.nativeElement as HTMLElement;
    const rotulos = [...pagina.querySelectorAll('nav[aria-label="Secciones"] a')].map((a) =>
      a.textContent?.trim(),
    );
    expect(rotulos).toEqual(['Inicio', 'Quincena', 'Anotar', 'Más']);
  });
});

describe('marco según el nivel', () => {
  async function marco(computadora: boolean) {
    await TestBed.configureTestingModule({
      imports: [App],
      providers: [
        provideRouter([]),
        { provide: Nivel, useValue: { computadora: signal(computadora) } },
      ],
    }).compileComponents();
    const fixture = TestBed.createComponent(App);
    await fixture.whenStable();
    return fixture.nativeElement as HTMLElement;
  }

  it('en la computadora, barra lateral con las secciones de escritorio', async () => {
    const raiz = await marco(true);
    expect(raiz.querySelector('fz-barra-lateral')).not.toBeNull();
    expect(raiz.querySelector('fz-barra-secciones')).toBeNull();
    const enlaces = [...raiz.querySelectorAll('fz-barra-lateral a')].map((a) =>
      a.textContent?.trim(),
    );
    expect(enlaces).toEqual([
      'Inicio',
      'Mes',
      'Quincena',
      'Anotar',
      'Extras',
      'Gastos',
      'Configuración',
    ]);
  });

  it('en el celular, la barra de secciones de siempre', async () => {
    const raiz = await marco(false);
    expect(raiz.querySelector('fz-barra-lateral')).toBeNull();
    expect(raiz.querySelector('fz-barra-secciones')).not.toBeNull();
  });
});
