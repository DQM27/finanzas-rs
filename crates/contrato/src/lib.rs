//! Pruebas de contrato de los puertos.
//!
//! Cada adaptador (memoria, SurrealDB) corre exactamente estas mismas pruebas.
//! Así un repositorio nuevo no puede comportarse distinto sin que se note: si
//! pasa aquí, los casos de uso no ven la diferencia.
//!
//! Las funciones reciben un repositorio vacío y fallan con un `assert` en el
//! primer incumplimiento.

#![expect(
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "crate de pruebas: fallar con un pánico es justo su trabajo"
)]
#![expect(
    clippy::future_not_send,
    reason = "los puertos no exigen `Send` (WebAssembly), así que estos futuros tampoco"
)]

use chrono::NaiveDate;
use finanzas_aplicacion::puertos::{RepositorioExtras, RepositorioGastos, RepositorioPerfiles};
use finanzas_dominio::caja_chica::CajaChica;
use finanzas_dominio::calendario::{CalendarioCortes, Corte};
use finanzas_dominio::dinero::Colones;
use finanzas_dominio::extra::{Extra, ExtraId, TipoExtra};
use finanzas_dominio::gasto::{Categoria, DatosGasto, Gasto, GastoId, Origen};
use finanzas_dominio::perfil::{Perfil, PerfilSalarial, TarifasExtra};
use finanzas_dominio::proporcion::Proporcion;
use finanzas_dominio::texto::Concepto;
use finanzas_dominio::usuario::UsuarioId;
use uuid::Uuid;

/// El usuario principal de las pruebas.
pub const DANIEL: UsuarioId = UsuarioId::new(Uuid::from_u128(0xDA));
/// Otro usuario, para comprobar el aislamiento.
pub const OTRO: UsuarioId = UsuarioId::new(Uuid::from_u128(0x07));

fn f(a: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(a, m, d).expect("fecha válida")
}

fn corte(inicio: NaiveDate, fin: NaiveDate, reportar: NaiveDate, pago: NaiveDate) -> Corte {
    Corte::new(inicio, fin, reportar, pago).expect("corte válido")
}

/// Un perfil completo con números reales de noviembre de 2026.
pub fn perfil(usuario: UsuarioId, tope_caja: i64) -> Perfil {
    Perfil::new(
        usuario,
        PerfilSalarial::new(
            Colones::new(293_000),
            Proporcion::desde_diezmilesimas(8617),
            Colones::new(20_253),
        )
        .expect("perfil salarial válido"),
        TarifasExtra::new(
            Colones::new(25_800),
            Colones::new(19_800),
            Colones::new(40_000),
            Colones::new(19_500),
        )
        .expect("tarifas válidas"),
        CajaChica::new(Colones::new(tope_caja)).expect("tope válido"),
        CalendarioCortes::new(vec![
            corte(
                f(2026, 10, 17),
                f(2026, 11, 1),
                f(2026, 11, 5),
                f(2026, 11, 13),
            ),
            corte(
                f(2026, 11, 2),
                f(2026, 11, 16),
                f(2026, 11, 17),
                f(2026, 11, 30),
            ),
        ])
        .expect("calendario válido"),
    )
}

fn gasto(n: u128, usuario: UsuarioId, fecha: NaiveDate, concepto: &str, monto: i64) -> Gasto {
    gasto_con(
        n,
        usuario,
        fecha,
        concepto,
        monto,
        Categoria::Otro,
        Origen::Planilla,
    )
}

fn gasto_con(
    n: u128,
    usuario: UsuarioId,
    fecha: NaiveDate,
    concepto: &str,
    monto: i64,
    categoria: Categoria,
    origen: Origen,
) -> Gasto {
    Gasto::new(
        GastoId::new(Uuid::from_u128(n)),
        usuario,
        DatosGasto {
            fecha,
            concepto: Concepto::new(concepto).expect("concepto válido"),
            monto: Colones::new(monto),
            categoria,
            origen,
        },
    )
    .expect("gasto válido")
}

fn extra(n: u128, usuario: UsuarioId, fecha: NaiveDate, tipo: TipoExtra, bruto: i64) -> Extra {
    Extra::reconstruir(
        ExtraId::new(Uuid::from_u128(n)),
        usuario,
        fecha,
        tipo,
        Colones::new(bruto),
    )
}

/// Contrato de [`RepositorioPerfiles`].
pub async fn perfiles<R: RepositorioPerfiles>(repo: &R) {
    assert_eq!(
        repo.perfil(DANIEL).await,
        Ok(None),
        "un usuario sin perfil no tiene perfil"
    );

    let original = perfil(DANIEL, 5_000);
    assert_eq!(repo.guardar(&original).await, Ok(()), "guarda el perfil");
    assert_eq!(
        repo.perfil(DANIEL).await,
        Ok(Some(original)),
        "devuelve el perfil completo, tal cual se guardó"
    );

    let cambiado = perfil(DANIEL, 6_000);
    assert_eq!(repo.guardar(&cambiado).await, Ok(()), "guarda otra vez");
    assert_eq!(
        repo.perfil(DANIEL).await,
        Ok(Some(cambiado)),
        "guardar otra vez reemplaza, no duplica"
    );
    assert_eq!(
        repo.perfil(OTRO).await,
        Ok(None),
        "el perfil de un usuario no lo ve otro"
    );
}

/// Contrato de [`RepositorioGastos`].
pub async fn gastos<R: RepositorioGastos>(repo: &R) {
    let jugo = gasto(2, DANIEL, f(2026, 11, 9), "JUGO NIETO", 2_950);
    let pasajes = gasto(1, DANIEL, f(2026, 11, 9), "PASAJES", 1_000);
    let antes = gasto(3, DANIEL, f(2026, 10, 31), "ANTES DEL PERIODO", 500);
    let despues = gasto(4, DANIEL, f(2026, 11, 13), "DESPUÉS DEL PERIODO", 700);
    let ajeno = gasto(5, OTRO, f(2026, 11, 9), "DE OTRA PERSONA", 9_999);
    // Un texto que parece fecha tiene que volver como texto.
    let raro = gasto(6, DANIEL, f(2026, 11, 12), "2026-11-12", 100);
    for g in [&jugo, &pasajes, &antes, &despues, &ajeno, &raro] {
        assert_eq!(repo.guardar(g).await, Ok(()), "guarda {:?}", g.id());
    }

    assert_eq!(
        repo.entre(DANIEL, f(2026, 11, 1), f(2026, 11, 12)).await,
        Ok(vec![pasajes, jugo, raro]),
        "filtra por usuario y por fechas incluidas, ordena por fecha y luego por id"
    );
    assert_eq!(
        repo.entre(DANIEL, f(2026, 12, 1), f(2026, 12, 31)).await,
        Ok(vec![]),
        "un periodo sin gastos da una lista vacía"
    );

    let categorias = [
        Categoria::Comida,
        Categoria::Casa,
        Categoria::Moto,
        Categoria::Salud,
        Categoria::Familia,
        Categoria::Ropa,
        Categoria::Tramites,
        Categoria::Gusto,
        Categoria::Otro,
    ];
    let origenes = [
        Origen::CuentaGastos,
        Origen::Planilla,
        Origen::SobraDelMes,
        Origen::Gusto,
        Origen::Fondo,
    ];
    let mut todos = Vec::new();
    for (i, categoria) in categorias.into_iter().enumerate() {
        let origen = origenes
            .get(i % origenes.len())
            .copied()
            .unwrap_or(Origen::Planilla);
        let n = 100 + u128::try_from(i).expect("índice pequeño");
        let g = gasto_con(n, OTRO, f(2027, 1, 1), "VARIOS", 1, categoria, origen);
        assert_eq!(repo.guardar(&g).await, Ok(()), "guarda {categoria:?}");
        todos.push(g);
    }
    assert_eq!(
        repo.entre(OTRO, f(2027, 1, 1), f(2027, 1, 1)).await,
        Ok(todos),
        "toda categoría y todo origen vuelven igual"
    );
}

/// Contrato de [`RepositorioExtras`].
pub async fn extras<R: RepositorioExtras>(repo: &R) {
    assert_eq!(
        repo.de_usuario(DANIEL).await,
        Ok(vec![]),
        "sin extras, lista vacía"
    );
    let catorce = extra(3, DANIEL, f(2026, 11, 14), TipoExtra::Nocturna, 25_800);
    let once = extra(2, DANIEL, f(2026, 11, 11), TipoExtra::Nocturna, 25_800);
    let especial = extra(1, DANIEL, f(2026, 11, 11), TipoExtra::Especial, 40_000);
    let otra = extra(4, DANIEL, f(2026, 10, 31), TipoExtra::Otra, 120_000);
    let diurna = extra(5, DANIEL, f(2026, 12, 1), TipoExtra::Diurna, 19_800);
    let feriado = extra(6, DANIEL, f(2026, 12, 25), TipoExtra::Feriado, 19_500);
    let ajena = extra(7, OTRO, f(2026, 11, 11), TipoExtra::Nocturna, 25_800);
    for e in [&catorce, &once, &especial, &otra, &diurna, &feriado, &ajena] {
        assert_eq!(repo.guardar(e).await, Ok(()), "guarda {:?}", e.id());
    }
    assert_eq!(
        repo.de_usuario(DANIEL).await,
        Ok(vec![otra, especial, once, catorce, diurna, feriado]),
        "sólo las del usuario, todo tipo intacto, por fecha y luego por id"
    );
    assert_eq!(
        repo.de_usuario(OTRO).await,
        Ok(vec![ajena]),
        "el otro usuario ve sólo lo suyo"
    );
}
