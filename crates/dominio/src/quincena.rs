//! Presupuesto de una quincena: lo que entra, lo que se paga y lo que sobra.

use crate::dinero::Colones;
use crate::texto::Concepto;
use thiserror::Error;

/// Una línea del presupuesto (un ingreso o un pago).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linea {
    concepto: Concepto,
    monto: Colones,
    hecho: bool,
}

/// Por qué una línea no es válida o no se encuentra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ErrorQuincena {
    /// Un monto presupuestado no puede ser negativo.
    #[error("el monto no puede ser negativo")]
    MontoNegativo,
    /// No hay una línea en esa posición.
    #[error("no existe esa línea")]
    LineaInexistente,
}

impl Linea {
    /// Crea una línea pendiente.
    pub fn new(concepto: Concepto, monto: Colones) -> Result<Self, ErrorQuincena> {
        if monto.es_negativo() {
            return Err(ErrorQuincena::MontoNegativo);
        }
        Ok(Self {
            concepto,
            monto,
            hecho: false,
        })
    }

    /// El concepto.
    pub const fn concepto(&self) -> &Concepto {
        &self.concepto
    }

    /// El monto.
    pub const fn monto(&self) -> Colones {
        self.monto
    }

    /// Si ya llegó (ingreso) o ya se pagó (pago).
    pub const fn hecho(&self) -> bool {
        self.hecho
    }
}

/// El presupuesto de una quincena.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Quincena {
    ingresos: Vec<Linea>,
    pagos: Vec<Linea>,
}

/// Qué lado del presupuesto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lado {
    /// Lo que entra.
    Ingreso,
    /// Lo que sale.
    Pago,
}

impl Quincena {
    /// Una quincena con sus líneas.
    pub const fn new(ingresos: Vec<Linea>, pagos: Vec<Linea>) -> Self {
        Self { ingresos, pagos }
    }

    /// Marca o desmarca una línea como hecha.
    pub fn marcar(&mut self, lado: Lado, indice: usize, hecho: bool) -> Result<(), ErrorQuincena> {
        let lineas = match lado {
            Lado::Ingreso => &mut self.ingresos,
            Lado::Pago => &mut self.pagos,
        };
        let linea = lineas
            .get_mut(indice)
            .ok_or(ErrorQuincena::LineaInexistente)?;
        linea.hecho = hecho;
        Ok(())
    }

    /// Total de ingresos presupuestados.
    pub fn total_ingresos(&self) -> Colones {
        self.ingresos.iter().map(Linea::monto).sum()
    }

    /// Total de pagos presupuestados.
    pub fn total_pagos(&self) -> Colones {
        self.pagos.iter().map(Linea::monto).sum()
    }

    /// Lo que falta pagar.
    pub fn falta_por_pagar(&self) -> Colones {
        self.pagos
            .iter()
            .filter(|l| !l.hecho)
            .map(Linea::monto)
            .sum()
    }

    /// Lo que sobra si todo sale como está presupuestado.
    pub fn sobra(&self) -> Colones {
        self.total_ingresos() - self.total_pagos()
    }

    /// Pagos hechos y total de pagos.
    pub fn avance(&self) -> (usize, usize) {
        (
            self.pagos.iter().filter(|l| l.hecho).count(),
            self.pagos.len(),
        )
    }

    /// Las líneas de ingreso.
    pub fn ingresos(&self) -> &[Linea] {
        &self.ingresos
    }

    /// Las líneas de pago.
    pub fn pagos(&self) -> &[Linea] {
        &self.pagos
    }
}

#[cfg(test)]
mod tests {
    use super::{ErrorQuincena, Lado, Linea, Quincena};
    use crate::dinero::Colones;
    use crate::texto::Concepto;

    fn l(concepto: &str, monto: i64) -> Linea {
        Linea::new(Concepto::new(concepto).unwrap(), Colones::new(monto)).unwrap()
    }

    /// La quincena del 13 de noviembre de 2026, tal como está en la hoja.
    fn q1_noviembre() -> Quincena {
        Quincena::new(
            vec![
                l("salario base", 232_225),
                l("extras", 103_404),
                l("papá", 65_000),
                l("saldo para operar", 14_300),
            ],
            vec![
                l("moto", 104_000),
                l("comida", 100_000),
                l("hijastra", 50_000),
                l("gasolina + caja chica", 40_000),
                l("internet", 23_000),
                l("luz", 20_000),
                l("línea", 20_000),
                l("claude", 11_000),
                l("ahorro moto", 6_000),
            ],
        )
    }

    #[test]
    fn totales_y_sobra_iguales_a_la_hoja() {
        let q = q1_noviembre();
        assert_eq!(q.total_ingresos(), Colones::new(414_929), "total ingresos");
        assert_eq!(q.total_pagos(), Colones::new(374_000), "total pagos");
        assert_eq!(q.sobra(), Colones::new(40_929), "sobra");
    }

    #[test]
    fn marcar_pagos_baja_lo_que_falta() {
        let mut q = q1_noviembre();
        q.marcar(Lado::Pago, 0, true).unwrap();
        assert_eq!(q.falta_por_pagar(), Colones::new(270_000), "pagada la moto");
        assert_eq!(q.avance(), (1, 9), "1 de 9");
        assert_eq!(
            q.marcar(Lado::Pago, 99, true),
            Err(ErrorQuincena::LineaInexistente),
            "índice fuera"
        );
    }

    #[test]
    fn no_hay_montos_negativos() {
        assert_eq!(
            Linea::new(Concepto::new("x").unwrap(), Colones::new(-1)),
            Err(ErrorQuincena::MontoNegativo),
            "negativo"
        );
    }
}
