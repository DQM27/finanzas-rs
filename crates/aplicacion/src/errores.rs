//! Errores de los casos de uso.

use finanzas_dominio::extra::ErrorExtra;
use finanzas_dominio::fondo::RechazoRetiro;
use finanzas_dominio::gasto::ErrorGasto;
use finanzas_dominio::movimientos_fondo::ErrorFondo;
use finanzas_dominio::texto::ErrorConcepto;
use thiserror::Error;

/// Un adaptador no pudo leer o guardar. Lleva el detalle técnico para el log.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("falló el almacenamiento: {0}")]
pub struct ErrorRepositorio(pub String);

/// Lo que puede salir mal en un caso de uso.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorAplicacion {
    /// El usuario no tiene configuración.
    #[error("el usuario no tiene perfil configurado")]
    PerfilNoEncontrado,
    /// Hoy no está cubierto por el calendario de cortes.
    #[error("el calendario de cortes no cubre la fecha de hoy")]
    FueraDeCalendario,
    /// El concepto no es válido.
    #[error(transparent)]
    Concepto(#[from] ErrorConcepto),
    /// El gasto no es válido.
    #[error(transparent)]
    Gasto(#[from] ErrorGasto),
    /// La extra no es válida.
    #[error(transparent)]
    Extra(#[from] ErrorExtra),
    /// El movimiento del fondo no es válido.
    #[error(transparent)]
    Fondo(#[from] ErrorFondo),
    /// El dominio no autorizó sacar del fondo.
    #[error(transparent)]
    RetiroRechazado(#[from] RechazoRetiro),
    /// Falló el almacenamiento.
    #[error(transparent)]
    Repositorio(#[from] ErrorRepositorio),
}
