//! Identidad de quien usa la app.
//!
//! La app está detrás de Cloudflare Access con Google: nadie llega al Worker
//! sin haber iniciado sesión. Aun así el Worker no le cree a la red: verifica
//! el JWT que Access pone en la cabecera `Cf-Access-Jwt-Assertion` (firma RS256
//! con las claves públicas del equipo, emisor, audiencia y vigencia) y sólo
//! entonces traduce el correo a un [`UsuarioId`] con el [`Directorio`].
//!
//! Todo es puro: la hora y las claves llegan como argumentos, así que se prueba
//! en nativo con tokens de prueba y compila igual para WebAssembly.

mod jwt;

use finanzas_dominio::usuario::UsuarioId;
use thiserror::Error;
use uuid::Uuid;

pub use jwt::{Claves, ConfigAcceso, verificar};

/// Por qué no se aceptó la identidad.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorAcceso {
    /// No vino la cabecera de Access.
    #[error("falta el token de Cloudflare Access")]
    SinToken,
    /// El token no tiene la forma de un JWT.
    #[error("el token no es un JWT válido: {0}")]
    Malformado(String),
    /// Sólo se acepta RS256.
    #[error("algoritmo no permitido: {0}")]
    AlgoritmoNoPermitido(String),
    /// La clave que firmó no está entre las del equipo.
    #[error("clave de firma desconocida: {0}")]
    ClaveDesconocida(String),
    /// La firma no corresponde.
    #[error("la firma del token no es válida")]
    FirmaInvalida,
    /// Lo emitió otro equipo de Access.
    #[error("el token lo emitió otro equipo")]
    EmisorIncorrecto,
    /// Es para otra aplicación.
    #[error("el token es para otra aplicación")]
    AudienciaIncorrecta,
    /// Venció o todavía no vale.
    #[error("el token venció o todavía no es válido")]
    FueraDeVigencia,
    /// No trae correo.
    #[error("el token no trae correo")]
    SinCorreo,
    /// El correo no está autorizado en esta app.
    #[error("el correo no tiene acceso a esta app")]
    CorreoNoAutorizado,
    /// Las claves públicas no se pudieron leer.
    #[error("las claves de Access no son válidas: {0}")]
    ClavesInvalidas(String),
    /// La lista de usuarios está mal escrita.
    #[error("la lista de usuarios no es válida: {0}")]
    DirectorioInvalido(String),
}

/// Qué correo corresponde a qué usuario.
///
/// Se configura como secreto del Worker con la forma
/// `correo=uuid` separados por coma. El correo se compara sin importar
/// mayúsculas. Un correo que Access deja pasar pero que no está aquí no entra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directorio(Vec<(String, UsuarioId)>);

impl Directorio {
    /// Lee la lista desde su texto de configuración.
    pub fn desde_config(texto: &str) -> Result<Self, ErrorAcceso> {
        let mut entradas: Vec<(String, UsuarioId)> = Vec::new();
        for entrada in texto.split(',').map(str::trim).filter(|e| !e.is_empty()) {
            let (correo, id) = entrada.split_once('=').ok_or_else(|| {
                ErrorAcceso::DirectorioInvalido(format!("falta `=` en `{entrada}`"))
            })?;
            let correo = correo.trim().to_lowercase();
            if !correo.contains('@') {
                return Err(ErrorAcceso::DirectorioInvalido(format!(
                    "`{correo}` no es un correo"
                )));
            }
            let id = Uuid::parse_str(id.trim()).map_err(|e| {
                ErrorAcceso::DirectorioInvalido(format!("UUID inválido para {correo}: {e}"))
            })?;
            if entradas.iter().any(|(c, _)| *c == correo) {
                return Err(ErrorAcceso::DirectorioInvalido(format!(
                    "`{correo}` está repetido"
                )));
            }
            entradas.push((correo, UsuarioId::new(id)));
        }
        Ok(Self(entradas))
    }

    /// El usuario de un correo verificado.
    pub fn usuario(&self, correo: &str) -> Result<UsuarioId, ErrorAcceso> {
        let correo = correo.to_lowercase();
        self.0
            .iter()
            .find(|(c, _)| *c == correo)
            .map(|(_, id)| *id)
            .ok_or(ErrorAcceso::CorreoNoAutorizado)
    }
}

#[cfg(test)]
mod tests {
    use super::{Directorio, ErrorAcceso};
    use finanzas_dominio::usuario::UsuarioId;
    use uuid::Uuid;

    const DANIEL: &str = "000000da-0000-4000-8000-000000000001";

    #[test]
    fn traduce_correos_sin_importar_mayusculas() {
        let d = Directorio::desde_config(&format!(" Daniel@Ejemplo.com = {DANIEL} , ")).unwrap();
        assert_eq!(
            d.usuario("daniel@ejemplo.COM"),
            Ok(UsuarioId::new(Uuid::parse_str(DANIEL).unwrap())),
            "mismo correo"
        );
        assert_eq!(
            d.usuario("otro@ejemplo.com"),
            Err(ErrorAcceso::CorreoNoAutorizado),
            "fuera de la lista no entra"
        );
    }

    #[test]
    fn rechaza_configuraciones_mal_escritas() {
        for malo in [
            "daniel@ejemplo.com",
            "daniel=000000da-0000-4000-8000-000000000001",
            "daniel@ejemplo.com=no-es-uuid",
            &format!("a@b.com={DANIEL},A@B.com={DANIEL}"),
        ] {
            assert!(
                matches!(
                    Directorio::desde_config(malo),
                    Err(ErrorAcceso::DirectorioInvalido(_))
                ),
                "debería rechazar `{malo}`"
            );
        }
        assert_eq!(
            Directorio::desde_config("").map(|d| d.usuario("x@y.com")),
            Ok(Err(ErrorAcceso::CorreoNoAutorizado)),
            "una lista vacía no deja entrar a nadie"
        );
    }
}
