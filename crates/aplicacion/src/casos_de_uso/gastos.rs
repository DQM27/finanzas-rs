//! Los gastos del día: anotarlos, listarlos, ver la caja chica y deshacer
//! el que se anotó mal.

use crate::errores::ErrorAplicacion;
use crate::puertos::{GeneradorIds, Reloj, RepositorioGastos, RepositorioPerfiles};
use chrono::NaiveDate;
use finanzas_dominio::base::dinero::Colones;
use finanzas_dominio::base::texto::Concepto;
use finanzas_dominio::base::usuario::UsuarioId;
use finanzas_dominio::gastos::caja_chica::EstadoCaja;
use finanzas_dominio::gastos::gastado_en_caja_chica;
use finanzas_dominio::gastos::{Categoria, DatosGasto, Gasto, GastoId, Origen};

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

/// Caso de uso: deshacer un gasto que se anotó mal.
#[derive(Debug)]
pub struct DeshacerGasto<'a, G> {
    gastos: &'a G,
}

impl<'a, G: RepositorioGastos> DeshacerGasto<'a, G> {
    /// Arma el caso de uso.
    pub const fn new(gastos: &'a G) -> Self {
        Self { gastos }
    }

    /// Borra el gasto si el usuario es su dueño (U1). Un id de otro es como
    /// si no existiera.
    pub async fn ejecutar(&self, usuario: UsuarioId, id: GastoId) -> Result<(), ErrorAplicacion> {
        if self.gastos.borrar(usuario, id).await? {
            Ok(())
        } else {
            Err(ErrorAplicacion::NoEncontrado)
        }
    }
}
