//! Implementaciones en memoria de los puertos.
//!
//! Sirven para probar los casos de uso sin base de datos y para desarrollar la
//! interfaz sin red. Se comportan como un repositorio real: filtran por usuario
//! y por fecha igual que lo hará SurrealDB.

use chrono::NaiveDate;
use finanzas_aplicacion::errores::ErrorRepositorio;
use finanzas_aplicacion::puertos::{
    GeneradorIds, Reloj, RepositorioExtras, RepositorioFondo, RepositorioGastos,
    RepositorioPerfiles, RepositorioQuincenas,
};
use finanzas_dominio::extra::{Extra, ExtraId};
use finanzas_dominio::gasto::{Gasto, GastoId};
use finanzas_dominio::movimientos_fondo::{MovimientoFondo, MovimientoFondoId};
use finanzas_dominio::perfil::Perfil;
use finanzas_dominio::quincena::Quincena;
use finanzas_dominio::usuario::UsuarioId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use uuid::Uuid;

fn cerrojo<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // Un pánico en otra prueba no debe envenenar a las demás.
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Agrega un registro. Igual que SurrealDB, guardar dos veces el mismo id es
/// un error, no una sobrescritura.
fn insertar<T: Clone, K: PartialEq + std::fmt::Debug>(
    m: &Mutex<Vec<T>>,
    nuevo: &T,
    id: impl Fn(&T) -> K,
) -> Result<(), ErrorRepositorio> {
    let mut lista = cerrojo(m);
    if lista.iter().any(|x| id(x) == id(nuevo)) {
        return Err(ErrorRepositorio(format!(
            "ya existe un registro con el id {:?}",
            id(nuevo)
        )));
    }
    lista.push(nuevo.clone());
    drop(lista);
    Ok(())
}

/// Quita el registro que cumpla `es_este`. `true` si había uno.
fn quitar<T>(m: &Mutex<Vec<T>>, es_este: impl Fn(&T) -> bool) -> bool {
    let mut lista = cerrojo(m);
    let antes = lista.len();
    lista.retain(|x| !es_este(x));
    antes != lista.len()
}

/// Un reloj que siempre dice la misma fecha.
#[derive(Debug, Clone, Copy)]
pub struct RelojFijo(pub NaiveDate);

impl Reloj for RelojFijo {
    fn hoy(&self) -> NaiveDate {
        self.0
    }
}

/// IDs predecibles: 1, 2, 3…
#[derive(Debug, Default)]
pub struct IdsSecuenciales(AtomicU64);

impl GeneradorIds for IdsSecuenciales {
    fn nuevo(&self) -> Uuid {
        Uuid::from_u128(u128::from(self.0.fetch_add(1, Ordering::Relaxed) + 1))
    }
}

/// Perfiles en memoria.
#[derive(Debug, Default)]
pub struct PerfilesEnMemoria(Mutex<Vec<Perfil>>);

impl PerfilesEnMemoria {
    /// Agrega o reemplaza el perfil de un usuario.
    pub fn poner(&self, perfil: Perfil) {
        let mut perfiles = cerrojo(&self.0);
        perfiles.retain(|p| p.usuario() != perfil.usuario());
        perfiles.push(perfil);
    }
}

impl RepositorioPerfiles for PerfilesEnMemoria {
    async fn perfil(&self, usuario: UsuarioId) -> Result<Option<Perfil>, ErrorRepositorio> {
        Ok(cerrojo(&self.0)
            .iter()
            .find(|p| p.usuario() == usuario)
            .cloned())
    }

    async fn guardar(&self, perfil: &Perfil) -> Result<(), ErrorRepositorio> {
        self.poner(perfil.clone());
        Ok(())
    }
}

/// Gastos en memoria.
#[derive(Debug, Default)]
pub struct GastosEnMemoria(Mutex<Vec<Gasto>>);

impl RepositorioGastos for GastosEnMemoria {
    async fn guardar(&self, gasto: &Gasto) -> Result<(), ErrorRepositorio> {
        insertar(&self.0, gasto, Gasto::id)
    }

    async fn entre(
        &self,
        usuario: UsuarioId,
        desde: NaiveDate,
        hasta: NaiveDate,
    ) -> Result<Vec<Gasto>, ErrorRepositorio> {
        let mut gastos: Vec<Gasto> = cerrojo(&self.0)
            .iter()
            .filter(|g| g.usuario() == usuario && desde <= g.fecha() && g.fecha() <= hasta)
            .cloned()
            .collect();
        gastos.sort_by_key(|g| (g.fecha(), g.id()));
        Ok(gastos)
    }

    async fn borrar(&self, usuario: UsuarioId, id: GastoId) -> Result<bool, ErrorRepositorio> {
        Ok(quitar(&self.0, |g| g.usuario() == usuario && g.id() == id))
    }
}

/// Extras en memoria.
#[derive(Debug, Default)]
pub struct ExtrasEnMemoria(Mutex<Vec<Extra>>);

impl RepositorioExtras for ExtrasEnMemoria {
    async fn guardar(&self, extra: &Extra) -> Result<(), ErrorRepositorio> {
        insertar(&self.0, extra, Extra::id)
    }

    async fn de_usuario(&self, usuario: UsuarioId) -> Result<Vec<Extra>, ErrorRepositorio> {
        let mut extras: Vec<Extra> = cerrojo(&self.0)
            .iter()
            .filter(|e| e.usuario() == usuario)
            .cloned()
            .collect();
        extras.sort_by_key(|e| (e.fecha(), e.id()));
        Ok(extras)
    }

    async fn borrar(&self, usuario: UsuarioId, id: ExtraId) -> Result<bool, ErrorRepositorio> {
        Ok(quitar(&self.0, |e| e.usuario() == usuario && e.id() == id))
    }
}

/// Movimientos del fondo en memoria.
#[derive(Debug, Default)]
pub struct FondoEnMemoria(Mutex<Vec<MovimientoFondo>>);

impl RepositorioFondo for FondoEnMemoria {
    async fn guardar(&self, movimiento: &MovimientoFondo) -> Result<(), ErrorRepositorio> {
        insertar(&self.0, movimiento, MovimientoFondo::id)
    }

    async fn de_usuario(
        &self,
        usuario: UsuarioId,
    ) -> Result<Vec<MovimientoFondo>, ErrorRepositorio> {
        let mut movimientos: Vec<MovimientoFondo> = cerrojo(&self.0)
            .iter()
            .filter(|m| m.usuario() == usuario)
            .cloned()
            .collect();
        movimientos.sort_by_key(|m| (m.fecha(), m.id()));
        Ok(movimientos)
    }

    async fn borrar(
        &self,
        usuario: UsuarioId,
        id: MovimientoFondoId,
    ) -> Result<bool, ErrorRepositorio> {
        Ok(quitar(&self.0, |m| m.usuario() == usuario && m.id() == id))
    }
}

/// Quincenas en memoria.
#[derive(Debug, Default)]
pub struct QuincenasEnMemoria(Mutex<Vec<Quincena>>);

impl RepositorioQuincenas for QuincenasEnMemoria {
    async fn guardar(&self, quincena: &Quincena) -> Result<(), ErrorRepositorio> {
        let mut quincenas = cerrojo(&self.0);
        quincenas.retain(|q| (q.usuario(), q.pago()) != (quincena.usuario(), quincena.pago()));
        quincenas.push(quincena.clone());
        drop(quincenas);
        Ok(())
    }

    async fn de(
        &self,
        usuario: UsuarioId,
        pago: NaiveDate,
    ) -> Result<Option<Quincena>, ErrorRepositorio> {
        Ok(cerrojo(&self.0)
            .iter()
            .find(|q| q.usuario() == usuario && q.pago() == pago)
            .cloned())
    }
}
