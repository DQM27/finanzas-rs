//! Los números de cada persona: su JSON y sus rutas.
//!
//! | Método | Ruta | Qué hace |
//! |---|---|---|
//! | `GET` | `/perfil` | El perfil propio |
//! | `PUT` | `/perfil` | Reemplaza el perfil propio |

use crate::calendario::CorteDto;
use crate::errores::{ErrorApi, Resultado};
use crate::{Contexto, Metodo, Peticion, cuerpo, json};
use chrono::NaiveDate;
use finanzas_aplicacion::casos_de_uso::perfil::{
    ConsultarPerfil, CortePedido, GuardarPerfil, ResumenPerfil, SolicitudPerfil,
};
use finanzas_dominio::base::dinero::Colones;
use finanzas_dominio::base::usuario::UsuarioId;
use finanzas_dominio::extras::TipoExtra;
use serde::{Deserialize, Serialize};

/// Tarifas brutas de cada tipo de extra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TarifasDto {
    pub nocturna: i64,
    pub diurna: i64,
    pub especial: i64,
    pub feriado: i64,
}

impl From<CorteDto> for CortePedido {
    fn from(c: CorteDto) -> Self {
        Self {
            inicio: c.inicio,
            fin: c.fin,
            reportar_antes: c.reportar_antes,
            pago: c.pago,
        }
    }
}

/// `PUT /perfil`: el perfil completo. Reemplaza al anterior.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevoPerfil {
    pub salario_base: i64,
    /// Diezmilésimas: 86,17 % = 8617.
    pub factor_neto: u32,
    pub rebajos: i64,
    pub tarifas: TarifasDto,
    pub tope_caja_chica: i64,
    pub cortes: Vec<CorteDto>,
}

/// El perfil con lo que se calcula de él.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PerfilDto {
    pub salario_base: i64,
    pub factor_neto: u32,
    pub rebajos: i64,
    pub salario_neto: i64,
    pub tarifas: TarifasDto,
    pub tope_caja_chica: i64,
    pub cortes: Vec<CorteDto>,
    /// Último día de pago que cubre el calendario.
    pub cubre_hasta: Option<NaiveDate>,
}

impl From<ResumenPerfil> for PerfilDto {
    fn from(r: ResumenPerfil) -> Self {
        let p = &r.perfil;
        let tarifa = |tipo| p.tarifas().tarifa(tipo).map_or(0, Colones::valor);
        Self {
            salario_base: p.salarial().salario_base_bruto().valor(),
            factor_neto: p.salarial().factor_neto().diezmilesimas(),
            rebajos: p.salarial().rebajos_fijos().valor(),
            salario_neto: r.salario_neto.valor(),
            tarifas: TarifasDto {
                nocturna: tarifa(TipoExtra::Nocturna),
                diurna: tarifa(TipoExtra::Diurna),
                especial: tarifa(TipoExtra::Especial),
                feriado: tarifa(TipoExtra::Feriado),
            },
            tope_caja_chica: p.caja_chica().tope().valor(),
            cortes: p.calendario().cortes().iter().map(CorteDto::from).collect(),
            cubre_hasta: r.cubre_hasta,
        }
    }
}

/// El perfil propio. `None` si el método no es de aquí.
pub(crate) async fn enrutar<C: Contexto>(
    ctx: &C,
    usuario: UsuarioId,
    p: &Peticion<'_>,
) -> Option<Resultado> {
    Some(match p.metodo {
        Metodo::Get => ConsultarPerfil::new(ctx.perfiles())
            .ejecutar(usuario)
            .await
            .map_err(ErrorApi::from)
            .and_then(|r| json(200, &PerfilDto::from(r))),
        Metodo::Put => guardar_perfil(ctx, usuario, p.cuerpo).await,
        _ => return None,
    })
}

async fn guardar_perfil<C: Contexto>(ctx: &C, usuario: UsuarioId, bytes: &[u8]) -> Resultado {
    let n: NuevoPerfil = cuerpo(bytes)?;
    let r = GuardarPerfil::new(ctx.perfiles())
        .ejecutar(SolicitudPerfil {
            usuario,
            salario_base: n.salario_base,
            factor_neto: n.factor_neto,
            rebajos: n.rebajos,
            nocturna: n.tarifas.nocturna,
            diurna: n.tarifas.diurna,
            especial: n.tarifas.especial,
            feriado: n.tarifas.feriado,
            tope_caja_chica: n.tope_caja_chica,
            cortes: n.cortes.into_iter().map(Into::into).collect(),
        })
        .await?;
    json(200, &PerfilDto::from(r))
}
