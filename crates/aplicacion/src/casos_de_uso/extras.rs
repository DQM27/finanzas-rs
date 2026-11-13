//! Las extras trabajadas: anotarlas, listarlas con su neto y su fecha de
//! pago, y deshacer la que se anotó mal.

use crate::errores::ErrorAplicacion;
use crate::puertos::{GeneradorIds, RepositorioExtras, RepositorioPerfiles};
use chrono::NaiveDate;
use finanzas_dominio::base::dinero::Colones;
use finanzas_dominio::base::usuario::UsuarioId;
use finanzas_dominio::extras::{Extra, ExtraId, TipoExtra};

/// Lo que llega desde la interfaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolicitudExtra {
    /// Dueño.
    pub usuario: UsuarioId,
    /// Día trabajado.
    pub fecha: NaiveDate,
    /// Tipo de extra.
    pub tipo: TipoExtra,
    /// Monto bruto escrito a mano (obligatorio en `Otra`).
    pub monto: Option<i64>,
}

/// La extra guardada con lo que el dominio calculó.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraRegistrada {
    /// La extra tal como quedó.
    pub extra: Extra,
    /// Lo que deja neto.
    pub neto: Colones,
    /// Cuándo se paga (`None` si el calendario aún no cubre esa fecha).
    pub fecha_pago: Option<NaiveDate>,
}

/// Caso de uso: registrar una extra.
#[derive(Debug)]
pub struct RegistrarExtra<'a, P, E, I> {
    perfiles: &'a P,
    extras: &'a E,
    ids: &'a I,
}

impl<'a, P, E, I> RegistrarExtra<'a, P, E, I>
where
    P: RepositorioPerfiles,
    E: RepositorioExtras,
    I: GeneradorIds,
{
    /// Arma el caso de uso con sus puertos.
    pub const fn new(perfiles: &'a P, extras: &'a E, ids: &'a I) -> Self {
        Self {
            perfiles,
            extras,
            ids,
        }
    }

    /// Valida con las tarifas del usuario, guarda y calcula neto y fecha de pago.
    pub async fn ejecutar(&self, s: SolicitudExtra) -> Result<ExtraRegistrada, ErrorAplicacion> {
        let perfil = self
            .perfiles
            .perfil(s.usuario)
            .await?
            .ok_or(ErrorAplicacion::PerfilNoEncontrado)?;
        let extra = Extra::new(
            ExtraId::new(self.ids.nuevo()),
            s.usuario,
            s.fecha,
            s.tipo,
            perfil.tarifas(),
            s.monto.map(Colones::new),
        )?;
        self.extras.guardar(&extra).await?;
        Ok(ExtraRegistrada {
            neto: extra.neto(perfil.salarial()),
            fecha_pago: extra.fecha_pago(perfil.calendario()),
            extra,
        })
    }
}

/// Caso de uso: todas las extras, con su neto y su fecha de pago.
#[derive(Debug)]
pub struct ListarExtras<'a, P, E> {
    perfiles: &'a P,
    extras: &'a E,
}

impl<'a, P, E> ListarExtras<'a, P, E>
where
    P: RepositorioPerfiles,
    E: RepositorioExtras,
{
    /// Arma el caso de uso.
    pub const fn new(perfiles: &'a P, extras: &'a E) -> Self {
        Self { perfiles, extras }
    }

    /// Las extras del usuario con lo que el dominio calcula de cada una.
    pub async fn ejecutar(
        &self,
        usuario: UsuarioId,
    ) -> Result<Vec<ExtraRegistrada>, ErrorAplicacion> {
        let perfil = self
            .perfiles
            .perfil(usuario)
            .await?
            .ok_or(ErrorAplicacion::PerfilNoEncontrado)?;
        Ok(self
            .extras
            .de_usuario(usuario)
            .await?
            .into_iter()
            .map(|extra| ExtraRegistrada {
                neto: extra.neto(perfil.salarial()),
                fecha_pago: extra.fecha_pago(perfil.calendario()),
                extra,
            })
            .collect())
    }
}

/// Caso de uso: deshacer una extra que se anotó mal.
#[derive(Debug)]
pub struct DeshacerExtra<'a, E> {
    extras: &'a E,
}

impl<'a, E: RepositorioExtras> DeshacerExtra<'a, E> {
    /// Arma el caso de uso.
    pub const fn new(extras: &'a E) -> Self {
        Self { extras }
    }

    /// Borra la extra si el usuario es su dueña (U1). Un id de otro es como
    /// si no existiera.
    pub async fn ejecutar(&self, usuario: UsuarioId, id: ExtraId) -> Result<(), ErrorAplicacion> {
        if self.extras.borrar(usuario, id).await? {
            Ok(())
        } else {
            Err(ErrorAplicacion::NoEncontrado)
        }
    }
}
