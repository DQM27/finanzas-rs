//! Anotar un gasto puntual y ver cómo queda la caja chica.

use crate::casos_de_uso::consultar_caja_chica::{ConsultarCajaChica, ResumenCaja};
use crate::errores::ErrorAplicacion;
use crate::puertos::{GeneradorIds, Reloj, RepositorioGastos, RepositorioPerfiles};
use chrono::NaiveDate;
use finanzas_dominio::dinero::Colones;
use finanzas_dominio::gasto::{Categoria, DatosGasto, Gasto, GastoId, Origen};
use finanzas_dominio::texto::Concepto;
use finanzas_dominio::usuario::UsuarioId;

/// Lo que llega desde la interfaz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolicitudGasto {
    /// Dueño.
    pub usuario: UsuarioId,
    /// Día del gasto.
    pub fecha: NaiveDate,
    /// En qué (texto libre; el dominio lo normaliza).
    pub concepto: String,
    /// Cuánto, en colones.
    pub monto: i64,
    /// Categoría.
    pub categoria: Categoria,
    /// De dónde salió.
    pub origen: Origen,
}

/// El gasto guardado y la caja chica después de anotarlo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GastoRegistrado {
    /// El gasto tal como quedó.
    pub gasto: Gasto,
    /// La caja chica del periodo en curso.
    pub caja: ResumenCaja,
}

/// Caso de uso: registrar un gasto puntual.
#[derive(Debug)]
pub struct RegistrarGasto<'a, P, G, I, R> {
    perfiles: &'a P,
    gastos: &'a G,
    ids: &'a I,
    reloj: &'a R,
}

impl<'a, P, G, I, R> RegistrarGasto<'a, P, G, I, R>
where
    P: RepositorioPerfiles,
    G: RepositorioGastos,
    I: GeneradorIds,
    R: Reloj,
{
    /// Arma el caso de uso con sus puertos.
    pub const fn new(perfiles: &'a P, gastos: &'a G, ids: &'a I, reloj: &'a R) -> Self {
        Self {
            perfiles,
            gastos,
            ids,
            reloj,
        }
    }

    /// Valida (en el dominio), guarda y devuelve la caja chica actualizada.
    pub async fn ejecutar(&self, s: SolicitudGasto) -> Result<GastoRegistrado, ErrorAplicacion> {
        let datos = DatosGasto {
            fecha: s.fecha,
            concepto: Concepto::new(&s.concepto)?,
            monto: Colones::new(s.monto),
            categoria: s.categoria,
            origen: s.origen,
        };
        let gasto = Gasto::new(GastoId::new(self.ids.nuevo()), s.usuario, datos)?;
        self.gastos.guardar(&gasto).await?;
        let caja = ConsultarCajaChica::new(self.perfiles, self.gastos, self.reloj)
            .ejecutar(s.usuario)
            .await?;
        Ok(GastoRegistrado { gasto, caja })
    }
}
