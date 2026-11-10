//! Proporciones exactas (factor neto, recargos) en diezmilésimas.
//!
//! `0.8617` se guarda como `8617`. Aplicarla a un monto redondea siempre igual:
//! al colón más cercano y, en el empate, alejándose del cero.

use crate::dinero::Colones;

const ESCALA: i128 = 10_000;

/// Una proporción no negativa, en diezmilésimas: `10_000` es el 100 %.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Proporcion(u32);

impl Proporcion {
    /// Cero por ciento.
    pub const CERO: Self = Self(0);
    /// Cien por ciento.
    pub const ENTERA: Self = Self(10_000);

    /// Crea una proporción a partir de diezmilésimas (`8617` = 86,17 %).
    pub const fn desde_diezmilesimas(diezmilesimas: u32) -> Self {
        Self(diezmilesimas)
    }

    /// Crea una proporción a partir de un porcentaje entero (`5` = 5 %).
    pub const fn porcentaje(por_ciento: u32) -> Self {
        Self(por_ciento * 100)
    }

    /// Las diezmilésimas guardadas.
    pub const fn diezmilesimas(self) -> u32 {
        self.0
    }

    /// Aplica la proporción a un monto, redondeando al colón más cercano.
    pub fn aplicar(self, monto: Colones) -> Colones {
        let producto = i128::from(monto.valor()) * i128::from(self.0);
        let medio = ESCALA / 2;
        let redondeado = if producto >= 0 {
            (producto + medio) / ESCALA
        } else {
            (producto - medio) / ESCALA
        };
        // Un monto por una proporción razonable cabe de sobra en i64; si no
        // cupiera, se satura en vez de envolver.
        Colones::new(i64::try_from(redondeado).unwrap_or(if redondeado > 0 {
            i64::MAX
        } else {
            i64::MIN
        }))
    }

    /// El monto más esta proporción de recargo (`45_000` + 5 % = `47_250`).
    pub fn con_recargo(self, monto: Colones) -> Colones {
        monto + self.aplicar(monto)
    }
}

#[cfg(test)]
mod tests {
    use super::Proporcion;
    use crate::dinero::Colones;

    #[test]
    fn el_factor_neto_redondea_al_colon_mas_cercano() {
        let factor = Proporcion::desde_diezmilesimas(8617);
        // 25 800 × 0,8617 = 22 231,86
        assert_eq!(
            factor.aplicar(Colones::new(25_800)),
            Colones::new(22_232),
            "nocturna"
        );
        // 40 000 × 0,8617 = 34 468
        assert_eq!(
            factor.aplicar(Colones::new(40_000)),
            Colones::new(34_468),
            "especial"
        );
    }

    #[test]
    fn el_empate_se_aleja_del_cero() {
        let mitad = Proporcion::desde_diezmilesimas(5_000);
        assert_eq!(
            mitad.aplicar(Colones::new(3)),
            Colones::new(2),
            "1,5 sube a 2"
        );
        assert_eq!(
            mitad.aplicar(Colones::new(-3)),
            Colones::new(-2),
            "-1,5 baja a -2"
        );
    }

    #[test]
    fn recargo_por_porcentaje() {
        let cinco = Proporcion::porcentaje(5);
        assert_eq!(
            cinco.con_recargo(Colones::new(45_000)),
            Colones::new(47_250),
            "5 %"
        );
        assert_eq!(
            Proporcion::CERO.con_recargo(Colones::new(60_000)),
            Colones::new(60_000),
            "sin recargo"
        );
    }
}
