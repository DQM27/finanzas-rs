//! Consultas de lectura para la interfaz: listas de lo ya guardado.
//!
//! No deciden nada: buscan, dejan que el dominio calcule lo derivado (neto,
//! fecha de pago) y devuelven.

use crate::casos_de_uso::registrar_extra::ExtraRegistrada;
use crate::errores::ErrorAplicacion;
use crate::puertos::{RepositorioExtras, RepositorioFondo, RepositorioGastos, RepositorioPerfiles};
use chrono::NaiveDate;
use finanzas_dominio::base::usuario::UsuarioId;
use finanzas_dominio::fondo::movimientos::MovimientoFondo;
use finanzas_dominio::gastos::Gasto;

/// Caso de uso: los gastos de un rango de fechas, ambos incluidos.
#[derive(Debug)]
pub struct ListarGastos<'a, G> {
    gastos: &'a G,
}

impl<'a, G: RepositorioGastos> ListarGastos<'a, G> {
    /// Arma el caso de uso.
    pub const fn new(gastos: &'a G) -> Self {
        Self { gastos }
    }

    /// Los gastos del usuario entre las dos fechas, por fecha y luego por id.
    pub async fn ejecutar(
        &self,
        usuario: UsuarioId,
        desde: NaiveDate,
        hasta: NaiveDate,
    ) -> Result<Vec<Gasto>, ErrorAplicacion> {
        Ok(self.gastos.entre(usuario, desde, hasta).await?)
    }
}

/// Caso de uso: todas las extras, con su neto y su fecha de pago.
#[derive(Debug)]
pub struct ListarExtras<'a, P, E> {
    perfiles: &'a P,
    extras: &'a E,
}

impl<'a, P, E> ListarExtras<'a, P, E>
where
    P: RepositorioPerfiles,
    E: RepositorioExtras,
{
    /// Arma el caso de uso.
    pub const fn new(perfiles: &'a P, extras: &'a E) -> Self {
        Self { perfiles, extras }
    }

    /// Las extras del usuario con lo que el dominio calcula de cada una.
    pub async fn ejecutar(
        &self,
        usuario: UsuarioId,
    ) -> Result<Vec<ExtraRegistrada>, ErrorAplicacion> {
        let perfil = self
            .perfiles
            .perfil(usuario)
            .await?
            .ok_or(ErrorAplicacion::PerfilNoEncontrado)?;
        Ok(self
            .extras
            .de_usuario(usuario)
            .await?
            .into_iter()
            .map(|extra| ExtraRegistrada {
                neto: extra.neto(perfil.salarial()),
                fecha_pago: extra.fecha_pago(perfil.calendario()),
                extra,
            })
            .collect())
    }
}

/// Caso de uso: los movimientos del fondo, del más viejo al más nuevo.
#[derive(Debug)]
pub struct ListarMovimientosFondo<'a, F> {
    fondo: &'a F,
}

impl<'a, F: RepositorioFondo> ListarMovimientosFondo<'a, F> {
    /// Arma el caso de uso.
    pub const fn new(fondo: &'a F) -> Self {
        Self { fondo }
    }

    /// Los movimientos del usuario.
    pub async fn ejecutar(
        &self,
        usuario: UsuarioId,
    ) -> Result<Vec<MovimientoFondo>, ErrorAplicacion> {
        Ok(self.fondo.de_usuario(usuario).await?)
    }
}
