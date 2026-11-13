//! El fondo de emergencia: su JSON y sus rutas.
//!
//! | Método | Ruta | Qué hace |
//! |---|---|---|
//! | `GET` | `/fondo` | Estado del fondo |
//! | `GET` | `/fondo/movimientos` | Los movimientos del fondo |
//! | `POST` | `/fondo/retiros` | Saca del fondo |
//! | `POST` | `/fondo/depositos` | Aporta o repone |
//! | `DELETE` | `/fondo/movimientos/{id}` | Deshace el último movimiento |

use crate::errores::{ErrorApi, Resultado};
use crate::{Contexto, Metodo, Peticion, cuerpo, id, json, sin_contenido};
use chrono::NaiveDate;
use finanzas_aplicacion::casos_de_uso::fondo::{
    ConsultarFondo, DepositarEnFondo, Deposito, DeshacerMovimientoFondo, ListarMovimientosFondo,
    RetiroHecho, SacarDelFondo, SolicitudDeposito, SolicitudSacar,
};
use finanzas_dominio::base::usuario::UsuarioId;
use finanzas_dominio::fondo::movimientos::{
    EstadoFondo, Movimiento, MovimientoFondo, MovimientoFondoId,
};
use finanzas_dominio::fondo::{Evaluacion, NivelEmergencia, PlanReposicion};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Nivel de una emergencia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NivelDto {
    Vital,
    Trabajo,
    Descuido,
    Falta,
}

impl From<NivelDto> for NivelEmergencia {
    fn from(n: NivelDto) -> Self {
        match n {
            NivelDto::Vital => Self::Vital,
            NivelDto::Trabajo => Self::Trabajo,
            NivelDto::Descuido => Self::Descuido,
            NivelDto::Falta => Self::Falta,
        }
    }
}

impl From<NivelEmergencia> for NivelDto {
    fn from(n: NivelEmergencia) -> Self {
        match n {
            NivelEmergencia::Vital => Self::Vital,
            NivelEmergencia::Trabajo => Self::Trabajo,
            NivelEmergencia::Descuido => Self::Descuido,
            NivelEmergencia::Falta => Self::Falta,
        }
    }
}

/// Aporte o reposición.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepositoDto {
    Aporte,
    Reposicion,
}

impl From<DepositoDto> for Deposito {
    fn from(d: DepositoDto) -> Self {
        match d {
            DepositoDto::Aporte => Self::Aporte,
            DepositoDto::Reposicion => Self::Reposicion,
        }
    }
}

/// `POST /fondo/retiros`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevoRetiro {
    pub concepto: String,
    pub monto: i64,
    pub nivel: NivelDto,
    pub inesperado: bool,
    pub necesario: bool,
    pub urgente: bool,
}

/// `POST /fondo/depositos`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevoDeposito {
    pub tipo: DepositoDto,
    pub concepto: String,
    pub monto: i64,
}

/// Cómo está el fondo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FondoDto {
    pub saldo: i64,
    pub por_reponer: i64,
    pub cuota_mensual: i64,
}

impl From<EstadoFondo> for FondoDto {
    fn from(e: EstadoFondo) -> Self {
        Self {
            saldo: e.saldo.valor(),
            por_reponer: e.por_reponer.valor(),
            cuota_mensual: e.cuota_mensual.valor(),
        }
    }
}

/// El plan para reponer un retiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PlanDto {
    pub nivel: NivelDto,
    pub sacado: i64,
    pub a_reponer: i64,
    pub meses: u8,
    pub cuota_mensual: i64,
}

impl From<PlanReposicion> for PlanDto {
    fn from(p: PlanReposicion) -> Self {
        Self {
            nivel: p.nivel.into(),
            sacado: p.sacado.valor(),
            a_reponer: p.a_reponer.valor(),
            meses: p.meses,
            cuota_mensual: p.cuota_mensual.valor(),
        }
    }
}

/// Respuesta de `POST /fondo/retiros`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RetiroDto {
    pub plan: PlanDto,
    pub fondo: FondoDto,
}

impl From<RetiroHecho> for RetiroDto {
    fn from(r: RetiroHecho) -> Self {
        Self {
            plan: r.plan.into(),
            fondo: r.estado.into(),
        }
    }
}

/// Un movimiento del fondo de emergencia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MovimientoFondoDto {
    pub id: Uuid,
    pub fecha: NaiveDate,
    pub concepto: String,
    pub monto: i64,
    pub tipo: DepositoOMovimientoDto,
    /// Sólo en los retiros.
    pub plan: Option<PlanDto>,
}

/// Clase de movimiento del fondo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepositoOMovimientoDto {
    Aporte,
    Retiro,
    Reposicion,
}

impl From<&MovimientoFondo> for MovimientoFondoDto {
    fn from(m: &MovimientoFondo) -> Self {
        let (tipo, plan) = match m.movimiento() {
            Movimiento::Aporte => (DepositoOMovimientoDto::Aporte, None),
            Movimiento::Reposicion => (DepositoOMovimientoDto::Reposicion, None),
            Movimiento::Retiro(plan) => (DepositoOMovimientoDto::Retiro, Some(plan.into())),
        };
        Self {
            id: m.id().uuid(),
            fecha: m.fecha(),
            concepto: m.concepto().como_str().to_owned(),
            monto: m.monto().valor(),
            tipo,
            plan,
        }
    }
}

/// El fondo de emergencia. `None` si la ruta no es de aquí.
pub(crate) async fn enrutar<C: Contexto>(
    ctx: &C,
    usuario: UsuarioId,
    p: &Peticion<'_>,
    partes: &[&str],
) -> Option<Resultado> {
    Some(match (p.metodo, partes) {
        (Metodo::Get, ["fondo"]) => ConsultarFondo::new(ctx.fondo())
            .ejecutar(usuario)
            .await
            .map_err(ErrorApi::from)
            .and_then(|e| json(200, &FondoDto::from(e))),
        (Metodo::Get, ["fondo", "movimientos"]) => ListarMovimientosFondo::new(ctx.fondo())
            .ejecutar(usuario)
            .await
            .map_err(ErrorApi::from)
            .and_then(|lista| {
                json(
                    200,
                    &lista
                        .iter()
                        .map(MovimientoFondoDto::from)
                        .collect::<Vec<_>>(),
                )
            }),
        (Metodo::Post, ["fondo", "retiros"]) => sacar(ctx, usuario, p.cuerpo).await,
        (Metodo::Post, ["fondo", "depositos"]) => depositar(ctx, usuario, p.cuerpo).await,
        (Metodo::Delete, ["fondo", "movimientos", texto]) => match id(texto) {
            Ok(uuid) => sin_contenido(
                DeshacerMovimientoFondo::new(ctx.fondo())
                    .ejecutar(usuario, MovimientoFondoId::new(uuid))
                    .await,
            ),
            Err(e) => Err(e),
        },
        _ => return None,
    })
}

async fn sacar<C: Contexto>(ctx: &C, usuario: UsuarioId, bytes: &[u8]) -> Resultado {
    let r: NuevoRetiro = cuerpo(bytes)?;
    let hecho = SacarDelFondo::new(ctx.fondo(), ctx.ids(), ctx.reloj())
        .ejecutar(SolicitudSacar {
            usuario,
            concepto: r.concepto,
            monto: r.monto,
            nivel: r.nivel.into(),
            evaluacion: Evaluacion {
                inesperado: r.inesperado,
                necesario: r.necesario,
                urgente: r.urgente,
            },
        })
        .await?;
    json(201, &RetiroDto::from(hecho))
}

async fn depositar<C: Contexto>(ctx: &C, usuario: UsuarioId, bytes: &[u8]) -> Resultado {
    let d: NuevoDeposito = cuerpo(bytes)?;
    let e = DepositarEnFondo::new(ctx.fondo(), ctx.ids(), ctx.reloj())
        .ejecutar(SolicitudDeposito {
            usuario,
            deposito: d.tipo.into(),
            concepto: d.concepto,
            monto: d.monto,
        })
        .await?;
    json(201, &FondoDto::from(e))
}
