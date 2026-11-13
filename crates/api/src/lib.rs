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
//! Está partida por tema: cada módulo trae el JSON de su tema y las rutas que
//! lo atienden, con su propia tabla de rutas.
//!
//! | Módulo | Rutas |
//! |---|---|
//! | [`perfil`] | `/perfil` |
//! | [`calendario`] | `/cortes/hoy` |
//! | [`gastos`] | `/caja-chica`, `/gastos` |
//! | [`extras`] | `/extras` |
//! | [`fondo`] | `/fondo` |
//! | [`quincena`] | `/quincenas`, `/meses` |
//!
//! `lib.rs` sólo reparte: mira la primera parte de la ruta y le pasa la
//! petición al módulo que le toca.

#![expect(
    clippy::future_not_send,
    reason = "sin `Send` a propósito: en el Worker (WebAssembly) los futuros no son `Send`"
)]

pub mod calendario;
pub mod extras;
pub mod fondo;
pub mod gastos;
pub mod perfil;
pub mod quincena;

mod errores;

use chrono::NaiveDate;
use errores::{ErrorApi, Resultado};
use finanzas_aplicacion::errores::ErrorAplicacion;
use finanzas_aplicacion::puertos::{
    GeneradorIds, Reloj, RepositorioExtras, RepositorioFondo, RepositorioGastos,
    RepositorioPerfiles, RepositorioQuincenas,
};
use finanzas_dominio::base::usuario::UsuarioId;
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

pub(crate) fn json<T: Serialize>(estado: u16, valor: &T) -> Resultado {
    serde_json::to_string(valor)
        .map(|cuerpo| (estado, cuerpo))
        .map_err(|e| ErrorApi::Interno(format!("no se pudo serializar: {e}")))
}

pub(crate) const SIN_CONTENIDO: Resultado = Ok((204, String::new()));

pub(crate) fn cuerpo<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ErrorApi> {
    serde_json::from_slice(bytes).map_err(|e| ErrorApi::JsonInvalido(e.to_string()))
}

pub(crate) fn id(texto: &str) -> Result<Uuid, ErrorApi> {
    Uuid::parse_str(texto).map_err(|_| ErrorApi::RutaInexistente)
}

pub(crate) fn fecha(texto: &str) -> Result<NaiveDate, ErrorApi> {
    NaiveDate::parse_from_str(texto, "%Y-%m-%d").map_err(|_| ErrorApi::RutaInexistente)
}

async fn enrutar<C: Contexto>(ctx: &C, usuario: UsuarioId, p: Peticion<'_>) -> Resultado {
    let partes: Vec<&str> = p.ruta.trim_matches('/').split('/').collect();
    let respuesta = match partes.as_slice() {
        ["perfil"] => perfil::enrutar(ctx, usuario, &p).await,
        ["cortes", "hoy"] => calendario::enrutar(ctx, usuario, &p).await,
        ["caja-chica" | "gastos", ..] => gastos::enrutar(ctx, usuario, &p, &partes).await,
        ["extras", ..] => extras::enrutar(ctx, usuario, &p, &partes).await,
        ["fondo", ..] => fondo::enrutar(ctx, usuario, &p, &partes).await,
        ["quincenas" | "meses", ..] => quincena::enrutar(ctx, usuario, &p, &partes).await,
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

/// Para distinguir 405 (la ruta existe, el método no) de 404.
fn es_ruta_conocida(partes: &[&str]) -> bool {
    matches!(
        partes,
        ["perfil" | "caja-chica" | "gastos" | "extras" | "fondo" | "quincenas"]
            | ["gastos" | "extras" | "quincenas" | "meses", _]
            | ["gastos", _, _]
            | ["fondo", "retiros" | "depositos"]
            | ["fondo", "movimientos", _]
            | ["quincenas", _, "marcas"]
            | ["cortes", "hoy"]
    )
}

/// Un borrado que salió bien no devuelve cuerpo.
pub(crate) fn sin_contenido(r: Result<(), ErrorAplicacion>) -> Resultado {
    r.map_err(ErrorApi::from)?;
    SIN_CONTENIDO
}
