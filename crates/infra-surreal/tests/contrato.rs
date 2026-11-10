//! Los repositorios de SurrealDB cumplen el mismo contrato que los de memoria.
//!
//! Necesitan un SurrealDB corriendo, por eso están marcadas `ignore`. Para
//! correrlas:
//!
//! ```text
//! surreal start --user root --pass root memory
//! cargo test -p finanzas-infra-surreal -- --ignored
//! ```
//!
//! `FINANZAS_SURREAL_URL`, `FINANZAS_SURREAL_USUARIO` y
//! `FINANZAS_SURREAL_CLAVE` cambian la dirección y la credencial (por defecto
//! `http://127.0.0.1:8000`, `root`, `root`). Cada prueba usa su propia base de
//! datos, recién creada.

#[cfg(test)]
mod tests {
    use finanzas_infra_surreal::{
        ClienteSurreal, Conexion, Credencial, EnvioHttp, PeticionHttp, RespuestaHttp, Surreal,
    };
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Transporte nativo y bloqueante, sólo para pruebas.
    #[derive(Debug)]
    struct Ureq(ureq::Agent);

    impl EnvioHttp for Ureq {
        async fn enviar(&self, p: PeticionHttp) -> Result<RespuestaHttp, String> {
            let mut peticion = self.0.post(&p.url);
            for (nombre, valor) in &p.cabeceras {
                peticion = peticion.header(*nombre, valor);
            }
            let mut respuesta = peticion.send(&p.cuerpo[..]).map_err(|e| e.to_string())?;
            Ok(RespuestaHttp {
                estado: respuesta.status().as_u16(),
                cuerpo: respuesta
                    .body_mut()
                    .read_to_vec()
                    .map_err(|e| e.to_string())?,
            })
        }
    }

    fn variable(nombre: &str, defecto: &str) -> String {
        std::env::var(nombre).unwrap_or_else(|_| defecto.to_owned())
    }

    /// Repositorios sobre una base de datos nueva y con el esquema aplicado.
    fn base_nueva() -> Surreal<Ureq> {
        static CONTADOR: AtomicU32 = AtomicU32::new(0);
        let marca = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = format!(
            "prueba_{marca}_{}",
            CONTADOR.fetch_add(1, Ordering::Relaxed)
        );
        let agente: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .into();
        let cliente = ClienteSurreal::new(
            Conexion {
                url: variable("FINANZAS_SURREAL_URL", "http://127.0.0.1:8000"),
                espacio: "finanzas_pruebas".to_owned(),
                base: base.clone(),
                credencial: Credencial::Basica {
                    usuario: variable("FINANZAS_SURREAL_USUARIO", "root"),
                    clave: variable("FINANZAS_SURREAL_CLAVE", "root"),
                },
            },
            Ureq(agente),
        );
        pollster::block_on(async {
            cliente
                .consulta(
                    &format!(
                        "DEFINE NAMESPACE IF NOT EXISTS finanzas_pruebas; \
                         DEFINE DATABASE IF NOT EXISTS {base};"
                    ),
                    [],
                )
                .await
                .unwrap();
            cliente.aplicar_esquema().await.unwrap();
            // Aplicarlo de nuevo no rompe nada.
            cliente.aplicar_esquema().await.unwrap();
        });
        Surreal::new(cliente)
    }

    #[test]
    #[ignore = "necesita SurrealDB: ver el comentario del archivo"]
    fn perfiles() {
        pollster::block_on(finanzas_contrato::perfiles(&base_nueva()));
    }

    #[test]
    #[ignore = "necesita SurrealDB: ver el comentario del archivo"]
    fn gastos() {
        pollster::block_on(finanzas_contrato::gastos(&base_nueva()));
    }

    #[test]
    #[ignore = "necesita SurrealDB: ver el comentario del archivo"]
    fn extras() {
        pollster::block_on(finanzas_contrato::extras(&base_nueva()));
    }

    #[test]
    #[ignore = "necesita SurrealDB: ver el comentario del archivo"]
    fn fondo() {
        pollster::block_on(finanzas_contrato::fondo(&base_nueva()));
    }

    #[test]
    #[ignore = "necesita SurrealDB: ver el comentario del archivo"]
    fn quincenas() {
        pollster::block_on(finanzas_contrato::quincenas(&base_nueva()));
    }
}
