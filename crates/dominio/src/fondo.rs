//! Fondo de emergencia: cuándo se puede sacar y cómo se repone.
//!
//! Para sacar tienen que ser tres «sí»: inesperado, necesario y urgente. El
//! nivel decide el recargo y el plazo de reposición; el recargo no se lo queda
//! nadie, vuelve al fondo. Mientras haya una reposición pendiente no se saca
//! otra vez, salvo una emergencia vital.

pub mod movimientos;

use crate::base::dinero::Colones;
use crate::base::proporcion::Proporcion;
use thiserror::Error;

/// Las tres preguntas antes de sacar del fondo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Evaluacion {
    /// No se sabía ni se podía planear.
    pub inesperado: bool,
    /// Es de salud, trabajo o techo.
    pub necesario: bool,
    /// No puede esperar a la próxima quincena.
    pub urgente: bool,
}

impl Evaluacion {
    /// `true` sólo si son los tres «sí».
    pub const fn es_emergencia(&self) -> bool {
        self.inesperado && self.necesario && self.urgente
    }
}

/// Qué tan grave es y cuánta culpa es propia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NivelEmergencia {
    /// Salud, accidente, perder la casa. Sin recargo.
    Vital,
    /// Algo sin lo que no se puede trabajar (la moto varada).
    Trabajo,
    /// Faltó para el mes por gastar de más o no planear.
    Descuido,
    /// No es emergencia (gustos, regalos, prestar). No debería sacarse; si se
    /// saca igual, queda anotado con el recargo más alto.
    Falta,
}

impl NivelEmergencia {
    /// Recargo sobre lo sacado.
    pub const fn recargo(self) -> Proporcion {
        match self {
            Self::Vital => Proporcion::CERO,
            Self::Trabajo => Proporcion::porcentaje(5),
            Self::Descuido => Proporcion::porcentaje(10),
            Self::Falta => Proporcion::porcentaje(20),
        }
    }

    /// Meses para reponer.
    pub const fn meses_para_reponer(self) -> u8 {
        match self {
            Self::Vital => 6,
            Self::Trabajo => 3,
            Self::Descuido | Self::Falta => 2,
        }
    }
}

/// Por qué no se puede sacar del fondo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RechazoRetiro {
    /// El monto debe ser mayor que cero.
    #[error("el monto a sacar debe ser mayor que cero")]
    MontoInvalido,
    /// No son los tres «sí».
    #[error("no es una emergencia: tienen que ser inesperado, necesario y urgente")]
    NoEsEmergencia,
    /// Ya hay una reposición pendiente (sólo una emergencia vital la salta).
    #[error("todavía hay una reposición pendiente")]
    ReposicionPendiente,
    /// El fondo no tiene esa plata.
    #[error("el fondo tiene {disponible} y se quieren sacar {pedido}")]
    FondoInsuficiente {
        /// Lo que hay en el fondo.
        disponible: Colones,
        /// Lo que se quiere sacar.
        pedido: Colones,
    },
}

/// Cómo se repone un retiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanReposicion {
    /// El nivel con que se sacó.
    pub nivel: NivelEmergencia,
    /// Lo que salió del fondo.
    pub sacado: Colones,
    /// Lo que hay que devolver al fondo (con recargo).
    pub a_reponer: Colones,
    /// En cuántos meses.
    pub meses: u8,
    /// Cuánto por mes, redondeado hacia arriba para no quedar debiendo.
    pub cuota_mensual: Colones,
}

/// Lo que hace falta para decidir un retiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolicitudRetiro {
    /// Cuánto se quiere sacar.
    pub monto: Colones,
    /// El nivel declarado.
    pub nivel: NivelEmergencia,
    /// Las tres preguntas.
    pub evaluacion: Evaluacion,
    /// Lo que hay hoy en el fondo.
    pub saldo_fondo: Colones,
    /// Si hay una reposición sin terminar.
    pub reposicion_pendiente: bool,
}

/// Decide si se puede sacar y, si sí, cómo se repone.
pub fn decidir_retiro(s: SolicitudRetiro) -> Result<PlanReposicion, RechazoRetiro> {
    if !s.monto.es_positivo() {
        return Err(RechazoRetiro::MontoInvalido);
    }
    if s.nivel != NivelEmergencia::Falta && !s.evaluacion.es_emergencia() {
        return Err(RechazoRetiro::NoEsEmergencia);
    }
    if s.reposicion_pendiente && s.nivel != NivelEmergencia::Vital {
        return Err(RechazoRetiro::ReposicionPendiente);
    }
    if s.monto > s.saldo_fondo {
        return Err(RechazoRetiro::FondoInsuficiente {
            disponible: s.saldo_fondo,
            pedido: s.monto,
        });
    }
    let a_reponer = s.nivel.recargo().con_recargo(s.monto);
    let meses = s.nivel.meses_para_reponer();
    let divisor = i64::from(meses);
    let cuota_mensual = Colones::new((a_reponer.valor() + divisor - 1) / divisor);
    Ok(PlanReposicion {
        nivel: s.nivel,
        sacado: s.monto,
        a_reponer,
        meses,
        cuota_mensual,
    })
}

#[cfg(test)]
mod tests {
    use super::{Evaluacion, NivelEmergencia, RechazoRetiro, SolicitudRetiro, decidir_retiro};
    use crate::base::dinero::Colones;

    const TRES_SI: Evaluacion = Evaluacion {
        inesperado: true,
        necesario: true,
        urgente: true,
    };

    fn pedir(monto: i64, nivel: NivelEmergencia) -> SolicitudRetiro {
        SolicitudRetiro {
            monto: Colones::new(monto),
            nivel,
            evaluacion: TRES_SI,
            saldo_fondo: Colones::new(500_000),
            reposicion_pendiente: false,
        }
    }

    #[test]
    fn los_ejemplos_de_las_reglas() {
        let vital = decidir_retiro(pedir(60_000, NivelEmergencia::Vital)).unwrap();
        assert_eq!(
            (vital.a_reponer, vital.cuota_mensual),
            (Colones::new(60_000), Colones::new(10_000)),
            "hijo enfermo: 10 000 × 6"
        );
        let cadena = decidir_retiro(pedir(45_000, NivelEmergencia::Trabajo)).unwrap();
        assert_eq!(
            (cadena.a_reponer, cadena.cuota_mensual),
            (Colones::new(47_250), Colones::new(15_750)),
            "cadena de la moto: 15 750 × 3"
        );
        let casa = decidir_retiro(pedir(30_000, NivelEmergencia::Descuido)).unwrap();
        assert_eq!(
            (casa.a_reponer, casa.cuota_mensual),
            (Colones::new(33_000), Colones::new(16_500)),
            "faltó para la casa: 16 500 × 2"
        );
    }

    #[test]
    fn una_falta_se_puede_anotar_pero_cuesta_veinte_por_ciento() {
        let mut celular = pedir(80_000, NivelEmergencia::Falta);
        celular.evaluacion = Evaluacion {
            inesperado: false,
            necesario: false,
            urgente: false,
        };
        let plan = decidir_retiro(celular).unwrap();
        assert_eq!(
            plan.a_reponer,
            Colones::new(96_000),
            "celular en oferta: 96 000"
        );
    }

    #[test]
    fn sin_los_tres_si_no_se_saca() {
        let mut s = pedir(45_000, NivelEmergencia::Trabajo);
        s.evaluacion.urgente = false;
        assert_eq!(
            decidir_retiro(s),
            Err(RechazoRetiro::NoEsEmergencia),
            "no es urgente"
        );
    }

    #[test]
    fn una_a_la_vez_salvo_vital() {
        let mut s = pedir(45_000, NivelEmergencia::Trabajo);
        s.reposicion_pendiente = true;
        assert_eq!(
            decidir_retiro(s),
            Err(RechazoRetiro::ReposicionPendiente),
            "trabajo con pendiente"
        );
        let mut vital = pedir(60_000, NivelEmergencia::Vital);
        vital.reposicion_pendiente = true;
        assert!(decidir_retiro(vital).is_ok(), "lo vital sí pasa");
    }

    #[test]
    fn no_se_saca_lo_que_no_hay_ni_cero() {
        let mut s = pedir(45_000, NivelEmergencia::Trabajo);
        s.saldo_fondo = Colones::new(31_619);
        assert_eq!(
            decidir_retiro(s),
            Err(RechazoRetiro::FondoInsuficiente {
                disponible: Colones::new(31_619),
                pedido: Colones::new(45_000)
            }),
            "fondo insuficiente"
        );
        assert_eq!(
            decidir_retiro(pedir(0, NivelEmergencia::Vital)),
            Err(RechazoRetiro::MontoInvalido),
            "monto cero"
        );
    }

    #[test]
    fn la_cuota_redondea_hacia_arriba() {
        let plan = decidir_retiro(pedir(10_001, NivelEmergencia::Trabajo)).unwrap();
        // 10 001 + 5 % = 10 501 → 3 cuotas de 3 501 (no se queda debiendo).
        assert_eq!(
            plan.cuota_mensual,
            Colones::new(3_501),
            "cuota hacia arriba"
        );
    }
}
