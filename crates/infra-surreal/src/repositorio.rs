//! Los repositorios sobre SurrealDB. Cada consulta filtra por usuario.

use crate::cbor::{self, Registro, error, filas, mapa, mapa_de};
use crate::cliente::{ClienteSurreal, EnvioHttp};
use crate::codigos;
use chrono::NaiveDate;
use ciborium::Value;
use finanzas_aplicacion::errores::ErrorRepositorio;
use finanzas_aplicacion::puertos::{
    RepositorioExtras, RepositorioFondo, RepositorioGastos, RepositorioPerfiles,
    RepositorioQuincenas,
};
use finanzas_dominio::caja_chica::CajaChica;
use finanzas_dominio::calendario::{CalendarioCortes, Corte};
use finanzas_dominio::extra::{Extra, ExtraId, TipoExtra};
use finanzas_dominio::fondo::PlanReposicion;
use finanzas_dominio::gasto::{DatosGasto, Gasto, GastoId};
use finanzas_dominio::movimientos_fondo::{Movimiento, MovimientoFondo, MovimientoFondoId};
use finanzas_dominio::perfil::{Perfil, PerfilSalarial, TarifasExtra};
use finanzas_dominio::proporcion::Proporcion;
use finanzas_dominio::quincena::{Linea, Quincena};
use finanzas_dominio::texto::Concepto;
use finanzas_dominio::usuario::UsuarioId;
use std::fmt::Display;

/// Todos los repositorios sobre una misma conexión.
#[derive(Debug)]
pub struct Surreal<H> {
    cliente: ClienteSurreal<H>,
}

impl<H: EnvioHttp> Surreal<H> {
    /// Repositorios sobre un cliente ya configurado.
    pub const fn new(cliente: ClienteSurreal<H>) -> Self {
        Self { cliente }
    }

    /// El cliente, para tareas de mantenimiento (aplicar el esquema).
    pub const fn cliente(&self) -> &ClienteSurreal<H> {
        &self.cliente
    }

    /// Borra un registro de una tabla sólo si es del usuario. `true` si
    /// borró algo.
    async fn borrar_de(
        &self,
        tabla: &str,
        dueno: UsuarioId,
        id: uuid::Uuid,
    ) -> Result<bool, ErrorRepositorio> {
        let filas = self
            .seleccionar(
                BORRAR,
                [
                    ("tabla", cbor::texto(tabla)),
                    ("id", cbor::uuid(id)),
                    ("usuario", usuario(dueno)),
                ],
            )
            .await?;
        Ok(!filas.is_empty())
    }

    /// Corre una consulta cuya última sentencia es un `SELECT` y devuelve sus
    /// filas.
    async fn seleccionar<const N: usize>(
        &self,
        sql: &str,
        variables: [(&str, Value); N],
    ) -> Result<Vec<Value>, ErrorRepositorio> {
        let resultados = self.cliente.consulta(sql, variables).await?;
        let ultimo = resultados
            .last()
            .ok_or_else(|| error("la consulta no devolvió resultados"))?;
        Ok(filas(ultimo)?.to_vec())
    }
}

/// Borra un registro por id sólo si es del usuario (U1); devuelve lo borrado.
const BORRAR: &str = "DELETE type::record($tabla, $id) WHERE usuario = $usuario RETURN BEFORE;";

/// Convierte un error de validación del dominio: un dato guardado que ya no es
/// válido es un problema del almacenamiento, no del usuario.
fn invalido(que: &str) -> impl Fn(&dyn Display) -> ErrorRepositorio + '_ {
    move |e| error(format!("{que} guardado no es válido: {e}"))
}

fn usuario(id: UsuarioId) -> Value {
    cbor::uuid(id.uuid())
}

// ---------------------------------------------------------------- perfiles

const LEER_PERFIL: &str = "SELECT * OMIT id FROM perfil WHERE usuario = $usuario;";
const GUARDAR_PERFIL: &str = "UPSERT type::record('perfil', $usuario) CONTENT $perfil RETURN NONE;";

fn perfil_a_cbor(p: &Perfil) -> Value {
    let salarial = p.salarial();
    let tarifa = |tipo| p.tarifas().tarifa(tipo).map_or(Value::Null, cbor::colones);
    let cortes = p
        .calendario()
        .cortes()
        .iter()
        .map(|c| {
            mapa([
                ("inicio", cbor::fecha(c.inicio())),
                ("fin", cbor::fecha(c.fin())),
                ("reportar_antes", cbor::fecha(c.reportar_antes())),
                ("pago", cbor::fecha(c.pago())),
            ])
        })
        .collect();
    mapa([
        ("usuario", usuario(p.usuario())),
        ("salario_base", cbor::colones(salarial.salario_base_bruto())),
        (
            "factor_neto",
            cbor::entero(i64::from(salarial.factor_neto().diezmilesimas())),
        ),
        ("rebajos", cbor::colones(salarial.rebajos_fijos())),
        (
            "tarifas",
            mapa([
                ("nocturna", tarifa(TipoExtra::Nocturna)),
                ("diurna", tarifa(TipoExtra::Diurna)),
                ("especial", tarifa(TipoExtra::Especial)),
                ("feriado", tarifa(TipoExtra::Feriado)),
            ]),
        ),
        ("tope_caja_chica", cbor::colones(p.caja_chica().tope())),
        ("cortes", Value::Array(cortes)),
    ])
}

fn perfil_de_cbor(valor: &Value) -> Result<Perfil, ErrorRepositorio> {
    let r = Registro::de(valor)?;
    let factor = u32::try_from(r.entero("factor_neto")?)
        .map_err(|_| error("factor neto guardado fuera de rango"))?;
    let salarial = PerfilSalarial::new(
        r.colones("salario_base")?,
        Proporcion::desde_diezmilesimas(factor),
        r.colones("rebajos")?,
    )
    .map_err(|e| invalido("el perfil salarial")(&e))?;
    let t = r.registro("tarifas")?;
    let tarifas = TarifasExtra::new(
        t.colones("nocturna")?,
        t.colones("diurna")?,
        t.colones("especial")?,
        t.colones("feriado")?,
    )
    .map_err(|e| invalido("las tarifas")(&e))?;
    let caja = CajaChica::new(r.colones("tope_caja_chica")?)
        .map_err(|e| invalido("el tope de caja chica")(&e))?;
    let cortes = r
        .lista("cortes")?
        .iter()
        .map(|c| {
            let c = Registro::de(c)?;
            Corte::new(
                c.fecha("inicio")?,
                c.fecha("fin")?,
                c.fecha("reportar_antes")?,
                c.fecha("pago")?,
            )
            .map_err(|e| invalido("un corte")(&e))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let calendario = CalendarioCortes::new(cortes).map_err(|e| invalido("el calendario")(&e))?;
    Ok(Perfil::new(
        UsuarioId::new(r.uuid("usuario")?),
        salarial,
        tarifas,
        caja,
        calendario,
    ))
}

impl<H: EnvioHttp> RepositorioPerfiles for Surreal<H> {
    async fn perfil(&self, id: UsuarioId) -> Result<Option<Perfil>, ErrorRepositorio> {
        let filas = self
            .seleccionar(LEER_PERFIL, [("usuario", usuario(id))])
            .await?;
        match filas.as_slice() {
            [] => Ok(None),
            [fila] => perfil_de_cbor(fila).map(Some),
            _ => Err(error("hay más de un perfil para el mismo usuario")),
        }
    }

    async fn guardar(&self, perfil: &Perfil) -> Result<(), ErrorRepositorio> {
        self.cliente
            .consulta(
                GUARDAR_PERFIL,
                [
                    ("usuario", usuario(perfil.usuario())),
                    ("perfil", perfil_a_cbor(perfil)),
                ],
            )
            .await
            .map(drop)
    }
}

// ------------------------------------------------------------------ gastos

const GUARDAR_GASTO: &str = "CREATE type::record('gasto', $id) CONTENT $gasto RETURN NONE;";
const GASTOS_ENTRE: &str = "\
    SELECT meta::id(id) AS id, usuario, fecha, concepto, monto, categoria, origen \
    FROM gasto \
    WHERE usuario = $usuario AND fecha >= $desde AND fecha <= $hasta \
    ORDER BY fecha, id;";

fn gasto_de_cbor(valor: &Value) -> Result<Gasto, ErrorRepositorio> {
    let r = Registro::de(valor)?;
    let concepto = Concepto::new(r.texto("concepto")?).map_err(|e| invalido("un concepto")(&e))?;
    Gasto::new(
        GastoId::new(r.uuid("id")?),
        UsuarioId::new(r.uuid("usuario")?),
        DatosGasto {
            fecha: r.fecha("fecha")?,
            concepto,
            monto: r.colones("monto")?,
            categoria: codigos::a_categoria(r.texto("categoria")?)?,
            origen: codigos::a_origen(r.texto("origen")?)?,
        },
    )
    .map_err(|e| invalido("un gasto")(&e))
}

impl<H: EnvioHttp> RepositorioGastos for Surreal<H> {
    async fn guardar(&self, gasto: &Gasto) -> Result<(), ErrorRepositorio> {
        let datos = mapa([
            ("usuario", usuario(gasto.usuario())),
            ("fecha", cbor::fecha(gasto.fecha())),
            ("concepto", cbor::texto(gasto.concepto().como_str())),
            ("monto", cbor::colones(gasto.monto())),
            (
                "categoria",
                cbor::texto(codigos::categoria(gasto.categoria())),
            ),
            ("origen", cbor::texto(codigos::origen(gasto.origen()))),
        ]);
        self.cliente
            .consulta(
                GUARDAR_GASTO,
                [("id", cbor::uuid(gasto.id().uuid())), ("gasto", datos)],
            )
            .await
            .map(drop)
    }

    async fn entre(
        &self,
        id: UsuarioId,
        desde: NaiveDate,
        hasta: NaiveDate,
    ) -> Result<Vec<Gasto>, ErrorRepositorio> {
        self.seleccionar(
            GASTOS_ENTRE,
            [
                ("usuario", usuario(id)),
                ("desde", cbor::fecha(desde)),
                ("hasta", cbor::fecha(hasta)),
            ],
        )
        .await?
        .iter()
        .map(gasto_de_cbor)
        .collect()
    }

    async fn borrar(&self, dueno: UsuarioId, id: GastoId) -> Result<bool, ErrorRepositorio> {
        self.borrar_de("gasto", dueno, id.uuid()).await
    }
}

// ------------------------------------------------------------------ extras

const GUARDAR_EXTRA: &str = "CREATE type::record('extra', $id) CONTENT $extra RETURN NONE;";
const EXTRAS_DE: &str = "\
    SELECT meta::id(id) AS id, usuario, fecha, tipo, bruto \
    FROM extra WHERE usuario = $usuario ORDER BY fecha, id;";

fn extra_de_cbor(valor: &Value) -> Result<Extra, ErrorRepositorio> {
    let r = Registro::de(valor)?;
    let bruto = r.colones("bruto")?;
    if !bruto.es_positivo() {
        return Err(error("una extra guardada tiene bruto no positivo"));
    }
    Ok(Extra::reconstruir(
        ExtraId::new(r.uuid("id")?),
        UsuarioId::new(r.uuid("usuario")?),
        r.fecha("fecha")?,
        codigos::a_tipo_extra(r.texto("tipo")?)?,
        bruto,
    ))
}

impl<H: EnvioHttp> RepositorioExtras for Surreal<H> {
    async fn guardar(&self, extra: &Extra) -> Result<(), ErrorRepositorio> {
        let datos = mapa([
            ("usuario", usuario(extra.usuario())),
            ("fecha", cbor::fecha(extra.fecha())),
            ("tipo", cbor::texto(codigos::tipo_extra(extra.tipo()))),
            ("bruto", cbor::colones(extra.bruto())),
        ]);
        self.cliente
            .consulta(
                GUARDAR_EXTRA,
                [("id", cbor::uuid(extra.id().uuid())), ("extra", datos)],
            )
            .await
            .map(drop)
    }

    async fn de_usuario(&self, id: UsuarioId) -> Result<Vec<Extra>, ErrorRepositorio> {
        self.seleccionar(EXTRAS_DE, [("usuario", usuario(id))])
            .await?
            .iter()
            .map(extra_de_cbor)
            .collect()
    }

    async fn borrar(&self, dueno: UsuarioId, id: ExtraId) -> Result<bool, ErrorRepositorio> {
        self.borrar_de("extra", dueno, id.uuid()).await
    }
}

// ------------------------------------------------------------------- fondo

const GUARDAR_MOVIMIENTO: &str =
    "CREATE type::record('movimiento_fondo', $id) CONTENT $movimiento RETURN NONE;";
const MOVIMIENTOS_DE: &str = "\
    SELECT meta::id(id) AS id, usuario, fecha, concepto, monto, tipo, plan \
    FROM movimiento_fondo WHERE usuario = $usuario ORDER BY fecha, id;";

fn movimiento_a_cbor(m: &MovimientoFondo) -> Value {
    let mut campos = vec![
        ("usuario", usuario(m.usuario())),
        ("fecha", cbor::fecha(m.fecha())),
        ("concepto", cbor::texto(m.concepto().como_str())),
        ("monto", cbor::colones(m.monto())),
    ];
    match m.movimiento() {
        Movimiento::Aporte => campos.push(("tipo", cbor::texto("APORTE"))),
        Movimiento::Reposicion => campos.push(("tipo", cbor::texto("REPOSICION"))),
        Movimiento::Retiro(plan) => {
            campos.push(("tipo", cbor::texto("RETIRO")));
            campos.push((
                "plan",
                mapa([
                    ("nivel", cbor::texto(codigos::nivel(plan.nivel))),
                    ("a_reponer", cbor::colones(plan.a_reponer)),
                    ("meses", cbor::entero(i64::from(plan.meses))),
                    ("cuota_mensual", cbor::colones(plan.cuota_mensual)),
                ]),
            ));
        }
    }
    mapa_de(campos)
}

fn movimiento_de_cbor(valor: &Value) -> Result<MovimientoFondo, ErrorRepositorio> {
    let r = Registro::de(valor)?;
    let monto = r.colones("monto")?;
    if !monto.es_positivo() {
        return Err(error(
            "un movimiento del fondo guardado tiene monto no positivo",
        ));
    }
    let movimiento = match (r.texto("tipo")?, r.registro_opcional("plan")?) {
        ("APORTE", None) => Movimiento::Aporte,
        ("REPOSICION", None) => Movimiento::Reposicion,
        ("RETIRO", Some(p)) => Movimiento::Retiro(PlanReposicion {
            nivel: codigos::a_nivel(p.texto("nivel")?)?,
            sacado: monto,
            a_reponer: p.colones("a_reponer")?,
            meses: u8::try_from(p.entero("meses")?)
                .map_err(|_| error("meses de reposición fuera de rango"))?,
            cuota_mensual: p.colones("cuota_mensual")?,
        }),
        (tipo, plan) => {
            return Err(error(format!(
                "movimiento del fondo incoherente: tipo {tipo}, con plan: {}",
                plan.is_some()
            )));
        }
    };
    let concepto = Concepto::new(r.texto("concepto")?).map_err(|e| invalido("un concepto")(&e))?;
    Ok(MovimientoFondo::reconstruir(
        MovimientoFondoId::new(r.uuid("id")?),
        UsuarioId::new(r.uuid("usuario")?),
        r.fecha("fecha")?,
        concepto,
        monto,
        movimiento,
    ))
}

impl<H: EnvioHttp> RepositorioFondo for Surreal<H> {
    async fn guardar(&self, movimiento: &MovimientoFondo) -> Result<(), ErrorRepositorio> {
        self.cliente
            .consulta(
                GUARDAR_MOVIMIENTO,
                [
                    ("id", cbor::uuid(movimiento.id().uuid())),
                    ("movimiento", movimiento_a_cbor(movimiento)),
                ],
            )
            .await
            .map(drop)
    }

    async fn de_usuario(&self, id: UsuarioId) -> Result<Vec<MovimientoFondo>, ErrorRepositorio> {
        self.seleccionar(MOVIMIENTOS_DE, [("usuario", usuario(id))])
            .await?
            .iter()
            .map(movimiento_de_cbor)
            .collect()
    }

    async fn borrar(
        &self,
        dueno: UsuarioId,
        id: MovimientoFondoId,
    ) -> Result<bool, ErrorRepositorio> {
        self.borrar_de("movimiento_fondo", dueno, id.uuid()).await
    }
}

// --------------------------------------------------------------- quincenas

const GUARDAR_QUINCENA: &str =
    "UPSERT type::record('quincena', [$usuario, $pago]) CONTENT $quincena RETURN NONE;";
const LEER_QUINCENA: &str =
    "SELECT * OMIT id FROM quincena WHERE usuario = $usuario AND pago = $pago;";

fn lineas_a_cbor(lineas: &[Linea]) -> Value {
    Value::Array(
        lineas
            .iter()
            .map(|l| {
                mapa([
                    ("concepto", cbor::texto(l.concepto().como_str())),
                    ("monto", cbor::colones(l.monto())),
                    ("hecho", cbor::booleano(l.hecho())),
                ])
            })
            .collect(),
    )
}

fn lineas_de_cbor(r: Registro<'_>, lado: &str) -> Result<Vec<Linea>, ErrorRepositorio> {
    r.lista(lado)?
        .iter()
        .map(|l| {
            let l = Registro::de(l)?;
            let concepto =
                Concepto::new(l.texto("concepto")?).map_err(|e| invalido("un concepto")(&e))?;
            Linea::reconstruir(concepto, l.colones("monto")?, l.booleano("hecho")?)
                .map_err(|e| invalido("una línea de quincena")(&e))
        })
        .collect()
}

fn quincena_de_cbor(valor: &Value) -> Result<Quincena, ErrorRepositorio> {
    let r = Registro::de(valor)?;
    Ok(Quincena::new(
        UsuarioId::new(r.uuid("usuario")?),
        r.fecha("pago")?,
        lineas_de_cbor(r, "ingresos")?,
        lineas_de_cbor(r, "pagos")?,
    ))
}

impl<H: EnvioHttp> RepositorioQuincenas for Surreal<H> {
    async fn guardar(&self, q: &Quincena) -> Result<(), ErrorRepositorio> {
        let datos = mapa([
            ("usuario", usuario(q.usuario())),
            ("pago", cbor::fecha(q.pago())),
            ("ingresos", lineas_a_cbor(q.ingresos())),
            ("pagos", lineas_a_cbor(q.pagos())),
        ]);
        self.cliente
            .consulta(
                GUARDAR_QUINCENA,
                [
                    ("usuario", usuario(q.usuario())),
                    ("pago", cbor::fecha(q.pago())),
                    ("quincena", datos),
                ],
            )
            .await
            .map(drop)
    }

    async fn de(
        &self,
        id: UsuarioId,
        pago: NaiveDate,
    ) -> Result<Option<Quincena>, ErrorRepositorio> {
        let filas = self
            .seleccionar(
                LEER_QUINCENA,
                [("usuario", usuario(id)), ("pago", cbor::fecha(pago))],
            )
            .await?;
        match filas.as_slice() {
            [] => Ok(None),
            [fila] => quincena_de_cbor(fila).map(Some),
            _ => Err(error(
                "hay más de una quincena para el mismo usuario y pago",
            )),
        }
    }
}
