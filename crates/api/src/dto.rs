//! Lo que viaja en JSON. Estos tipos son el contrato con la interfaz: cambiar
//! un nombre rompe a Angular. El dinero va siempre en colones enteros y las
//! fechas como `AAAA-MM-DD`.

use chrono::NaiveDate;
use finanzas_aplicacion::casos_de_uso::calendario::CortesHoy;
use finanzas_aplicacion::casos_de_uso::extras::ExtraRegistrada;
use finanzas_aplicacion::casos_de_uso::fondo::{Deposito, RetiroHecho};
use finanzas_aplicacion::casos_de_uso::gastos::GastoRegistrado;
use finanzas_aplicacion::casos_de_uso::gastos::ResumenCaja;
use finanzas_aplicacion::casos_de_uso::perfil::{CortePedido, ResumenPerfil};
use finanzas_aplicacion::casos_de_uso::quincena::{LineaPedida, ResumenMes, ResumenQuincena};
use finanzas_dominio::base::dinero::Colones;
use finanzas_dominio::calendario::Corte;
use finanzas_dominio::extras::TipoExtra;
use finanzas_dominio::fondo::movimientos::{EstadoFondo, Movimiento, MovimientoFondo};
use finanzas_dominio::fondo::{NivelEmergencia, PlanReposicion};
use finanzas_dominio::gastos::caja_chica::NivelCaja;
use finanzas_dominio::gastos::{Categoria, Gasto, Origen};
use finanzas_dominio::quincena::{Balance, Lado, Linea};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ------------------------------------------------------------- enumerados

/// Categoría de un gasto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CategoriaDto {
    Comida,
    Casa,
    Moto,
    Salud,
    Familia,
    Ropa,
    Tramites,
    Gusto,
    Otro,
}

impl From<CategoriaDto> for Categoria {
    fn from(c: CategoriaDto) -> Self {
        match c {
            CategoriaDto::Comida => Self::Comida,
            CategoriaDto::Casa => Self::Casa,
            CategoriaDto::Moto => Self::Moto,
            CategoriaDto::Salud => Self::Salud,
            CategoriaDto::Familia => Self::Familia,
            CategoriaDto::Ropa => Self::Ropa,
            CategoriaDto::Tramites => Self::Tramites,
            CategoriaDto::Gusto => Self::Gusto,
            CategoriaDto::Otro => Self::Otro,
        }
    }
}

impl From<Categoria> for CategoriaDto {
    fn from(c: Categoria) -> Self {
        match c {
            Categoria::Comida => Self::Comida,
            Categoria::Casa => Self::Casa,
            Categoria::Moto => Self::Moto,
            Categoria::Salud => Self::Salud,
            Categoria::Familia => Self::Familia,
            Categoria::Ropa => Self::Ropa,
            Categoria::Tramites => Self::Tramites,
            Categoria::Gusto => Self::Gusto,
            Categoria::Otro => Self::Otro,
        }
    }
}

/// De dónde salió la plata de un gasto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrigenDto {
    CuentaGastos,
    Planilla,
    SobraDelMes,
    Gusto,
    Fondo,
}

impl From<OrigenDto> for Origen {
    fn from(o: OrigenDto) -> Self {
        match o {
            OrigenDto::CuentaGastos => Self::CuentaGastos,
            OrigenDto::Planilla => Self::Planilla,
            OrigenDto::SobraDelMes => Self::SobraDelMes,
            OrigenDto::Gusto => Self::Gusto,
            OrigenDto::Fondo => Self::Fondo,
        }
    }
}

impl From<Origen> for OrigenDto {
    fn from(o: Origen) -> Self {
        match o {
            Origen::CuentaGastos => Self::CuentaGastos,
            Origen::Planilla => Self::Planilla,
            Origen::SobraDelMes => Self::SobraDelMes,
            Origen::Gusto => Self::Gusto,
            Origen::Fondo => Self::Fondo,
        }
    }
}

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

/// Nivel de una emergencia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NivelDto {
    Vital,
    Trabajo,
    Descuido,
    Falta,
}

impl From<NivelDto> for NivelEmergencia {
    fn from(n: NivelDto) -> Self {
        match n {
            NivelDto::Vital => Self::Vital,
            NivelDto::Trabajo => Self::Trabajo,
            NivelDto::Descuido => Self::Descuido,
            NivelDto::Falta => Self::Falta,
        }
    }
}

impl From<NivelEmergencia> for NivelDto {
    fn from(n: NivelEmergencia) -> Self {
        match n {
            NivelEmergencia::Vital => Self::Vital,
            NivelEmergencia::Trabajo => Self::Trabajo,
            NivelEmergencia::Descuido => Self::Descuido,
            NivelEmergencia::Falta => Self::Falta,
        }
    }
}

/// Lado de una quincena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LadoDto {
    Ingreso,
    Pago,
}

impl From<LadoDto> for Lado {
    fn from(l: LadoDto) -> Self {
        match l {
            LadoDto::Ingreso => Self::Ingreso,
            LadoDto::Pago => Self::Pago,
        }
    }
}

/// Aporte o reposición.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepositoDto {
    Aporte,
    Reposicion,
}

impl From<DepositoDto> for Deposito {
    fn from(d: DepositoDto) -> Self {
        match d {
            DepositoDto::Aporte => Self::Aporte,
            DepositoDto::Reposicion => Self::Reposicion,
        }
    }
}

/// Cómo está la caja chica.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NivelCajaDto {
    Holgada,
    Baja,
    Agotada,
}

impl From<NivelCaja> for NivelCajaDto {
    fn from(n: NivelCaja) -> Self {
        match n {
            NivelCaja::Holgada => Self::Holgada,
            NivelCaja::Baja => Self::Baja,
            NivelCaja::Agotada => Self::Agotada,
        }
    }
}

// ------------------------------------------------------------- peticiones

/// `POST /gastos`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevoGasto {
    pub fecha: NaiveDate,
    pub concepto: String,
    pub monto: i64,
    pub categoria: CategoriaDto,
    pub origen: OrigenDto,
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

/// `POST /fondo/retiros`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevoRetiro {
    pub concepto: String,
    pub monto: i64,
    pub nivel: NivelDto,
    pub inesperado: bool,
    pub necesario: bool,
    pub urgente: bool,
}

/// `POST /fondo/depositos`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuevoDeposito {
    pub tipo: DepositoDto,
    pub concepto: String,
    pub monto: i64,
}

/// Una línea de presupuesto pedida.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineaNueva {
    pub concepto: String,
    pub monto: i64,
}

impl From<LineaNueva> for LineaPedida {
    fn from(l: LineaNueva) -> Self {
        Self {
            concepto: l.concepto,
            monto: l.monto,
        }
    }
}

/// `PUT /quincenas/{pago}`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Presupuesto {
    pub ingresos: Vec<LineaNueva>,
    pub pagos: Vec<LineaNueva>,
}

/// `PUT /quincenas/{pago}/marcas`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Marca {
    pub lado: LadoDto,
    pub indice: usize,
    pub hecho: bool,
}

// ------------------------------------------------------------- respuestas

/// La caja chica del periodo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CajaDto {
    pub desde: NaiveDate,
    pub hasta: NaiveDate,
    pub queda: i64,
    pub exceso: i64,
    pub nivel: NivelCajaDto,
}

impl From<ResumenCaja> for CajaDto {
    fn from(r: ResumenCaja) -> Self {
        Self {
            desde: r.desde,
            hasta: r.hasta,
            queda: r.estado.queda.valor(),
            exceso: r.estado.exceso.valor(),
            nivel: r.estado.nivel.into(),
        }
    }
}

/// Un gasto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GastoDto {
    pub id: Uuid,
    pub fecha: NaiveDate,
    pub concepto: String,
    pub monto: i64,
    pub categoria: CategoriaDto,
    pub origen: OrigenDto,
}

impl From<&Gasto> for GastoDto {
    fn from(g: &Gasto) -> Self {
        Self {
            id: g.id().uuid(),
            fecha: g.fecha(),
            concepto: g.concepto().como_str().to_owned(),
            monto: g.monto().valor(),
            categoria: g.categoria().into(),
            origen: g.origen().into(),
        }
    }
}

/// Respuesta de `POST /gastos`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GastoRegistradoDto {
    pub gasto: GastoDto,
    pub caja: CajaDto,
}

impl From<GastoRegistrado> for GastoRegistradoDto {
    fn from(r: GastoRegistrado) -> Self {
        Self {
            gasto: (&r.gasto).into(),
            caja: r.caja.into(),
        }
    }
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

/// Cómo está el fondo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FondoDto {
    pub saldo: i64,
    pub por_reponer: i64,
    pub cuota_mensual: i64,
}

impl From<EstadoFondo> for FondoDto {
    fn from(e: EstadoFondo) -> Self {
        Self {
            saldo: e.saldo.valor(),
            por_reponer: e.por_reponer.valor(),
            cuota_mensual: e.cuota_mensual.valor(),
        }
    }
}

/// El plan para reponer un retiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PlanDto {
    pub nivel: NivelDto,
    pub sacado: i64,
    pub a_reponer: i64,
    pub meses: u8,
    pub cuota_mensual: i64,
}

impl From<PlanReposicion> for PlanDto {
    fn from(p: PlanReposicion) -> Self {
        Self {
            nivel: p.nivel.into(),
            sacado: p.sacado.valor(),
            a_reponer: p.a_reponer.valor(),
            meses: p.meses,
            cuota_mensual: p.cuota_mensual.valor(),
        }
    }
}

/// Respuesta de `POST /fondo/retiros`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RetiroDto {
    pub plan: PlanDto,
    pub fondo: FondoDto,
}

impl From<RetiroHecho> for RetiroDto {
    fn from(r: RetiroHecho) -> Self {
        Self {
            plan: r.plan.into(),
            fondo: r.estado.into(),
        }
    }
}

/// Una línea de la quincena.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LineaDto {
    pub concepto: String,
    pub monto: i64,
    pub hecho: bool,
}

impl From<&Linea> for LineaDto {
    fn from(l: &Linea) -> Self {
        Self {
            concepto: l.concepto().como_str().to_owned(),
            monto: l.monto().valor(),
            hecho: l.hecho(),
        }
    }
}

/// Pagos hechos de cuántos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AvanceDto {
    pub hechos: usize,
    pub total: usize,
}

/// Lo que entra solo en una quincena (Q7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutomaticosDto {
    /// Salario neto.
    pub salario: i64,
    /// Neto de las extras que se pagan ese día (E4).
    pub extras: i64,
    /// `true` si el pago ya llegó.
    pub llegaron: bool,
}

/// Los totales de una quincena o de un mes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BalanceDto {
    pub salario: i64,
    pub extras: i64,
    pub otros_ingresos: i64,
    pub total_ingresos: i64,
    pub total_pagos: i64,
    pub sobra: i64,
    pub falta_por_pagar: i64,
}

impl From<Balance> for BalanceDto {
    fn from(b: Balance) -> Self {
        Self {
            salario: b.salario().valor(),
            extras: b.extras().valor(),
            otros_ingresos: b.otros_ingresos().valor(),
            total_ingresos: b.total_ingresos().valor(),
            total_pagos: b.total_pagos().valor(),
            sobra: b.sobra().valor(),
            falta_por_pagar: b.falta_por_pagar().valor(),
        }
    }
}

/// Una quincena con sus números.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuincenaDto {
    pub pago: NaiveDate,
    /// `false` si todavía no se le armó presupuesto.
    pub presupuestada: bool,
    pub automaticos: AutomaticosDto,
    /// Los otros ingresos, escritos a mano.
    pub ingresos: Vec<LineaDto>,
    pub pagos: Vec<LineaDto>,
    /// Suma de los otros ingresos.
    pub otros_ingresos: i64,
    pub total_ingresos: i64,
    pub total_pagos: i64,
    pub sobra: i64,
    pub falta_por_pagar: i64,
    pub avance: AvanceDto,
}

impl From<ResumenQuincena> for QuincenaDto {
    fn from(r: ResumenQuincena) -> Self {
        Self {
            pago: r.quincena.pago(),
            presupuestada: r.presupuestada,
            automaticos: AutomaticosDto {
                salario: r.automaticos.salario().valor(),
                extras: r.automaticos.extras().valor(),
                llegaron: r.automaticos.llegaron(),
            },
            ingresos: r.quincena.ingresos().iter().map(LineaDto::from).collect(),
            pagos: r.quincena.pagos().iter().map(LineaDto::from).collect(),
            otros_ingresos: r.balance.otros_ingresos().valor(),
            total_ingresos: r.balance.total_ingresos().valor(),
            total_pagos: r.balance.total_pagos().valor(),
            sobra: r.balance.sobra().valor(),
            falta_por_pagar: r.balance.falta_por_pagar().valor(),
            avance: AvanceDto {
                hechos: r.avance.0,
                total: r.avance.1,
            },
        }
    }
}

/// Un mes: sus quincenas y la suma.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MesDto {
    /// `AAAA-MM`.
    pub mes: String,
    pub quincenas: Vec<QuincenaDto>,
    pub balance: BalanceDto,
}

impl From<ResumenMes> for MesDto {
    fn from(r: ResumenMes) -> Self {
        Self {
            mes: format!("{:04}-{:02}", r.anio, r.mes),
            quincenas: r.quincenas.into_iter().map(QuincenaDto::from).collect(),
            balance: r.balance.into(),
        }
    }
}

/// Un error, siempre con un código estable y un mensaje para mostrar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDto {
    pub codigo: String,
    pub mensaje: String,
}

// ------------------------------------------------------------------ perfil

/// Tarifas brutas de cada tipo de extra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TarifasDto {
    pub nocturna: i64,
    pub diurna: i64,
    pub especial: i64,
    pub feriado: i64,
}

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

/// Un movimiento del fondo de emergencia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MovimientoFondoDto {
    pub id: Uuid,
    pub fecha: NaiveDate,
    pub concepto: String,
    pub monto: i64,
    pub tipo: DepositoOMovimientoDto,
    /// Sólo en los retiros.
    pub plan: Option<PlanDto>,
}

/// Clase de movimiento del fondo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepositoOMovimientoDto {
    Aporte,
    Retiro,
    Reposicion,
}

impl From<&MovimientoFondo> for MovimientoFondoDto {
    fn from(m: &MovimientoFondo) -> Self {
        let (tipo, plan) = match m.movimiento() {
            Movimiento::Aporte => (DepositoOMovimientoDto::Aporte, None),
            Movimiento::Reposicion => (DepositoOMovimientoDto::Reposicion, None),
            Movimiento::Retiro(plan) => (DepositoOMovimientoDto::Retiro, Some(plan.into())),
        };
        Self {
            id: m.id().uuid(),
            fecha: m.fecha(),
            concepto: m.concepto().como_str().to_owned(),
            monto: m.monto().valor(),
            tipo,
            plan,
        }
    }
}
