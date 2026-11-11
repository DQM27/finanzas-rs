//! Verificación de tokens de Cloudflare Access con claves y tokens de prueba
//! generados aparte (`tokens.json`): dos claves RSA de 2048 bits del equipo
//! `https://quintana.cloudflareaccess.com` y la audiencia `aud-finanzas`. El
//! token válido rige entre 1794300000 y 1794400000.

#[cfg(test)]
mod tests {
    use finanzas_acceso::{Claves, ConfigAcceso, ErrorAcceso, verificar};
    use serde_json::Value;

    const VIGENTE: i64 = 1_794_350_000;

    fn datos() -> Value {
        serde_json::from_str(include_str!("tokens.json")).unwrap()
    }

    fn claves() -> Claves {
        Claves::desde_json(&datos()["jwks"].to_string()).unwrap()
    }

    fn config() -> ConfigAcceso {
        ConfigAcceso {
            emisor: "https://quintana.cloudflareaccess.com".to_owned(),
            audiencia: "aud-finanzas".to_owned(),
        }
    }

    fn probar(nombre: &str, ahora: i64) -> Result<String, ErrorAcceso> {
        let d = datos();
        let token = d["tokens"][nombre].as_str().unwrap();
        verificar(token, &claves(), &config(), ahora)
    }

    #[test]
    fn un_token_valido_da_el_correo() {
        assert_eq!(
            probar("valido", VIGENTE),
            Ok("daniel@ejemplo.com".to_owned()),
            "clave 1"
        );
        assert_eq!(
            probar("otra_clave", VIGENTE),
            Ok("daniel@ejemplo.com".to_owned()),
            "clave 2: Access rota claves y hay que aceptar todas las publicadas"
        );
    }

    #[test]
    fn una_firma_falsa_no_pasa() {
        assert_eq!(
            probar("firmado_con_otra", VIGENTE),
            Err(ErrorAcceso::FirmaInvalida),
            "dice clave 1 pero lo firmó la 2"
        );
        let d = datos();
        let valido = d["tokens"]["valido"].as_str().unwrap();
        let (sin_firma, _) = valido.rsplit_once('.').unwrap();
        // Se cambia el correo en el cuerpo y se conserva la firma original.
        let (cab, _) = sin_firma.split_once('.').unwrap();
        let cuerpo_falso = base64_url(
            br#"{"aud":["aud-finanzas"],"email":"intruso@ejemplo.com","exp":1794400000,"iss":"https://quintana.cloudflareaccess.com"}"#,
        );
        let firma = valido.rsplit('.').next().unwrap();
        let alterado = format!("{cab}.{cuerpo_falso}.{firma}");
        assert_eq!(
            verificar(&alterado, &claves(), &config(), VIGENTE),
            Err(ErrorAcceso::FirmaInvalida),
            "cambiar el correo invalida la firma"
        );
    }

    #[test]
    fn otro_algoritmo_ni_se_intenta() {
        assert_eq!(
            probar("alg_hs256", VIGENTE),
            Err(ErrorAcceso::AlgoritmoNoPermitido("HS256".to_owned())),
            "confusión de algoritmo"
        );
        let cab = base64_url(br#"{"alg":"none","kid":"clave-1"}"#);
        let cuerpo = base64_url(br#"{"email":"intruso@ejemplo.com"}"#);
        assert_eq!(
            verificar(&format!("{cab}.{cuerpo}."), &claves(), &config(), VIGENTE),
            Err(ErrorAcceso::AlgoritmoNoPermitido("none".to_owned())),
            "alg none"
        );
    }

    #[test]
    fn emisor_audiencia_y_vigencia() {
        assert_eq!(
            probar("otro_emisor", VIGENTE),
            Err(ErrorAcceso::EmisorIncorrecto),
            "otro equipo"
        );
        assert_eq!(
            probar("otra_audiencia", VIGENTE),
            Err(ErrorAcceso::AudienciaIncorrecta),
            "otra app del mismo equipo"
        );
        assert_eq!(
            probar("valido", 1_794_400_000 + 61),
            Err(ErrorAcceso::FueraDeVigencia),
            "vencido, pasada la tolerancia de 60 s"
        );
        assert!(
            probar("valido", 1_794_400_000 + 59).is_ok(),
            "dentro de la tolerancia"
        );
        assert_eq!(
            probar("valido", 1_794_300_000 - 61),
            Err(ErrorAcceso::FueraDeVigencia),
            "antes de nbf"
        );
        assert_eq!(
            probar("sin_correo", VIGENTE),
            Err(ErrorAcceso::SinCorreo),
            "sin correo"
        );
        assert_eq!(
            probar("kid_desconocido", VIGENTE),
            Err(ErrorAcceso::ClaveDesconocida("clave-9".to_owned())),
            "kid que no está publicado"
        );
    }

    #[test]
    fn basura_no_rompe_nada() {
        for malo in ["", "   ", "a.b", "a.b.c.d", "%%%.%%%.%%%", "e30.e30.e30"] {
            let r = verificar(malo, &claves(), &config(), VIGENTE);
            assert!(r.is_err(), "`{malo}` debería fallar: {r:?}");
        }
        assert!(
            matches!(
                Claves::desde_json(r#"{"keys":[]}"#),
                Err(ErrorAcceso::ClavesInvalidas(_))
            ),
            "sin claves"
        );
        assert!(Claves::desde_json("no es json").is_err(), "JSON roto");
    }

    #[test]
    fn la_url_de_las_claves() {
        assert_eq!(
            config().url_claves(),
            "https://quintana.cloudflareaccess.com/cdn-cgi/access/certs",
            "url"
        );
    }

    fn base64_url(bytes: &[u8]) -> String {
        use base64::Engine;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    }
}
