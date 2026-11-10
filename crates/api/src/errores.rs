//! Cómo se traduce cada error a HTTP. El código es estable (la interfaz decide
//! con él) y el mensaje se puede mostrar tal cual. Un error interno nunca
//! expone su detalle al cliente.

use crate::Respuesta;
use crate::dto::ErrorDto;
use finanzas_aplicacion::errores::ErrorAplicacion;

pub type Resultado = Result<(u16, String), ErrorApi>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorApi {
    Aplicacion(ErrorAplicacion),
    JsonInvalido(String),
    RutaInexistente,
    MetodoNoPermitido,
    SinPresupuesto,
    Interno(String),
}

impl From<ErrorAplicacion> for ErrorApi {
    fn from(e: ErrorAplicacion) -> Self {
        match e {
            ErrorAplicacion::Repositorio(r) => Self::Interno(r.0),
            otro => Self::Aplicacion(otro),
        }
    }
}

const MENSAJE_INTERNO: &str = "algo falló de nuestro lado; intentá de nuevo en un momento";

impl ErrorApi {
    /// Código HTTP, código estable y mensaje.
    fn partes(&self) -> (u16, &'static str, String) {
        match self {
            Self::JsonInvalido(detalle) => (
                400,
                "JSON_INVALIDO",
                format!("los datos enviados no son válidos: {detalle}"),
            ),
            Self::RutaInexistente => (404, "RUTA_INEXISTENTE", "esa ruta no existe".to_owned()),
            Self::MetodoNoPermitido => (
                405,
                "METODO_NO_PERMITIDO",
                "esa ruta no admite ese método".to_owned(),
            ),
            Self::SinPresupuesto => (
                404,
                "SIN_PRESUPUESTO",
                "esa quincena todavía no tiene presupuesto".to_owned(),
            ),
            Self::Interno(_) => (500, "ERROR_INTERNO", MENSAJE_INTERNO.to_owned()),
            Self::Aplicacion(e) => {
                let (estado, codigo) = match e {
                    ErrorAplicacion::PerfilNoEncontrado => (404, "PERFIL_NO_ENCONTRADO"),
                    ErrorAplicacion::QuincenaNoEncontrada(_) => (404, "SIN_PRESUPUESTO"),
                    ErrorAplicacion::NoEncontrado => (404, "NO_ENCONTRADO"),
                    ErrorAplicacion::FueraDeCalendario => (409, "FUERA_DE_CALENDARIO"),
                    ErrorAplicacion::RetiroRechazado(_) => (409, "RETIRO_RECHAZADO"),
                    ErrorAplicacion::NoEsDiaDePago(_)
                    | ErrorAplicacion::Quincena(_)
                    | ErrorAplicacion::Concepto(_)
                    | ErrorAplicacion::Gasto(_)
                    | ErrorAplicacion::Extra(_)
                    | ErrorAplicacion::Fondo(_) => (422, "DATOS_INVALIDOS"),
                    ErrorAplicacion::Repositorio(_) => (500, "ERROR_INTERNO"),
                };
                let mensaje = if estado == 500 {
                    MENSAJE_INTERNO.to_owned()
                } else {
                    e.to_string()
                };
                (estado, codigo, mensaje)
            }
        }
    }

    pub fn respuesta(&self) -> Respuesta {
        let (estado, codigo, mensaje) = self.partes();
        let cuerpo = serde_json::to_string(&ErrorDto {
            codigo: codigo.to_owned(),
            mensaje,
        })
        .unwrap_or_else(|_| format!(r#"{{"codigo":"{codigo}","mensaje":""}}"#));
        let detalle_interno = match self {
            Self::Interno(detalle) => Some(detalle.clone()),
            _ => None,
        };
        Respuesta {
            estado,
            cuerpo,
            detalle_interno,
        }
    }
}
