//! Gastos puntuales: lo que no es fijo ni se repite.

use crate::dinero::Colones;
use crate::texto::Concepto;
use crate::usuario::UsuarioId;
use chrono::NaiveDate;
use thiserror::Error;
use uuid::Uuid;

/// Identificador de un gasto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GastoId(Uuid);

impl GastoId {
    /// Envuelve un UUID ya generado.
    pub const fn new(id: Uuid) -> Self {
        Self(id)
    }

    /// El UUID.
    pub const fn uuid(self) -> Uuid {
        self.0
    }
}

/// En qué se gastó.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Categoria {
    /// Comida o antojo.
    Comida,
    /// La casa.
    Casa,
    /// La moto.
    Moto,
    /// Salud.
    Salud,
    /// Familia.
    Familia,
    /// Ropa.
    Ropa,
    /// Trámites.
    Tramites,
    /// Gusto personal.
    Gusto,
    /// Otra cosa.
    Otro,
}

/// De dónde salió la plata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origen {
    /// La cuenta de gastos (gasolina y caja chica).
    CuentaGastos,
    /// La cuenta donde cae el salario.
    Planilla,
    /// Lo que sobró del mes.
    SobraDelMes,
    /// El gusto de la quincena.
    Gusto,
    /// El fondo de emergencia (no cuenta contra la caja chica).
    Fondo,
}

/// Por qué no se puede registrar un gasto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ErrorGasto {
    /// El monto debe ser mayor que cero.
    #[error("el monto del gasto debe ser mayor que cero")]
    MontoInvalido,
}

/// Un gasto puntual.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gasto {
    id: GastoId,
    usuario: UsuarioId,
    fecha: NaiveDate,
    concepto: Concepto,
    monto: Colones,
    categoria: Categoria,
    origen: Origen,
}

/// Los datos de un gasto nuevo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatosGasto {
    /// Día del gasto.
    pub fecha: NaiveDate,
    /// En qué.
    pub concepto: Concepto,
    /// Cuánto.
    pub monto: Colones,
    /// Categoría.
    pub categoria: Categoria,
    /// De dónde salió.
    pub origen: Origen,
}

impl Gasto {
    /// Registra un gasto.
    pub fn new(id: GastoId, usuario: UsuarioId, datos: DatosGasto) -> Result<Self, ErrorGasto> {
        if !datos.monto.es_positivo() {
            return Err(ErrorGasto::MontoInvalido);
        }
        Ok(Self {
            id,
            usuario,
            fecha: datos.fecha,
            concepto: datos.concepto,
            monto: datos.monto,
            categoria: datos.categoria,
            origen: datos.origen,
        })
    }

    /// `true` si cuenta contra la caja chica (todo lo que no sale del fondo).
    pub fn cuenta_para_caja_chica(&self) -> bool {
        self.origen != Origen::Fondo
    }

    /// Identificador.
    pub const fn id(&self) -> GastoId {
        self.id
    }

    /// Dueño.
    pub const fn usuario(&self) -> UsuarioId {
        self.usuario
    }

    /// Día del gasto.
    pub const fn fecha(&self) -> NaiveDate {
        self.fecha
    }

    /// Concepto.
    pub const fn concepto(&self) -> &Concepto {
        &self.concepto
    }

    /// Monto.
    pub const fn monto(&self) -> Colones {
        self.monto
    }

    /// Categoría.
    pub const fn categoria(&self) -> Categoria {
        self.categoria
    }

    /// Origen.
    pub const fn origen(&self) -> Origen {
        self.origen
    }
}

/// Lo gastado contra la caja chica entre dos fechas (incluidas).
pub fn gastado_en_caja_chica(gastos: &[Gasto], desde: NaiveDate, hasta: NaiveDate) -> Colones {
    gastos
        .iter()
        .filter(|g| g.cuenta_para_caja_chica() && desde <= g.fecha && g.fecha <= hasta)
        .map(Gasto::monto)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{Categoria, DatosGasto, ErrorGasto, Gasto, GastoId, Origen, gastado_en_caja_chica};
    use crate::dinero::Colones;
    use crate::texto::Concepto;
    use crate::usuario::UsuarioId;
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn f(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, d).unwrap()
    }

    fn gasto(n: u128, dia: u32, monto: i64, origen: Origen) -> Gasto {
        Gasto::new(
            GastoId::new(Uuid::from_u128(n)),
            UsuarioId::new(Uuid::from_u128(1)),
            DatosGasto {
                fecha: f(dia),
                concepto: Concepto::new("prueba").unwrap(),
                monto: Colones::new(monto),
                categoria: Categoria::Otro,
                origen,
            },
        )
        .unwrap()
    }

    #[test]
    fn lo_del_fondo_no_cuenta_para_la_caja_chica() {
        let gastos = [
            gasto(1, 7, 1_000, Origen::Planilla),
            gasto(2, 9, 2_950, Origen::CuentaGastos),
            gasto(3, 9, 60_000, Origen::Fondo),
            gasto(4, 13, 4_000, Origen::CuentaGastos),
        ];
        assert_eq!(
            gastado_en_caja_chica(
                &gastos,
                NaiveDate::from_ymd_opt(2026, 10, 30).unwrap(),
                f(12)
            ),
            Colones::new(3_950),
            "pasajes + jugo; el fondo y el gasto del 13 no entran"
        );
    }

    #[test]
    fn monto_cero_se_rechaza() {
        let r = Gasto::new(
            GastoId::new(Uuid::from_u128(1)),
            UsuarioId::new(Uuid::from_u128(1)),
            DatosGasto {
                fecha: f(9),
                concepto: Concepto::new("x").unwrap(),
                monto: Colones::CERO,
                categoria: Categoria::Otro,
                origen: Origen::Planilla,
            },
        );
        assert_eq!(r, Err(ErrorGasto::MontoInvalido), "monto cero");
    }
}
