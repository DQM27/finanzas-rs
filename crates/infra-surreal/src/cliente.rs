//! Cliente mínimo del RPC de SurrealDB: arma la petición, la manda por el
//! transporte que le den y revisa la respuesta sentencia por sentencia.

use crate::cbor::{Registro, error, es_vacio, mapa, mapa_de, texto};
use ciborium::Value;
use finanzas_aplicacion::errores::ErrorRepositorio;
use std::cell::{Cell, RefCell};
use std::fmt;

/// Cómo se identifica el cliente ante SurrealDB.
#[derive(Clone, PartialEq, Eq)]
pub enum Credencial {
    /// Usuario de sistema de la raíz (`DEFINE USER … ON ROOT`).
    Raiz {
        /// Usuario.
        usuario: String,
        /// Clave.
        clave: String,
    },
    /// Usuario de sistema de la base de datos de la conexión
    /// (`DEFINE USER … ON DATABASE`). Es con el que entra el Worker.
    DeBase {
        /// Usuario.
        usuario: String,
        /// Clave.
        clave: String,
    },
    /// Un token ya emitido por SurrealDB.
    Token(String),
}

impl fmt::Debug for Credencial {
    // Nunca se imprimen claves ni tokens, ni siquiera en un log de depuración.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raiz { usuario, .. } => f
                .debug_struct("Raiz")
                .field("usuario", usuario)
                .finish_non_exhaustive(),
            Self::DeBase { usuario, .. } => f
                .debug_struct("DeBase")
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

/// El token de la sesión, una vez que se inició. Nunca se imprime.
#[derive(Default)]
struct Sesion(RefCell<Option<String>>);

impl fmt::Debug for Sesion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if self.0.borrow().is_some() {
            "Sesion(<oculta>)"
        } else {
            "Sesion(sin iniciar)"
        })
    }
}

/// Cliente del RPC de SurrealDB.
///
/// Con usuario y clave inicia sesión una vez (`signin`) y consulta con el
/// token que le dan, igual que los SDK oficiales. SurrealDB Earth no acepta
/// `Authorization: Basic` para un usuario de base de datos.
#[derive(Debug)]
pub struct ClienteSurreal<H> {
    conexion: Conexion,
    http: H,
    siguiente_id: Cell<u64>,
    sesion: Sesion,
}

impl<H: EnvioHttp> ClienteSurreal<H> {
    /// Un cliente sobre una conexión y un transporte.
    pub fn new(conexion: Conexion, http: H) -> Self {
        Self {
            conexion,
            http,
            siguiente_id: Cell::new(1),
            sesion: Sesion::default(),
        }
    }

    /// El valor de `Authorization`, iniciando sesión si todavía no hay token.
    async fn autorizacion(&self) -> Result<String, ErrorRepositorio> {
        let (usuario, clave, de_base) = match &self.conexion.credencial {
            Credencial::Token(token) => return Ok(format!("Bearer {token}")),
            Credencial::Raiz { usuario, clave } => (usuario, clave, false),
            Credencial::DeBase { usuario, clave } => (usuario, clave, true),
        };
        if let Some(token) = self.sesion.0.borrow().clone() {
            return Ok(format!("Bearer {token}"));
        }
        let mut datos = vec![("user", texto(usuario)), ("pass", texto(clave))];
        if de_base {
            datos.push(("ns", texto(&self.conexion.espacio)));
            datos.push(("db", texto(&self.conexion.base)));
        }
        let respuesta = self
            .rpc("signin", vec![mapa_de(datos)], None)
            .await
            .map_err(|e| error(format!("no se pudo iniciar sesión en SurrealDB: {}", e.0)))?;
        let token = match Registro::de(&respuesta)?.campo("result")? {
            Value::Text(token) => token.clone(),
            _ => return Err(error("SurrealDB no devolvió un token al iniciar sesión")),
        };
        *self.sesion.0.borrow_mut() = Some(token.clone());
        Ok(format!("Bearer {token}"))
    }

    /// Manda un mensaje RPC y devuelve la respuesta, ya revisada: un `error`
    /// del RPC o un estado HTTP distinto de 200 son error.
    async fn rpc(
        &self,
        metodo: &str,
        params: Vec<Value>,
        autorizacion: Option<String>,
    ) -> Result<Value, ErrorRepositorio> {
        let id = self.siguiente_id.get();
        self.siguiente_id.set(id.wrapping_add(1));
        let mensaje = mapa([
            ("id", Value::Integer(id.into())),
            ("method", texto(metodo)),
            ("params", Value::Array(params)),
        ]);
        let mut cuerpo = Vec::new();
        ciborium::into_writer(&mensaje, &mut cuerpo)
            .map_err(|e| error(format!("no se pudo codificar la petición: {e}")))?;

        let mut cabeceras = vec![
            ("Content-Type", "application/cbor".to_owned()),
            ("Accept", "application/cbor".to_owned()),
            ("Surreal-NS", self.conexion.espacio.clone()),
            ("Surreal-DB", self.conexion.base.clone()),
        ];
        if let Some(autorizacion) = autorizacion {
            cabeceras.push(("Authorization", autorizacion));
        }
        let respuesta = self
            .http
            .enviar(PeticionHttp {
                url: format!("{}/rpc", self.conexion.url.trim_end_matches('/')),
                cabeceras,
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
        if let Ok(fallo) = Registro::de(&valor)?.campo("error") {
            let mensaje = Registro::de(fallo)
                .and_then(|f| f.texto("message"))
                .unwrap_or("error sin mensaje");
            return Err(error(format!("SurrealDB rechazó la petición: {mensaje}")));
        }
        Ok(valor)
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
        let autorizacion = self.autorizacion().await?;
        let valor = self
            .rpc(
                "query",
                vec![texto(sql), mapa(variables)],
                Some(autorizacion),
            )
            .await?;
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
        let de_base = Credencial::DeBase {
            usuario: "worker".to_owned(),
            clave: "secreta".to_owned(),
        };
        let raiz = Credencial::Raiz {
            usuario: "root".to_owned(),
            clave: "secreta".to_owned(),
        };
        let token = Credencial::Token("abc.def".to_owned());
        assert!(!format!("{de_base:?}").contains("secreta"), "clave oculta");
        assert!(!format!("{raiz:?}").contains("secreta"), "clave oculta");
        assert!(!format!("{token:?}").contains("abc"), "token oculto");
    }

    /// Guarda cada petición y contesta como SurrealDB: un token a `signin` y
    /// una sentencia vacía a `query`.
    #[derive(Debug, Default)]
    struct Grabadora(RefCell<Vec<PeticionHttp>>);

    impl Grabadora {
        fn metodo(peticion: &PeticionHttp) -> String {
            let cuerpo: Value = ciborium::from_reader(peticion.cuerpo.as_slice()).unwrap();
            Registro::de(&cuerpo)
                .unwrap()
                .texto("method")
                .unwrap()
                .to_owned()
        }
    }

    impl EnvioHttp for Grabadora {
        async fn enviar(&self, peticion: PeticionHttp) -> Result<RespuestaHttp, String> {
            let respuesta = if Self::metodo(&peticion) == "signin" {
                mapa([("result", texto("token.de.sesion"))])
            } else {
                mapa([("result", Value::Array(vec![sentencia("OK", Value::Null)]))])
            };
            self.0.borrow_mut().push(peticion);
            let mut cuerpo = Vec::new();
            ciborium::into_writer(&respuesta, &mut cuerpo).unwrap();
            Ok(RespuestaHttp {
                estado: 200,
                cuerpo,
            })
        }
    }

    fn conexion(credencial: Credencial) -> Conexion {
        Conexion {
            url: "https://db.ejemplo/".to_owned(),
            espacio: "finanzas".to_owned(),
            base: "produccion".to_owned(),
            credencial,
        }
    }

    fn autorizacion(peticion: &PeticionHttp) -> Option<&str> {
        peticion
            .cabeceras
            .iter()
            .find(|(nombre, _)| *nombre == "Authorization")
            .map(|(_, valor)| valor.as_str())
    }

    #[test]
    fn arma_la_peticion_rpc_en_cbor() {
        let cliente = ClienteSurreal::new(
            conexion(Credencial::Token("t".to_owned())),
            Grabadora::default(),
        );
        let r = pollster::block_on(cliente.consulta("RETURN $x;", [("x", entero(5))]));
        assert_eq!(r, Ok(vec![Value::Null]), "respuesta");

        let peticiones = cliente.http.0.borrow();
        assert_eq!(
            peticiones.len(),
            1,
            "con token no hace falta iniciar sesión"
        );
        let peticion = &peticiones[0];
        assert_eq!(peticion.url, "https://db.ejemplo/rpc", "sin doble barra");
        assert!(
            peticion
                .cabeceras
                .contains(&("Surreal-NS", "finanzas".to_owned()))
                && peticion
                    .cabeceras
                    .contains(&("Surreal-DB", "produccion".to_owned())),
            "cabeceras: {:?}",
            peticion.cabeceras
        );
        assert_eq!(autorizacion(peticion), Some("Bearer t"), "token");
        let cuerpo: Value = ciborium::from_reader(peticion.cuerpo.as_slice()).unwrap();
        let cuerpo = Registro::de(&cuerpo).unwrap();
        assert_eq!(cuerpo.texto("method"), Ok("query"), "método");
        assert_eq!(
            cuerpo.lista("params").unwrap().to_vec(),
            vec![texto("RETURN $x;"), mapa([("x", entero(5))])],
            "consulta y variables"
        );
    }

    #[test]
    fn con_usuario_de_base_inicia_sesion_una_vez_y_usa_el_token() {
        let cliente = ClienteSurreal::new(
            conexion(Credencial::DeBase {
                usuario: "worker".to_owned(),
                clave: "secreta".to_owned(),
            }),
            Grabadora::default(),
        );
        pollster::block_on(cliente.consulta("RETURN 1;", [])).unwrap();
        pollster::block_on(cliente.consulta("RETURN 2;", [])).unwrap();

        let peticiones = cliente.http.0.borrow();
        let metodos: Vec<String> = peticiones.iter().map(Grabadora::metodo).collect();
        assert_eq!(metodos, ["signin", "query", "query"], "una sola sesión");

        let signin: Value = ciborium::from_reader(peticiones[0].cuerpo.as_slice()).unwrap();
        let datos = Registro::de(&signin).unwrap().lista("params").unwrap()[0].clone();
        let datos = Registro::de(&datos).unwrap();
        assert_eq!(datos.texto("user"), Ok("worker"), "usuario");
        assert_eq!(datos.texto("pass"), Ok("secreta"), "clave");
        assert_eq!(
            datos.texto("ns"),
            Ok("finanzas"),
            "el usuario vive en la base"
        );
        assert_eq!(
            datos.texto("db"),
            Ok("produccion"),
            "el usuario vive en la base"
        );
        assert_eq!(autorizacion(&peticiones[0]), None, "signin va sin token");
        for consulta in &peticiones[1..] {
            assert_eq!(
                autorizacion(consulta),
                Some("Bearer token.de.sesion"),
                "las consultas usan el token de la sesión"
            );
        }
        assert!(
            !format!("{:?}", cliente.sesion).contains("token.de.sesion"),
            "el token de la sesión no se imprime"
        );
    }

    #[test]
    fn con_usuario_de_raiz_no_manda_namespace_ni_base_al_iniciar_sesion() {
        let cliente = ClienteSurreal::new(
            conexion(Credencial::Raiz {
                usuario: "root".to_owned(),
                clave: "root".to_owned(),
            }),
            Grabadora::default(),
        );
        pollster::block_on(cliente.consulta("RETURN 1;", [])).unwrap();
        let peticiones = cliente.http.0.borrow();
        let signin: Value = ciborium::from_reader(peticiones[0].cuerpo.as_slice()).unwrap();
        let datos = Registro::de(&signin).unwrap().lista("params").unwrap()[0].clone();
        let datos = Registro::de(&datos).unwrap();
        assert!(
            datos.campo("ns").is_err() && datos.campo("db").is_err(),
            "root se busca en la raíz"
        );
    }
}
