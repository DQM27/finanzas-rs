//! El perfil de cada usuario: su salario, tarifas, caja chica y calendario.
//!
//! Las validaciones viven en el dominio (los constructores de cada pieza); aquí
//! sólo se arman las piezas y se guarda el perfil completo.

use crate::errores::ErrorAplicacion;
use crate::puertos::RepositorioPerfiles;
use chrono::NaiveDate;
use finanzas_dominio::caja_chica::CajaChica;
use finanzas_dominio::calendario::{CalendarioCortes, Corte};
use finanzas_dominio::dinero::Colones;
use finanzas_dominio::perfil::{Perfil, PerfilSalarial, TarifasExtra};
use finanzas_dominio::proporcion::Proporcion;
use finanzas_dominio::usuario::UsuarioId;

/// Un corte tal como llega de la interfaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CortePedido {
    /// Primer día del corte.
    pub inicio: NaiveDate,
    /// Último día del corte.
    pub fin: NaiveDate,
    /// Último día para reportar extras.
    pub reportar_antes: NaiveDate,
    /// Día de pago.
    pub pago: NaiveDate,
}

/// El perfil completo pedido desde la interfaz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolicitudPerfil {
    /// Dueño: siempre el usuario autenticado (P5).
    pub usuario: UsuarioId,
    /// Salario base bruto de la quincena.
    pub salario_base: i64,
    /// Factor neto en diezmilésimas (86,17 % = 8617).
    pub factor_neto: u32,
    /// Rebajos fijos de la quincena.
    pub rebajos: i64,
    /// Tarifa bruta de una extra nocturna.
    pub nocturna: i64,
    /// Tarifa bruta de una extra diurna.
    pub diurna: i64,
    /// Tarifa bruta de una extra especial.
    pub especial: i64,
    /// Tarifa bruta de un feriado.
    pub feriado: i64,
    /// Tope de la caja chica por quincena.
    pub tope_caja_chica: i64,
    /// El calendario completo de cortes.
    pub cortes: Vec<CortePedido>,
}

/// El perfil con lo que el dominio calcula de él.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumenPerfil {
    /// El perfil.
    pub perfil: Perfil,
    /// Salario neto de la quincena (P2).
    pub salario_neto: Colones,
    /// Último día de pago que cubre el calendario.
    pub cubre_hasta: Option<NaiveDate>,
}

impl ResumenPerfil {
    fn de(perfil: Perfil) -> Self {
        Self {
            salario_neto: perfil.salarial().salario_neto_quincena(),
            cubre_hasta: perfil.calendario().cortes().last().map(Corte::pago),
            perfil,
        }
    }
}

/// Caso de uso: ver el perfil propio.
#[derive(Debug)]
pub struct ConsultarPerfil<'a, P> {
    perfiles: &'a P,
}

impl<'a, P: RepositorioPerfiles> ConsultarPerfil<'a, P> {
    /// Arma el caso de uso.
    pub const fn new(perfiles: &'a P) -> Self {
        Self { perfiles }
    }

    /// El perfil del usuario, o `PerfilNoEncontrado`.
    pub async fn ejecutar(&self, usuario: UsuarioId) -> Result<ResumenPerfil, ErrorAplicacion> {
        self.perfiles
            .perfil(usuario)
            .await?
            .map(ResumenPerfil::de)
            .ok_or(ErrorAplicacion::PerfilNoEncontrado)
    }
}

/// Caso de uso: guardar el perfil propio completo (P6).
#[derive(Debug)]
pub struct GuardarPerfil<'a, P> {
    perfiles: &'a P,
}

impl<'a, P: RepositorioPerfiles> GuardarPerfil<'a, P> {
    /// Arma el caso de uso.
    pub const fn new(perfiles: &'a P) -> Self {
        Self { perfiles }
    }

    /// Valida cada pieza con el dominio y reemplaza el perfil entero.
    pub async fn ejecutar(&self, s: SolicitudPerfil) -> Result<ResumenPerfil, ErrorAplicacion> {
        let salarial = PerfilSalarial::new(
            Colones::new(s.salario_base),
            Proporcion::desde_diezmilesimas(s.factor_neto),
            Colones::new(s.rebajos),
        )?;
        let tarifas = TarifasExtra::new(
            Colones::new(s.nocturna),
            Colones::new(s.diurna),
            Colones::new(s.especial),
            Colones::new(s.feriado),
        )?;
        let caja = CajaChica::new(Colones::new(s.tope_caja_chica))?;
        let cortes = s
            .cortes
            .into_iter()
            .map(|c| Corte::new(c.inicio, c.fin, c.reportar_antes, c.pago))
            .collect::<Result<Vec<_>, _>>()?;
        let calendario = CalendarioCortes::new(cortes)?;
        let perfil = Perfil::new(s.usuario, salarial, tarifas, caja, calendario);
        self.perfiles.guardar(&perfil).await?;
        Ok(ResumenPerfil::de(perfil))
    }
}
