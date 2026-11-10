//! Puertos: lo que los casos de uso necesitan del mundo exterior.
//!
//! Se definen aquí y los implementa la infraestructura (SurrealDB, memoria
//! para pruebas). Son asíncronos porque la base de datos y el Worker lo son;
//! sin exigir `Send`, para que funcionen también en WebAssembly.

use crate::errores::ErrorRepositorio;
use chrono::NaiveDate;
use finanzas_dominio::extra::Extra;
use finanzas_dominio::gasto::Gasto;
use finanzas_dominio::perfil::Perfil;
use finanzas_dominio::usuario::UsuarioId;
use uuid::Uuid;

/// La fecha de hoy, en la zona del usuario.
pub trait Reloj {
    /// Hoy.
    fn hoy(&self) -> NaiveDate;
}

/// Genera identificadores nuevos.
pub trait GeneradorIds {
    /// Un UUID nuevo.
    fn nuevo(&self) -> Uuid;
}

/// La configuración de cada usuario.
#[expect(
    async_fn_in_trait,
    reason = "sin `Send` a propósito: en WebAssembly los futuros no son `Send`"
)]
pub trait RepositorioPerfiles {
    /// El perfil del usuario, si tiene.
    async fn perfil(&self, usuario: UsuarioId) -> Result<Option<Perfil>, ErrorRepositorio>;
}

/// Los gastos puntuales.
#[expect(
    async_fn_in_trait,
    reason = "sin `Send` a propósito: en WebAssembly los futuros no son `Send`"
)]
pub trait RepositorioGastos {
    /// Guarda un gasto nuevo.
    async fn guardar(&self, gasto: &Gasto) -> Result<(), ErrorRepositorio>;
    /// Los gastos de un usuario entre dos fechas (incluidas).
    async fn entre(
        &self,
        usuario: UsuarioId,
        desde: NaiveDate,
        hasta: NaiveDate,
    ) -> Result<Vec<Gasto>, ErrorRepositorio>;
}

/// Las extras trabajadas.
#[expect(
    async_fn_in_trait,
    reason = "sin `Send` a propósito: en WebAssembly los futuros no son `Send`"
)]
pub trait RepositorioExtras {
    /// Guarda una extra nueva.
    async fn guardar(&self, extra: &Extra) -> Result<(), ErrorRepositorio>;
    /// Todas las extras de un usuario.
    async fn de_usuario(&self, usuario: UsuarioId) -> Result<Vec<Extra>, ErrorRepositorio>;
}
