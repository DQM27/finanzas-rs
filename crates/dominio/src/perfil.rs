//! Configuración personal de cada usuario.
//!
//! Ningún número personal está fijo en el código: el salario, los rebajos, las
//! tarifas de las extras y el tope de la caja chica son de cada persona.

use crate::base::dinero::Colones;
use crate::base::proporcion::Proporcion;
use crate::base::usuario::UsuarioId;
use crate::calendario::CalendarioCortes;
use crate::extras::TipoExtra;
use crate::gastos::caja_chica::CajaChica;
use thiserror::Error;

/// Cómo se calcula el neto de lo que se gana.
///
/// `neto = bruto × factor_neto`; al salario de la quincena además se le
/// restan los rebajos fijos (préstamos, pólizas, ahorros de planilla).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerfilSalarial {
    salario_base_bruto: Colones,
    factor_neto: Proporcion,
    rebajos_fijos: Colones,
}

/// Por qué un perfil salarial no es válido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ErrorPerfil {
    /// El salario base debe ser mayor que cero.
    #[error("el salario base debe ser mayor que cero")]
    SalarioInvalido,
    /// El factor neto no puede pasar del 100 %.
    #[error("el factor neto no puede pasar del 100 %")]
    FactorInvalido,
    /// Los rebajos fijos no pueden ser negativos.
    #[error("los rebajos fijos no pueden ser negativos")]
    RebajosInvalidos,
    /// Una tarifa de extra debe ser mayor que cero.
    #[error("la tarifa de una extra debe ser mayor que cero")]
    TarifaInvalida,
}

impl PerfilSalarial {
    /// Crea y valida un perfil salarial (montos por quincena).
    pub fn new(
        salario_base_bruto: Colones,
        factor_neto: Proporcion,
        rebajos_fijos: Colones,
    ) -> Result<Self, ErrorPerfil> {
        if !salario_base_bruto.es_positivo() {
            return Err(ErrorPerfil::SalarioInvalido);
        }
        if factor_neto > Proporcion::ENTERA {
            return Err(ErrorPerfil::FactorInvalido);
        }
        if rebajos_fijos.es_negativo() {
            return Err(ErrorPerfil::RebajosInvalidos);
        }
        Ok(Self {
            salario_base_bruto,
            factor_neto,
            rebajos_fijos,
        })
    }

    /// Lo que queda neto de un monto bruto variable (una extra, un feriado).
    pub fn neto_de(&self, bruto: Colones) -> Colones {
        self.factor_neto.aplicar(bruto)
    }

    /// El salario base neto de una quincena, ya con los rebajos fijos.
    pub fn salario_neto_quincena(&self) -> Colones {
        self.neto_de(self.salario_base_bruto) - self.rebajos_fijos
    }

    /// El salario base bruto por quincena.
    pub const fn salario_base_bruto(&self) -> Colones {
        self.salario_base_bruto
    }

    /// El factor neto.
    pub const fn factor_neto(&self) -> Proporcion {
        self.factor_neto
    }

    /// Los rebajos fijos por quincena.
    pub const fn rebajos_fijos(&self) -> Colones {
        self.rebajos_fijos
    }
}

/// Lo que se paga bruto por cada tipo de extra con tarifa fija.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TarifasExtra {
    nocturna: Colones,
    diurna: Colones,
    especial: Colones,
    feriado: Colones,
}

impl TarifasExtra {
    /// Crea y valida las tarifas (todas mayores que cero).
    pub fn new(
        nocturna: Colones,
        diurna: Colones,
        especial: Colones,
        feriado: Colones,
    ) -> Result<Self, ErrorPerfil> {
        if [nocturna, diurna, especial, feriado]
            .iter()
            .any(|t| !t.es_positivo())
        {
            return Err(ErrorPerfil::TarifaInvalida);
        }
        Ok(Self {
            nocturna,
            diurna,
            especial,
            feriado,
        })
    }

    /// La tarifa de un tipo de extra. `Otra` no tiene tarifa: se escribe a mano.
    pub const fn tarifa(&self, tipo: TipoExtra) -> Option<Colones> {
        match tipo {
            TipoExtra::Nocturna => Some(self.nocturna),
            TipoExtra::Diurna => Some(self.diurna),
            TipoExtra::Especial => Some(self.especial),
            TipoExtra::Feriado => Some(self.feriado),
            TipoExtra::Otra => None,
        }
    }
}

/// Toda la configuración de un usuario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Perfil {
    usuario: UsuarioId,
    salarial: PerfilSalarial,
    tarifas: TarifasExtra,
    caja_chica: CajaChica,
    calendario: CalendarioCortes,
}

impl Perfil {
    /// Junta la configuración de un usuario (cada parte ya viene validada).
    pub const fn new(
        usuario: UsuarioId,
        salarial: PerfilSalarial,
        tarifas: TarifasExtra,
        caja_chica: CajaChica,
        calendario: CalendarioCortes,
    ) -> Self {
        Self {
            usuario,
            salarial,
            tarifas,
            caja_chica,
            calendario,
        }
    }

    /// El dueño de esta configuración.
    pub const fn usuario(&self) -> UsuarioId {
        self.usuario
    }

    /// Cómo se calcula el neto.
    pub const fn salarial(&self) -> &PerfilSalarial {
        &self.salarial
    }

    /// Las tarifas de las extras.
    pub const fn tarifas(&self) -> &TarifasExtra {
        &self.tarifas
    }

    /// La caja chica.
    pub const fn caja_chica(&self) -> &CajaChica {
        &self.caja_chica
    }

    /// El calendario de cortes y pagos del patrono.
    pub const fn calendario(&self) -> &CalendarioCortes {
        &self.calendario
    }
}

#[cfg(test)]
mod tests {
    use super::{ErrorPerfil, PerfilSalarial};
    use crate::base::dinero::Colones;
    use crate::base::proporcion::Proporcion;

    fn perfil_de_daniel() -> PerfilSalarial {
        PerfilSalarial::new(
            Colones::new(293_000),
            Proporcion::desde_diezmilesimas(8617),
            Colones::new(20_253),
        )
        .unwrap()
    }

    #[test]
    fn salario_neto_de_la_quincena() {
        // 293 000 × 0,8617 = 252 478,1 → 252 478 − 20 253 = 232 225
        assert_eq!(
            perfil_de_daniel().salario_neto_quincena(),
            Colones::new(232_225),
            "salario base neto real de noviembre 2026"
        );
    }

    #[test]
    fn rechaza_valores_imposibles() {
        let factor = Proporcion::desde_diezmilesimas(8617);
        assert_eq!(
            PerfilSalarial::new(Colones::CERO, factor, Colones::CERO),
            Err(ErrorPerfil::SalarioInvalido),
            "salario cero"
        );
        assert_eq!(
            PerfilSalarial::new(
                Colones::new(1),
                Proporcion::desde_diezmilesimas(10_001),
                Colones::CERO
            ),
            Err(ErrorPerfil::FactorInvalido),
            "factor mayor al 100 %"
        );
        assert_eq!(
            PerfilSalarial::new(Colones::new(1), factor, Colones::new(-1)),
            Err(ErrorPerfil::RebajosInvalidos),
            "rebajos negativos"
        );
    }
}
