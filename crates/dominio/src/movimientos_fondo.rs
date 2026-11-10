//! Los movimientos del fondo de emergencia y el estado que resulta de ellos.
//!
//! El fondo no guarda un saldo: el saldo, lo que falta reponer y la cuota del
//! mes salen siempre de la lista de movimientos. Así no hay dos verdades que se
//! puedan desincronizar.
//!
//! Las reposiciones pagan primero el retiro más viejo. Como cada retiro se
//! repone con su recargo, el recargo vuelve al fondo y nadie se lo queda (F7).

use crate::dinero::Colones;
use crate::fondo::{Evaluacion, NivelEmergencia, PlanReposicion, SolicitudRetiro};
use crate::texto::Concepto;
use crate::usuario::UsuarioId;
use chrono::NaiveDate;
use thiserror::Error;
use uuid::Uuid;

/// Identificador de un movimiento del fondo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MovimientoFondoId(Uuid);

impl MovimientoFondoId {
    /// Envuelve un UUID ya generado.
    pub const fn new(id: Uuid) -> Self {
        Self(id)
    }

    /// El UUID.
    pub const fn uuid(self) -> Uuid {
        self.0
    }
}

/// Qué clase de movimiento es.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Movimiento {
    /// Plata que entra para hacer crecer el fondo.
    Aporte,
    /// Una emergencia: sale plata y queda un plan para reponerla.
    Retiro(PlanReposicion),
    /// Plata que entra para pagar lo que se sacó.
    Reposicion,
}

/// Por qué no se puede registrar un movimiento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ErrorFondo {
    /// El monto debe ser mayor que cero.
    #[error("el monto debe ser mayor que cero")]
    MontoInvalido,
    /// No hay nada que reponer: lo que entra es un aporte.
    #[error("no hay nada pendiente de reponer; anotalo como aporte")]
    SinReposicionPendiente,
    /// Se quiere reponer más de lo que falta.
    #[error("sólo faltan {por_reponer} por reponer; el resto es un aporte")]
    ReposicionDeMas {
        /// Lo que de verdad falta.
        por_reponer: Colones,
    },
}

/// Un movimiento del fondo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovimientoFondo {
    id: MovimientoFondoId,
    usuario: UsuarioId,
    fecha: NaiveDate,
    concepto: Concepto,
    monto: Colones,
    movimiento: Movimiento,
}

impl MovimientoFondo {
    /// Un aporte al fondo.
    pub fn aporte(
        id: MovimientoFondoId,
        usuario: UsuarioId,
        fecha: NaiveDate,
        concepto: Concepto,
        monto: Colones,
    ) -> Result<Self, ErrorFondo> {
        if !monto.es_positivo() {
            return Err(ErrorFondo::MontoInvalido);
        }
        Ok(Self::reconstruir(
            id,
            usuario,
            fecha,
            concepto,
            monto,
            Movimiento::Aporte,
        ))
    }

    /// Un retiro ya aprobado por [`crate::fondo::decidir_retiro`]: el plan
    /// trae el monto sacado.
    pub const fn retiro(
        id: MovimientoFondoId,
        usuario: UsuarioId,
        fecha: NaiveDate,
        concepto: Concepto,
        plan: PlanReposicion,
    ) -> Self {
        Self::reconstruir(
            id,
            usuario,
            fecha,
            concepto,
            plan.sacado,
            Movimiento::Retiro(plan),
        )
    }

    /// Una reposición. Tiene que haber algo pendiente y no puede pasarse de lo
    /// que falta: lo que sobre se anota aparte como aporte.
    pub fn reposicion(
        id: MovimientoFondoId,
        usuario: UsuarioId,
        fecha: NaiveDate,
        concepto: Concepto,
        monto: Colones,
        estado: &EstadoFondo,
    ) -> Result<Self, ErrorFondo> {
        if !monto.es_positivo() {
            return Err(ErrorFondo::MontoInvalido);
        }
        if !estado.reposicion_pendiente() {
            return Err(ErrorFondo::SinReposicionPendiente);
        }
        if monto > estado.por_reponer {
            return Err(ErrorFondo::ReposicionDeMas {
                por_reponer: estado.por_reponer,
            });
        }
        Ok(Self::reconstruir(
            id,
            usuario,
            fecha,
            concepto,
            monto,
            Movimiento::Reposicion,
        ))
    }

    /// Reconstruye un movimiento ya guardado (para los repositorios).
    pub const fn reconstruir(
        id: MovimientoFondoId,
        usuario: UsuarioId,
        fecha: NaiveDate,
        concepto: Concepto,
        monto: Colones,
        movimiento: Movimiento,
    ) -> Self {
        Self {
            id,
            usuario,
            fecha,
            concepto,
            monto,
            movimiento,
        }
    }

    /// Cuánto cambia el saldo: positivo si entra, negativo si sale.
    pub fn efecto(&self) -> Colones {
        match self.movimiento {
            Movimiento::Aporte | Movimiento::Reposicion => self.monto,
            Movimiento::Retiro(_) => -self.monto,
        }
    }

    /// Identificador.
    pub const fn id(&self) -> MovimientoFondoId {
        self.id
    }

    /// Dueño.
    pub const fn usuario(&self) -> UsuarioId {
        self.usuario
    }

    /// Día del movimiento.
    pub const fn fecha(&self) -> NaiveDate {
        self.fecha
    }

    /// Concepto.
    pub const fn concepto(&self) -> &Concepto {
        &self.concepto
    }

    /// Monto (siempre positivo; el sentido lo da [`Self::efecto`]).
    pub const fn monto(&self) -> Colones {
        self.monto
    }

    /// Clase de movimiento.
    pub const fn movimiento(&self) -> Movimiento {
        self.movimiento
    }
}

/// Cómo está el fondo según sus movimientos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EstadoFondo {
    /// Lo que hay.
    pub saldo: Colones,
    /// Lo que falta reponer de los retiros, con su recargo.
    pub por_reponer: Colones,
    /// Lo que toca reponer este mes: la cuota de cada retiro no repuesto,
    /// sin pasarse de lo que le falta.
    pub cuota_mensual: Colones,
}

impl EstadoFondo {
    /// Calcula el estado a partir de los movimientos, en orden cronológico.
    pub fn de(movimientos: &[MovimientoFondo]) -> Self {
        let mut saldo = Colones::CERO;
        // Lo que le falta a cada retiro y su cuota, del más viejo al más nuevo.
        let mut deudas: Vec<(Colones, Colones)> = Vec::new();
        for m in movimientos {
            saldo += m.efecto();
            match m.movimiento {
                Movimiento::Aporte => {}
                Movimiento::Retiro(plan) => deudas.push((plan.a_reponer, plan.cuota_mensual)),
                Movimiento::Reposicion => {
                    let mut resto = m.monto;
                    for (falta, _) in &mut deudas {
                        let pago = if resto < *falta { resto } else { *falta };
                        *falta -= pago;
                        resto -= pago;
                    }
                }
            }
        }
        let por_reponer = deudas.iter().map(|(falta, _)| *falta).sum();
        let cuota_mensual = deudas
            .iter()
            .map(|&(falta, cuota)| if cuota < falta { cuota } else { falta })
            .sum();
        Self {
            saldo,
            por_reponer,
            cuota_mensual,
        }
    }

    /// `true` si queda algo por reponer.
    pub const fn reposicion_pendiente(&self) -> bool {
        self.por_reponer.es_positivo()
    }

    /// Arma la solicitud para [`crate::fondo::decidir_retiro`] con el saldo y
    /// lo pendiente de este estado.
    pub const fn solicitud(
        &self,
        monto: Colones,
        nivel: NivelEmergencia,
        evaluacion: Evaluacion,
    ) -> SolicitudRetiro {
        SolicitudRetiro {
            monto,
            nivel,
            evaluacion,
            saldo_fondo: self.saldo,
            reposicion_pendiente: self.reposicion_pendiente(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ErrorFondo, EstadoFondo, MovimientoFondo, MovimientoFondoId};
    use crate::dinero::Colones;
    use crate::fondo::{Evaluacion, NivelEmergencia, RechazoRetiro, decidir_retiro};
    use crate::texto::Concepto;
    use crate::usuario::UsuarioId;
    use chrono::NaiveDate;
    use uuid::Uuid;

    const TRES_SI: Evaluacion = Evaluacion {
        inesperado: true,
        necesario: true,
        urgente: true,
    };
    const YO: UsuarioId = UsuarioId::new(Uuid::from_u128(1));

    fn id(n: u128) -> MovimientoFondoId {
        MovimientoFondoId::new(Uuid::from_u128(n))
    }

    fn dia(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, d).unwrap()
    }

    fn c(t: &str) -> Concepto {
        Concepto::new(t).unwrap()
    }

    /// Saca del fondo como lo hará el caso de uso: el dominio decide.
    fn sacar(
        movs: &mut Vec<MovimientoFondo>,
        n: u128,
        monto: i64,
        nivel: NivelEmergencia,
    ) -> Result<(), RechazoRetiro> {
        let estado = EstadoFondo::de(movs);
        let plan = decidir_retiro(estado.solicitud(Colones::new(monto), nivel, TRES_SI))?;
        movs.push(MovimientoFondo::retiro(id(n), YO, dia(10), c("x"), plan));
        Ok(())
    }

    fn reponer(movs: &mut Vec<MovimientoFondo>, n: u128, monto: i64) -> Result<(), ErrorFondo> {
        let estado = EstadoFondo::de(movs);
        let m = MovimientoFondo::reposicion(
            id(n),
            YO,
            dia(13),
            c("reposición"),
            Colones::new(monto),
            &estado,
        )?;
        movs.push(m);
        Ok(())
    }

    fn base() -> Vec<MovimientoFondo> {
        vec![
            MovimientoFondo::aporte(id(1), YO, dia(1), c("saldo inicial"), Colones::new(31_619))
                .unwrap(),
        ]
    }

    #[test]
    fn un_retiro_de_trabajo_se_repone_con_su_recargo() {
        let mut movs = base();
        sacar(&mut movs, 2, 20_000, NivelEmergencia::Trabajo).unwrap();
        assert_eq!(
            EstadoFondo::de(&movs),
            EstadoFondo {
                saldo: Colones::new(11_619),
                por_reponer: Colones::new(21_000),
                cuota_mensual: Colones::new(7_000),
            },
            "sale 20 000; se reponen 21 000 en 3 cuotas de 7 000"
        );

        reponer(&mut movs, 3, 7_000).unwrap();
        reponer(&mut movs, 4, 7_000).unwrap();
        assert_eq!(
            EstadoFondo::de(&movs),
            EstadoFondo {
                saldo: Colones::new(25_619),
                por_reponer: Colones::new(7_000),
                cuota_mensual: Colones::new(7_000),
            },
            "dos cuotas pagadas"
        );

        assert_eq!(
            reponer(&mut movs, 5, 8_000),
            Err(ErrorFondo::ReposicionDeMas {
                por_reponer: Colones::new(7_000)
            }),
            "no se repone de más"
        );
        reponer(&mut movs, 6, 7_000).unwrap();
        assert_eq!(
            EstadoFondo::de(&movs),
            EstadoFondo {
                saldo: Colones::new(32_619),
                por_reponer: Colones::CERO,
                cuota_mensual: Colones::CERO,
            },
            "repuesto: el fondo ganó los 1 000 del recargo (F7)"
        );
        assert_eq!(
            reponer(&mut movs, 7, 1_000),
            Err(ErrorFondo::SinReposicionPendiente),
            "sin nada pendiente, lo que entra es un aporte"
        );
    }

    #[test]
    fn con_algo_pendiente_solo_se_saca_para_lo_vital() {
        let mut movs = base();
        sacar(&mut movs, 2, 20_000, NivelEmergencia::Trabajo).unwrap();
        assert_eq!(
            sacar(&mut movs, 3, 1_000, NivelEmergencia::Descuido),
            Err(RechazoRetiro::ReposicionPendiente),
            "F4: una a la vez"
        );
        sacar(&mut movs, 4, 5_000, NivelEmergencia::Vital).unwrap();
        assert_eq!(
            EstadoFondo::de(&movs),
            EstadoFondo {
                saldo: Colones::new(6_619),
                por_reponer: Colones::new(26_000),
                cuota_mensual: Colones::new(7_834),
            },
            "lo vital entra igual: 5 000 sin recargo en 6 cuotas de 834"
        );
    }

    #[test]
    fn la_reposicion_paga_primero_el_retiro_mas_viejo() {
        let mut movs = base();
        sacar(&mut movs, 2, 20_000, NivelEmergencia::Trabajo).unwrap();
        sacar(&mut movs, 3, 6_000, NivelEmergencia::Vital).unwrap();
        reponer(&mut movs, 4, 18_000).unwrap();
        assert_eq!(
            EstadoFondo::de(&movs),
            EstadoFondo {
                saldo: Colones::new(23_619),
                por_reponer: Colones::new(9_000),
                cuota_mensual: Colones::new(4_000),
            },
            "al de trabajo le faltan 3 000 (cuota 3 000) y al vital 6 000 (cuota 1 000)"
        );
    }

    #[test]
    fn no_se_saca_mas_de_lo_que_hay() {
        let mut movs = base();
        assert_eq!(
            sacar(&mut movs, 2, 40_000, NivelEmergencia::Vital),
            Err(RechazoRetiro::FondoInsuficiente {
                disponible: Colones::new(31_619),
                pedido: Colones::new(40_000),
            }),
            "F5 con el saldo calculado de los movimientos"
        );
    }

    #[test]
    fn montos_de_cero_se_rechazan() {
        assert_eq!(
            MovimientoFondo::aporte(id(1), YO, dia(1), c("x"), Colones::CERO),
            Err(ErrorFondo::MontoInvalido),
            "aporte de cero"
        );
        let estado = EstadoFondo::de(&base());
        assert_eq!(
            MovimientoFondo::reposicion(id(2), YO, dia(1), c("x"), Colones::new(-5), &estado),
            Err(ErrorFondo::MontoInvalido),
            "reposición negativa"
        );
    }
}
