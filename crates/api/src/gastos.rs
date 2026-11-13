//! Los gastos del día y la caja chica: su JSON y sus rutas.
//!
//! | Método | Ruta | Qué hace |
//! |---|---|---|
//! | `GET` | `/caja-chica` | Caja chica del periodo |
//! | `GET` | `/gastos/{desde}/{hasta}` | Los gastos entre dos fechas |
//! | `POST` | `/gastos` | Anota un gasto |
//! | `DELETE` | `/gastos/{id}` | Deshace un gasto |

use crate::errores::{ErrorApi, Resultado};
use crate::{Contexto, Metodo, Peticion, cuerpo, fecha, id, json, sin_contenido};
use chrono::NaiveDate;
use finanzas_aplicacion::casos_de_uso::gastos::{
    ConsultarCajaChica, DeshacerGasto, GastoRegistrado, ListarGastos, RegistrarGasto, ResumenCaja,
    SolicitudGasto,
};
use finanzas_dominio::base::usuario::UsuarioId;
use finanzas_dominio::gastos::caja_chica::NivelCaja;
use finanzas_dominio::gastos::{Categoria, Gasto, GastoId, Origen};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Categoría de un gasto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CategoriaDto {
    Comida,
    Casa,
    Moto,
    Salud,
    Familia,
    Ropa,
    Tramites,
    Gusto,
    Otro,
}

impl From<CategoriaDto> for Categoria {
    fn from(c: CategoriaDto) -> Self {
        match c {
            CategoriaDto::Comida => Self::Comida,
            CategoriaDto::Casa => Self::Casa,
            CategoriaDto::Moto => Self::Moto,
            CategoriaDto::Salud => Self::Salud,
            CategoriaDto::Familia => Self::Familia,
            CategoriaDto::Ropa => Self::Ropa,
            CategoriaDto::Tramites => Self::Tramites,
            CategoriaDto::Gusto => Self::Gusto,
            CategoriaDto::Otro => Self::Otro,
        }
    }
}

impl From<Categoria> for CategoriaDto {
    fn from(c: Categoria) -> Self {
        match c {
            Categoria::Comida => Self::Comida,
            Categoria::Casa => Self::Casa,
            Categoria::Moto => Self::Moto,
            Categoria::Salud => Self::Salud,
            Categoria::Familia => Self::Familia,
            Categoria::Ropa => Self::Ropa,
            Categoria::Tramites => Self::Tramites,
            Categoria::Gusto => Self::Gusto,
            Categoria::Otro => Self::Otro,
        }
    }
}

/// De dónde salió la plata de un gasto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrigenDto {
    CuentaGastos,
    Planilla,
    SobraDelMes,
    Gusto,
    Fondo,
}

impl From<OrigenDto> for Origen {
    fn from(o: OrigenDto) -> Self {
        match o {
            OrigenDto::CuentaGastos => Self::CuentaGastos,
            OrigenDto::Planilla => Self::Planilla,
            OrigenDto::SobraDelMes => Self::SobraDelMes,
            OrigenDto::Gusto => Self::Gusto,
            OrigenDto::Fondo => Self::Fondo,
        }
    }
}

impl From<Origen> for OrigenDto {
    fn from(o: Origen) -> Self {
        match o {
            Origen::CuentaGastos => Self::CuentaGastos,
            Origen::Planilla => Self::Planilla,
            Origen::SobraDelMes => Self::SobraDelMes,
            Origen::Gusto => Self::Gusto,
            Origen::Fondo => Self::Fondo,
        }
    }
}

/// Cómo está la caja chica.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NivelCajaDto {
    Holgada,
    Baja,
    Agotada,
}

impl From<NivelCaja> for NivelCajaDto {
    fn from(n: NivelCaja) -> Self {
        match n {
            NivelCaja::Holgada => Self::Holgada,
            NivelCaja::Baja => Self::Baja,
            NivelCaja::Agotada => Self::Agotada,
        }
    }
}

/// `POST /gastos`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevoGasto {
    pub fecha: NaiveDate,
    pub concepto: String,
    pub monto: i64,
    pub categoria: CategoriaDto,
    pub origen: OrigenDto,
}

/// La caja chica del periodo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CajaDto {
    pub desde: NaiveDate,
    pub hasta: NaiveDate,
    pub queda: i64,
    pub exceso: i64,
    pub nivel: NivelCajaDto,
}

impl From<ResumenCaja> for CajaDto {
    fn from(r: ResumenCaja) -> Self {
        Self {
            desde: r.desde,
            hasta: r.hasta,
            queda: r.estado.queda.valor(),
            exceso: r.estado.exceso.valor(),
            nivel: r.estado.nivel.into(),
        }
    }
}

/// Un gasto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GastoDto {
    pub id: Uuid,
    pub fecha: NaiveDate,
    pub concepto: String,
    pub monto: i64,
    pub categoria: CategoriaDto,
    pub origen: OrigenDto,
}

impl From<&Gasto> for GastoDto {
    fn from(g: &Gasto) -> Self {
        Self {
            id: g.id().uuid(),
            fecha: g.fecha(),
            concepto: g.concepto().como_str().to_owned(),
            monto: g.monto().valor(),
            categoria: g.categoria().into(),
            origen: g.origen().into(),
        }
    }
}

/// Respuesta de `POST /gastos`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GastoRegistradoDto {
    pub gasto: GastoDto,
    pub caja: CajaDto,
}

impl From<GastoRegistrado> for GastoRegistradoDto {
    fn from(r: GastoRegistrado) -> Self {
        Self {
            gasto: (&r.gasto).into(),
            caja: r.caja.into(),
        }
    }
}

async fn registrar_gasto<C: Contexto>(ctx: &C, usuario: UsuarioId, bytes: &[u8]) -> Resultado {
    let g: NuevoGasto = cuerpo(bytes)?;
    let r = RegistrarGasto::new(ctx.perfiles(), ctx.gastos(), ctx.ids(), ctx.reloj())
        .ejecutar(SolicitudGasto {
            usuario,
            fecha: g.fecha,
            concepto: g.concepto,
            monto: g.monto,
            categoria: g.categoria.into(),
            origen: g.origen.into(),
        })
        .await?;
    json(201, &GastoRegistradoDto::from(r))
}

/// Las rutas de gastos y caja chica. `None` si la ruta no es de aquí.
pub(crate) async fn enrutar<C: Contexto>(
    ctx: &C,
    usuario: UsuarioId,
    p: &Peticion<'_>,
    partes: &[&str],
) -> Option<Resultado> {
    Some(match (p.metodo, partes) {
        (Metodo::Get, ["caja-chica"]) => {
            ConsultarCajaChica::new(ctx.perfiles(), ctx.gastos(), ctx.reloj())
                .ejecutar(usuario)
                .await
                .map_err(ErrorApi::from)
                .and_then(|r| json(200, &CajaDto::from(r)))
        }
        (Metodo::Post, ["gastos"]) => registrar_gasto(ctx, usuario, p.cuerpo).await,
        (Metodo::Get, ["gastos", desde, hasta]) => match (fecha(desde), fecha(hasta)) {
            (Ok(desde), Ok(hasta)) => ListarGastos::new(ctx.gastos())
                .ejecutar(usuario, desde, hasta)
                .await
                .map_err(ErrorApi::from)
                .and_then(|lista| json(200, &lista.iter().map(GastoDto::from).collect::<Vec<_>>())),
            _ => Err(ErrorApi::RutaInexistente),
        },
        (Metodo::Delete, ["gastos", texto]) => match id(texto) {
            Ok(uuid) => sin_contenido(
                DeshacerGasto::new(ctx.gastos())
                    .ejecutar(usuario, GastoId::new(uuid))
                    .await,
            ),
            Err(e) => Err(e),
        },
        _ => return None,
    })
}
