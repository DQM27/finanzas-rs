//! Cuánto queda en la caja chica del periodo en curso.

use crate::errores::ErrorAplicacion;
use crate::puertos::{Reloj, RepositorioGastos, RepositorioPerfiles};
use chrono::NaiveDate;
use finanzas_dominio::caja_chica::EstadoCaja;
use finanzas_dominio::gasto::gastado_en_caja_chica;
use finanzas_dominio::usuario::UsuarioId;

/// El estado de la caja chica en el periodo de gasto actual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResumenCaja {
    /// Primer día del periodo (el día de pago).
    pub desde: NaiveDate,
    /// Último día del periodo (el día antes del próximo pago).
    pub hasta: NaiveDate,
    /// Lo que queda, el exceso y el nivel.
    pub estado: EstadoCaja,
}

/// Caso de uso: consultar la caja chica.
#[derive(Debug)]
pub struct ConsultarCajaChica<'a, P, G, R> {
    perfiles: &'a P,
    gastos: &'a G,
    reloj: &'a R,
}

impl<'a, P, G, R> ConsultarCajaChica<'a, P, G, R>
where
    P: RepositorioPerfiles,
    G: RepositorioGastos,
    R: Reloj,
{
    /// Arma el caso de uso con sus puertos.
    pub const fn new(perfiles: &'a P, gastos: &'a G, reloj: &'a R) -> Self {
        Self {
            perfiles,
            gastos,
            reloj,
        }
    }

    /// Calcula el estado de la caja chica de hoy.
    pub async fn ejecutar(&self, usuario: UsuarioId) -> Result<ResumenCaja, ErrorAplicacion> {
        let perfil = self
            .perfiles
            .perfil(usuario)
            .await?
            .ok_or(ErrorAplicacion::PerfilNoEncontrado)?;
        let (desde, hasta) = perfil
            .calendario()
            .periodo_de_gasto(self.reloj.hoy())
            .ok_or(ErrorAplicacion::FueraDeCalendario)?;
        let gastos = self.gastos.entre(usuario, desde, hasta).await?;
        let estado = perfil
            .caja_chica()
            .estado(gastado_en_caja_chica(&gastos, desde, hasta));
        Ok(ResumenCaja {
            desde,
            hasta,
            estado,
        })
    }
}
