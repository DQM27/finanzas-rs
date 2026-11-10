//! Casos de uso de finanzas-rs.
//!
//! Cada caso de uso es un struct que recibe sus puertos por constructor, busca
//! los datos, se los pasa al dominio y guarda lo que el dominio decide. Aquí no
//! se decide nada sobre la plata.

// Los puertos no exigen `Send` (en WebAssembly los futuros no lo son), así que
// los casos de uso tampoco pueden prometerlo.
#![expect(
    clippy::future_not_send,
    reason = "los puertos son asíncronos sin `Send` para funcionar en WebAssembly"
)]

pub mod casos_de_uso;
pub mod errores;
pub mod puertos;
