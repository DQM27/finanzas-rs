//! Los casos de uso de punta a punta con los adaptadores en memoria.
//!
//! Usan la configuración real de noviembre de 2026 y comprueban que la app
//! llega a los mismos números que la hoja de cálculo.

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use finanzas_aplicacion::casos_de_uso::extras::{RegistrarExtra, SolicitudExtra};
    use finanzas_aplicacion::casos_de_uso::gastos::ConsultarCajaChica;
    use finanzas_aplicacion::casos_de_uso::gastos::{RegistrarGasto, SolicitudGasto};
    use finanzas_aplicacion::errores::ErrorAplicacion;
    use finanzas_dominio::base::dinero::Colones;
    use finanzas_dominio::base::proporcion::Proporcion;
    use finanzas_dominio::base::usuario::UsuarioId;
    use finanzas_dominio::calendario::{CalendarioCortes, Corte};
    use finanzas_dominio::extras::{ErrorExtra, TipoExtra};
    use finanzas_dominio::gastos::caja_chica::{CajaChica, NivelCaja};
    use finanzas_dominio::gastos::{Categoria, Origen};
    use finanzas_dominio::perfil::{Perfil, PerfilSalarial, TarifasExtra};
    use finanzas_infra_memoria::{
        ExtrasEnMemoria, GastosEnMemoria, IdsSecuenciales, PerfilesEnMemoria, RelojFijo,
    };
    use uuid::Uuid;

    fn f(a: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(a, m, d).unwrap()
    }

    const DANIEL: UsuarioId = UsuarioId::new(Uuid::from_u128(0xDA));
    const OTRO: UsuarioId = UsuarioId::new(Uuid::from_u128(0x07));

    fn perfil(usuario: UsuarioId) -> Perfil {
        Perfil::new(
            usuario,
            PerfilSalarial::new(
                Colones::new(293_000),
                Proporcion::desde_diezmilesimas(8617),
                Colones::new(20_253),
            )
            .unwrap(),
            TarifasExtra::new(
                Colones::new(25_800),
                Colones::new(19_800),
                Colones::new(40_000),
                Colones::new(19_500),
            )
            .unwrap(),
            CajaChica::new(Colones::new(5_000)).unwrap(),
            CalendarioCortes::new(vec![
                Corte::new(
                    f(2026, 10, 2),
                    f(2026, 10, 16),
                    f(2026, 10, 21),
                    f(2026, 10, 30),
                )
                .unwrap(),
                Corte::new(
                    f(2026, 10, 17),
                    f(2026, 11, 1),
                    f(2026, 11, 4),
                    f(2026, 11, 13),
                )
                .unwrap(),
                Corte::new(
                    f(2026, 11, 2),
                    f(2026, 11, 13),
                    f(2026, 11, 17),
                    f(2026, 11, 30),
                )
                .unwrap(),
                Corte::new(
                    f(2026, 11, 14),
                    f(2026, 12, 1),
                    f(2026, 12, 7),
                    f(2026, 12, 15),
                )
                .unwrap(),
            ])
            .unwrap(),
        )
    }

    fn gasto(
        usuario: UsuarioId,
        dia: u32,
        concepto: &str,
        monto: i64,
        origen: Origen,
    ) -> SolicitudGasto {
        SolicitudGasto {
            usuario,
            fecha: f(2026, 11, dia),
            concepto: concepto.to_owned(),
            monto,
            categoria: Categoria::Otro,
            origen,
        }
    }

    #[test]
    fn anotar_el_jugo_deja_la_caja_chica_en_1050() {
        let perfiles = PerfilesEnMemoria::default();
        perfiles.poner(perfil(DANIEL));
        let gastos = GastosEnMemoria::default();
        let ids = IdsSecuenciales::default();
        let reloj = RelojFijo(f(2026, 11, 10));
        let registrar = RegistrarGasto::new(&perfiles, &gastos, &ids, &reloj);

        pollster::block_on(registrar.ejecutar(gasto(
            DANIEL,
            7,
            "pasajes",
            1_000,
            Origen::Planilla,
        )))
        .unwrap();
        let r = pollster::block_on(registrar.ejecutar(gasto(
            DANIEL,
            9,
            " jugo  nieto",
            2_950,
            Origen::CuentaGastos,
        )))
        .unwrap();

        assert_eq!(
            r.gasto.concepto().como_str(),
            "JUGO NIETO",
            "concepto normalizado"
        );
        assert_eq!(r.caja.estado.queda, Colones::new(1_050), "quedan 1 050");
        assert_eq!(r.caja.estado.nivel, NivelCaja::Baja, "caja baja");
        assert_eq!(
            (r.caja.desde, r.caja.hasta),
            (f(2026, 10, 30), f(2026, 11, 12)),
            "periodo"
        );
    }

    #[test]
    fn cada_usuario_ve_solo_su_caja_chica() {
        let perfiles = PerfilesEnMemoria::default();
        perfiles.poner(perfil(DANIEL));
        perfiles.poner(perfil(OTRO));
        let gastos = GastosEnMemoria::default();
        let ids = IdsSecuenciales::default();
        let reloj = RelojFijo(f(2026, 11, 10));
        pollster::block_on(
            RegistrarGasto::new(&perfiles, &gastos, &ids, &reloj).ejecutar(gasto(
                OTRO,
                9,
                "almuerzo",
                4_000,
                Origen::Planilla,
            )),
        )
        .unwrap();

        let caja = pollster::block_on(
            ConsultarCajaChica::new(&perfiles, &gastos, &reloj).ejecutar(DANIEL),
        )
        .unwrap();
        assert_eq!(
            caja.estado.queda,
            Colones::new(5_000),
            "el gasto del otro no me toca"
        );
    }

    #[test]
    fn la_extra_del_14_se_paga_el_15_de_diciembre() {
        let perfiles = PerfilesEnMemoria::default();
        perfiles.poner(perfil(DANIEL));
        let extras = ExtrasEnMemoria::default();
        let ids = IdsSecuenciales::default();
        let registrar = RegistrarExtra::new(&perfiles, &extras, &ids);

        let r = pollster::block_on(registrar.ejecutar(SolicitudExtra {
            usuario: DANIEL,
            fecha: f(2026, 11, 14),
            tipo: TipoExtra::Nocturna,
            monto: None,
        }))
        .unwrap();
        assert_eq!(r.neto, Colones::new(22_232), "neto de la nocturna");
        assert_eq!(
            r.fecha_pago,
            Some(f(2026, 12, 15)),
            "cae en el pago del 15 de diciembre"
        );
    }

    #[test]
    fn errores_del_dominio_y_usuario_sin_perfil() {
        let perfiles = PerfilesEnMemoria::default();
        let extras = ExtrasEnMemoria::default();
        let ids = IdsSecuenciales::default();
        let registrar = RegistrarExtra::new(&perfiles, &extras, &ids);
        let solicitud = SolicitudExtra {
            usuario: DANIEL,
            fecha: f(2026, 11, 14),
            tipo: TipoExtra::Otra,
            monto: None,
        };

        assert_eq!(
            pollster::block_on(registrar.ejecutar(solicitud)),
            Err(ErrorAplicacion::PerfilNoEncontrado),
            "sin perfil"
        );
        perfiles.poner(perfil(DANIEL));
        assert_eq!(
            pollster::block_on(registrar.ejecutar(solicitud)),
            Err(ErrorAplicacion::Extra(ErrorExtra::FaltaMonto)),
            "OTRA sin monto: lo decide el dominio"
        );
    }
}
