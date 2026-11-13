//! El presupuesto de cada pago y la suma del mes: su JSON y sus rutas.
//!
//! | Método | Ruta | Qué hace |
//! |---|---|---|
//! | `GET` | `/quincenas/actual` | La quincena que se está gastando |
//! | `GET` | `/quincenas/{pago}` | Una quincena |
//! | `PUT` | `/quincenas/{pago}` | Guarda su presupuesto |
//! | `PUT` | `/quincenas/{pago}/marcas` | Marca o desmarca una línea |
//! | `GET` | `/meses/actual` | Las quincenas del mes de hoy |
//! | `GET` | `/meses/{AAAA-MM}` | Las quincenas de un mes |

use crate::errores::{ErrorApi, Resultado};
use crate::{Contexto, Metodo, Peticion, cuerpo, fecha, json};
use chrono::{Datelike, NaiveDate};
use finanzas_aplicacion::casos_de_uso::quincena::{
    LineaPedida, Presupuestos, ResumenMes, ResumenQuincena, SolicitudMarca, SolicitudPresupuesto,
};
use finanzas_dominio::base::usuario::UsuarioId;
use finanzas_dominio::quincena::{Balance, Lado, Linea};
use serde::{Deserialize, Serialize};

/// Lado de una quincena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LadoDto {
    Ingreso,
    Pago,
}

impl From<LadoDto> for Lado {
    fn from(l: LadoDto) -> Self {
        match l {
            LadoDto::Ingreso => Self::Ingreso,
            LadoDto::Pago => Self::Pago,
        }
    }
}

/// Una línea de presupuesto pedida.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineaNueva {
    pub concepto: String,
    pub monto: i64,
}

impl From<LineaNueva> for LineaPedida {
    fn from(l: LineaNueva) -> Self {
        Self {
            concepto: l.concepto,
            monto: l.monto,
        }
    }
}

/// `PUT /quincenas/{pago}`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Presupuesto {
    pub ingresos: Vec<LineaNueva>,
    pub pagos: Vec<LineaNueva>,
}

/// `PUT /quincenas/{pago}/marcas`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Marca {
    pub lado: LadoDto,
    pub indice: usize,
    pub hecho: bool,
}

/// Una línea de la quincena.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LineaDto {
    pub concepto: String,
    pub monto: i64,
    pub hecho: bool,
}

impl From<&Linea> for LineaDto {
    fn from(l: &Linea) -> Self {
        Self {
            concepto: l.concepto().como_str().to_owned(),
            monto: l.monto().valor(),
            hecho: l.hecho(),
        }
    }
}

/// Pagos hechos de cuántos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AvanceDto {
    pub hechos: usize,
    pub total: usize,
}

/// Lo que entra solo en una quincena (Q7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutomaticosDto {
    /// Salario neto.
    pub salario: i64,
    /// Neto de las extras que se pagan ese día (E4).
    pub extras: i64,
    /// `true` si el pago ya llegó.
    pub llegaron: bool,
}

/// Los totales de una quincena o de un mes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BalanceDto {
    pub salario: i64,
    pub extras: i64,
    pub otros_ingresos: i64,
    pub total_ingresos: i64,
    pub total_pagos: i64,
    pub sobra: i64,
    pub falta_por_pagar: i64,
}

impl From<Balance> for BalanceDto {
    fn from(b: Balance) -> Self {
        Self {
            salario: b.salario().valor(),
            extras: b.extras().valor(),
            otros_ingresos: b.otros_ingresos().valor(),
            total_ingresos: b.total_ingresos().valor(),
            total_pagos: b.total_pagos().valor(),
            sobra: b.sobra().valor(),
            falta_por_pagar: b.falta_por_pagar().valor(),
        }
    }
}

/// Una quincena con sus números.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuincenaDto {
    pub pago: NaiveDate,
    /// `false` si todavía no se le armó presupuesto.
    pub presupuestada: bool,
    pub automaticos: AutomaticosDto,
    /// Los otros ingresos, escritos a mano.
    pub ingresos: Vec<LineaDto>,
    pub pagos: Vec<LineaDto>,
    /// Suma de los otros ingresos.
    pub otros_ingresos: i64,
    pub total_ingresos: i64,
    pub total_pagos: i64,
    pub sobra: i64,
    pub falta_por_pagar: i64,
    pub avance: AvanceDto,
}

impl From<ResumenQuincena> for QuincenaDto {
    fn from(r: ResumenQuincena) -> Self {
        Self {
            pago: r.quincena.pago(),
            presupuestada: r.presupuestada,
            automaticos: AutomaticosDto {
                salario: r.automaticos.salario().valor(),
                extras: r.automaticos.extras().valor(),
                llegaron: r.automaticos.llegaron(),
            },
            ingresos: r.quincena.ingresos().iter().map(LineaDto::from).collect(),
            pagos: r.quincena.pagos().iter().map(LineaDto::from).collect(),
            otros_ingresos: r.balance.otros_ingresos().valor(),
            total_ingresos: r.balance.total_ingresos().valor(),
            total_pagos: r.balance.total_pagos().valor(),
            sobra: r.balance.sobra().valor(),
            falta_por_pagar: r.balance.falta_por_pagar().valor(),
            avance: AvanceDto {
                hechos: r.avance.0,
                total: r.avance.1,
            },
        }
    }
}

/// Un mes: sus quincenas y la suma.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MesDto {
    /// `AAAA-MM`.
    pub mes: String,
    pub quincenas: Vec<QuincenaDto>,
    pub balance: BalanceDto,
}

impl From<ResumenMes> for MesDto {
    fn from(r: ResumenMes) -> Self {
        Self {
            mes: format!("{:04}-{:02}", r.anio, r.mes),
            quincenas: r.quincenas.into_iter().map(QuincenaDto::from).collect(),
            balance: r.balance.into(),
        }
    }
}

/// Las quincenas. `None` si la ruta no es de aquí.
pub(crate) async fn enrutar<C: Contexto>(
    ctx: &C,
    usuario: UsuarioId,
    p: &Peticion<'_>,
    partes: &[&str],
) -> Option<Resultado> {
    Some(match (p.metodo, partes) {
        (Metodo::Get, ["quincenas", cual]) => consultar_quincena(ctx, usuario, cual).await,
        (Metodo::Put, ["quincenas", texto]) => {
            guardar_presupuesto(ctx, usuario, texto, p.cuerpo).await
        }
        (Metodo::Put, ["quincenas", texto, "marcas"]) => {
            marcar(ctx, usuario, texto, p.cuerpo).await
        }
        (Metodo::Get, ["meses", cual]) => consultar_mes(ctx, usuario, cual).await,
        _ => return None,
    })
}

async fn consultar_quincena<C: Contexto>(ctx: &C, usuario: UsuarioId, cual: &str) -> Resultado {
    let pago = if cual == "actual" {
        None
    } else {
        Some(fecha(cual)?)
    };
    let r = presupuestos(ctx).consultar(usuario, pago).await?;
    json(200, &QuincenaDto::from(r))
}

fn presupuestos<C: Contexto>(
    ctx: &C,
) -> Presupuestos<'_, C::Perfiles, C::Quincenas, C::Extras, C::Reloj> {
    Presupuestos::new(ctx.perfiles(), ctx.quincenas(), ctx.extras(), ctx.reloj())
}

/// `actual` o `AAAA-MM`.
async fn consultar_mes<C: Contexto>(ctx: &C, usuario: UsuarioId, cual: &str) -> Resultado {
    let mes = if cual == "actual" {
        None
    } else {
        if cual.len() != 7 {
            return Err(ErrorApi::RutaInexistente);
        }
        let primero = fecha(&format!("{cual}-01"))?;
        Some((primero.year(), primero.month()))
    };
    let r = presupuestos(ctx).mes(usuario, mes).await?;
    json(200, &MesDto::from(r))
}

async fn guardar_presupuesto<C: Contexto>(
    ctx: &C,
    usuario: UsuarioId,
    texto: &str,
    bytes: &[u8],
) -> Resultado {
    let pago = fecha(texto)?;
    let q: Presupuesto = cuerpo(bytes)?;
    let r = presupuestos(ctx)
        .guardar(SolicitudPresupuesto {
            usuario,
            pago,
            ingresos: q.ingresos.into_iter().map(Into::into).collect(),
            pagos: q.pagos.into_iter().map(Into::into).collect(),
        })
        .await?;
    json(200, &QuincenaDto::from(r))
}

async fn marcar<C: Contexto>(ctx: &C, usuario: UsuarioId, texto: &str, bytes: &[u8]) -> Resultado {
    let pago = fecha(texto)?;
    let m: Marca = cuerpo(bytes)?;
    let r = presupuestos(ctx)
        .marcar(SolicitudMarca {
            usuario,
            pago,
            lado: m.lado.into(),
            indice: m.indice,
            hecho: m.hecho,
        })
        .await?;
    json(200, &QuincenaDto::from(r))
}
