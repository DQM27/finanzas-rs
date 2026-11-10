//! Conversión entre valores CBOR y los tipos del dominio.

use chrono::NaiveDate;
use ciborium::Value;
use finanzas_aplicacion::errores::ErrorRepositorio;
use finanzas_dominio::dinero::Colones;
use uuid::Uuid;

/// Etiqueta CBOR con que SurrealDB marca `NONE`.
const ETIQUETA_NONE: u64 = 6;

const FORMATO_FECHA: &str = "%Y-%m-%d";

pub fn error(mensaje: impl Into<String>) -> ErrorRepositorio {
    ErrorRepositorio(mensaje.into())
}

/// Arma un mapa CBOR con claves de texto.
pub fn mapa<const N: usize>(campos: [(&str, Value); N]) -> Value {
    Value::Map(
        campos
            .into_iter()
            .map(|(clave, valor)| (Value::Text(clave.to_owned()), valor))
            .collect(),
    )
}

/// Arma un mapa CBOR con una cantidad variable de campos.
pub fn mapa_de(campos: Vec<(&str, Value)>) -> Value {
    Value::Map(
        campos
            .into_iter()
            .map(|(clave, valor)| (Value::Text(clave.to_owned()), valor))
            .collect(),
    )
}

pub const fn booleano(b: bool) -> Value {
    Value::Bool(b)
}

pub fn texto(s: &str) -> Value {
    Value::Text(s.to_owned())
}

pub fn entero(n: i64) -> Value {
    Value::Integer(n.into())
}

pub fn colones(monto: Colones) -> Value {
    entero(monto.valor())
}

pub fn fecha(f: NaiveDate) -> Value {
    Value::Text(f.format(FORMATO_FECHA).to_string())
}

pub fn uuid(id: Uuid) -> Value {
    Value::Text(id.hyphenated().to_string())
}

/// `true` si el valor es `NULL` o `NONE`.
pub fn es_vacio(valor: &Value) -> bool {
    matches!(valor, Value::Null) || matches!(valor, Value::Tag(ETIQUETA_NONE, _))
}

/// Lee los campos de un registro.
#[derive(Debug, Clone, Copy)]
pub struct Registro<'a>(&'a [(Value, Value)]);

impl<'a> Registro<'a> {
    pub fn de(valor: &'a Value) -> Result<Self, ErrorRepositorio> {
        match valor {
            Value::Map(campos) => Ok(Self(campos)),
            otro => Err(error(format!("se esperaba un registro y llegó {otro:?}"))),
        }
    }

    pub fn campo(self, nombre: &str) -> Result<&'a Value, ErrorRepositorio> {
        self.0
            .iter()
            .find(|(clave, _)| matches!(clave, Value::Text(t) if t == nombre))
            .map(|(_, valor)| valor)
            .ok_or_else(|| error(format!("falta el campo `{nombre}`")))
    }

    pub fn texto(self, nombre: &str) -> Result<&'a str, ErrorRepositorio> {
        match self.campo(nombre)? {
            Value::Text(t) => Ok(t),
            otro => Err(error(format!("`{nombre}` no es texto: {otro:?}"))),
        }
    }

    pub fn entero(self, nombre: &str) -> Result<i64, ErrorRepositorio> {
        match self.campo(nombre)? {
            Value::Integer(n) => {
                i64::try_from(*n).map_err(|_| error(format!("`{nombre}` se sale de i64")))
            }
            otro => Err(error(format!("`{nombre}` no es entero: {otro:?}"))),
        }
    }

    pub fn booleano(self, nombre: &str) -> Result<bool, ErrorRepositorio> {
        match self.campo(nombre)? {
            Value::Bool(b) => Ok(*b),
            otro => Err(error(format!("`{nombre}` no es booleano: {otro:?}"))),
        }
    }

    pub fn colones(self, nombre: &str) -> Result<Colones, ErrorRepositorio> {
        self.entero(nombre).map(Colones::new)
    }

    pub fn fecha(self, nombre: &str) -> Result<NaiveDate, ErrorRepositorio> {
        let t = self.texto(nombre)?;
        NaiveDate::parse_from_str(t, FORMATO_FECHA)
            .map_err(|e| error(format!("`{nombre}` no es una fecha ({t}): {e}")))
    }

    pub fn uuid(self, nombre: &str) -> Result<Uuid, ErrorRepositorio> {
        let t = self.texto(nombre)?;
        Uuid::parse_str(t).map_err(|e| error(format!("`{nombre}` no es un UUID ({t}): {e}")))
    }

    /// Un subregistro que puede faltar (`NONE`).
    pub fn registro_opcional(self, nombre: &str) -> Result<Option<Self>, ErrorRepositorio> {
        match self
            .0
            .iter()
            .find(|(clave, _)| matches!(clave, Value::Text(t) if t == nombre))
        {
            None => Ok(None),
            Some((_, valor)) if es_vacio(valor) => Ok(None),
            Some((_, valor)) => Self::de(valor).map(Some),
        }
    }

    pub fn registro(self, nombre: &str) -> Result<Self, ErrorRepositorio> {
        Self::de(self.campo(nombre)?)
    }

    pub fn lista(self, nombre: &str) -> Result<&'a [Value], ErrorRepositorio> {
        match self.campo(nombre)? {
            Value::Array(lista) => Ok(lista),
            otro => Err(error(format!("`{nombre}` no es una lista: {otro:?}"))),
        }
    }
}

/// Las filas que devolvió un `SELECT`.
pub fn filas(resultado: &Value) -> Result<&[Value], ErrorRepositorio> {
    match resultado {
        Value::Array(filas) => Ok(filas),
        otro => Err(error(format!("se esperaba una lista de filas: {otro:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::{Registro, entero, es_vacio, fecha, mapa, texto};
    use chrono::NaiveDate;
    use ciborium::Value;

    #[test]
    fn lee_lo_que_escribe() {
        let dia = NaiveDate::from_ymd_opt(2026, 11, 9).unwrap();
        let v = mapa([
            ("monto", entero(2_950)),
            ("fecha", fecha(dia)),
            ("q", texto("x")),
        ]);
        let r = Registro::de(&v).unwrap();
        assert_eq!(r.entero("monto"), Ok(2_950), "entero");
        assert_eq!(r.fecha("fecha"), Ok(dia), "fecha");
        assert_eq!(r.texto("q"), Ok("x"), "texto");
        assert!(r.campo("nada").is_err(), "un campo que falta es un error");
        assert!(r.entero("q").is_err(), "un tipo equivocado es un error");
    }

    #[test]
    fn none_y_null_son_vacios() {
        assert!(es_vacio(&Value::Null), "NULL");
        assert!(
            es_vacio(&Value::Tag(6, Box::new(Value::Null))),
            "NONE de SurrealDB"
        );
        assert!(!es_vacio(&texto("")), "un texto vacío no es NONE");
    }
}
