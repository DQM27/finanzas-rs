//! Reglas de negocio de finanzas-rs.
//!
//! Todo lo que decide algo sobre la plata vive aquí: cuánto queda neto de una
//! extra, en qué pago cae, cuándo la caja chica se acabó, cuánto se repone
//! después de sacar del fondo. Las capas de afuera sólo buscan datos, se los
//! pasan al dominio y guardan lo que el dominio decide.
//!
//! El dominio no lee la hora ni genera IDs: los recibe.
//!
//! Está partido por tema, un módulo por responsabilidad:
//!
//! | Módulo | De qué responde |
//! |---|---|
//! | [`base`] | Plata, proporciones, conceptos y dueño: el vocabulario común |
//! | [`calendario`] | Los cortes de la empresa: qué se reporta cuándo y qué día se paga |
//! | [`perfil`] | Los números de cada persona: salario, tarifas y tope de caja chica |
//! | [`gastos`] | Lo que se gasta en el día y la caja chica |
//! | [`extras`] | Las extras trabajadas: cuánto dejan netas y en qué pago caen |
//! | [`quincena`] | El presupuesto de cada pago y la suma del mes |
//! | [`fondo`] | El fondo de emergencia: cuándo se puede sacar y cómo se repone |

pub mod base;
pub mod calendario;
pub mod extras;
pub mod fondo;
pub mod gastos;
pub mod perfil;
pub mod quincena;
