import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable, catchError, throwError } from 'rxjs';
import type {
  Caja, CortesHoy, Extra, Fondo, Gasto, GastoRegistrado, MovimientoFondo, Perfil, PerfilPedido,
  Quincena, Retiro,
} from './api';

/** Un error ya traducido a algo que se puede mostrar. */
export class FallaApi extends Error {
  constructor(
    readonly codigo: string,
    mensaje: string,
  ) {
    super(mensaje);
  }
}

const SIN_RED = 'No hay conexión. Revisá el internet e intentá de nuevo.';
const SESION = 'Se cerró la sesión. Recargá la página para entrar de nuevo.';

/**
 * El único punto que habla con el Worker. Todo va al mismo origen, así que la
 * sesión de Cloudflare Access viaja sola y no hay nada que guardar aquí.
 */
@Injectable({ providedIn: 'root' })
export class Finanzas {
  readonly #http = inject(HttpClient);

  perfil(): Observable<Perfil> {
    return this.#get('/api/perfil');
  }

  guardarPerfil(perfil: PerfilPedido): Observable<Perfil> {
    return this.#enviar('put', '/api/perfil', perfil);
  }

  caja(): Observable<Caja> {
    return this.#get('/api/caja-chica');
  }

  cortesHoy(): Observable<CortesHoy> {
    return this.#get('/api/cortes/hoy');
  }

  gastos(desde: string, hasta: string): Observable<Gasto[]> {
    return this.#get(`/api/gastos/${desde}/${hasta}`);
  }

  anotarGasto(gasto: {
    fecha: string;
    concepto: string;
    monto: number;
    categoria: string;
    origen: string;
  }): Observable<GastoRegistrado> {
    return this.#enviar('post', '/api/gastos', gasto);
  }

  borrarGasto(id: string): Observable<void> {
    return this.#borrar(`/api/gastos/${id}`);
  }

  extras(): Observable<Extra[]> {
    return this.#get('/api/extras');
  }

  anotarExtra(extra: {
    fecha: string;
    tipo: string;
    monto?: number | null;
  }): Observable<Extra> {
    return this.#enviar('post', '/api/extras', extra);
  }

  borrarExtra(id: string): Observable<void> {
    return this.#borrar(`/api/extras/${id}`);
  }

  fondo(): Observable<Fondo> {
    return this.#get('/api/fondo');
  }

  movimientosFondo(): Observable<MovimientoFondo[]> {
    return this.#get('/api/fondo/movimientos');
  }

  sacarDelFondo(retiro: {
    concepto: string;
    monto: number;
    nivel: string;
    inesperado: boolean;
    necesario: boolean;
    urgente: boolean;
  }): Observable<Retiro> {
    return this.#enviar('post', '/api/fondo/retiros', retiro);
  }

  depositarEnFondo(deposito: {
    tipo: string;
    concepto: string;
    monto: number;
  }): Observable<Fondo> {
    return this.#enviar('post', '/api/fondo/depositos', deposito);
  }

  borrarMovimientoFondo(id: string): Observable<void> {
    return this.#borrar(`/api/fondo/movimientos/${id}`);
  }

  quincena(pago: string | 'actual'): Observable<Quincena> {
    return this.#get(`/api/quincenas/${pago}`);
  }

  guardarPresupuesto(
    pago: string,
    presupuesto: {
      ingresos: { concepto: string; monto: number }[];
      pagos: { concepto: string; monto: number }[];
    },
  ): Observable<Quincena> {
    return this.#enviar('put', `/api/quincenas/${pago}`, presupuesto);
  }

  marcar(
    pago: string,
    marca: { lado: string; indice: number; hecho: boolean },
  ): Observable<Quincena> {
    return this.#enviar('put', `/api/quincenas/${pago}/marcas`, marca);
  }

  #get<T>(ruta: string): Observable<T> {
    return this.#http.get<T>(ruta).pipe(catchError(traducir));
  }

  #enviar<T>(metodo: 'post' | 'put', ruta: string, cuerpo: unknown): Observable<T> {
    return this.#http[metodo]<T>(ruta, cuerpo).pipe(catchError(traducir));
  }

  #borrar(ruta: string): Observable<void> {
    return this.#http.delete<void>(ruta).pipe(catchError(traducir));
  }
}

/** Convierte cualquier falla en una `FallaApi` con su código y su mensaje. */
function traducir(e: HttpErrorResponse): Observable<never> {
  if (e.status === 0) {
    return throwError(() => new FallaApi('SIN_RED', SIN_RED));
  }
  // Access devuelve 302 a su login; el navegador lo sigue y falla el CORS,
  // así que un 401 o 403 aquí es sesión vencida.
  if (e.status === 401 || e.status === 403) {
    return throwError(() => new FallaApi('SESION', SESION));
  }
  const cuerpo: unknown = e.error;
  if (
    typeof cuerpo === 'object' &&
    cuerpo !== null &&
    'codigo' in cuerpo &&
    'mensaje' in cuerpo
  ) {
    const { codigo, mensaje } = cuerpo as { codigo: unknown; mensaje: unknown };
    if (typeof codigo === 'string' && typeof mensaje === 'string') {
      return throwError(() => new FallaApi(codigo, mensaje));
    }
  }
  return throwError(
    () => new FallaApi('DESCONOCIDO', 'Algo falló. Intentá de nuevo en un momento.'),
  );
}
