//! Worker de Cloudflare de finanzas-rs.
//!
//! Es sólo pegamento: lee la configuración del entorno, verifica la identidad
//! que pone Cloudflare Access (`finanzas-acceso`), arma los repositorios de
//! SurrealDB con el `fetch` del Worker como transporte y le pasa la petición a
//! `finanzas-api`. Ninguna regla de negocio vive aquí.
//!
//! Configuración (ver `wrangler.toml`):
//!
//! | Nombre | Tipo | Qué es |
//! |---|---|---|
//! | `SURREAL_URL` | variable | `https://<instancia>.surrealdb.earth` |
//! | `SURREAL_ESPACIO`, `SURREAL_BASE` | variable | Namespace y base de datos |
//! | `SURREAL_USUARIO`, `SURREAL_CLAVE` | secreto | Usuario de sistema con rol `EDITOR` |
//! | `ACCESO_EMISOR` | variable | `https://<equipo>.cloudflareaccess.com` |
//! | `ACCESO_AUDIENCIA` | variable | Etiqueta AUD de la aplicación en Access |
//! | `USUARIOS` | secreto | `correo=uuid,correo=uuid` |
//! | `ZONA_HORARIA_MINUTOS` | variable | Desfase respecto de UTC, `-360` en Costa Rica |

#![expect(
    clippy::future_not_send,
    reason = "en el Worker (WebAssembly) los futuros no son `Send`"
)]

mod pegamento;

use finanzas_acceso::{Claves, ConfigAcceso, Directorio, ErrorAcceso, verificar};
use finanzas_api::{Contexto, Peticion, atender};
use finanzas_aplicacion::puertos::{GeneradorIds, Reloj};
use finanzas_infra_surreal::{
    ClienteSurreal, Conexion, Credencial, EnvioHttp, PeticionHttp, RespuestaHttp, Surreal,
};
use pegamento::{CABECERA_ACCESO, fecha_local, metodo, ruta_api};
use std::cell::RefCell;
use uuid::Uuid;
use worker::{
    Date, Env, Fetch, Headers, Method, Request, RequestInit, Response, console_error, event,
};

/// Cuánto se guardan las claves de Access antes de pedirlas otra vez.
const VIDA_CLAVES_MS: u64 = 60 * 60 * 1000;

thread_local! {
    /// Claves de Access y hasta cuándo valen. Un isolate atiende muchas
    /// peticiones; así no se piden en cada una.
    static CLAVES: RefCell<Option<(Claves, u64)>> = const { RefCell::new(None) };
}

#[event(fetch)]
async fn fetch(mut req: Request, env: Env, _ctx: worker::Context) -> worker::Result<Response> {
    let ruta = req.path();
    let Some(ruta) = ruta_api(&ruta) else {
        return respuesta(
            404,
            r#"{"codigo":"RUTA_INEXISTENTE","mensaje":"esa ruta no existe"}"#,
        );
    };
    let ruta = ruta.to_owned();

    let usuario = match identificar(&req, &env).await {
        Ok(usuario) => usuario,
        Err(Fallo::Acceso(e)) => {
            console_error!("acceso rechazado: {e}");
            return respuesta(
                401,
                r#"{"codigo":"NO_AUTORIZADO","mensaje":"iniciá sesión de nuevo"}"#,
            );
        }
        Err(Fallo::Config(e)) => {
            console_error!("configuración: {e}");
            return error_interno();
        }
    };

    let ctx = match contexto(&env) {
        Ok(ctx) => ctx,
        Err(e) => {
            console_error!("configuración: {e}");
            return error_interno();
        }
    };
    let cuerpo = req.bytes().await.unwrap_or_default();
    let r = atender(
        &ctx,
        usuario,
        Peticion {
            metodo: metodo(&req.method()),
            ruta: &ruta,
            cuerpo: &cuerpo,
        },
    )
    .await;
    if let Some(detalle) = r.detalle_interno {
        console_error!("{} {ruta}: {detalle}", req.method());
    }
    respuesta(r.estado, &r.cuerpo)
}

/// Por qué no se pudo identificar a la persona.
enum Fallo {
    /// La identidad no es válida: 401.
    Acceso(ErrorAcceso),
    /// El Worker está mal configurado: 500.
    Config(String),
}

fn texto(env: &Env, nombre: &str) -> Result<String, String> {
    env.var(nombre)
        .map(|v| v.to_string())
        .or_else(|_| env.secret(nombre).map(|s| s.to_string()))
        .map_err(|_| format!("falta `{nombre}`"))
}

async fn identificar(
    req: &Request,
    env: &Env,
) -> Result<finanzas_dominio::usuario::UsuarioId, Fallo> {
    let config = ConfigAcceso {
        emisor: texto(env, "ACCESO_EMISOR").map_err(Fallo::Config)?,
        audiencia: texto(env, "ACCESO_AUDIENCIA").map_err(Fallo::Config)?,
    };
    let directorio = Directorio::desde_config(&texto(env, "USUARIOS").map_err(Fallo::Config)?)
        .map_err(|e| Fallo::Config(e.to_string()))?;
    let token = req
        .headers()
        .get(CABECERA_ACCESO)
        .ok()
        .flatten()
        .ok_or(Fallo::Acceso(ErrorAcceso::SinToken))?;
    let ahora_ms = Date::now().as_millis();
    let ahora = i64::try_from(ahora_ms / 1000).unwrap_or(i64::MAX);

    let correo = match verificar(
        &token,
        &claves(&config, ahora_ms, false).await?,
        &config,
        ahora,
    ) {
        // Access rota sus claves: ante una clave nueva se piden otra vez.
        Err(ErrorAcceso::ClaveDesconocida(_)) => verificar(
            &token,
            &claves(&config, ahora_ms, true).await?,
            &config,
            ahora,
        ),
        otro => otro,
    }
    .map_err(Fallo::Acceso)?;
    directorio.usuario(&correo).map_err(Fallo::Acceso)
}

/// Las claves de Access, de la memoria del isolate o pedidas de nuevo.
async fn claves(config: &ConfigAcceso, ahora_ms: u64, renovar: bool) -> Result<Claves, Fallo> {
    if !renovar {
        let guardadas = CLAVES.with(|c| {
            c.borrow()
                .as_ref()
                .filter(|(_, vence)| ahora_ms < *vence)
                .map(|(claves, _)| claves.clone())
        });
        if let Some(claves) = guardadas {
            return Ok(claves);
        }
    }
    let url = config.url_claves();
    let mut r = Fetch::Url(
        url.parse()
            .map_err(|e| Fallo::Config(format!("{url}: {e}")))?,
    )
    .send()
    .await
    .map_err(|e| Fallo::Config(format!("no se pudieron pedir las claves: {e}")))?;
    let json = r
        .text()
        .await
        .map_err(|e| Fallo::Config(format!("claves ilegibles: {e}")))?;
    let claves = Claves::desde_json(&json).map_err(|e| Fallo::Config(e.to_string()))?;
    CLAVES.with(|c| *c.borrow_mut() = Some((claves.clone(), ahora_ms + VIDA_CLAVES_MS)));
    Ok(claves)
}

fn respuesta(estado: u16, cuerpo: &str) -> worker::Result<Response> {
    let cabeceras = Headers::new();
    cabeceras.set("Content-Type", "application/json; charset=utf-8")?;
    cabeceras.set("Cache-Control", "no-store")?;
    cabeceras.set("X-Content-Type-Options", "nosniff")?;
    Ok(Response::ok(cuerpo)?
        .with_status(estado)
        .with_headers(cabeceras))
}

fn error_interno() -> worker::Result<Response> {
    respuesta(
        500,
        r#"{"codigo":"ERROR_INTERNO","mensaje":"algo falló de nuestro lado; intentá de nuevo en un momento"}"#,
    )
}

// ------------------------------------------------------------- adaptadores

/// Todo lo que la API necesita, sobre SurrealDB Earth.
struct ContextoWorker {
    repos: Surreal<FetchHttp>,
    ids: IdsWorker,
    reloj: RelojWorker,
}

fn contexto(env: &Env) -> Result<ContextoWorker, String> {
    let minutos: i64 = texto(env, "ZONA_HORARIA_MINUTOS")?
        .trim()
        .parse()
        .map_err(|e| format!("ZONA_HORARIA_MINUTOS: {e}"))?;
    let conexion = Conexion {
        url: texto(env, "SURREAL_URL")?,
        espacio: texto(env, "SURREAL_ESPACIO")?,
        base: texto(env, "SURREAL_BASE")?,
        credencial: Credencial::DeBase {
            usuario: texto(env, "SURREAL_USUARIO")?,
            clave: texto(env, "SURREAL_CLAVE")?,
        },
    };
    Ok(ContextoWorker {
        repos: Surreal::new(ClienteSurreal::new(conexion, FetchHttp)),
        ids: IdsWorker,
        reloj: RelojWorker { minutos },
    })
}

impl Contexto for ContextoWorker {
    type Perfiles = Surreal<FetchHttp>;
    type Gastos = Surreal<FetchHttp>;
    type Extras = Surreal<FetchHttp>;
    type Fondo = Surreal<FetchHttp>;
    type Quincenas = Surreal<FetchHttp>;
    type Ids = IdsWorker;
    type Reloj = RelojWorker;

    fn perfiles(&self) -> &Surreal<FetchHttp> {
        &self.repos
    }
    fn gastos(&self) -> &Surreal<FetchHttp> {
        &self.repos
    }
    fn extras(&self) -> &Surreal<FetchHttp> {
        &self.repos
    }
    fn fondo(&self) -> &Surreal<FetchHttp> {
        &self.repos
    }
    fn quincenas(&self) -> &Surreal<FetchHttp> {
        &self.repos
    }
    fn ids(&self) -> &IdsWorker {
        &self.ids
    }
    fn reloj(&self) -> &RelojWorker {
        &self.reloj
    }
}

/// El `fetch` del Worker como transporte hacia SurrealDB.
#[derive(Debug)]
struct FetchHttp;

impl EnvioHttp for FetchHttp {
    async fn enviar(&self, p: PeticionHttp) -> Result<RespuestaHttp, String> {
        let cabeceras = Headers::new();
        for (nombre, valor) in &p.cabeceras {
            cabeceras.set(nombre, valor).map_err(|e| e.to_string())?;
        }
        let mut init = RequestInit::new();
        init.with_method(Method::Post)
            .with_headers(cabeceras)
            .with_body(Some(js_sys::Uint8Array::from(p.cuerpo.as_slice()).into()));
        let peticion = Request::new_with_init(&p.url, &init).map_err(|e| e.to_string())?;
        let mut r = Fetch::Request(peticion)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Ok(RespuestaHttp {
            estado: r.status_code(),
            cuerpo: r.bytes().await.map_err(|e| e.to_string())?,
        })
    }
}

/// UUID v4 con `crypto.randomUUID()` del runtime.
#[derive(Debug)]
struct IdsWorker;

impl GeneradorIds for IdsWorker {
    fn nuevo(&self) -> Uuid {
        aleatorio_del_runtime().unwrap_or_else(|| {
            // Respaldo si el runtime no tuviera `randomUUID`: los ids no son
            // secretos, sólo tienen que ser únicos.
            let mut bytes = [0_u8; 16];
            for b in &mut bytes {
                *b = pegamento::byte_de(js_sys::Math::random());
            }
            uuid::Builder::from_random_bytes(bytes).into_uuid()
        })
    }
}

fn aleatorio_del_runtime() -> Option<Uuid> {
    let crypto = js_sys::Reflect::get(&js_sys::global(), &"crypto".into()).ok()?;
    let funcion = js_sys::Reflect::get(&crypto, &"randomUUID".into()).ok()?;
    let funcion: js_sys::Function = funcion.dyn_into().ok()?;
    let texto = funcion.call0(&crypto).ok()?.as_string()?;
    Uuid::parse_str(&texto).ok()
}

use worker::wasm_bindgen::JsCast;

/// La fecha de hoy en la zona horaria configurada.
#[derive(Debug)]
struct RelojWorker {
    minutos: i64,
}

impl Reloj for RelojWorker {
    fn hoy(&self) -> chrono::NaiveDate {
        fecha_local(Date::now().as_millis(), self.minutos)
    }
}
