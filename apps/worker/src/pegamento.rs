//! Piezas puras del Worker, separadas para poder probarlas en nativo.

use chrono::{DateTime, NaiveDate};
use finanzas_api::Metodo;
use worker::Method;

/// Cabecera donde Cloudflare Access pone el JWT de la persona.
pub const CABECERA_ACCESO: &str = "Cf-Access-Jwt-Assertion";

/// Lo que va después de `/api`, o `None` si la ruta no es de la API.
pub fn ruta_api(ruta: &str) -> Option<&str> {
    match ruta.strip_prefix("/api")? {
        "" => Some("/"),
        resto if resto.starts_with('/') => Some(resto),
        _ => None,
    }
}

/// El método del Worker en los términos de la API.
pub const fn metodo(m: &Method) -> Metodo {
    match m {
        Method::Get => Metodo::Get,
        Method::Post => Metodo::Post,
        Method::Put => Metodo::Put,
        Method::Delete => Metodo::Delete,
        _ => Metodo::Otro,
    }
}

/// La fecha local de un instante (milisegundos desde 1970, UTC) con un
/// desfase fijo en minutos. Costa Rica no tiene horario de verano: −360.
pub fn fecha_local(milis: u64, desfase_minutos: i64) -> NaiveDate {
    let milis = i64::try_from(milis).unwrap_or(i64::MAX);
    let local = milis.saturating_add(desfase_minutos.saturating_mul(60_000));
    DateTime::from_timestamp_millis(local)
        .map(|d| d.date_naive())
        .unwrap_or_default()
}

/// Un byte a partir de un número en `[0, 1)`.
pub fn byte_de(x: f64) -> u8 {
    let escalado = (x.clamp(0.0, 1.0) * 255.0).round();
    // `escalado` está entre 0 y 255: la conversión no pierde nada.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "el valor ya está acotado a 0..=255"
    )]
    let byte = escalado as u8;
    byte
}

#[cfg(test)]
mod tests {
    use super::{byte_de, fecha_local, metodo, ruta_api};
    use chrono::NaiveDate;
    use finanzas_api::Metodo;
    use worker::Method;

    #[test]
    fn solo_lo_que_empieza_con_api_es_de_la_api() {
        assert_eq!(ruta_api("/api/fondo"), Some("/fondo"), "ruta");
        assert_eq!(ruta_api("/api"), Some("/"), "raíz");
        assert_eq!(ruta_api("/apis/fondo"), None, "otro prefijo");
        assert_eq!(ruta_api("/fondo"), None, "sin prefijo");
        assert_eq!(ruta_api("/"), None, "la PWA");
    }

    #[test]
    fn metodos() {
        assert_eq!(metodo(&Method::Put), Metodo::Put, "PUT");
        assert_eq!(metodo(&Method::Patch), Metodo::Otro, "PATCH no se usa");
    }

    #[test]
    fn la_fecha_es_la_de_costa_rica() {
        // 2026-11-14 05:30 UTC = 2026-11-13 23:30 en Costa Rica.
        let instante = 1_794_634_200_000;
        assert_eq!(
            fecha_local(instante, -360),
            NaiveDate::from_ymd_opt(2026, 11, 13).unwrap(),
            "todavía es viernes 13 en Costa Rica"
        );
        assert_eq!(
            fecha_local(instante, 0),
            NaiveDate::from_ymd_opt(2026, 11, 14).unwrap(),
            "en UTC ya es sábado"
        );
    }

    #[test]
    fn bytes_acotados() {
        assert_eq!(byte_de(0.0), 0, "mínimo");
        assert_eq!(byte_de(0.999_999), 255, "máximo");
        assert_eq!(byte_de(7.0), 255, "fuera de rango se acota");
    }
}
