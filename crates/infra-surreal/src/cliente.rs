//! Cliente mínimo del RPC de SurrealDB: arma la petición, la manda por el
//! transporte que le den y revisa la respuesta sentencia por sentencia.

use crate::cbor::{Registro, error, es_vacio, mapa, texto};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ciborium::Value;
use finanzas_aplicacion::errores::ErrorRepositorio;
use std::cell::Cell;
use std::fmt;

/// Cómo se identifica el cliente ante SurrealDB.
#[derive(Clone, PartialEq, Eq)]
pub enum Credencial {
    /// Usuario y clave de sistema (`DEFINE USER`).
    Basica {
        /// Usuario.
        usuario: String,
        /// Clave.
        clave: String,
    },
    /// Un token ya emitido por SurrealDB.
    Token(String),
}

impl Credencial {
    fn cabecera(&self) -> String {
        match self {
            Self::Basica { usuario, clave } => {
                format!("Basic {}", STANDARD.encode(format!("{usuario}:{clave}")))
            }
            Self::Token(token) => format!("Bearer {token}"),
        }
    }
}

impl fmt::Debug for Credencial {
    // Nunca se imprimen claves ni tokens, ni siquiera en un log de depuración.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Basica { usuario, .. } => f
                .debug_struct("Basica")
                .field("usuario", usuario)
                .finish_non_exhaustive(),
            Self::Token(_) => f.write_str("Token(<oculto>)"),
        }
    }
}

/// A dónde y cómo conectarse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conexion {
    /// Dirección base, por ejemplo `https://xxxx.aws-use1.surreal.cloud`.
    pub url: String,
    /// Espacio de nombres (`NAMESPACE`).
    pub espacio: String,
    /// Base de datos (`DATABASE`).
    pub base: String,
    /// Credencial.
    pub credencial: Credencial,
}

/// Una petición HTTP `POST` lista para mandar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeticionHttp {
    /// Dirección completa.
    pub url: String,
    /// Cabeceras.
    pub cabeceras: Vec<(&'static str, String)>,
    /// Cuerpo en CBOR.
    pub cuerpo: Vec<u8>,
}

/// La respuesta HTTP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespuestaHttp {
    /// Código de estado.
    pub estado: u16,
    /// Cuerpo.
    pub cuerpo: Vec<u8>,
}

/// El transporte HTTP. Lo implementa quien usa el adaptador.
#[expect(
    async_fn_in_trait,
    reason = "sin `Send` a propósito: en WebAssembly los futuros no son `Send`"
)]
pub trait EnvioHttp {
    /// Manda la petición. Un error aquí es de red, no de SurrealDB.
    async fn enviar(&self, peticion: PeticionHttp) -> Result<RespuestaHttp, String>;
}

/// Cliente del RPC de SurrealDB.
#[derive(Debug)]
pub struct ClienteSurreal<H> {
    conexion: Conexion,
    http: H,
    siguiente_id: Cell<u64>,
}

impl<H: EnvioHttp> ClienteSurreal<H> {
    /// Un cliente sobre una conexión y un transporte.
    pub const fn new(conexion: Conexion, http: H) -> Self {
        Self {
            conexion,
            http,
            siguiente_id: Cell::new(1),
        }
    }

    /// Aplica el esquema completo ([`crate::ESQUEMA`]).
    pub async fn aplicar_esquema(&self) -> Result<(), ErrorRepositorio> {
        self.consulta(crate::ESQUEMA, []).await.map(drop)
    }

    /// Corre una o varias sentencias con sus variables y devuelve el resultado
    /// de cada una, en orden. Si cualquiera falla, falla todo.
    pub async fn consulta<const N: usize>(
        &self,
        sql: &str,
        variables: [(&str, Value); N],
    ) -> Result<Vec<Value>, ErrorRepositorio> {
        let id = self.siguiente_id.get();
        self.siguiente_id.set(id.wrapping_add(1));
        let peticion = mapa([
            ("id", Value::Integer(id.into())),
            ("method", texto("query")),
            ("params", Value::Array(vec![texto(sql), mapa(variables)])),
        ]);
        let mut cuerpo = Vec::new();
        ciborium::into_writer(&peticion, &mut cuerpo)
            .map_err(|e| error(format!("no se pudo codificar la consulta: {e}")))?;

        let respuesta = self
            .http
            .enviar(PeticionHttp {
                url: format!("{}/rpc", self.conexion.url.trim_end_matches('/')),
                cabeceras: vec![
                    ("Content-Type", "application/cbor".to_owned()),
                    ("Accept", "application/cbor".to_owned()),
                    ("Surreal-NS", self.conexion.espacio.clone()),
                    ("Surreal-DB", self.conexion.base.clone()),
                    ("Authorization", self.conexion.credencial.cabecera()),
                ],
                cuerpo,
            })
            .await
            .map_err(|e| error(format!("SurrealDB no respondió: {e}")))?;
        if respuesta.estado != 200 {
            return Err(error(format!(
                "SurrealDB respondió {}: {}",
                respuesta.estado,
                String::from_utf8_lossy(&respuesta.cuerpo)
            )));
        }
        let valor: Value = ciborium::from_reader(respuesta.cuerpo.as_slice())
            .map_err(|e| error(format!("respuesta ilegible de SurrealDB: {e}")))?;
        resultados(&valor)
    }
}

/// Saca el resultado de cada sentencia de una respuesta RPC.
fn resultados(respuesta: &Value) -> Result<Vec<Value>, ErrorRepositorio> {
    let respuesta = Registro::de(respuesta)?;
    if let Ok(fallo) = respuesta.campo("error") {
        let mensaje = Registro::de(fallo)
            .and_then(|f| f.texto("message"))
            .unwrap_or("error sin mensaje");
        return Err(error(format!("SurrealDB rechazó la petición: {mensaje}")));
    }
    respuesta
        .lista("result")?
        .iter()
        .enumerate()
        .map(|(i, sentencia)| {
            let sentencia = Registro::de(sentencia)?;
            let resultado = sentencia.campo("result")?;
            if sentencia.texto("status")? == "OK" {
                Ok(if es_vacio(resultado) {
                    Value::Null
                } else {
                    resultado.clone()
                })
            } else {
                let detalle = match resultado {
                    Value::Text(t) => t.clone(),
                    otro => format!("{otro:?}"),
                };
                Err(error(format!("la sentencia {} falló: {detalle}", i + 1)))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        ClienteSurreal, Conexion, Credencial, EnvioHttp, PeticionHttp, RespuestaHttp, resultados,
    };
    use crate::cbor::{Registro, entero, mapa, texto};
    use ciborium::Value;
    use std::cell::RefCell;

    fn sentencia(estado: &str, resultado: Value) -> Value {
        mapa([("status", texto(estado)), ("result", resultado)])
    }

    #[test]
    fn devuelve_el_resultado_de_cada_sentencia() {
        let r = mapa([(
            "result",
            Value::Array(vec![
                sentencia("OK", Value::Tag(6, Box::new(Value::Null))),
                sentencia("OK", Value::Array(vec![entero(1)])),
            ]),
        )]);
        assert_eq!(
            resultados(&r),
            Ok(vec![Value::Null, Value::Array(vec![entero(1)])]),
            "NONE pasa a NULL y el resto queda igual"
        );
    }

    #[test]
    fn una_sentencia_fallida_hace_fallar_todo() {
        let r = mapa([(
            "result",
            Value::Array(vec![
                sentencia("OK", Value::Null),
                sentencia("ERR", texto("ya existe")),
            ]),
        )]);
        let e = resultados(&r).unwrap_err();
        assert!(
            e.0.contains("sentencia 2") && e.0.contains("ya existe"),
            "{e}"
        );
    }

    #[test]
    fn un_error_del_rpc_se_reporta() {
        let r = mapa([("error", mapa([("message", texto("sin permiso"))]))]);
        assert!(
            resultados(&r).unwrap_err().0.contains("sin permiso"),
            "mensaje"
        );
    }

    #[test]
    fn la_credencial_no_se_imprime() {
        let basica = Credencial::Basica {
            usuario: "worker".to_owned(),
            clave: "secreta".to_owned(),
        };
        let token = Credencial::Token("abc.def".to_owned());
        assert!(!format!("{basica:?}").contains("secreta"), "clave oculta");
        assert!(!format!("{token:?}").contains("abc"), "token oculto");
        assert_eq!(
            basica.cabecera(),
            "Basic d29ya2VyOnNlY3JldGE=",
            "base64 de worker:secreta"
        );
    }

    /// Guarda la petición y contesta con una respuesta fija.
    #[derive(Debug, Default)]
    struct Grabadora(RefCell<Option<PeticionHttp>>);

    impl EnvioHttp for Grabadora {
        async fn enviar(&self, peticion: PeticionHttp) -> Result<RespuestaHttp, String> {
            *self.0.borrow_mut() = Some(peticion);
            let mut cuerpo = Vec::new();
            ciborium::into_writer(
                &mapa([("result", Value::Array(vec![sentencia("OK", Value::Null)]))]),
                &mut cuerpo,
            )
            .unwrap();
            Ok(RespuestaHttp {
                estado: 200,
                cuerpo,
            })
        }
    }

    #[test]
    fn arma_la_peticion_rpc_en_cbor() {
        let cliente = ClienteSurreal::new(
            Conexion {
                url: "https://db.ejemplo/".to_owned(),
                espacio: "finanzas".to_owned(),
                base: "produccion".to_owned(),
                credencial: Credencial::Token("t".to_owned()),
            },
            Grabadora::default(),
        );
        let r = pollster::block_on(cliente.consulta("RETURN $x;", [("x", entero(5))]));
        assert_eq!(r, Ok(vec![Value::Null]), "respuesta");

        let peticion = cliente.http.0.borrow_mut().take().unwrap();
        assert_eq!(peticion.url, "https://db.ejemplo/rpc", "sin doble barra");
        assert!(
            peticion
                .cabeceras
                .contains(&("Surreal-NS", "finanzas".to_owned()))
                && peticion
                    .cabeceras
                    .contains(&("Surreal-DB", "produccion".to_owned()))
                && peticion
                    .cabeceras
                    .contains(&("Authorization", "Bearer t".to_owned())),
            "cabeceras: {:?}",
            peticion.cabeceras
        );
        let cuerpo: Value = ciborium::from_reader(peticion.cuerpo.as_slice()).unwrap();
        let cuerpo = Registro::de(&cuerpo).unwrap();
        assert_eq!(cuerpo.texto("method"), Ok("query"), "método");
        assert_eq!(
            cuerpo.lista("params").unwrap().to_vec(),
            vec![texto("RETURN $x;"), mapa([("x", entero(5))])],
            "consulta y variables"
        );
    }
}
