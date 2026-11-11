//! Verificación del JWT de Cloudflare Access.
//!
//! Sólo se acepta RS256 con una clave del equipo (`kid`). Se revisa la firma
//! antes de mirar cualquier dato del cuerpo; después, emisor, audiencia y
//! vigencia. No hay algoritmo `none` ni HMAC: un token que diga otro algoritmo
//! se rechaza sin intentar verificarlo.

use crate::ErrorAcceso;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::signature::Verifier;
use rsa::{BoxedUint, PublicKey};
use serde::Deserialize;
use sha2::Sha256;

/// Margen para relojes un poco desfasados, en segundos.
const TOLERANCIA_S: i64 = 60;

/// Contra qué se valida el token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigAcceso {
    /// El equipo de Access, por ejemplo `https://quintana.cloudflareaccess.com`.
    pub emisor: String,
    /// La etiqueta de audiencia (AUD) de la aplicación en Access.
    pub audiencia: String,
}

impl ConfigAcceso {
    /// Dónde publica Access sus claves públicas.
    pub fn url_claves(&self) -> String {
        format!("{}/cdn-cgi/access/certs", self.emisor.trim_end_matches('/'))
    }
}

/// Las claves públicas del equipo de Access.
#[derive(Debug, Clone)]
pub struct Claves(Vec<(String, PublicKey)>);

#[derive(Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
}

fn b64(texto: &str) -> Result<Vec<u8>, ErrorAcceso> {
    URL_SAFE_NO_PAD
        .decode(texto)
        .map_err(|e| ErrorAcceso::Malformado(format!("base64: {e}")))
}

impl Claves {
    /// Lee el JSON de `/cdn-cgi/access/certs`. Ignora lo que no sea RSA.
    pub fn desde_json(json: &str) -> Result<Self, ErrorAcceso> {
        let jwks: Jwks =
            serde_json::from_str(json).map_err(|e| ErrorAcceso::ClavesInvalidas(e.to_string()))?;
        let mut claves = Vec::new();
        for jwk in jwks.keys.into_iter().filter(|k| k.kty == "RSA") {
            let invalida = |e: &dyn std::fmt::Display| {
                ErrorAcceso::ClavesInvalidas(format!("clave {}: {e}", jwk.kid))
            };
            let n = b64(&jwk.n).map_err(|e| invalida(&e))?;
            let e = b64(&jwk.e).map_err(|e| invalida(&e))?;
            let bits = |bytes: &[u8]| u32::try_from(bytes.len() * 8).unwrap_or(u32::MAX);
            let n = BoxedUint::from_be_slice(&n, bits(&n)).map_err(|e| invalida(&e))?;
            let e = BoxedUint::from_be_slice(&e, bits(&e).max(64)).map_err(|e| invalida(&e))?;
            let clave = PublicKey::new(n, e).map_err(|e| invalida(&e))?;
            claves.push((jwk.kid, clave));
        }
        if claves.is_empty() {
            return Err(ErrorAcceso::ClavesInvalidas("no hay claves RSA".to_owned()));
        }
        Ok(Self(claves))
    }

    fn clave(&self, kid: &str) -> Option<&PublicKey> {
        self.0.iter().find(|(k, _)| k == kid).map(|(_, c)| c)
    }
}

#[derive(Deserialize)]
struct Cabecera {
    alg: String,
    #[serde(default)]
    kid: Option<String>,
}

/// La audiencia puede venir como texto o como lista.
#[derive(Deserialize)]
#[serde(untagged)]
enum Audiencia {
    Una(String),
    Varias(Vec<String>),
}

impl Audiencia {
    fn incluye(&self, aud: &str) -> bool {
        match self {
            Self::Una(a) => a == aud,
            Self::Varias(lista) => lista.iter().any(|a| a == aud),
        }
    }
}

#[derive(Deserialize)]
struct Datos {
    iss: String,
    aud: Audiencia,
    exp: i64,
    #[serde(default)]
    nbf: Option<i64>,
    #[serde(default)]
    email: Option<String>,
}

/// Verifica el token y devuelve el correo de la persona.
///
/// `ahora` son segundos desde 1970 (UTC); lo da quien llama.
pub fn verificar(
    token: &str,
    claves: &Claves,
    config: &ConfigAcceso,
    ahora: i64,
) -> Result<String, ErrorAcceso> {
    let token = token.trim();
    if token.is_empty() {
        return Err(ErrorAcceso::SinToken);
    }
    let mut partes = token.split('.');
    let (Some(cab), Some(cuerpo), Some(firma), None) =
        (partes.next(), partes.next(), partes.next(), partes.next())
    else {
        return Err(ErrorAcceso::Malformado("no tiene tres partes".to_owned()));
    };

    let cabecera: Cabecera = serde_json::from_slice(&b64(cab)?)
        .map_err(|e| ErrorAcceso::Malformado(format!("cabecera: {e}")))?;
    if cabecera.alg != "RS256" {
        return Err(ErrorAcceso::AlgoritmoNoPermitido(cabecera.alg));
    }
    let kid = cabecera.kid.unwrap_or_default();
    let clave = claves
        .clave(&kid)
        .ok_or_else(|| ErrorAcceso::ClaveDesconocida(kid.clone()))?;
    let firma =
        Signature::try_from(b64(firma)?.as_slice()).map_err(|_| ErrorAcceso::FirmaInvalida)?;
    VerifyingKey::<Sha256>::new(clave.clone())
        .verify(format!("{cab}.{cuerpo}").as_bytes(), &firma)
        .map_err(|_| ErrorAcceso::FirmaInvalida)?;

    // A partir de aquí el contenido es auténtico.
    let datos: Datos = serde_json::from_slice(&b64(cuerpo)?)
        .map_err(|e| ErrorAcceso::Malformado(format!("cuerpo: {e}")))?;
    if datos.iss.trim_end_matches('/') != config.emisor.trim_end_matches('/') {
        return Err(ErrorAcceso::EmisorIncorrecto);
    }
    if !datos.aud.incluye(&config.audiencia) {
        return Err(ErrorAcceso::AudienciaIncorrecta);
    }
    let vencido = ahora > datos.exp + TOLERANCIA_S;
    let temprano = datos.nbf.is_some_and(|nbf| ahora + TOLERANCIA_S < nbf);
    if vencido || temprano {
        return Err(ErrorAcceso::FueraDeVigencia);
    }
    datos
        .email
        .filter(|c| c.contains('@'))
        .ok_or(ErrorAcceso::SinCorreo)
}
