//! El calendario de cortes visto desde hoy: su JSON y su ruta.
//!
//! | Método | Ruta | Qué hace |
//! |---|---|---|
//! | `GET` | `/cortes/hoy` | El corte en curso y el próximo pago |

use crate::errores::{ErrorApi, Resultado};
use crate::{Contexto, Metodo, Peticion, json};
use chrono::NaiveDate;
use finanzas_aplicacion::casos_de_uso::calendario::{ConsultarCortesHoy, CortesHoy};
use finanzas_dominio::base::usuario::UsuarioId;
use finanzas_dominio::calendario::Corte;
use serde::{Deserialize, Serialize};

/// Un corte del calendario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorteDto {
    pub inicio: NaiveDate,
    pub fin: NaiveDate,
    pub reportar_antes: NaiveDate,
    pub pago: NaiveDate,
}

impl From<&Corte> for CorteDto {
    fn from(c: &Corte) -> Self {
        Self {
            inicio: c.inicio(),
            fin: c.fin(),
            reportar_antes: c.reportar_antes(),
            pago: c.pago(),
        }
    }
}

/// `GET /cortes/hoy`: el calendario visto desde hoy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CortesHoyDto {
    /// El corte en que cae hoy: lo trabajado hoy se reporta y se paga según él.
    pub en_curso: Option<CorteDto>,
    /// El último pago recibido, hoy incluido: la quincena que se está gastando.
    pub ultimo_pago: Option<CorteDto>,
    /// El próximo pago, hoy incluido.
    pub proximo_pago: Option<CorteDto>,
    /// Días hasta ese pago (0 si es hoy).
    pub dias_para_pago: Option<i64>,
}

impl From<CortesHoy> for CortesHoyDto {
    fn from(c: CortesHoy) -> Self {
        Self {
            en_curso: c.en_curso.as_ref().map(CorteDto::from),
            ultimo_pago: c.ultimo_pago.as_ref().map(CorteDto::from),
            proximo_pago: c.proximo_pago.as_ref().map(CorteDto::from),
            dias_para_pago: c.dias_para_pago,
        }
    }
}

/// La ruta del calendario. `None` si el método no es de aquí.
pub(crate) async fn enrutar<C: Contexto>(
    ctx: &C,
    usuario: UsuarioId,
    p: &Peticion<'_>,
) -> Option<Resultado> {
    Some(match p.metodo {
        Metodo::Get => ConsultarCortesHoy::new(ctx.perfiles(), ctx.reloj())
            .ejecutar(usuario)
            .await
            .map_err(ErrorApi::from)
            .and_then(|c| json(200, &CortesHoyDto::from(c))),
        _ => return None,
    })
}
