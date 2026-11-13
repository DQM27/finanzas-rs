//! Puertos: lo que los casos de uso necesitan del mundo exterior.
//!
//! Se definen aquí y los implementa la infraestructura (SurrealDB, memoria
//! para pruebas). Son asíncronos porque la base de datos y el Worker lo son;
//! sin exigir `Send`, para que funcionen también en WebAssembly.

use crate::errores::ErrorRepositorio;
use chrono::NaiveDate;
use finanzas_dominio::base::usuario::UsuarioId;
use finanzas_dominio::extras::{Extra, ExtraId};
use finanzas_dominio::fondo::movimientos::{MovimientoFondo, MovimientoFondoId};
use finanzas_dominio::gastos::{Gasto, GastoId};
use finanzas_dominio::perfil::Perfil;
use finanzas_dominio::quincena::Quincena;
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
    /// Guarda el perfil; si el usuario ya tenía uno, lo reemplaza.
    async fn guardar(&self, perfil: &Perfil) -> Result<(), ErrorRepositorio>;
}

/// Los gastos puntuales.
#[expect(
    async_fn_in_trait,
    reason = "sin `Send` a propósito: en WebAssembly los futuros no son `Send`"
)]
pub trait RepositorioGastos {
    /// Guarda un gasto nuevo.
    async fn guardar(&self, gasto: &Gasto) -> Result<(), ErrorRepositorio>;
    /// Los gastos de un usuario entre dos fechas (incluidas), ordenados por
    /// fecha y, el mismo día, por identificador.
    async fn entre(
        &self,
        usuario: UsuarioId,
        desde: NaiveDate,
        hasta: NaiveDate,
    ) -> Result<Vec<Gasto>, ErrorRepositorio>;
    /// Borra un gasto del usuario. `false` si no existe o es de otro.
    async fn borrar(&self, usuario: UsuarioId, id: GastoId) -> Result<bool, ErrorRepositorio>;
}

/// Las extras trabajadas.
#[expect(
    async_fn_in_trait,
    reason = "sin `Send` a propósito: en WebAssembly los futuros no son `Send`"
)]
pub trait RepositorioExtras {
    /// Guarda una extra nueva.
    async fn guardar(&self, extra: &Extra) -> Result<(), ErrorRepositorio>;
    /// Todas las extras de un usuario, ordenadas por fecha y, el mismo día,
    /// por identificador.
    async fn de_usuario(&self, usuario: UsuarioId) -> Result<Vec<Extra>, ErrorRepositorio>;
    /// Borra una extra del usuario. `false` si no existe o es de otro.
    async fn borrar(&self, usuario: UsuarioId, id: ExtraId) -> Result<bool, ErrorRepositorio>;
}

/// Los movimientos del fondo de emergencia.
#[expect(
    async_fn_in_trait,
    reason = "sin `Send` a propósito: en WebAssembly los futuros no son `Send`"
)]
pub trait RepositorioFondo {
    /// Guarda un movimiento nuevo.
    async fn guardar(&self, movimiento: &MovimientoFondo) -> Result<(), ErrorRepositorio>;
    /// Todos los movimientos de un usuario, ordenados por fecha y, el mismo
    /// día, por identificador.
    async fn de_usuario(
        &self,
        usuario: UsuarioId,
    ) -> Result<Vec<MovimientoFondo>, ErrorRepositorio>;
    /// Borra un movimiento del usuario. `false` si no existe o es de otro.
    async fn borrar(
        &self,
        usuario: UsuarioId,
        id: MovimientoFondoId,
    ) -> Result<bool, ErrorRepositorio>;
}

/// El presupuesto de cada quincena.
#[expect(
    async_fn_in_trait,
    reason = "sin `Send` a propósito: en WebAssembly los futuros no son `Send`"
)]
pub trait RepositorioQuincenas {
    /// Guarda la quincena; si ya había una del mismo usuario y pago, la
    /// reemplaza.
    async fn guardar(&self, quincena: &Quincena) -> Result<(), ErrorRepositorio>;
    /// La quincena de un usuario financiada por el pago de esa fecha.
    async fn de(
        &self,
        usuario: UsuarioId,
        pago: NaiveDate,
    ) -> Result<Option<Quincena>, ErrorRepositorio>;
}
