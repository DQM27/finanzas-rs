//! Adaptadores de los puertos sobre SurrealDB.
//!
//! Hablan con SurrealDB por su RPC binario (CBOR) sobre HTTP. Se usa CBOR y no
//! JSON porque en JSON SurrealDB convierte por su cuenta todo texto que parezca
//! una fecha o un UUID: un concepto como `"2026-11-12"` dejaría de ser texto.
//!
//! El transporte HTTP no está aquí: lo pone quien use el adaptador a través de
//! [`EnvioHttp`] (el `fetch` del Worker en producción, un cliente nativo en las
//! pruebas). Así este crate compila igual para WebAssembly y no elige runtime.
//!
//! Las consultas se escriben a mano, sin ORM, y viven junto al repositorio que
//! las usa.

#![expect(
    clippy::future_not_send,
    reason = "sin `Send` a propósito: en el Worker (WebAssembly) los futuros no son `Send`"
)]

mod cbor;
mod cliente;
mod codigos;
mod repositorio;

pub use cliente::{ClienteSurreal, Conexion, Credencial, EnvioHttp, PeticionHttp, RespuestaHttp};
pub use repositorio::Surreal;

/// El esquema completo (tablas, campos, índices y permisos). Se puede aplicar
/// varias veces.
pub const ESQUEMA: &str = include_str!("esquema.surql");
