//! Deshacer algo que se anotó mal.
//!
//! Sólo se borra lo del propio usuario (U1). En el fondo además decide el
//! dominio: sólo el último movimiento (U2).

use crate::errores::ErrorAplicacion;
use crate::puertos::{RepositorioExtras, RepositorioFondo, RepositorioGastos};
use finanzas_dominio::extra::ExtraId;
use finanzas_dominio::gasto::GastoId;
use finanzas_dominio::movimientos_fondo::{MovimientoFondoId, deshacible};
use finanzas_dominio::usuario::UsuarioId;

/// Qué se quiere deshacer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueDeshacer {
    /// Un gasto puntual.
    Gasto(GastoId),
    /// Una extra.
    Extra(ExtraId),
    /// Un movimiento del fondo de emergencia.
    MovimientoFondo(MovimientoFondoId),
}

/// Caso de uso: deshacer un registro.
#[derive(Debug)]
pub struct Deshacer<'a, G, E, F> {
    gastos: &'a G,
    extras: &'a E,
    fondo: &'a F,
}

impl<'a, G, E, F> Deshacer<'a, G, E, F>
where
    G: RepositorioGastos,
    E: RepositorioExtras,
    F: RepositorioFondo,
{
    /// Arma el caso de uso.
    pub const fn new(gastos: &'a G, extras: &'a E, fondo: &'a F) -> Self {
        Self {
            gastos,
            extras,
            fondo,
        }
    }

    /// Borra el registro si el usuario es su dueño y el dominio lo permite.
    pub async fn ejecutar(
        &self,
        usuario: UsuarioId,
        que: QueDeshacer,
    ) -> Result<(), ErrorAplicacion> {
        let borrado = match que {
            QueDeshacer::Gasto(id) => self.gastos.borrar(usuario, id).await?,
            QueDeshacer::Extra(id) => self.extras.borrar(usuario, id).await?,
            QueDeshacer::MovimientoFondo(id) => {
                let movimientos = self.fondo.de_usuario(usuario).await?;
                deshacible(&movimientos, id).ok_or(ErrorAplicacion::NoEncontrado)??;
                self.fondo.borrar(usuario, id).await?
            }
        };
        if borrado {
            Ok(())
        } else {
            Err(ErrorAplicacion::NoEncontrado)
        }
    }
}
