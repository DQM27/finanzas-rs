//! Identidad del dueño de los datos.
//!
//! Todo dato pertenece a un usuario. El sistema nace para una persona, pero
//! así puede abrirse a varias sin rehacer el modelo.

use uuid::Uuid;

/// Identificador de un usuario. Lo genera quien llama, nunca el dominio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UsuarioId(Uuid);

impl UsuarioId {
    /// Envuelve un UUID ya generado.
    pub const fn new(id: Uuid) -> Self {
        Self(id)
    }

    /// El UUID.
    pub const fn uuid(self) -> Uuid {
        self.0
    }
}
