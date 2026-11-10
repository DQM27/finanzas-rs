//! Textos cortos que escribe el usuario (conceptos, notas).

use thiserror::Error;

/// Largo máximo de un concepto.
pub const LARGO_MAXIMO: usize = 80;

/// Un concepto no vacío, sin espacios sobrantes y en mayúscula (así se ve en
/// el celular y así se compara).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Concepto(String);

/// Por qué un concepto no es válido.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorConcepto {
    /// Vino vacío o sólo con espacios.
    #[error("falta el concepto")]
    Vacio,
    /// Pasa del largo máximo.
    #[error("el concepto tiene más de {LARGO_MAXIMO} caracteres")]
    MuyLargo,
}

impl Concepto {
    /// Normaliza y valida un concepto.
    pub fn new(texto: &str) -> Result<Self, ErrorConcepto> {
        let limpio = texto.split_whitespace().collect::<Vec<_>>().join(" ");
        if limpio.is_empty() {
            return Err(ErrorConcepto::Vacio);
        }
        if limpio.chars().count() > LARGO_MAXIMO {
            return Err(ErrorConcepto::MuyLargo);
        }
        Ok(Self(limpio.to_uppercase()))
    }

    /// El texto ya normalizado.
    pub fn como_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Concepto, ErrorConcepto};

    #[test]
    fn normaliza_espacios_y_mayusculas() {
        let c = Concepto::new("  jugo   nieto ").unwrap();
        assert_eq!(c.como_str(), "JUGO NIETO", "texto normalizado");
    }

    #[test]
    fn rechaza_vacio_y_muy_largo() {
        assert_eq!(Concepto::new("   "), Err(ErrorConcepto::Vacio), "vacío");
        assert_eq!(
            Concepto::new(&"x".repeat(81)),
            Err(ErrorConcepto::MuyLargo),
            "más de 80"
        );
    }
}
