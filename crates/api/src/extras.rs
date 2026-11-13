//! Las extras trabajadas: su JSON y sus rutas.
//!
//! | Método | Ruta | Qué hace |
//! |---|---|---|
//! | `GET` | `/extras` | Todas las extras |
//! | `POST` | `/extras` | Anota una extra |
//! | `DELETE` | `/extras/{id}` | Deshace una extra |

use crate::errores::{ErrorApi, Resultado};
use crate::{Contexto, Metodo, Peticion, cuerpo, id, json, sin_contenido};
use chrono::NaiveDate;
use finanzas_aplicacion::casos_de_uso::extras::{
    DeshacerExtra, ExtraRegistrada, ListarExtras, RegistrarExtra, SolicitudExtra,
};
use finanzas_dominio::base::usuario::UsuarioId;
use finanzas_dominio::extras::{ExtraId, TipoExtra};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tipo de extra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoExtraDto {
    Nocturna,
    Diurna,
    Especial,
    Feriado,
    Otra,
}

impl From<TipoExtraDto> for TipoExtra {
    fn from(t: TipoExtraDto) -> Self {
        match t {
            TipoExtraDto::Nocturna => Self::Nocturna,
            TipoExtraDto::Diurna => Self::Diurna,
            TipoExtraDto::Especial => Self::Especial,
            TipoExtraDto::Feriado => Self::Feriado,
            TipoExtraDto::Otra => Self::Otra,
        }
    }
}

impl From<TipoExtra> for TipoExtraDto {
    fn from(t: TipoExtra) -> Self {
        match t {
            TipoExtra::Nocturna => Self::Nocturna,
            TipoExtra::Diurna => Self::Diurna,
            TipoExtra::Especial => Self::Especial,
            TipoExtra::Feriado => Self::Feriado,
            TipoExtra::Otra => Self::Otra,
        }
    }
}

/// `POST /extras`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevaExtra {
    pub fecha: NaiveDate,
    pub tipo: TipoExtraDto,
    /// Sólo para el tipo `OTRA`.
    #[serde(default)]
    pub monto: Option<i64>,
}

/// Respuesta de `POST /extras`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtraDto {
    pub id: Uuid,
    pub fecha: NaiveDate,
    pub tipo: TipoExtraDto,
    pub bruto: i64,
    pub neto: i64,
    /// `null` si el calendario todavía no cubre ese día.
    pub fecha_pago: Option<NaiveDate>,
}

impl From<ExtraRegistrada> for ExtraDto {
    fn from(r: ExtraRegistrada) -> Self {
        Self {
            id: r.extra.id().uuid(),
            fecha: r.extra.fecha(),
            tipo: r.extra.tipo().into(),
            bruto: r.extra.bruto().valor(),
            neto: r.neto.valor(),
            fecha_pago: r.fecha_pago,
        }
    }
}

async fn registrar_extra<C: Contexto>(ctx: &C, usuario: UsuarioId, bytes: &[u8]) -> Resultado {
    let e: NuevaExtra = cuerpo(bytes)?;
    let r = RegistrarExtra::new(ctx.perfiles(), ctx.extras(), ctx.ids())
        .ejecutar(SolicitudExtra {
            usuario,
            fecha: e.fecha,
            tipo: e.tipo.into(),
            monto: e.monto,
        })
        .await?;
    json(201, &ExtraDto::from(r))
}

/// Las rutas de extras. `None` si la ruta no es de aquí.
pub(crate) async fn enrutar<C: Contexto>(
    ctx: &C,
    usuario: UsuarioId,
    p: &Peticion<'_>,
    partes: &[&str],
) -> Option<Resultado> {
    Some(match (p.metodo, partes) {
        (Metodo::Get, ["extras"]) => ListarExtras::new(ctx.perfiles(), ctx.extras())
            .ejecutar(usuario)
            .await
            .map_err(ErrorApi::from)
            .and_then(|lista| {
                json(
                    200,
                    &lista.into_iter().map(ExtraDto::from).collect::<Vec<_>>(),
                )
            }),
        (Metodo::Post, ["extras"]) => registrar_extra(ctx, usuario, p.cuerpo).await,
        (Metodo::Delete, ["extras", texto]) => match id(texto) {
            Ok(uuid) => sin_contenido(
                DeshacerExtra::new(ctx.extras())
                    .ejecutar(usuario, ExtraId::new(uuid))
                    .await,
            ),
            Err(e) => Err(e),
        },
        _ => return None,
    })
}
