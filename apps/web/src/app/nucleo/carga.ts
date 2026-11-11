import { Signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { Observable, catchError, map, of, switchMap } from 'rxjs';
import { FallaApi } from './finanzas';

/** Lo que se sabe de un dato que se pidió a la API. */
export type Carga<T> =
  | { estado: 'cargando' }
  | { estado: 'listo'; valor: T }
  | { estado: 'falla'; codigo: string; mensaje: string };

/** Un pedido que nunca falla: termina en `listo` o en `falla`. */
function resultado<T>(pedido: Observable<T>): Observable<Carga<T>> {
  return pedido.pipe(
    map((valor): Carga<T> => ({ estado: 'listo', valor })),
    catchError((e: unknown) => {
      const falla =
        e instanceof FallaApi
          ? e
          : new FallaApi('DESCONOCIDO', 'Algo falló. Intentá de nuevo en un momento.');
      return of<Carga<T>>({ estado: 'falla', codigo: falla.codigo, mensaje: falla.message });
    }),
  );
}

/**
 * Pide un dato y lo expone como señal: arranca en `cargando` y termina en
 * `listo` o en `falla`, con el código y el mensaje que mandó la API. Se llama
 * desde un contexto de inyección (un campo o el constructor).
 */
export function cargar<T>(fuente: Observable<T>): Signal<Carga<T>> {
  return toSignal(resultado(fuente), { initialValue: { estado: 'cargando' } });
}

/**
 * Como `cargar`, pero pide de nuevo cada vez que `disparo` emite. Mientras
 * llega la respuesta nueva se sigue viendo la anterior: no vuelve a
 * `cargando`, así que no parpadea.
 */
export function recargable<D, T>(
  disparo: Observable<D>,
  pedir: (d: D) => Observable<T>,
): Signal<Carga<T>> {
  return toSignal(disparo.pipe(switchMap((d) => resultado(pedir(d)))), {
    initialValue: { estado: 'cargando' },
  });
}

/** El valor, si ya llegó. */
export function valor<T>(c: Carga<T>): T | undefined {
  return c.estado === 'listo' ? c.valor : undefined;
}
