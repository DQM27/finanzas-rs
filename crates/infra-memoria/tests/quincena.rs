//! La quincena y el mes de punta a punta, con los adaptadores en memoria y el
//! presupuesto real de la primera quincena de noviembre de 2026.

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use finanzas_aplicacion::casos_de_uso::quincena::{
        LineaPedida, Presupuestos, ResumenMes, ResumenQuincena, SolicitudMarca,
        SolicitudPresupuesto,
    };
    use finanzas_aplicacion::errores::ErrorAplicacion;
    use finanzas_aplicacion::puertos::{RepositorioExtras, RepositorioPerfiles};
    use finanzas_contrato::{DANIEL, OTRO, perfil};
    use finanzas_dominio::base::dinero::Colones;
    use finanzas_dominio::extras::{Extra, ExtraId, TipoExtra};
    use finanzas_dominio::quincena::{ErrorQuincena, Lado};
    use finanzas_infra_memoria::{
        ExtrasEnMemoria, PerfilesEnMemoria, QuincenasEnMemoria, RelojFijo,
    };
    use uuid::Uuid;

    fn f(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, d).unwrap()
    }

    fn l(concepto: &str, monto: i64) -> LineaPedida {
        LineaPedida {
            concepto: concepto.to_owned(),
            monto,
        }
    }

    struct Mundo {
        perfiles: PerfilesEnMemoria,
        quincenas: QuincenasEnMemoria,
        extras: ExtrasEnMemoria,
        reloj: RelojFijo,
    }

    impl Mundo {
        /// Hoy es 15 de noviembre: se está gastando el pago del 13. Hay una
        /// extra de ₡120 000 trabajada el 31 de octubre (se paga el 13) y una
        /// nocturna del 5 de noviembre (se paga el 30).
        fn new() -> Self {
            let m = Self {
                perfiles: PerfilesEnMemoria::default(),
                quincenas: QuincenasEnMemoria::default(),
                extras: ExtrasEnMemoria::default(),
                reloj: RelojFijo(f(15)),
            };
            pollster::block_on(m.perfiles.guardar(&perfil(DANIEL, 5_000))).unwrap();
            for (n, fecha, tipo, bruto) in [
                (
                    1,
                    NaiveDate::from_ymd_opt(2026, 10, 31).unwrap(),
                    TipoExtra::Otra,
                    120_000,
                ),
                (2, f(5), TipoExtra::Nocturna, 25_800),
            ] {
                let extra = Extra::reconstruir(
                    ExtraId::new(Uuid::from_u128(n)),
                    DANIEL,
                    fecha,
                    tipo,
                    Colones::new(bruto),
                );
                pollster::block_on(m.extras.guardar(&extra)).unwrap();
            }
            m
        }

        fn casos(
            &self,
        ) -> Presupuestos<'_, PerfilesEnMemoria, QuincenasEnMemoria, ExtrasEnMemoria, RelojFijo>
        {
            Presupuestos::new(&self.perfiles, &self.quincenas, &self.extras, &self.reloj)
        }

        fn guardar(
            &self,
            pago: NaiveDate,
            pagos: Vec<LineaPedida>,
        ) -> Result<ResumenQuincena, ErrorAplicacion> {
            pollster::block_on(self.casos().guardar(SolicitudPresupuesto {
                usuario: DANIEL,
                pago,
                ingresos: vec![l("papá", 65_000), l("saldo para operar", 14_300)],
                pagos,
            }))
        }

        fn marcar(&self, lado: Lado, indice: usize) -> Result<ResumenQuincena, ErrorAplicacion> {
            pollster::block_on(self.casos().marcar(SolicitudMarca {
                usuario: DANIEL,
                pago: f(13),
                lado,
                indice,
                hecho: true,
            }))
        }

        fn consultar(&self, pago: Option<NaiveDate>) -> Result<ResumenQuincena, ErrorAplicacion> {
            pollster::block_on(self.casos().consultar(DANIEL, pago))
        }

        fn mes(&self, mes: Option<(i32, u32)>) -> ResumenMes {
            pollster::block_on(self.casos().mes(DANIEL, mes)).unwrap()
        }
    }

    fn pagos_q1() -> Vec<LineaPedida> {
        vec![
            l("moto", 104_000),
            l("comida", 100_000),
            l("hijastra", 50_000),
            l("gasolina + caja chica", 40_000),
            l("internet", 23_000),
            l("luz", 20_000),
            l("línea", 20_000),
            l("claude", 11_000),
            l("ahorro moto", 6_000),
        ]
    }

    #[test]
    fn presupuesto_igual_a_la_hoja() {
        let m = Mundo::new();
        let r = m.guardar(f(13), pagos_q1()).unwrap();
        assert_eq!(
            r.automaticos.salario(),
            Colones::new(232_225),
            "salario neto"
        );
        assert_eq!(
            r.automaticos.extras(),
            Colones::new(103_404),
            "extras del 13"
        );
        assert!(r.automaticos.llegaron(), "el 15 ya llegó el pago del 13");
        assert_eq!(
            r.balance.total_ingresos(),
            Colones::new(414_929),
            "ingresos"
        );
        assert_eq!(r.balance.total_pagos(), Colones::new(374_000), "pagos");
        assert_eq!(r.balance.sobra(), Colones::new(40_929), "sobra de la Q1");
        assert_eq!(r.avance, (0, 9), "nada pagado todavía");
        assert_eq!(
            r.quincena.pagos()[3].concepto().como_str(),
            "GASOLINA + CAJA CHICA",
            "el dominio normaliza el concepto"
        );
    }

    #[test]
    fn sin_presupuesto_igual_se_ve_lo_que_entra_solo() {
        let m = Mundo::new();
        let hoy = m.consultar(None).unwrap();
        assert_eq!(hoy.quincena.pago(), f(13), "el 15 se gasta el pago del 13");
        assert!(!hoy.presupuestada, "sin presupuesto todavía");
        assert_eq!(
            hoy.balance.sobra(),
            Colones::new(335_629),
            "salario + extras"
        );

        m.guardar(f(13), pagos_q1()).unwrap();
        assert!(m.consultar(None).unwrap().presupuestada, "ya tiene");
        let q2 = m.consultar(Some(f(30))).unwrap();
        assert!(!q2.presupuestada, "la Q2 no tiene presupuesto");
        assert_eq!(q2.automaticos.extras(), Colones::new(22_232), "la nocturna");
        assert!(!q2.automaticos.llegaron(), "el 30 todavía no llega");
        assert_eq!(
            m.consultar(Some(f(15))),
            Err(ErrorAplicacion::NoEsDiaDePago(f(15))),
            "el 15 no es día de pago"
        );
    }

    #[test]
    fn el_mes_trae_sus_dos_quincenas() {
        let m = Mundo::new();
        m.guardar(f(13), pagos_q1()).unwrap();
        let nov = m.mes(None);
        assert_eq!((nov.anio, nov.mes), (2026, 11), "sin mes, el de hoy");
        let pagos: Vec<NaiveDate> = nov.quincenas.iter().map(|q| q.quincena.pago()).collect();
        assert_eq!(pagos, vec![f(13), f(30)], "Q1 y Q2");
        assert_eq!(nov.balance.salario(), Colones::new(464_450), "dos salarios");
        assert_eq!(
            nov.balance.total_pagos(),
            Colones::new(374_000),
            "sólo la Q1 tiene pagos"
        );
        assert_eq!(
            nov.balance.sobra(),
            Colones::new(40_929 + 232_225 + 22_232),
            "sobra de la Q1 más todo lo de la Q2"
        );
        let dic = m.mes(Some((2026, 12)));
        assert!(
            dic.quincenas.is_empty(),
            "el calendario no llega a diciembre"
        );
        assert_eq!(dic.balance.sobra(), Colones::new(0), "nada");
    }

    #[test]
    fn marcar_pagos_y_rehacer_el_presupuesto_sin_perderlos() {
        let m = Mundo::new();
        m.guardar(f(13), pagos_q1()).unwrap();
        m.marcar(Lado::Pago, 0).unwrap(); // moto
        let r = m.marcar(Lado::Pago, 4).unwrap(); // internet
        assert_eq!(
            r.balance.falta_por_pagar(),
            Colones::new(247_000),
            "374 000 − 104 000 − 23 000"
        );
        assert_eq!(r.avance, (2, 9), "2 de 9");
        assert_eq!(
            m.marcar(Lado::Pago, 9),
            Err(ErrorAplicacion::Quincena(ErrorQuincena::LineaInexistente)),
            "fuera de rango"
        );

        // Se agrega una línea al inicio y la moto cambia de monto: lo marcado se queda.
        let mut nuevos = vec![l("refri", 30_000)];
        nuevos.extend(pagos_q1());
        let r = m.guardar(f(13), nuevos).unwrap();
        assert_eq!(r.avance, (2, 10), "siguen 2 pagos hechos");
        assert_eq!(
            r.balance.falta_por_pagar(),
            Colones::new(277_000),
            "lo pendiente de antes más el refri"
        );
        assert_eq!(m.consultar(Some(f(13))), Ok(r), "quedó guardado así");
    }

    #[test]
    fn solo_se_presupuesta_un_dia_de_pago() {
        let m = Mundo::new();
        assert_eq!(
            m.guardar(f(15), pagos_q1()),
            Err(ErrorAplicacion::NoEsDiaDePago(f(15))),
            "el 15 de noviembre no es pago en el calendario de 2026"
        );
        assert_eq!(
            m.marcar(Lado::Pago, 0),
            Err(ErrorAplicacion::QuincenaNoEncontrada(f(13))),
            "no se marca lo que no existe"
        );
    }

    #[test]
    fn la_quincena_es_de_cada_usuario() {
        let m = Mundo::new();
        m.guardar(f(13), pagos_q1()).unwrap();
        pollster::block_on(m.perfiles.guardar(&perfil(OTRO, 5_000))).unwrap();
        let ajena = pollster::block_on(m.casos().consultar(OTRO, Some(f(13)))).unwrap();
        assert!(
            !ajena.presupuestada,
            "el otro no ve el presupuesto de Daniel"
        );
        assert_eq!(ajena.automaticos.extras(), Colones::new(0), "ni sus extras");
    }
}
