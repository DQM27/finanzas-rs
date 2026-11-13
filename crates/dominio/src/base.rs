//! Las piezas que usa todo el dominio: la plata, las proporciones, el texto
//! de los conceptos y a quién pertenece cada dato.
//!
//! No deciden nada de negocio por sí solas; son el vocabulario con que los
//! demás módulos escriben sus reglas.

pub mod dinero;
pub mod proporcion;
pub mod texto;
pub mod usuario;
