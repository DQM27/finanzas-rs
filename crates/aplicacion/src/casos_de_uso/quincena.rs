//! El presupuesto de cada quincena y del mes: armarlo, verlo y marcar lo
//! pagado. Lo que entra solo (salario y extras, Q7) lo calcula el dominio con
//! el perfil y las extras del usuario; aquí sólo se buscan los datos.

use crate::errores::ErrorAplicacion;
use crate::puertos::{Reloj, RepositorioExtras, RepositorioPerfiles, RepositorioQuincenas};
use chrono::{Datelike, NaiveDate};
use finanzas_dominio::base::dinero::Colones;
use finanzas_dominio::base::texto::Concepto;
use finanzas_dominio::base::usuario::UsuarioId;
use finanzas_dominio::calendario::Corte;
use finanzas_dominio::extras::Extra;
use finanzas_dominio::perfil::Perfil;
use finanzas_dominio::quincena::{Balance, IngresosAutomaticos, Lado, Linea, Quincena};

/// Una quincena con sus números ya calculados por el dominio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumenQuincena {
    /// La quincena con sus líneas y marcas (vacía si no tiene presupuesto).
    pub quincena: Quincena,
    /// `false` si todavía no se le armó presupuesto.
    pub presupuestada: bool,
    /// Lo que entra solo (Q7).
    pub automaticos: IngresosAutomaticos,
    /// Totales, sobra (Q1) y falta por pagar (Q2).
    pub balance: Balance,
    /// Pagos hechos y pagos en total.
    pub avance: (usize, usize),
}

/// Un mes: sus quincenas (C6) y la suma de todas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumenMes {
    /// Año.
    pub anio: i32,
    /// Mes (1 a 12).
    pub mes: u32,
    /// Las quincenas cuyo pago cae en el mes, en orden.
    pub quincenas: Vec<ResumenQuincena>,
    /// La suma de sus balances.
    pub balance: Balance,
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
    /// Los otros ingresos (el salario y las extras entran solos).
    pub ingresos: Vec<LineaPedida>,
    /// Lo que se paga.
    pub pagos: Vec<LineaPedida>,
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

/// Las fuentes que necesitan todos los casos de uso de quincenas.
#[derive(Debug)]
pub struct Presupuestos<'a, P, Q, E, R> {
    perfiles: &'a P,
    quincenas: &'a Q,
    extras: &'a E,
    reloj: &'a R,
}

/// Lo que se lee una vez por petición para resumir quincenas.
struct Datos {
    perfil: Perfil,
    extras: Vec<Extra>,
    hoy: NaiveDate,
}

impl Datos {
    /// Resume una quincena: el dominio calcula lo automático y el balance.
    fn resumir(&self, quincena: Quincena, presupuestada: bool) -> ResumenQuincena {
        let automaticos = IngresosAutomaticos::de(
            self.perfil.salarial(),
            self.perfil.calendario(),
            &self.extras,
            quincena.pago(),
            self.hoy,
        );
        ResumenQuincena {
            balance: quincena.balance(&automaticos),
            avance: quincena.avance(),
            automaticos,
            presupuestada,
            quincena,
        }
    }
}

impl<'a, P, Q, E, R> Presupuestos<'a, P, Q, E, R>
where
    P: RepositorioPerfiles,
    Q: RepositorioQuincenas,
    E: RepositorioExtras,
    R: Reloj,
{
    /// Arma los casos de uso.
    pub const fn new(perfiles: &'a P, quincenas: &'a Q, extras: &'a E, reloj: &'a R) -> Self {
        Self {
            perfiles,
            quincenas,
            extras,
            reloj,
        }
    }

    async fn datos(&self, usuario: UsuarioId) -> Result<Datos, ErrorAplicacion> {
        let perfil = self
            .perfiles
            .perfil(usuario)
            .await?
            .ok_or(ErrorAplicacion::PerfilNoEncontrado)?;
        Ok(Datos {
            perfil,
            extras: self.extras.de_usuario(usuario).await?,
            hoy: self.reloj.hoy(),
        })
    }

    /// La quincena guardada o, si no tiene presupuesto, una vacía.
    async fn quincena(
        &self,
        d: &Datos,
        usuario: UsuarioId,
        pago: NaiveDate,
    ) -> Result<ResumenQuincena, ErrorAplicacion> {
        Ok(self.quincenas.de(usuario, pago).await?.map_or_else(
            || d.resumir(Quincena::new(usuario, pago, vec![], vec![]), false),
            |q| d.resumir(q, true),
        ))
    }

    /// La quincena del pago pedido o, sin pago, la que se está gastando hoy
    /// (la del último pago, C4). Siempre trae lo que entra solo, aunque no
    /// tenga presupuesto; el pago tiene que estar en el calendario (Q5).
    pub async fn consultar(
        &self,
        usuario: UsuarioId,
        pago: Option<NaiveDate>,
    ) -> Result<ResumenQuincena, ErrorAplicacion> {
        let d = self.datos(usuario).await?;
        let calendario = d.perfil.calendario();
        let pago = match pago {
            Some(p) if calendario.es_dia_de_pago(p) => p,
            Some(p) => return Err(ErrorAplicacion::NoEsDiaDePago(p)),
            None => calendario
                .ultimo_pago(d.hoy)
                .map(Corte::pago)
                .ok_or(ErrorAplicacion::FueraDeCalendario)?,
        };
        self.quincena(&d, usuario, pago).await
    }

    /// Las quincenas de un mes (C6) con su suma. Sin mes, el de hoy.
    pub async fn mes(
        &self,
        usuario: UsuarioId,
        mes: Option<(i32, u32)>,
    ) -> Result<ResumenMes, ErrorAplicacion> {
        let d = self.datos(usuario).await?;
        let (anio, mes) = mes.unwrap_or_else(|| (d.hoy.year(), d.hoy.month()));
        let mut quincenas = Vec::new();
        for pago in d.perfil.calendario().pagos_del_mes(anio, mes) {
            quincenas.push(self.quincena(&d, usuario, pago).await?);
        }
        Ok(ResumenMes {
            anio,
            mes,
            balance: quincenas.iter().map(|q| q.balance).sum(),
            quincenas,
        })
    }

    /// Guarda (o rehace) el presupuesto. El pago tiene que estar en el
    /// calendario (Q5). Si ya había presupuesto se reemplaza conservando lo
    /// marcado (Q4).
    pub async fn guardar(
        &self,
        s: SolicitudPresupuesto,
    ) -> Result<ResumenQuincena, ErrorAplicacion> {
        let d = self.datos(s.usuario).await?;
        if !d.perfil.calendario().es_dia_de_pago(s.pago) {
            return Err(ErrorAplicacion::NoEsDiaDePago(s.pago));
        }
        let ingresos = lineas(s.ingresos)?;
        let pagos = lineas(s.pagos)?;
        let quincena = match self.quincenas.de(s.usuario, s.pago).await? {
            Some(anterior) => anterior.con_presupuesto(ingresos, pagos),
            None => Quincena::new(s.usuario, s.pago, ingresos, pagos),
        };
        self.quincenas.guardar(&quincena).await?;
        Ok(d.resumir(quincena, true))
    }

    /// Marca una línea como pagada o recibida y devuelve la quincena.
    pub async fn marcar(&self, s: SolicitudMarca) -> Result<ResumenQuincena, ErrorAplicacion> {
        let d = self.datos(s.usuario).await?;
        let mut quincena = self
            .quincenas
            .de(s.usuario, s.pago)
            .await?
            .ok_or(ErrorAplicacion::QuincenaNoEncontrada(s.pago))?;
        quincena.marcar(s.lado, s.indice, s.hecho)?;
        self.quincenas.guardar(&quincena).await?;
        Ok(d.resumir(quincena, true))
    }
}
