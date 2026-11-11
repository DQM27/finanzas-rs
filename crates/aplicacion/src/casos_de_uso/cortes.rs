//! Dónde está hoy el usuario en su calendario de cortes.
//!
//! No decide nada: el calendario del dominio dice en qué corte cae hoy (C1) y
//! cuál es el próximo pago; aquí sólo se busca el perfil y se pregunta.

use crate::errores::ErrorAplicacion;
use crate::puertos::{Reloj, RepositorioPerfiles};
use finanzas_dominio::calendario::Corte;
use finanzas_dominio::usuario::UsuarioId;

/// El calendario visto desde hoy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CortesHoy {
    /// El corte en que cae hoy: lo trabajado hoy se reporta y se paga según él.
    /// `None` si hoy queda fuera del calendario.
    pub en_curso: Option<Corte>,
    /// El próximo pago, hoy incluido. `None` si el calendario ya no cubre más.
    pub proximo_pago: Option<Corte>,
    /// Días hasta ese pago (0 si es hoy).
    pub dias_para_pago: Option<i64>,
}

/// Caso de uso: el calendario de cortes visto desde hoy.
#[derive(Debug)]
pub struct ConsultarCortesHoy<'a, P, R> {
    perfiles: &'a P,
    reloj: &'a R,
}

impl<'a, P, R> ConsultarCortesHoy<'a, P, R>
where
    P: RepositorioPerfiles,
    R: Reloj,
{
    /// Arma el caso de uso.
    pub const fn new(perfiles: &'a P, reloj: &'a R) -> Self {
        Self { perfiles, reloj }
    }

    /// El corte en curso y el próximo pago del usuario.
    pub async fn ejecutar(&self, usuario: UsuarioId) -> Result<CortesHoy, ErrorAplicacion> {
        let perfil = self
            .perfiles
            .perfil(usuario)
            .await?
            .ok_or(ErrorAplicacion::PerfilNoEncontrado)?;
        let calendario = perfil.calendario();
        let hoy = self.reloj.hoy();
        Ok(CortesHoy {
            en_curso: calendario.corte_de(hoy).copied(),
            proximo_pago: calendario.proximo_pago(hoy).copied(),
            dias_para_pago: calendario.dias_para_pago(hoy),
        })
    }
}
