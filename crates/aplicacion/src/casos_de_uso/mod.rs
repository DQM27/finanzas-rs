//! Los casos de uso, agrupados por tema: cada archivo tiene todo lo que se
//! puede hacer con un tema y nada de otro.
//!
//! Orquestan, no deciden: buscan los datos, se los dan al dominio y guardan lo
//! que el dominio resuelve.

pub mod calendario;
pub mod extras;
pub mod fondo;
pub mod gastos;
pub mod perfil;
pub mod quincena;
