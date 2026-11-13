//! Montos de dinero.
//!
//! El dinero se guarda en colones enteros. Nunca en `f64`: los decimales
//! binarios no representan bien los montos y los totales terminan
//! descuadrando por céntimos.

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

/// Un monto en colones enteros. Puede ser negativo (un saldo en rojo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Colones(i64);

impl Colones {
    /// Cero colones.
    pub const CERO: Self = Self(0);

    /// Crea un monto.
    pub const fn new(colones: i64) -> Self {
        Self(colones)
    }

    /// El monto como número.
    pub const fn valor(self) -> i64 {
        self.0
    }

    /// `true` si el monto es mayor que cero.
    pub const fn es_positivo(self) -> bool {
        self.0 > 0
    }

    /// `true` si el monto es menor que cero.
    pub const fn es_negativo(self) -> bool {
        self.0 < 0
    }

    /// El mayor entre este monto y cero.
    pub const fn o_cero(self) -> Self {
        if self.0 < 0 { Self::CERO } else { self }
    }
}

impl Add for Colones {
    type Output = Self;
    fn add(self, otro: Self) -> Self {
        Self(self.0 + otro.0)
    }
}

impl AddAssign for Colones {
    fn add_assign(&mut self, otro: Self) {
        self.0 += otro.0;
    }
}

impl Sub for Colones {
    type Output = Self;
    fn sub(self, otro: Self) -> Self {
        Self(self.0 - otro.0)
    }
}

impl SubAssign for Colones {
    fn sub_assign(&mut self, otro: Self) {
        self.0 -= otro.0;
    }
}

impl Neg for Colones {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl Sum for Colones {
    fn sum<I: Iterator<Item = Self>>(montos: I) -> Self {
        montos.fold(Self::CERO, Add::add)
    }
}

impl<'a> Sum<&'a Self> for Colones {
    fn sum<I: Iterator<Item = &'a Self>>(montos: I) -> Self {
        montos.copied().sum()
    }
}

/// Se muestra como `₡25,800` o `-₡1,000`.
impl fmt::Display for Colones {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digitos = self.0.unsigned_abs().to_string();
        let mut con_comas = String::with_capacity(digitos.len() + digitos.len() / 3);
        for (i, c) in digitos.chars().enumerate() {
            if i > 0 && (digitos.len() - i).is_multiple_of(3) {
                con_comas.push(',');
            }
            con_comas.push(c);
        }
        let signo = if self.0 < 0 { "-" } else { "" };
        write!(f, "{signo}₡{con_comas}")
    }
}

#[cfg(test)]
mod tests {
    use super::Colones;

    #[test]
    fn suma_resta_y_total() {
        let total: Colones = [Colones::new(104_000), Colones::new(100_000)].iter().sum();
        assert_eq!(total, Colones::new(204_000), "la suma de montos");
        assert_eq!(
            total - Colones::new(4_000),
            Colones::new(200_000),
            "la resta"
        );
    }

    #[test]
    fn se_muestra_con_separador_de_miles() {
        assert_eq!(
            Colones::new(25_800).to_string(),
            "₡25,800",
            "monto positivo"
        );
        assert_eq!(
            Colones::new(-1_000).to_string(),
            "-₡1,000",
            "monto negativo"
        );
        assert_eq!(Colones::new(500).to_string(), "₡500", "menos de mil");
        assert_eq!(
            Colones::new(1_234_567).to_string(),
            "₡1,234,567",
            "millones"
        );
    }

    #[test]
    fn o_cero_no_deja_pasar_negativos() {
        assert_eq!(
            Colones::new(-5).o_cero(),
            Colones::CERO,
            "negativo queda en cero"
        );
        assert_eq!(
            Colones::new(5).o_cero(),
            Colones::new(5),
            "positivo no cambia"
        );
    }
}
