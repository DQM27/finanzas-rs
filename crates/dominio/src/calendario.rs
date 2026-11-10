//! Calendario de cortes y pagos del patrono.
//!
//! Las extras no se pagan "una quincena después" sino según el corte en que
//! caen: cada corte tiene un rango de fechas trabajadas, una fecha límite para
//! reportar y una fecha de pago. Los cortes cambian de quincena en quincena,
//! así que vienen del calendario oficial y no de una regla fija.

use chrono::NaiveDate;
use thiserror::Error;

/// Un corte: lo trabajado entre `inicio` y `fin` (incluidos) se paga en `pago`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Corte {
    inicio: NaiveDate,
    fin: NaiveDate,
    reportar_antes: NaiveDate,
    pago: NaiveDate,
}

/// Por qué un corte o un calendario no es válido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ErrorCalendario {
    /// El corte termina antes de empezar.
    #[error("el corte que empieza el {0} termina antes de empezar")]
    RangoInvertido(NaiveDate),
    /// La fecha límite para reportar es anterior al fin del corte.
    #[error("el corte que termina el {0} pide reportar antes de terminar")]
    ReporteAntesDelFin(NaiveDate),
    /// El pago llega antes de la fecha límite para reportar.
    #[error("el corte que termina el {0} se paga antes de cerrar el reporte")]
    PagoAntesDelReporte(NaiveDate),
    /// El calendario no tiene cortes.
    #[error("el calendario no tiene cortes")]
    Vacio,
    /// Hay un hueco o un traslape entre dos cortes seguidos.
    #[error("después del corte que termina el {0} el siguiente no empieza al día siguiente")]
    NoContiguo(NaiveDate),
}

impl Corte {
    /// Crea y valida un corte.
    pub fn new(
        inicio: NaiveDate,
        fin: NaiveDate,
        reportar_antes: NaiveDate,
        pago: NaiveDate,
    ) -> Result<Self, ErrorCalendario> {
        if fin < inicio {
            return Err(ErrorCalendario::RangoInvertido(inicio));
        }
        if reportar_antes < fin {
            return Err(ErrorCalendario::ReporteAntesDelFin(fin));
        }
        if pago < reportar_antes {
            return Err(ErrorCalendario::PagoAntesDelReporte(fin));
        }
        Ok(Self {
            inicio,
            fin,
            reportar_antes,
            pago,
        })
    }

    /// `true` si la fecha cae dentro del corte.
    pub fn contiene(&self, fecha: NaiveDate) -> bool {
        self.inicio <= fecha && fecha <= self.fin
    }

    /// Primer día del corte.
    pub const fn inicio(&self) -> NaiveDate {
        self.inicio
    }

    /// Último día del corte.
    pub const fn fin(&self) -> NaiveDate {
        self.fin
    }

    /// Fecha límite para reportar lo trabajado en el corte.
    pub const fn reportar_antes(&self) -> NaiveDate {
        self.reportar_antes
    }

    /// Fecha de pago.
    pub const fn pago(&self) -> NaiveDate {
        self.pago
    }
}

/// Un calendario de cortes seguidos, sin huecos ni traslapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarioCortes {
    cortes: Vec<Corte>,
}

impl CalendarioCortes {
    /// Ordena y valida los cortes: cada uno empieza el día después del anterior.
    pub fn new(mut cortes: Vec<Corte>) -> Result<Self, ErrorCalendario> {
        if cortes.is_empty() {
            return Err(ErrorCalendario::Vacio);
        }
        cortes.sort();
        for par in cortes.windows(2) {
            if let [anterior, siguiente] = par
                && anterior.fin.succ_opt() != Some(siguiente.inicio)
            {
                return Err(ErrorCalendario::NoContiguo(anterior.fin));
            }
        }
        Ok(Self { cortes })
    }

    /// El corte en que cae una fecha trabajada.
    pub fn corte_de(&self, fecha: NaiveDate) -> Option<&Corte> {
        self.cortes.iter().find(|c| c.contiene(fecha))
    }

    /// El próximo pago desde `hoy` (incluido hoy).
    pub fn proximo_pago(&self, hoy: NaiveDate) -> Option<&Corte> {
        self.cortes
            .iter()
            .filter(|c| c.pago >= hoy)
            .min_by_key(|c| c.pago)
    }

    /// El último pago antes de `hoy` o el de hoy mismo.
    pub fn ultimo_pago(&self, hoy: NaiveDate) -> Option<&Corte> {
        self.cortes
            .iter()
            .filter(|c| c.pago <= hoy)
            .max_by_key(|c| c.pago)
    }

    /// Días que faltan para el próximo pago (0 si es hoy).
    pub fn dias_para_pago(&self, hoy: NaiveDate) -> Option<i64> {
        self.proximo_pago(hoy).map(|c| (c.pago - hoy).num_days())
    }

    /// El periodo de gasto en curso: desde el último día de pago hasta el día
    /// anterior al siguiente. La caja chica y la gasolina se reinician con
    /// cada pago.
    pub fn periodo_de_gasto(&self, hoy: NaiveDate) -> Option<(NaiveDate, NaiveDate)> {
        let desde = self.ultimo_pago(hoy)?.pago;
        let hasta = self
            .cortes
            .iter()
            .filter(|c| c.pago > hoy)
            .min_by_key(|c| c.pago)?
            .pago
            .pred_opt()?;
        Some((desde, hasta))
    }

    /// Los cortes, ordenados.
    pub fn cortes(&self) -> &[Corte] {
        &self.cortes
    }
}

#[cfg(test)]
mod tests {
    use super::{CalendarioCortes, Corte, ErrorCalendario};
    use chrono::NaiveDate;

    fn f(a: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(a, m, d).unwrap()
    }

    /// Calendario oficial de octubre a diciembre de 2026.
    fn fin_de_2026() -> CalendarioCortes {
        CalendarioCortes::new(vec![
            Corte::new(
                f(2026, 10, 2),
                f(2026, 10, 16),
                f(2026, 10, 21),
                f(2026, 10, 30),
            )
            .unwrap(),
            Corte::new(
                f(2026, 10, 17),
                f(2026, 11, 1),
                f(2026, 11, 4),
                f(2026, 11, 13),
            )
            .unwrap(),
            Corte::new(
                f(2026, 11, 2),
                f(2026, 11, 13),
                f(2026, 11, 17),
                f(2026, 11, 30),
            )
            .unwrap(),
            Corte::new(
                f(2026, 11, 14),
                f(2026, 12, 1),
                f(2026, 12, 7),
                f(2026, 12, 15),
            )
            .unwrap(),
            Corte::new(
                f(2026, 12, 2),
                f(2026, 12, 15),
                f(2026, 12, 18),
                f(2026, 12, 30),
            )
            .unwrap(),
        ])
        .unwrap()
    }

    #[test]
    fn cada_extra_cae_en_el_pago_de_su_corte() {
        let cal = fin_de_2026();
        let pago = |fecha| cal.corte_de(fecha).map(Corte::pago);
        assert_eq!(
            pago(f(2026, 10, 31)),
            Some(f(2026, 11, 13)),
            "31 oct se paga el 13 nov"
        );
        assert_eq!(
            pago(f(2026, 11, 11)),
            Some(f(2026, 11, 30)),
            "11 nov se paga el 30 nov"
        );
        assert_eq!(
            pago(f(2026, 11, 13)),
            Some(f(2026, 11, 30)),
            "13 nov aún entra el 30"
        );
        assert_eq!(
            pago(f(2026, 11, 14)),
            Some(f(2026, 12, 15)),
            "14 nov ya es del 15 dic"
        );
        assert_eq!(
            pago(f(2026, 11, 17)),
            Some(f(2026, 12, 15)),
            "17 nov es del 15 dic"
        );
        assert_eq!(pago(f(2027, 3, 1)), None, "fuera del calendario");
    }

    #[test]
    fn proximo_pago_y_dias_que_faltan() {
        let cal = fin_de_2026();
        let hoy = f(2026, 11, 10);
        assert_eq!(
            cal.proximo_pago(hoy).map(Corte::pago),
            Some(f(2026, 11, 13)),
            "viernes 13"
        );
        assert_eq!(cal.dias_para_pago(hoy), Some(3), "faltan 3 días");
        assert_eq!(
            cal.dias_para_pago(f(2026, 11, 13)),
            Some(0),
            "el día de pago"
        );
    }

    #[test]
    fn el_periodo_de_gasto_va_de_pago_a_pago() {
        let cal = fin_de_2026();
        assert_eq!(
            cal.periodo_de_gasto(f(2026, 11, 10)),
            Some((f(2026, 10, 30), f(2026, 11, 12))),
            "del 30 oct al 12 nov"
        );
        assert_eq!(
            cal.periodo_de_gasto(f(2026, 11, 13)),
            Some((f(2026, 11, 13), f(2026, 11, 29))),
            "el día de pago arranca el periodo nuevo"
        );
    }

    #[test]
    fn rechaza_cortes_mal_armados() {
        assert_eq!(
            Corte::new(
                f(2026, 11, 2),
                f(2026, 11, 1),
                f(2026, 11, 4),
                f(2026, 11, 13)
            ),
            Err(ErrorCalendario::RangoInvertido(f(2026, 11, 2))),
            "fin antes del inicio"
        );
        assert_eq!(
            Corte::new(
                f(2026, 11, 2),
                f(2026, 11, 13),
                f(2026, 11, 20),
                f(2026, 11, 15)
            ),
            Err(ErrorCalendario::PagoAntesDelReporte(f(2026, 11, 13))),
            "pago antes del reporte"
        );
        let hueco = CalendarioCortes::new(vec![
            Corte::new(
                f(2026, 11, 2),
                f(2026, 11, 13),
                f(2026, 11, 17),
                f(2026, 11, 30),
            )
            .unwrap(),
            Corte::new(
                f(2026, 11, 15),
                f(2026, 12, 1),
                f(2026, 12, 7),
                f(2026, 12, 15),
            )
            .unwrap(),
        ]);
        assert_eq!(
            hueco,
            Err(ErrorCalendario::NoContiguo(f(2026, 11, 13))),
            "falta el 14 nov"
        );
        assert_eq!(
            CalendarioCortes::new(vec![]),
            Err(ErrorCalendario::Vacio),
            "vacío"
        );
    }
}
