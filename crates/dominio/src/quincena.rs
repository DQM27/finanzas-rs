//! Presupuesto de una quincena: lo que entra, lo que se paga y lo que sobra.
//!
//! Cada quincena es de un usuario y se identifica por la fecha del pago que la
//! financia (la fecha de pago de un corte del calendario).
//!
//! Lo que entra se arma de dos partes: lo que entra solo con el pago (el
//! salario neto y las extras de ese pago, Q7) y los otros ingresos que se
//! escriben a mano (una ayuda, un saldo que quedó).

use crate::base::dinero::Colones;
use crate::base::texto::Concepto;
use crate::base::usuario::UsuarioId;
use crate::calendario::CalendarioCortes;
use crate::extras::{Extra, neto_en_pago};
use crate::perfil::PerfilSalarial;
use chrono::NaiveDate;
use std::iter::Sum;
use thiserror::Error;

/// Lo que entra solo en una quincena, sin escribirlo (Q7): el salario neto
/// del perfil (P) y lo neto de las extras que se pagan ese día (E4). Llega
/// con el pago: el día de pago ya cuenta como recibido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IngresosAutomaticos {
    salario: Colones,
    extras: Colones,
    llegaron: bool,
}

impl IngresosAutomaticos {
    /// Lo que entra solo en el pago `pago`, visto desde `hoy`.
    pub fn de(
        salarial: &PerfilSalarial,
        calendario: &CalendarioCortes,
        extras: &[Extra],
        pago: NaiveDate,
        hoy: NaiveDate,
    ) -> Self {
        Self {
            salario: salarial.salario_neto_quincena(),
            extras: neto_en_pago(extras, pago, salarial, calendario),
            llegaron: pago <= hoy,
        }
    }

    /// El salario neto de la quincena.
    pub const fn salario(&self) -> Colones {
        self.salario
    }

    /// Lo neto de las extras que se pagan ese día.
    pub const fn extras(&self) -> Colones {
        self.extras
    }

    /// `true` si el pago ya llegó (es hoy o ya pasó).
    pub const fn llegaron(&self) -> bool {
        self.llegaron
    }
}

/// Los números de una quincena, o de varias sumadas (un mes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Balance {
    salario: Colones,
    extras: Colones,
    otros_ingresos: Colones,
    total_pagos: Colones,
    falta_por_pagar: Colones,
}

impl Balance {
    /// El salario neto.
    pub const fn salario(&self) -> Colones {
        self.salario
    }

    /// Lo neto de las extras.
    pub const fn extras(&self) -> Colones {
        self.extras
    }

    /// Los ingresos escritos a mano.
    pub const fn otros_ingresos(&self) -> Colones {
        self.otros_ingresos
    }

    /// Todo lo que entra: salario, extras y otros ingresos.
    pub fn total_ingresos(&self) -> Colones {
        self.salario + self.extras + self.otros_ingresos
    }

    /// Todo lo presupuestado para pagar.
    pub const fn total_pagos(&self) -> Colones {
        self.total_pagos
    }

    /// Lo que sobra si todo sale como está presupuestado (Q1).
    pub fn sobra(&self) -> Colones {
        self.total_ingresos() - self.total_pagos
    }

    /// Los pagos sin marcar (Q2).
    pub const fn falta_por_pagar(&self) -> Colones {
        self.falta_por_pagar
    }
}

impl Sum for Balance {
    fn sum<I: Iterator<Item = Self>>(balances: I) -> Self {
        balances.fold(Self::default(), |a, b| Self {
            salario: a.salario + b.salario,
            extras: a.extras + b.extras,
            otros_ingresos: a.otros_ingresos + b.otros_ingresos,
            total_pagos: a.total_pagos + b.total_pagos,
            falta_por_pagar: a.falta_por_pagar + b.falta_por_pagar,
        })
    }
}

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
        Self::reconstruir(concepto, monto, false)
    }

    /// Reconstruye una línea guardada, con su marca.
    pub fn reconstruir(
        concepto: Concepto,
        monto: Colones,
        hecho: bool,
    ) -> Result<Self, ErrorQuincena> {
        if monto.es_negativo() {
            return Err(ErrorQuincena::MontoNegativo);
        }
        Ok(Self {
            concepto,
            monto,
            hecho,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quincena {
    usuario: UsuarioId,
    pago: NaiveDate,
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
    pub const fn new(
        usuario: UsuarioId,
        pago: NaiveDate,
        ingresos: Vec<Linea>,
        pagos: Vec<Linea>,
    ) -> Self {
        Self {
            usuario,
            pago,
            ingresos,
            pagos,
        }
    }

    /// Reemplaza el presupuesto por uno nuevo sin perder lo ya marcado: una
    /// línea nueva queda hecha si en la anterior había una del mismo lado, con
    /// el mismo concepto, marcada (Q4). Si el concepto se repite, se empareja
    /// en orden.
    pub fn con_presupuesto(&self, ingresos: Vec<Linea>, pagos: Vec<Linea>) -> Self {
        fn heredar(anteriores: &[Linea], nuevas: Vec<Linea>) -> Vec<Linea> {
            let mut usadas = vec![false; anteriores.len()];
            nuevas
                .into_iter()
                .map(|mut linea| {
                    let pareja = anteriores
                        .iter()
                        .zip(usadas.iter_mut())
                        .find(|(a, usada)| !**usada && a.concepto == linea.concepto);
                    if let Some((anterior, usada)) = pareja {
                        *usada = true;
                        linea.hecho = anterior.hecho;
                    }
                    linea
                })
                .collect()
        }
        Self {
            usuario: self.usuario,
            pago: self.pago,
            ingresos: heredar(&self.ingresos, ingresos),
            pagos: heredar(&self.pagos, pagos),
        }
    }

    /// Dueño.
    pub const fn usuario(&self) -> UsuarioId {
        self.usuario
    }

    /// Fecha del pago que financia esta quincena.
    pub const fn pago(&self) -> NaiveDate {
        self.pago
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

    /// Los números de la quincena con lo que entra solo (Q1, Q2, Q7).
    pub fn balance(&self, automaticos: &IngresosAutomaticos) -> Balance {
        Balance {
            salario: automaticos.salario,
            extras: automaticos.extras,
            otros_ingresos: self.ingresos.iter().map(Linea::monto).sum(),
            total_pagos: self.pagos.iter().map(Linea::monto).sum(),
            falta_por_pagar: self
                .pagos
                .iter()
                .filter(|l| !l.hecho)
                .map(Linea::monto)
                .sum(),
        }
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
    use super::{Balance, ErrorQuincena, IngresosAutomaticos, Lado, Linea, Quincena};
    use crate::base::dinero::Colones;
    use crate::base::proporcion::Proporcion;
    use crate::base::texto::Concepto;
    use crate::base::usuario::UsuarioId;
    use crate::calendario::{CalendarioCortes, Corte};
    use crate::extras::{Extra, ExtraId, TipoExtra};
    use crate::perfil::PerfilSalarial;
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn f(a: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(a, m, d).unwrap()
    }

    fn pago_13() -> NaiveDate {
        f(2026, 11, 13)
    }

    fn l(concepto: &str, monto: i64) -> Linea {
        Linea::new(Concepto::new(concepto).unwrap(), Colones::new(monto)).unwrap()
    }

    fn salarial() -> PerfilSalarial {
        PerfilSalarial::new(
            Colones::new(293_000),
            Proporcion::desde_diezmilesimas(8617),
            Colones::new(20_253),
        )
        .unwrap()
    }

    fn calendario() -> CalendarioCortes {
        CalendarioCortes::new(vec![
            Corte::new(f(2026, 10, 17), f(2026, 11, 1), f(2026, 11, 4), pago_13()).unwrap(),
            Corte::new(
                f(2026, 11, 2),
                f(2026, 11, 13),
                f(2026, 11, 17),
                f(2026, 11, 30),
            )
            .unwrap(),
        ])
        .unwrap()
    }

    fn extra(n: u128, fecha: NaiveDate, bruto: i64) -> Extra {
        Extra::reconstruir(
            ExtraId::new(Uuid::from_u128(n)),
            UsuarioId::new(Uuid::from_u128(1)),
            fecha,
            TipoExtra::Otra,
            Colones::new(bruto),
        )
    }

    /// Lo que entra solo el 13 de noviembre: el salario y una extra de
    /// ₡120 000 brutos trabajada el 31 de octubre (₡103 404 netos).
    fn automaticos_13(hoy: NaiveDate) -> IngresosAutomaticos {
        let extras = [
            extra(1, f(2026, 10, 31), 120_000),
            extra(2, f(2026, 11, 5), 25_800),
        ];
        IngresosAutomaticos::de(&salarial(), &calendario(), &extras, pago_13(), hoy)
    }

    /// La quincena del 13 de noviembre de 2026, tal como está en la hoja:
    /// el salario y las extras entran solos; a mano, sólo lo demás.
    fn q1_noviembre() -> Quincena {
        Quincena::new(
            UsuarioId::new(Uuid::from_u128(1)),
            pago_13(),
            vec![l("papá", 65_000), l("saldo para operar", 14_300)],
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
    fn el_salario_y_las_extras_del_pago_entran_solos() {
        let a = automaticos_13(f(2026, 11, 11));
        assert_eq!(a.salario(), Colones::new(232_225), "salario neto (P)");
        assert_eq!(
            a.extras(),
            Colones::new(103_404),
            "sólo la extra cuyo corte paga el 13 (E4)"
        );
        assert!(!a.llegaron(), "el 11 todavía no llega");
        assert!(
            automaticos_13(pago_13()).llegaron(),
            "el día de pago ya llegó"
        );
    }

    #[test]
    fn totales_y_sobra_iguales_a_la_hoja() {
        let b = q1_noviembre().balance(&automaticos_13(pago_13()));
        assert_eq!(b.otros_ingresos(), Colones::new(79_300), "papá + saldo");
        assert_eq!(b.total_ingresos(), Colones::new(414_929), "total ingresos");
        assert_eq!(b.total_pagos(), Colones::new(374_000), "total pagos");
        assert_eq!(b.sobra(), Colones::new(40_929), "sobra");
    }

    #[test]
    fn sin_presupuesto_igual_entra_el_salario() {
        let vacia = Quincena::new(
            UsuarioId::new(Uuid::from_u128(1)),
            pago_13(),
            vec![],
            vec![],
        );
        let b = vacia.balance(&automaticos_13(pago_13()));
        assert_eq!(
            b.total_ingresos(),
            Colones::new(335_629),
            "salario + extras"
        );
        assert_eq!(b.sobra(), Colones::new(335_629), "nada que pagar");
    }

    #[test]
    fn el_mes_suma_sus_quincenas() {
        let q1 = q1_noviembre().balance(&automaticos_13(pago_13()));
        let q2 = Quincena::new(
            UsuarioId::new(Uuid::from_u128(1)),
            f(2026, 11, 30),
            vec![],
            vec![l("comida", 100_000)],
        )
        .balance(&IngresosAutomaticos::de(
            &salarial(),
            &calendario(),
            &[extra(2, f(2026, 11, 5), 25_800)],
            f(2026, 11, 30),
            pago_13(),
        ));
        let mes: Balance = [q1, q2].into_iter().sum();
        assert_eq!(mes.salario(), Colones::new(464_450), "dos salarios");
        assert_eq!(
            mes.extras(),
            Colones::new(103_404 + 22_232),
            "extras de los dos pagos"
        );
        assert_eq!(mes.total_pagos(), Colones::new(474_000), "pagos de las dos");
        assert_eq!(mes.sobra(), q1.sobra() + q2.sobra(), "sobra del mes");
        assert_eq!(
            std::iter::empty().sum::<Balance>(),
            Balance::default(),
            "un mes sin quincenas"
        );
    }

    #[test]
    fn marcar_pagos_baja_lo_que_falta() {
        let mut q = q1_noviembre();
        q.marcar(Lado::Pago, 0, true).unwrap();
        let falta = q.balance(&automaticos_13(pago_13())).falta_por_pagar();
        assert_eq!(falta, Colones::new(270_000), "pagada la moto");
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

    #[test]
    fn cambiar_el_presupuesto_conserva_lo_marcado() {
        let mut q = q1_noviembre();
        q.marcar(Lado::Pago, 0, true).unwrap(); // moto
        q.marcar(Lado::Pago, 4, true).unwrap(); // internet
        q.marcar(Lado::Ingreso, 0, true).unwrap(); // papá
        let nueva = q.con_presupuesto(
            vec![l("papá", 70_000), l("venta", 5_000)],
            vec![
                l("moto", 125_000),
                l("comida", 100_000),
                l("refri", 30_000),
                l("internet", 23_000),
            ],
        );
        assert_eq!(nueva.pago(), pago_13(), "misma quincena");
        let marcas: Vec<bool> = nueva.pagos().iter().map(Linea::hecho).collect();
        assert_eq!(
            marcas,
            vec![true, false, false, true],
            "la moto y el internet siguen pagados aunque cambie el monto o la posición"
        );
        assert!(nueva.ingresos()[0].hecho(), "lo de papá sigue recibido");
        assert_eq!(
            nueva.balance(&automaticos_13(pago_13())).falta_por_pagar(),
            Colones::new(130_000),
            "comida + refri"
        );
    }

    #[test]
    fn un_concepto_repetido_hereda_en_orden() {
        let mut q = Quincena::new(
            UsuarioId::new(Uuid::from_u128(1)),
            pago_13(),
            vec![],
            vec![l("cuota", 10), l("cuota", 20)],
        );
        q.marcar(Lado::Pago, 0, true).unwrap();
        let nueva = q.con_presupuesto(vec![], vec![l("cuota", 10), l("cuota", 20), l("cuota", 30)]);
        let marcas: Vec<bool> = nueva.pagos().iter().map(Linea::hecho).collect();
        assert_eq!(
            marcas,
            vec![true, false, false],
            "sólo la primera estaba pagada"
        );
    }
}
