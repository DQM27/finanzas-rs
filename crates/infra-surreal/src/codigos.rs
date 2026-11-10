//! Cómo se guardan los enumerados del dominio. Los códigos son parte del
//! esquema: cambiar uno obliga a migrar los datos.

use crate::cbor::error;
use finanzas_aplicacion::errores::ErrorRepositorio;
use finanzas_dominio::extra::TipoExtra;
use finanzas_dominio::fondo::NivelEmergencia;
use finanzas_dominio::gasto::{Categoria, Origen};

pub const fn categoria(c: Categoria) -> &'static str {
    match c {
        Categoria::Comida => "COMIDA",
        Categoria::Casa => "CASA",
        Categoria::Moto => "MOTO",
        Categoria::Salud => "SALUD",
        Categoria::Familia => "FAMILIA",
        Categoria::Ropa => "ROPA",
        Categoria::Tramites => "TRAMITES",
        Categoria::Gusto => "GUSTO",
        Categoria::Otro => "OTRO",
    }
}

pub fn a_categoria(codigo: &str) -> Result<Categoria, ErrorRepositorio> {
    Ok(match codigo {
        "COMIDA" => Categoria::Comida,
        "CASA" => Categoria::Casa,
        "MOTO" => Categoria::Moto,
        "SALUD" => Categoria::Salud,
        "FAMILIA" => Categoria::Familia,
        "ROPA" => Categoria::Ropa,
        "TRAMITES" => Categoria::Tramites,
        "GUSTO" => Categoria::Gusto,
        "OTRO" => Categoria::Otro,
        otro => return Err(error(format!("categoría desconocida: {otro}"))),
    })
}

pub const fn origen(o: Origen) -> &'static str {
    match o {
        Origen::CuentaGastos => "CUENTA_GASTOS",
        Origen::Planilla => "PLANILLA",
        Origen::SobraDelMes => "SOBRA_DEL_MES",
        Origen::Gusto => "GUSTO",
        Origen::Fondo => "FONDO",
    }
}

pub fn a_origen(codigo: &str) -> Result<Origen, ErrorRepositorio> {
    Ok(match codigo {
        "CUENTA_GASTOS" => Origen::CuentaGastos,
        "PLANILLA" => Origen::Planilla,
        "SOBRA_DEL_MES" => Origen::SobraDelMes,
        "GUSTO" => Origen::Gusto,
        "FONDO" => Origen::Fondo,
        otro => return Err(error(format!("origen desconocido: {otro}"))),
    })
}

pub const fn tipo_extra(t: TipoExtra) -> &'static str {
    match t {
        TipoExtra::Nocturna => "NOCTURNA",
        TipoExtra::Diurna => "DIURNA",
        TipoExtra::Especial => "ESPECIAL",
        TipoExtra::Feriado => "FERIADO",
        TipoExtra::Otra => "OTRA",
    }
}

pub fn a_tipo_extra(codigo: &str) -> Result<TipoExtra, ErrorRepositorio> {
    Ok(match codigo {
        "NOCTURNA" => TipoExtra::Nocturna,
        "DIURNA" => TipoExtra::Diurna,
        "ESPECIAL" => TipoExtra::Especial,
        "FERIADO" => TipoExtra::Feriado,
        "OTRA" => TipoExtra::Otra,
        otro => return Err(error(format!("tipo de extra desconocido: {otro}"))),
    })
}

pub const fn nivel(n: NivelEmergencia) -> &'static str {
    match n {
        NivelEmergencia::Vital => "VITAL",
        NivelEmergencia::Trabajo => "TRABAJO",
        NivelEmergencia::Descuido => "DESCUIDO",
        NivelEmergencia::Falta => "FALTA",
    }
}

pub fn a_nivel(codigo: &str) -> Result<NivelEmergencia, ErrorRepositorio> {
    Ok(match codigo {
        "VITAL" => NivelEmergencia::Vital,
        "TRABAJO" => NivelEmergencia::Trabajo,
        "DESCUIDO" => NivelEmergencia::Descuido,
        "FALTA" => NivelEmergencia::Falta,
        otro => return Err(error(format!("nivel de emergencia desconocido: {otro}"))),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        a_categoria, a_nivel, a_origen, a_tipo_extra, categoria, nivel, origen, tipo_extra,
    };
    use finanzas_dominio::extra::TipoExtra;
    use finanzas_dominio::fondo::NivelEmergencia;
    use finanzas_dominio::gasto::{Categoria, Origen};

    #[test]
    fn todo_codigo_va_y_vuelve() {
        for c in [
            Categoria::Comida,
            Categoria::Casa,
            Categoria::Moto,
            Categoria::Salud,
            Categoria::Familia,
            Categoria::Ropa,
            Categoria::Tramites,
            Categoria::Gusto,
            Categoria::Otro,
        ] {
            assert_eq!(a_categoria(categoria(c)), Ok(c), "{c:?}");
        }
        for o in [
            Origen::CuentaGastos,
            Origen::Planilla,
            Origen::SobraDelMes,
            Origen::Gusto,
            Origen::Fondo,
        ] {
            assert_eq!(a_origen(origen(o)), Ok(o), "{o:?}");
        }
        for t in [
            TipoExtra::Nocturna,
            TipoExtra::Diurna,
            TipoExtra::Especial,
            TipoExtra::Feriado,
            TipoExtra::Otra,
        ] {
            assert_eq!(a_tipo_extra(tipo_extra(t)), Ok(t), "{t:?}");
        }
        for n in [
            NivelEmergencia::Vital,
            NivelEmergencia::Trabajo,
            NivelEmergencia::Descuido,
            NivelEmergencia::Falta,
        ] {
            assert_eq!(a_nivel(nivel(n)), Ok(n), "{n:?}");
        }
        assert!(a_categoria("comida").is_err(), "distingue mayúsculas");
    }

    #[test]
    fn los_codigos_coinciden_con_el_esquema() {
        let esquema = crate::ESQUEMA;
        for codigo in [
            "TRAMITES",
            "CUENTA_GASTOS",
            "SOBRA_DEL_MES",
            "FERIADO",
            "DESCUIDO",
            "REPOSICION",
        ] {
            assert!(
                esquema.contains(&format!("\"{codigo}\"")),
                "el esquema no admite {codigo}"
            );
        }
    }
}
