//! Horas extra y feriados trabajados.

use crate::calendario::CalendarioCortes;
use crate::dinero::Colones;
use crate::perfil::{PerfilSalarial, TarifasExtra};
use crate::usuario::UsuarioId;
use chrono::NaiveDate;
use thiserror::Error;
use uuid::Uuid;

/// Identificador de una extra registrada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtraId(Uuid);

impl ExtraId {
    /// Envuelve un UUID ya generado.
    pub const fn new(id: Uuid) -> Self {
        Self(id)
    }

    /// El UUID.
    pub const fn uuid(self) -> Uuid {
        self.0
    }
}

/// Tipos de extra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoExtra {
    /// Turno extra de noche.
    Nocturna,
    /// Turno extra de día.
    Diurna,
    /// Extra especial, mejor pagada y esporádica.
    Especial,
    /// Feriado trabajado.
    Feriado,
    /// Cualquier otro monto: se escribe a mano.
    Otra,
}

/// Por qué no se puede registrar una extra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ErrorExtra {
    /// Una extra `Otra` necesita el monto bruto escrito a mano.
    #[error("una extra de tipo OTRA necesita el monto bruto")]
    FaltaMonto,
    /// El monto escrito a mano debe ser mayor que cero.
    #[error("el monto bruto debe ser mayor que cero")]
    MontoInvalido,
}

/// Una extra trabajada: cuándo, de qué tipo y cuánto paga bruto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extra {
    id: ExtraId,
    usuario: UsuarioId,
    fecha: NaiveDate,
    tipo: TipoExtra,
    bruto: Colones,
}

impl Extra {
    /// Registra una extra. El bruto sale de la tarifa del usuario, salvo en
    /// `Otra`, que lo trae escrito; si se escribe un monto en un tipo con
    /// tarifa, manda lo escrito (el comprobante puede traer otra cifra).
    pub fn new(
        id: ExtraId,
        usuario: UsuarioId,
        fecha: NaiveDate,
        tipo: TipoExtra,
        tarifas: &TarifasExtra,
        monto_escrito: Option<Colones>,
    ) -> Result<Self, ErrorExtra> {
        let bruto = match (monto_escrito, tarifas.tarifa(tipo)) {
            (Some(monto), _) if !monto.es_positivo() => return Err(ErrorExtra::MontoInvalido),
            (Some(monto), _) => monto,
            (None, Some(tarifa)) => tarifa,
            (None, None) => return Err(ErrorExtra::FaltaMonto),
        };
        Ok(Self {
            id,
            usuario,
            fecha,
            tipo,
            bruto,
        })
    }

    /// Reconstruye una extra ya guardada (para los repositorios).
    pub const fn reconstruir(
        id: ExtraId,
        usuario: UsuarioId,
        fecha: NaiveDate,
        tipo: TipoExtra,
        bruto: Colones,
    ) -> Self {
        Self {
            id,
            usuario,
            fecha,
            tipo,
            bruto,
        }
    }

    /// Lo que queda neto según el perfil del usuario.
    pub fn neto(&self, perfil: &PerfilSalarial) -> Colones {
        perfil.neto_de(self.bruto)
    }

    /// La fecha en que se paga, según el corte en que cae. `None` si el
    /// calendario todavía no cubre esa fecha.
    pub fn fecha_pago(&self, calendario: &CalendarioCortes) -> Option<NaiveDate> {
        calendario
            .corte_de(self.fecha)
            .map(crate::calendario::Corte::pago)
    }

    /// Identificador.
    pub const fn id(&self) -> ExtraId {
        self.id
    }

    /// Dueño.
    pub const fn usuario(&self) -> UsuarioId {
        self.usuario
    }

    /// Día trabajado.
    pub const fn fecha(&self) -> NaiveDate {
        self.fecha
    }

    /// Tipo.
    pub const fn tipo(&self) -> TipoExtra {
        self.tipo
    }

    /// Monto bruto.
    pub const fn bruto(&self) -> Colones {
        self.bruto
    }
}

/// Lo neto de las extras que se pagan en una fecha dada.
pub fn neto_en_pago(
    extras: &[Extra],
    pago: NaiveDate,
    perfil: &PerfilSalarial,
    calendario: &CalendarioCortes,
) -> Colones {
    extras
        .iter()
        .filter(|e| e.fecha_pago(calendario) == Some(pago))
        .map(|e| e.neto(perfil))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{ErrorExtra, Extra, ExtraId, TipoExtra, neto_en_pago};
    use crate::calendario::{CalendarioCortes, Corte};
    use crate::dinero::Colones;
    use crate::perfil::{PerfilSalarial, TarifasExtra};
    use crate::proporcion::Proporcion;
    use crate::usuario::UsuarioId;
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn f(a: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(a, m, d).unwrap()
    }

    fn tarifas() -> TarifasExtra {
        TarifasExtra::new(
            Colones::new(25_800),
            Colones::new(19_800),
            Colones::new(40_000),
            Colones::new(19_500),
        )
        .unwrap()
    }

    fn perfil() -> PerfilSalarial {
        PerfilSalarial::new(
            Colones::new(293_000),
            Proporcion::desde_diezmilesimas(8617),
            Colones::new(20_253),
        )
        .unwrap()
    }

    fn extra(n: u128, fecha: NaiveDate, tipo: TipoExtra, monto: Option<Colones>) -> Extra {
        Extra::new(
            ExtraId::new(Uuid::from_u128(n)),
            UsuarioId::new(Uuid::from_u128(1)),
            fecha,
            tipo,
            &tarifas(),
            monto,
        )
        .unwrap()
    }

    #[test]
    fn el_bruto_sale_de_la_tarifa_o_de_lo_escrito() {
        let noche = extra(1, f(2026, 11, 11), TipoExtra::Nocturna, None);
        assert_eq!(noche.bruto(), Colones::new(25_800), "tarifa nocturna");
        assert_eq!(noche.neto(&perfil()), Colones::new(22_232), "neto nocturna");
        let otra = extra(
            2,
            f(2026, 10, 31),
            TipoExtra::Otra,
            Some(Colones::new(120_000)),
        );
        assert_eq!(otra.bruto(), Colones::new(120_000), "monto escrito");
    }

    #[test]
    fn otra_sin_monto_o_monto_cero_se_rechaza() {
        let crear = |monto| {
            Extra::new(
                ExtraId::new(Uuid::from_u128(9)),
                UsuarioId::new(Uuid::from_u128(1)),
                f(2026, 11, 11),
                TipoExtra::Otra,
                &tarifas(),
                monto,
            )
        };
        assert_eq!(crear(None), Err(ErrorExtra::FaltaMonto), "sin monto");
        assert_eq!(
            crear(Some(Colones::CERO)),
            Err(ErrorExtra::MontoInvalido),
            "monto cero"
        );
    }

    #[test]
    fn neto_de_las_extras_del_pago_del_30_de_noviembre() {
        let cal = CalendarioCortes::new(vec![
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
        ])
        .unwrap();
        let extras = [
            extra(
                1,
                f(2026, 10, 31),
                TipoExtra::Otra,
                Some(Colones::new(120_000)),
            ),
            extra(2, f(2026, 11, 5), TipoExtra::Especial, None),
            extra(3, f(2026, 11, 6), TipoExtra::Especial, None),
            extra(4, f(2026, 11, 11), TipoExtra::Nocturna, None),
            extra(5, f(2026, 11, 14), TipoExtra::Nocturna, None),
        ];
        // 34 468 + 34 468 + 22 232: los mismos números que la hoja.
        assert_eq!(
            neto_en_pago(&extras, f(2026, 11, 30), &perfil(), &cal),
            Colones::new(91_168),
            "extras netas del 30 de noviembre"
        );
        assert_eq!(
            neto_en_pago(&extras, f(2026, 11, 13), &perfil(), &cal),
            Colones::new(103_404),
            "extras netas del 13 de noviembre"
        );
    }
}
