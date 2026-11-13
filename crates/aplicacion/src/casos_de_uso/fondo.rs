//! El fondo de emergencia: consultar, sacar, reponer, aportar, listar sus
//! movimientos y deshacer el último.
//!
//! Los casos de uso sólo leen los movimientos, se los dan al dominio y guardan
//! lo que el dominio autoriza. Ninguna regla del fondo vive aquí.

use crate::errores::ErrorAplicacion;
use crate::puertos::{GeneradorIds, Reloj, RepositorioFondo};
use finanzas_dominio::base::dinero::Colones;
use finanzas_dominio::base::texto::Concepto;
use finanzas_dominio::base::usuario::UsuarioId;
use finanzas_dominio::fondo::movimientos::{
    EstadoFondo, MovimientoFondo, MovimientoFondoId, deshacible,
};
use finanzas_dominio::fondo::{Evaluacion, NivelEmergencia, PlanReposicion, decidir_retiro};

/// Lo que entra al fondo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deposito {
    /// Para hacerlo crecer.
    Aporte,
    /// Para pagar lo que se sacó.
    Reposicion,
}

/// Un depósito pedido desde la interfaz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolicitudDeposito {
    /// Dueño.
    pub usuario: UsuarioId,
    /// Aporte o reposición.
    pub deposito: Deposito,
    /// De dónde viene (texto libre; el dominio lo normaliza).
    pub concepto: String,
    /// Cuánto, en colones.
    pub monto: i64,
}

/// Un retiro pedido desde la interfaz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolicitudSacar {
    /// Dueño.
    pub usuario: UsuarioId,
    /// Para qué (texto libre; el dominio lo normaliza).
    pub concepto: String,
    /// Cuánto, en colones.
    pub monto: i64,
    /// Qué tan grave es.
    pub nivel: NivelEmergencia,
    /// Los tres «sí».
    pub evaluacion: Evaluacion,
}

/// El retiro autorizado y cómo queda el fondo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetiroHecho {
    /// Cuánto hay que reponer y en cuántas cuotas.
    pub plan: PlanReposicion,
    /// El fondo después del retiro.
    pub estado: EstadoFondo,
}

/// Caso de uso: ver cómo está el fondo.
#[derive(Debug)]
pub struct ConsultarFondo<'a, F> {
    fondo: &'a F,
}

impl<'a, F: RepositorioFondo> ConsultarFondo<'a, F> {
    /// Arma el caso de uso.
    pub const fn new(fondo: &'a F) -> Self {
        Self { fondo }
    }

    /// El estado calculado a partir de los movimientos.
    pub async fn ejecutar(&self, usuario: UsuarioId) -> Result<EstadoFondo, ErrorAplicacion> {
        let movimientos = self.fondo.de_usuario(usuario).await?;
        Ok(EstadoFondo::de(&movimientos))
    }
}

/// Caso de uso: sacar del fondo por una emergencia.
#[derive(Debug)]
pub struct SacarDelFondo<'a, F, I, R> {
    fondo: &'a F,
    ids: &'a I,
    reloj: &'a R,
}

impl<'a, F, I, R> SacarDelFondo<'a, F, I, R>
where
    F: RepositorioFondo,
    I: GeneradorIds,
    R: Reloj,
{
    /// Arma el caso de uso.
    pub const fn new(fondo: &'a F, ids: &'a I, reloj: &'a R) -> Self {
        Self { fondo, ids, reloj }
    }

    /// El dominio decide si se puede; si sí, se guarda el retiro con su plan.
    pub async fn ejecutar(&self, s: SolicitudSacar) -> Result<RetiroHecho, ErrorAplicacion> {
        let concepto = Concepto::new(&s.concepto)?;
        let mut movimientos = self.fondo.de_usuario(s.usuario).await?;
        let antes = EstadoFondo::de(&movimientos);
        let plan = decidir_retiro(antes.solicitud(Colones::new(s.monto), s.nivel, s.evaluacion))?;
        let retiro = MovimientoFondo::retiro(
            MovimientoFondoId::new(self.ids.nuevo()),
            s.usuario,
            self.reloj.hoy(),
            concepto,
            plan,
        );
        self.fondo.guardar(&retiro).await?;
        movimientos.push(retiro);
        Ok(RetiroHecho {
            plan,
            estado: EstadoFondo::de(&movimientos),
        })
    }
}

/// Caso de uso: meter plata al fondo (aporte o reposición).
#[derive(Debug)]
pub struct DepositarEnFondo<'a, F, I, R> {
    fondo: &'a F,
    ids: &'a I,
    reloj: &'a R,
}

impl<'a, F, I, R> DepositarEnFondo<'a, F, I, R>
where
    F: RepositorioFondo,
    I: GeneradorIds,
    R: Reloj,
{
    /// Arma el caso de uso.
    pub const fn new(fondo: &'a F, ids: &'a I, reloj: &'a R) -> Self {
        Self { fondo, ids, reloj }
    }

    /// Valida con el dominio, guarda y devuelve el fondo actualizado.
    pub async fn ejecutar(&self, s: SolicitudDeposito) -> Result<EstadoFondo, ErrorAplicacion> {
        let concepto = Concepto::new(&s.concepto)?;
        let mut movimientos = self.fondo.de_usuario(s.usuario).await?;
        let id = MovimientoFondoId::new(self.ids.nuevo());
        let monto = Colones::new(s.monto);
        let hoy = self.reloj.hoy();
        let movimiento = match s.deposito {
            Deposito::Aporte => MovimientoFondo::aporte(id, s.usuario, hoy, concepto, monto)?,
            Deposito::Reposicion => MovimientoFondo::reposicion(
                id,
                s.usuario,
                hoy,
                concepto,
                monto,
                &EstadoFondo::de(&movimientos),
            )?,
        };
        self.fondo.guardar(&movimiento).await?;
        movimientos.push(movimiento);
        Ok(EstadoFondo::de(&movimientos))
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

/// Caso de uso: deshacer el último movimiento del fondo.
#[derive(Debug)]
pub struct DeshacerMovimientoFondo<'a, F> {
    fondo: &'a F,
}

impl<'a, F: RepositorioFondo> DeshacerMovimientoFondo<'a, F> {
    /// Arma el caso de uso.
    pub const fn new(fondo: &'a F) -> Self {
        Self { fondo }
    }

    /// Borra el movimiento si el usuario es su dueño (U1) y el dominio lo
    /// permite: sólo el último, porque borrar uno del medio dejaría
    /// reposiciones pagando un retiro que ya no existe (U2).
    pub async fn ejecutar(
        &self,
        usuario: UsuarioId,
        id: MovimientoFondoId,
    ) -> Result<(), ErrorAplicacion> {
        let movimientos = self.fondo.de_usuario(usuario).await?;
        deshacible(&movimientos, id).ok_or(ErrorAplicacion::NoEncontrado)??;
        if self.fondo.borrar(usuario, id).await? {
            Ok(())
        } else {
            Err(ErrorAplicacion::NoEncontrado)
        }
    }
}
