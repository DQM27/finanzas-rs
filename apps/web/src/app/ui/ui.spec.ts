import { Component, signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { Avisos, DURACION_AVISO_MS } from './aviso';
import { Boton } from './boton';
import { Grupo } from './grupo';
import { Renglon } from './renglon';
import { Segmentado } from './segmentado';
import { Tarjeta, estadoBarra } from './tarjeta';

describe('estadoBarra', () => {
  it('sana arriba del 30 %, baja al 30 % o menos, agotada en 0', () => {
    expect(estadoBarra(0.31)).toBe('sana');
    expect(estadoBarra(0.3)).toBe('baja');
    expect(estadoBarra(0.01)).toBe('baja');
    expect(estadoBarra(0)).toBe('agotada');
    expect(estadoBarra(-0.2)).toBe('agotada');
  });
});

@Component({
  imports: [Boton, Grupo, Renglon, Segmentado, Tarjeta],
  template: `
    <fz-tarjeta clave="Cierre" [monto]="monto()" [proporcion]="0.2" />
    <fz-tarjeta clave="Fondo" [monto]="1000" [proporcion]="0.1" [crece]="true" />
    <fz-segmentado etiqueta="Quincena" [segmentos]="mitades" [(valor)]="mitad" />
    <fz-grupo titulo="Datos">
      <fz-renglon [interactivo]="true" (click)="toques.set(toques() + 1)">Tocable</fz-renglon>
      <fz-renglon>Fijo</fz-renglon>
    </fz-grupo>
    <button fz-boton type="button" [guardando]="guardando()">Guardar</button>
  `,
})
class Banco {
  readonly monto = signal(-500);
  readonly mitades = [
    { valor: 'Q1', etiqueta: 'Q1' },
    { valor: 'Q2', etiqueta: 'Q2' },
  ] as const;
  readonly mitad = signal<'Q1' | 'Q2'>('Q1');
  readonly toques = signal(0);
  readonly guardando = signal(false);
}

describe('componentes', () => {
  async function montar() {
    await TestBed.configureTestingModule({ imports: [Banco] }).compileComponents();
    const fixture = TestBed.createComponent(Banco);
    await fixture.whenStable();
    return { fixture, raiz: fixture.nativeElement as HTMLElement };
  }

  it('la tarjeta marca el negativo y colorea la barra según lo que queda', async () => {
    const { raiz } = await montar();
    const [cierre, fondo] = raiz.querySelectorAll('fz-tarjeta');
    expect(cierre.querySelector('.cifra')?.textContent).toContain('−₡500');
    expect(cierre.querySelector('.cifra')?.classList).toContain('negativo');
    expect(cierre.querySelector('.barra')?.getAttribute('data-estado')).toBe('baja');
    expect(cierre.querySelector('[role="progressbar"]')?.getAttribute('aria-valuenow')).toBe('20');
    expect(fondo.querySelector('.barra')?.getAttribute('data-estado')).toBe('acento');
  });

  it('el segmentado cambia con un toque y con las flechas', async () => {
    const { fixture, raiz } = await montar();
    const [q1, q2] = raiz.querySelectorAll<HTMLButtonElement>('[role="radio"]');
    expect(q1.getAttribute('aria-checked')).toBe('true');

    q2.click();
    await fixture.whenStable();
    expect(fixture.componentInstance.mitad()).toBe('Q2');

    raiz
      .querySelector('fz-segmentado')!
      .dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
    await fixture.whenStable();
    expect(fixture.componentInstance.mitad()).toBe('Q1');
  });

  it('un renglón interactivo responde a Enter como a un toque; uno fijo no es botón', async () => {
    const { fixture, raiz } = await montar();
    const [tocable, fijo] = raiz.querySelectorAll<HTMLElement>('fz-renglon');
    expect(tocable.getAttribute('role')).toBe('button');
    expect(tocable.getAttribute('tabindex')).toBe('0');
    expect(fijo.getAttribute('role')).toBeNull();

    tocable.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }));
    expect(fixture.componentInstance.toques()).toBe(1);
  });

  it('el botón dice «Guardando…» y se deshabilita mientras guarda', async () => {
    const { fixture, raiz } = await montar();
    const boton = raiz.querySelector<HTMLButtonElement>('button[fz-boton]')!;
    expect(boton.disabled).toBe(false);
    fixture.componentInstance.guardando.set(true);
    await fixture.whenStable();
    expect(boton.disabled).toBe(true);
    expect(boton.textContent?.trim()).toBe('Guardando…');
  });
});

describe('Avisos', () => {
  afterEach(() => vi.useRealTimers());

  it('muestra un aviso, lo reemplaza con el siguiente y lo quita a los 2,4 s', () => {
    vi.useFakeTimers();
    const avisos = TestBed.inject(Avisos);
    avisos.exito('Gasto guardado');
    expect(avisos.actual()?.texto).toBe('Gasto guardado');

    vi.advanceTimersByTime(DURACION_AVISO_MS - 100);
    avisos.error('Sin conexión');
    expect(avisos.actual()).toMatchObject({ texto: 'Sin conexión', tipo: 'error' });

    vi.advanceTimersByTime(DURACION_AVISO_MS - 1);
    expect(avisos.actual()).not.toBeNull();
    vi.advanceTimersByTime(1);
    expect(avisos.actual()).toBeNull();
  });
});
