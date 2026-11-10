//! El presupuesto de cada quincena: armarlo, verlo y marcar lo pagado.

use crate::errores::ErrorAplicacion;
use crate::puertos::{Reloj, RepositorioPerfiles, RepositorioQuincenas};
use chrono::NaiveDate;
use finanzas_dominio::calendario::Corte;
use finanzas_dominio::dinero::Colones;
use finanzas_dominio::quincena::{Lado, Linea, Quincena};
use finanzas_dominio::texto::Concepto;
use finanzas_dominio::usuario::UsuarioId;

/// Una quincena con sus números ya calculados por el dominio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumenQuincena {
    /// La quincena con sus líneas y marcas.
    pub quincena: Quincena,
    /// Suma de ingresos.
    pub total_ingresos: Colones,
    /// Suma de pagos.
    pub total_pagos: Colones,
    /// Ingresos menos pagos (Q1).
    pub sobra: Colones,
    /// Pagos sin marcar (Q2).
    pub falta_por_pagar: Colones,
    /// Pagos hechos y pagos en total.
    pub avance: (usize, usize),
}

impl ResumenQuincena {
    fn de(quincena: Quincena) -> Self {
        Self {
            total_ingresos: quincena.total_ingresos(),
            total_pagos: quincena.total_pagos(),
            sobra: quincena.sobra(),
            falta_por_pagar: quincena.falta_por_pagar(),
            avance: quincena.avance(),
            quincena,
        }
    }
}

/// Una línea tal como llega de la interfaz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineaPedida {
    /// Concepto (texto libre; el dominio lo normaliza).
    pub concepto: String,
    /// Monto en colones.
    pub monto: i64,
}

/// El presupuesto de una quincena, tal como llega de la interfaz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolicitudPresupuesto {
    /// Dueño.
    pub usuario: UsuarioId,
    /// Fecha del pago que financia la quincena.
    pub pago: NaiveDate,
    /// Lo que entra.
    pub ingresos: Vec<LineaPedida>,
    /// Lo que se paga.
    pub pagos: Vec<LineaPedida>,
}

fn lineas(pedidas: Vec<LineaPedida>) -> Result<Vec<Linea>, ErrorAplicacion> {
    pedidas
        .into_iter()
        .map(|l| {
            Ok(Linea::new(
                Concepto::new(&l.concepto)?,
                Colones::new(l.monto),
            )?)
        })
        .collect()
}

/// Caso de uso: guardar (o rehacer) el presupuesto de una quincena.
#[derive(Debug)]
pub struct GuardarPresupuesto<'a, P, Q> {
    perfiles: &'a P,
    quincenas: &'a Q,
}

impl<'a, P, Q> GuardarPresupuesto<'a, P, Q>
where
    P: RepositorioPerfiles,
    Q: RepositorioQuincenas,
{
    /// Arma el caso de uso.
    pub const fn new(perfiles: &'a P, quincenas: &'a Q) -> Self {
        Self {
            perfiles,
            quincenas,
        }
    }

    /// El pago tiene que estar en el calendario (Q5). Si ya había presupuesto
    /// para esa quincena se reemplaza conservando lo marcado (Q4).
    pub async fn ejecutar(
        &self,
        s: SolicitudPresupuesto,
    ) -> Result<ResumenQuincena, ErrorAplicacion> {
        let perfil = self
            .perfiles
            .perfil(s.usuario)
            .await?
            .ok_or(ErrorAplicacion::PerfilNoEncontrado)?;
        if !perfil.calendario().es_dia_de_pago(s.pago) {
            return Err(ErrorAplicacion::NoEsDiaDePago(s.pago));
        }
        let ingresos = lineas(s.ingresos)?;
        let pagos = lineas(s.pagos)?;
        let quincena = match self.quincenas.de(s.usuario, s.pago).await? {
            Some(anterior) => anterior.con_presupuesto(ingresos, pagos),
            None => Quincena::new(s.usuario, s.pago, ingresos, pagos),
        };
        self.quincenas.guardar(&quincena).await?;
        Ok(ResumenQuincena::de(quincena))
    }
}

/// Caso de uso: ver una quincena.
#[derive(Debug)]
pub struct ConsultarQuincena<'a, P, Q, R> {
    perfiles: &'a P,
    quincenas: &'a Q,
    reloj: &'a R,
}

impl<'a, P, Q, R> ConsultarQuincena<'a, P, Q, R>
where
    P: RepositorioPerfiles,
    Q: RepositorioQuincenas,
    R: Reloj,
{
    /// Arma el caso de uso.
    pub const fn new(perfiles: &'a P, quincenas: &'a Q, reloj: &'a R) -> Self {
        Self {
            perfiles,
            quincenas,
            reloj,
        }
    }

    /// El último pago recibido según el calendario del usuario.
    async fn pago_en_curso(&self, usuario: UsuarioId) -> Result<NaiveDate, ErrorAplicacion> {
        let perfil = self
            .perfiles
            .perfil(usuario)
            .await?
            .ok_or(ErrorAplicacion::PerfilNoEncontrado)?;
        perfil
            .calendario()
            .ultimo_pago(self.reloj.hoy())
            .map(Corte::pago)
            .ok_or(ErrorAplicacion::FueraDeCalendario)
    }

    /// La quincena del pago pedido o, sin pago, la que se está gastando hoy
    /// (la del último pago, C4). `None` si todavía no tiene presupuesto.
    pub async fn ejecutar(
        &self,
        usuario: UsuarioId,
        pago: Option<NaiveDate>,
    ) -> Result<Option<ResumenQuincena>, ErrorAplicacion> {
        let pago = if let Some(p) = pago {
            p
        } else {
            self.pago_en_curso(usuario).await?
        };
        Ok(self
            .quincenas
            .de(usuario, pago)
            .await?
            .map(ResumenQuincena::de))
    }
}

/// Marcar o desmarcar una línea.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolicitudMarca {
    /// Dueño.
    pub usuario: UsuarioId,
    /// Quincena (fecha del pago).
    pub pago: NaiveDate,
    /// Ingreso o pago.
    pub lado: Lado,
    /// Posición de la línea en su lado.
    pub indice: usize,
    /// `true` para marcar como hecho.
    pub hecho: bool,
}

/// Caso de uso: marcar una línea como pagada o recibida.
#[derive(Debug)]
pub struct MarcarLinea<'a, Q> {
    quincenas: &'a Q,
}

impl<'a, Q: RepositorioQuincenas> MarcarLinea<'a, Q> {
    /// Arma el caso de uso.
    pub const fn new(quincenas: &'a Q) -> Self {
        Self { quincenas }
    }

    /// Marca la línea y devuelve la quincena actualizada.
    pub async fn ejecutar(&self, s: SolicitudMarca) -> Result<ResumenQuincena, ErrorAplicacion> {
        let mut quincena = self
            .quincenas
            .de(s.usuario, s.pago)
            .await?
            .ok_or(ErrorAplicacion::QuincenaNoEncontrada(s.pago))?;
        quincena.marcar(s.lado, s.indice, s.hecho)?;
        self.quincenas.guardar(&quincena).await?;
        Ok(ResumenQuincena::de(quincena))
    }
}
