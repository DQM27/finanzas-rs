import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { App } from './app';

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
