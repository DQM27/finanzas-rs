//! Implementaciones en memoria de los puertos.
//!
//! Sirven para probar los casos de uso sin base de datos y para desarrollar la
//! interfaz sin red. Se comportan como un repositorio real: filtran por usuario
//! y por fecha igual que lo hará SurrealDB.

use chrono::NaiveDate;
use finanzas_aplicacion::errores::ErrorRepositorio;
use finanzas_aplicacion::puertos::{
    GeneradorIds, Reloj, RepositorioExtras, RepositorioGastos, RepositorioPerfiles,
};
use finanzas_dominio::extra::Extra;
use finanzas_dominio::gasto::Gasto;
use finanzas_dominio::perfil::Perfil;
use finanzas_dominio::usuario::UsuarioId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use uuid::Uuid;

fn cerrojo<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // Un pánico en otra prueba no debe envenenar a las demás.
    m.lock().unwrap_or_else(PoisonError::into_inner)
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
        cerrojo(&self.0).push(gasto.clone());
        Ok(())
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
}

/// Extras en memoria.
#[derive(Debug, Default)]
pub struct ExtrasEnMemoria(Mutex<Vec<Extra>>);

impl RepositorioExtras for ExtrasEnMemoria {
    async fn guardar(&self, extra: &Extra) -> Result<(), ErrorRepositorio> {
        cerrojo(&self.0).push(extra.clone());
        Ok(())
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
}
