//! Errores de los casos de uso.

use chrono::NaiveDate;
use finanzas_dominio::base::texto::ErrorConcepto;
use finanzas_dominio::calendario::ErrorCalendario;
use finanzas_dominio::extras::ErrorExtra;
use finanzas_dominio::fondo::RechazoRetiro;
use finanzas_dominio::fondo::movimientos::ErrorFondo;
use finanzas_dominio::gastos::ErrorGasto;
use finanzas_dominio::gastos::caja_chica::ErrorCajaChica;
use finanzas_dominio::perfil::ErrorPerfil;
use finanzas_dominio::quincena::ErrorQuincena;
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
    /// Esa fecha no es un pago del calendario del usuario.
    #[error("el {0} no es un día de pago del calendario")]
    NoEsDiaDePago(NaiveDate),
    /// Todavía no hay presupuesto para esa quincena.
    #[error("no hay presupuesto para la quincena del {0}")]
    QuincenaNoEncontrada(NaiveDate),
    /// No hay nada con ese id para ese usuario.
    #[error("no se encontró lo que se quería deshacer")]
    NoEncontrado,
    /// Los números del perfil no son válidos.
    #[error(transparent)]
    Perfil(#[from] ErrorPerfil),
    /// El calendario de cortes no es válido.
    #[error(transparent)]
    Calendario(#[from] ErrorCalendario),
    /// El tope de la caja chica no es válido.
    #[error(transparent)]
    CajaChica(#[from] ErrorCajaChica),
    /// La quincena no es válida.
    #[error(transparent)]
    Quincena(#[from] ErrorQuincena),
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
