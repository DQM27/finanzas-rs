//! API HTTP de finanzas-rs.
//!
//! Es un adaptador de entrada: recibe una petición ya separada en método, ruta
//! y cuerpo, llama al caso de uso que toca y devuelve JSON. No sabe nada de
//! Cloudflare ni de SurrealDB: el Worker traduce su `Request` a [`Peticion`] y
//! le pasa un [`Contexto`] con los repositorios. Así toda la API se prueba en
//! nativo con los adaptadores en memoria.
//!
//! El usuario llega ya autenticado: identificarlo es trabajo del Worker.
//!
//! | Método | Ruta | Qué hace |
//! |---|---|---|
//! | `GET` | `/perfil` | El perfil propio |
//! | `PUT` | `/perfil` | Reemplaza el perfil propio |
//! | `GET` | `/caja-chica` | Caja chica del periodo |
//! | `POST` | `/gastos` | Anota un gasto |
//! | `DELETE` | `/gastos/{id}` | Deshace un gasto |
//! | `POST` | `/extras` | Anota una extra |
//! | `DELETE` | `/extras/{id}` | Deshace una extra |
//! | `GET` | `/fondo` | Estado del fondo |
//! | `POST` | `/fondo/retiros` | Saca del fondo |
//! | `POST` | `/fondo/depositos` | Aporta o repone |
//! | `DELETE` | `/fondo/movimientos/{id}` | Deshace el último movimiento |
//! | `GET` | `/quincenas/actual` | La quincena que se está gastando |
//! | `GET` | `/quincenas/{pago}` | Una quincena |
//! | `PUT` | `/quincenas/{pago}` | Guarda su presupuesto |
//! | `PUT` | `/quincenas/{pago}/marcas` | Marca o desmarca una línea |

#![expect(
    clippy::future_not_send,
    reason = "sin `Send` a propósito: en el Worker (WebAssembly) los futuros no son `Send`"
)]

pub mod dto;
mod errores;

use chrono::NaiveDate;
use dto::{
    CajaDto, ExtraDto, FondoDto, GastoRegistradoDto, Marca, NuevaExtra, NuevoDeposito, NuevoGasto,
    NuevoPerfil, NuevoRetiro, PerfilDto, Presupuesto, QuincenaDto, RetiroDto,
};
use errores::{ErrorApi, Resultado};
use finanzas_aplicacion::casos_de_uso::consultar_caja_chica::ConsultarCajaChica;
use finanzas_aplicacion::casos_de_uso::deshacer::{Deshacer, QueDeshacer};
use finanzas_aplicacion::casos_de_uso::fondo::{
    ConsultarFondo, DepositarEnFondo, SacarDelFondo, SolicitudDeposito, SolicitudSacar,
};
use finanzas_aplicacion::casos_de_uso::perfil::{ConsultarPerfil, GuardarPerfil, SolicitudPerfil};
use finanzas_aplicacion::casos_de_uso::quincena::{
    ConsultarQuincena, GuardarPresupuesto, MarcarLinea, SolicitudMarca, SolicitudPresupuesto,
};
use finanzas_aplicacion::casos_de_uso::registrar_extra::{RegistrarExtra, SolicitudExtra};
use finanzas_aplicacion::casos_de_uso::registrar_gasto::{RegistrarGasto, SolicitudGasto};
use finanzas_aplicacion::puertos::{
    GeneradorIds, Reloj, RepositorioExtras, RepositorioFondo, RepositorioGastos,
    RepositorioPerfiles, RepositorioQuincenas,
};
use finanzas_dominio::extra::ExtraId;
use finanzas_dominio::fondo::Evaluacion;
use finanzas_dominio::gasto::GastoId;
use finanzas_dominio::movimientos_fondo::MovimientoFondoId;
use finanzas_dominio::usuario::UsuarioId;
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

/// Método HTTP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metodo {
    /// Leer.
    Get,
    /// Crear.
    Post,
    /// Reemplazar.
    Put,
    /// Borrar.
    Delete,
    /// Cualquier otro.
    Otro,
}

/// Una petición ya separada por el Worker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peticion<'a> {
    /// Método.
    pub metodo: Metodo,
    /// Ruta sin el prefijo `/api`, por ejemplo `/fondo/retiros`.
    pub ruta: &'a str,
    /// Cuerpo en JSON (vacío si no hay).
    pub cuerpo: &'a [u8],
}

/// La respuesta para el Worker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Respuesta {
    /// Código HTTP.
    pub estado: u16,
    /// Cuerpo JSON.
    pub cuerpo: String,
    /// Detalle de un error interno para el registro del servidor. Nunca se
    /// manda al cliente.
    pub detalle_interno: Option<String>,
}

/// Lo que la API necesita para atender: los puertos.
pub trait Contexto {
    /// Perfiles.
    type Perfiles: RepositorioPerfiles;
    /// Gastos.
    type Gastos: RepositorioGastos;
    /// Extras.
    type Extras: RepositorioExtras;
    /// Fondo de emergencia.
    type Fondo: RepositorioFondo;
    /// Quincenas.
    type Quincenas: RepositorioQuincenas;
    /// Generador de identificadores.
    type Ids: GeneradorIds;
    /// Reloj.
    type Reloj: Reloj;

    /// Perfiles.
    fn perfiles(&self) -> &Self::Perfiles;
    /// Gastos.
    fn gastos(&self) -> &Self::Gastos;
    /// Extras.
    fn extras(&self) -> &Self::Extras;
    /// Fondo de emergencia.
    fn fondo(&self) -> &Self::Fondo;
    /// Quincenas.
    fn quincenas(&self) -> &Self::Quincenas;
    /// Generador de identificadores.
    fn ids(&self) -> &Self::Ids;
    /// Reloj.
    fn reloj(&self) -> &Self::Reloj;
}

/// Atiende una petición de un usuario ya autenticado.
pub async fn atender<C: Contexto>(ctx: &C, usuario: UsuarioId, p: Peticion<'_>) -> Respuesta {
    match enrutar(ctx, usuario, p).await {
        Ok((estado, cuerpo)) => Respuesta {
            estado,
            cuerpo,
            detalle_interno: None,
        },
        Err(e) => e.respuesta(),
    }
}

fn json<T: Serialize>(estado: u16, valor: &T) -> Resultado {
    serde_json::to_string(valor)
        .map(|cuerpo| (estado, cuerpo))
        .map_err(|e| ErrorApi::Interno(format!("no se pudo serializar: {e}")))
}

const SIN_CONTENIDO: Resultado = Ok((204, String::new()));

fn cuerpo<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ErrorApi> {
    serde_json::from_slice(bytes).map_err(|e| ErrorApi::JsonInvalido(e.to_string()))
}

fn id(texto: &str) -> Result<Uuid, ErrorApi> {
    Uuid::parse_str(texto).map_err(|_| ErrorApi::RutaInexistente)
}

fn fecha(texto: &str) -> Result<NaiveDate, ErrorApi> {
    NaiveDate::parse_from_str(texto, "%Y-%m-%d").map_err(|_| ErrorApi::RutaInexistente)
}

async fn enrutar<C: Contexto>(ctx: &C, usuario: UsuarioId, p: Peticion<'_>) -> Resultado {
    let partes: Vec<&str> = p.ruta.trim_matches('/').split('/').collect();
    let respuesta = match partes.as_slice() {
        ["perfil"] => perfil(ctx, usuario, &p).await,
        ["caja-chica" | "gastos" | "extras", ..] => registro(ctx, usuario, &p, &partes).await,
        ["fondo", ..] => fondo(ctx, usuario, &p, &partes).await,
        ["quincenas", ..] => quincenas(ctx, usuario, &p, &partes).await,
        _ => None,
    };
    respuesta.unwrap_or_else(|| {
        if es_ruta_conocida(&partes) {
            Err(ErrorApi::MetodoNoPermitido)
        } else {
            Err(ErrorApi::RutaInexistente)
        }
    })
}

/// El perfil propio. `None` si el método no es de aquí.
async fn perfil<C: Contexto>(ctx: &C, usuario: UsuarioId, p: &Peticion<'_>) -> Option<Resultado> {
    Some(match p.metodo {
        Metodo::Get => ConsultarPerfil::new(ctx.perfiles())
            .ejecutar(usuario)
            .await
            .map_err(ErrorApi::from)
            .and_then(|r| json(200, &PerfilDto::from(r))),
        Metodo::Put => guardar_perfil(ctx, usuario, p.cuerpo).await,
        _ => return None,
    })
}

async fn guardar_perfil<C: Contexto>(ctx: &C, usuario: UsuarioId, bytes: &[u8]) -> Resultado {
    let n: NuevoPerfil = cuerpo(bytes)?;
    let r = GuardarPerfil::new(ctx.perfiles())
        .ejecutar(SolicitudPerfil {
            usuario,
            salario_base: n.salario_base,
            factor_neto: n.factor_neto,
            rebajos: n.rebajos,
            nocturna: n.tarifas.nocturna,
            diurna: n.tarifas.diurna,
            especial: n.tarifas.especial,
            feriado: n.tarifas.feriado,
            tope_caja_chica: n.tope_caja_chica,
            cortes: n.cortes.into_iter().map(Into::into).collect(),
        })
        .await?;
    json(200, &PerfilDto::from(r))
}

/// Caja chica, gastos y extras. `None` si la ruta no es de aquí.
async fn registro<C: Contexto>(
    ctx: &C,
    usuario: UsuarioId,
    p: &Peticion<'_>,
    partes: &[&str],
) -> Option<Resultado> {
    Some(match (p.metodo, partes) {
        (Metodo::Get, ["caja-chica"]) => {
            ConsultarCajaChica::new(ctx.perfiles(), ctx.gastos(), ctx.reloj())
                .ejecutar(usuario)
                .await
                .map_err(ErrorApi::from)
                .and_then(|r| json(200, &CajaDto::from(r)))
        }
        (Metodo::Post, ["gastos"]) => registrar_gasto(ctx, usuario, p.cuerpo).await,
        (Metodo::Post, ["extras"]) => registrar_extra(ctx, usuario, p.cuerpo).await,
        (Metodo::Delete, ["gastos", texto]) => match id(texto) {
            Ok(uuid) => deshacer(ctx, usuario, QueDeshacer::Gasto(GastoId::new(uuid))).await,
            Err(e) => Err(e),
        },
        (Metodo::Delete, ["extras", texto]) => match id(texto) {
            Ok(uuid) => deshacer(ctx, usuario, QueDeshacer::Extra(ExtraId::new(uuid))).await,
            Err(e) => Err(e),
        },
        _ => return None,
    })
}

async fn registrar_gasto<C: Contexto>(ctx: &C, usuario: UsuarioId, bytes: &[u8]) -> Resultado {
    let g: NuevoGasto = cuerpo(bytes)?;
    let r = RegistrarGasto::new(ctx.perfiles(), ctx.gastos(), ctx.ids(), ctx.reloj())
        .ejecutar(SolicitudGasto {
            usuario,
            fecha: g.fecha,
            concepto: g.concepto,
            monto: g.monto,
            categoria: g.categoria.into(),
            origen: g.origen.into(),
        })
        .await?;
    json(201, &GastoRegistradoDto::from(r))
}

async fn registrar_extra<C: Contexto>(ctx: &C, usuario: UsuarioId, bytes: &[u8]) -> Resultado {
    let e: NuevaExtra = cuerpo(bytes)?;
    let r = RegistrarExtra::new(ctx.perfiles(), ctx.extras(), ctx.ids())
        .ejecutar(SolicitudExtra {
            usuario,
            fecha: e.fecha,
            tipo: e.tipo.into(),
            monto: e.monto,
        })
        .await?;
    json(201, &ExtraDto::from(r))
}

/// El fondo de emergencia. `None` si la ruta no es de aquí.
async fn fondo<C: Contexto>(
    ctx: &C,
    usuario: UsuarioId,
    p: &Peticion<'_>,
    partes: &[&str],
) -> Option<Resultado> {
    Some(match (p.metodo, partes) {
        (Metodo::Get, ["fondo"]) => ConsultarFondo::new(ctx.fondo())
            .ejecutar(usuario)
            .await
            .map_err(ErrorApi::from)
            .and_then(|e| json(200, &FondoDto::from(e))),
        (Metodo::Post, ["fondo", "retiros"]) => sacar(ctx, usuario, p.cuerpo).await,
        (Metodo::Post, ["fondo", "depositos"]) => depositar(ctx, usuario, p.cuerpo).await,
        (Metodo::Delete, ["fondo", "movimientos", texto]) => match id(texto) {
            Ok(uuid) => {
                let que = QueDeshacer::MovimientoFondo(MovimientoFondoId::new(uuid));
                deshacer(ctx, usuario, que).await
            }
            Err(e) => Err(e),
        },
        _ => return None,
    })
}

async fn sacar<C: Contexto>(ctx: &C, usuario: UsuarioId, bytes: &[u8]) -> Resultado {
    let r: NuevoRetiro = cuerpo(bytes)?;
    let hecho = SacarDelFondo::new(ctx.fondo(), ctx.ids(), ctx.reloj())
        .ejecutar(SolicitudSacar {
            usuario,
            concepto: r.concepto,
            monto: r.monto,
            nivel: r.nivel.into(),
            evaluacion: Evaluacion {
                inesperado: r.inesperado,
                necesario: r.necesario,
                urgente: r.urgente,
            },
        })
        .await?;
    json(201, &RetiroDto::from(hecho))
}

async fn depositar<C: Contexto>(ctx: &C, usuario: UsuarioId, bytes: &[u8]) -> Resultado {
    let d: NuevoDeposito = cuerpo(bytes)?;
    let e = DepositarEnFondo::new(ctx.fondo(), ctx.ids(), ctx.reloj())
        .ejecutar(SolicitudDeposito {
            usuario,
            deposito: d.tipo.into(),
            concepto: d.concepto,
            monto: d.monto,
        })
        .await?;
    json(201, &FondoDto::from(e))
}

/// Las quincenas. `None` si la ruta no es de aquí.
async fn quincenas<C: Contexto>(
    ctx: &C,
    usuario: UsuarioId,
    p: &Peticion<'_>,
    partes: &[&str],
) -> Option<Resultado> {
    Some(match (p.metodo, partes) {
        (Metodo::Get, ["quincenas", cual]) => consultar_quincena(ctx, usuario, cual).await,
        (Metodo::Put, ["quincenas", texto]) => {
            guardar_presupuesto(ctx, usuario, texto, p.cuerpo).await
        }
        (Metodo::Put, ["quincenas", texto, "marcas"]) => {
            marcar(ctx, usuario, texto, p.cuerpo).await
        }
        _ => return None,
    })
}

async fn consultar_quincena<C: Contexto>(ctx: &C, usuario: UsuarioId, cual: &str) -> Resultado {
    let pago = if cual == "actual" {
        None
    } else {
        Some(fecha(cual)?)
    };
    let r = ConsultarQuincena::new(ctx.perfiles(), ctx.quincenas(), ctx.reloj())
        .ejecutar(usuario, pago)
        .await?
        .ok_or(ErrorApi::SinPresupuesto)?;
    json(200, &QuincenaDto::from(r))
}

async fn guardar_presupuesto<C: Contexto>(
    ctx: &C,
    usuario: UsuarioId,
    texto: &str,
    bytes: &[u8],
) -> Resultado {
    let pago = fecha(texto)?;
    let q: Presupuesto = cuerpo(bytes)?;
    let r = GuardarPresupuesto::new(ctx.perfiles(), ctx.quincenas())
        .ejecutar(SolicitudPresupuesto {
            usuario,
            pago,
            ingresos: q.ingresos.into_iter().map(Into::into).collect(),
            pagos: q.pagos.into_iter().map(Into::into).collect(),
        })
        .await?;
    json(200, &QuincenaDto::from(r))
}

async fn marcar<C: Contexto>(ctx: &C, usuario: UsuarioId, texto: &str, bytes: &[u8]) -> Resultado {
    let pago = fecha(texto)?;
    let m: Marca = cuerpo(bytes)?;
    let r = MarcarLinea::new(ctx.quincenas())
        .ejecutar(SolicitudMarca {
            usuario,
            pago,
            lado: m.lado.into(),
            indice: m.indice,
            hecho: m.hecho,
        })
        .await?;
    json(200, &QuincenaDto::from(r))
}

/// Para distinguir 405 (la ruta existe, el método no) de 404.
fn es_ruta_conocida(partes: &[&str]) -> bool {
    matches!(
        partes,
        ["perfil" | "caja-chica" | "gastos" | "extras" | "fondo" | "quincenas"]
            | ["gastos" | "extras" | "quincenas", _]
            | ["fondo", "retiros" | "depositos"]
            | ["fondo", "movimientos", _]
            | ["quincenas", _, "marcas"]
    )
}

async fn deshacer<C: Contexto>(ctx: &C, usuario: UsuarioId, que: QueDeshacer) -> Resultado {
    Deshacer::new(ctx.gastos(), ctx.extras(), ctx.fondo())
        .ejecutar(usuario, que)
        .await?;
    SIN_CONTENIDO
}
