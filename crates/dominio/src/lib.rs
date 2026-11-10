//! Reglas de negocio de finanzas-rs.
//!
//! Todo lo que decide algo sobre la plata vive aquí: cuánto queda neto de una
//! extra, en qué pago cae, cuándo la caja chica se acabó, cuánto se repone
//! después de sacar del fondo. Las capas de afuera sólo buscan datos, se los
//! pasan al dominio y guardan lo que el dominio decide.
//!
//! El dominio no lee la hora ni genera IDs: los recibe.

pub mod caja_chica;
pub mod calendario;
pub mod dinero;
pub mod extra;
pub mod fondo;
pub mod gasto;
pub mod movimientos_fondo;
pub mod perfil;
pub mod proporcion;
pub mod quincena;
pub mod texto;
pub mod usuario;
