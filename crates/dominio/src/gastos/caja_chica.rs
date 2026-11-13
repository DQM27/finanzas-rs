//! Caja chica: el tope para gastos chicos y puntuales de cada periodo de pago.
//!
//! Lo que pasa del tope sale de la sobra de la quincena; si la sobra no
//! alcanza, es un retiro del fondo de nivel «descuido».

use crate::base::dinero::Colones;
use crate::base::proporcion::Proporcion;
use thiserror::Error;

/// Debajo de esta parte del tope, la caja está baja (30 %).
pub const UMBRAL_BAJA: Proporcion = Proporcion::porcentaje(30);

/// La caja chica de un usuario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CajaChica {
    tope: Colones,
}

/// Por qué una caja chica no es válida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ErrorCajaChica {
    /// El tope debe ser mayor que cero.
    #[error("el tope de la caja chica debe ser mayor que cero")]
    TopeInvalido,
}

/// Cómo está la caja.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NivelCaja {
    /// Hay holgura.
    Holgada,
    /// Queda el 30 % o menos.
    Baja,
    /// Se acabó (o se pasó).
    Agotada,
}

/// El estado de la caja en un periodo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EstadoCaja {
    /// Lo que queda (nunca negativo).
    pub queda: Colones,
    /// Lo que se gastó de más sobre el tope: sale de la sobra.
    pub exceso: Colones,
    /// Holgada, baja o agotada.
    pub nivel: NivelCaja,
}

impl CajaChica {
    /// Crea una caja chica con su tope por periodo.
    pub fn new(tope: Colones) -> Result<Self, ErrorCajaChica> {
        if !tope.es_positivo() {
            return Err(ErrorCajaChica::TopeInvalido);
        }
        Ok(Self { tope })
    }

    /// El tope por periodo.
    pub const fn tope(&self) -> Colones {
        self.tope
    }

    /// El estado después de gastar `gastado` en el periodo.
    pub fn estado(&self, gastado: Colones) -> EstadoCaja {
        let restante = self.tope - gastado;
        let queda = restante.o_cero();
        let exceso = (-restante).o_cero();
        let nivel = if !queda.es_positivo() {
            NivelCaja::Agotada
        } else if queda <= UMBRAL_BAJA.aplicar(self.tope) {
            NivelCaja::Baja
        } else {
            NivelCaja::Holgada
        };
        EstadoCaja {
            queda,
            exceso,
            nivel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CajaChica, ErrorCajaChica, NivelCaja};
    use crate::base::dinero::Colones;

    #[test]
    fn pasajes_y_jugo_dejan_la_caja_baja() {
        let caja = CajaChica::new(Colones::new(5_000)).unwrap();
        let estado = caja.estado(Colones::new(1_000 + 2_950));
        assert_eq!(estado.queda, Colones::new(1_050), "quedan 1 050");
        assert_eq!(estado.exceso, Colones::CERO, "sin exceso");
        assert_eq!(
            estado.nivel,
            NivelCaja::Baja,
            "1 050 es menos del 30 % de 5 000"
        );
    }

    #[test]
    fn pasarse_del_tope_marca_el_exceso() {
        let caja = CajaChica::new(Colones::new(5_000)).unwrap();
        let estado = caja.estado(Colones::new(8_000));
        assert_eq!(estado.queda, Colones::CERO, "no queda nada");
        assert_eq!(
            estado.exceso,
            Colones::new(3_000),
            "3 000 salen de la sobra"
        );
        assert_eq!(estado.nivel, NivelCaja::Agotada, "agotada");
    }

    #[test]
    fn holgada_y_tope_invalido() {
        let caja = CajaChica::new(Colones::new(5_000)).unwrap();
        assert_eq!(
            caja.estado(Colones::CERO).nivel,
            NivelCaja::Holgada,
            "sin gastos"
        );
        assert_eq!(
            CajaChica::new(Colones::CERO),
            Err(ErrorCajaChica::TopeInvalido),
            "tope 0"
        );
    }
}
