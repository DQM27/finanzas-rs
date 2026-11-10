//! La quincena de punta a punta, con los adaptadores en memoria y el
//! presupuesto real de la primera quincena de noviembre de 2026.

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use finanzas_aplicacion::casos_de_uso::quincena::{
        ConsultarQuincena, GuardarPresupuesto, LineaPedida, MarcarLinea, ResumenQuincena,
        SolicitudMarca, SolicitudPresupuesto,
    };
    use finanzas_aplicacion::errores::ErrorAplicacion;
    use finanzas_aplicacion::puertos::RepositorioPerfiles;
    use finanzas_contrato::{DANIEL, OTRO, perfil};
    use finanzas_dominio::dinero::Colones;
    use finanzas_dominio::quincena::{ErrorQuincena, Lado};
    use finanzas_infra_memoria::{PerfilesEnMemoria, QuincenasEnMemoria, RelojFijo};

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
        reloj: RelojFijo,
    }

    impl Mundo {
        /// Hoy es 15 de noviembre: se está gastando el pago del 13.
        fn new() -> Self {
            let m = Self {
                perfiles: PerfilesEnMemoria::default(),
                quincenas: QuincenasEnMemoria::default(),
                reloj: RelojFijo(f(15)),
            };
            pollster::block_on(m.perfiles.guardar(&perfil(DANIEL, 5_000))).unwrap();
            m
        }

        fn guardar(
            &self,
            pago: NaiveDate,
            pagos: Vec<LineaPedida>,
        ) -> Result<ResumenQuincena, ErrorAplicacion> {
            pollster::block_on(
                GuardarPresupuesto::new(&self.perfiles, &self.quincenas).ejecutar(
                    SolicitudPresupuesto {
                        usuario: DANIEL,
                        pago,
                        ingresos: vec![
                            l("salario base", 232_225),
                            l("extras", 103_404),
                            l("papá", 65_000),
                            l("saldo para operar", 14_300),
                        ],
                        pagos,
                    },
                ),
            )
        }

        fn marcar(&self, lado: Lado, indice: usize) -> Result<ResumenQuincena, ErrorAplicacion> {
            pollster::block_on(MarcarLinea::new(&self.quincenas).ejecutar(SolicitudMarca {
                usuario: DANIEL,
                pago: f(13),
                lado,
                indice,
                hecho: true,
            }))
        }

        fn consultar(&self, pago: Option<NaiveDate>) -> Option<ResumenQuincena> {
            pollster::block_on(
                ConsultarQuincena::new(&self.perfiles, &self.quincenas, &self.reloj)
                    .ejecutar(DANIEL, pago),
            )
            .unwrap()
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
        assert_eq!(r.total_ingresos, Colones::new(414_929), "ingresos");
        assert_eq!(r.total_pagos, Colones::new(374_000), "pagos");
        assert_eq!(r.sobra, Colones::new(40_929), "sobra de la Q1");
        assert_eq!(r.avance, (0, 9), "nada pagado todavía");
        assert_eq!(
            r.quincena.pagos()[3].concepto().como_str(),
            "GASOLINA + CAJA CHICA",
            "el dominio normaliza el concepto"
        );
    }

    #[test]
    fn sin_fecha_muestra_la_quincena_que_se_esta_gastando() {
        let m = Mundo::new();
        assert_eq!(m.consultar(None), None, "sin presupuesto todavía");
        m.guardar(f(13), pagos_q1()).unwrap();
        let hoy = m.consultar(None).unwrap();
        assert_eq!(hoy.quincena.pago(), f(13), "el 15 se gasta el pago del 13");
        assert_eq!(m.consultar(Some(f(30))), None, "la Q2 no tiene presupuesto");
    }

    #[test]
    fn marcar_pagos_y_rehacer_el_presupuesto_sin_perderlos() {
        let m = Mundo::new();
        m.guardar(f(13), pagos_q1()).unwrap();
        m.marcar(Lado::Pago, 0).unwrap(); // moto
        let r = m.marcar(Lado::Pago, 4).unwrap(); // internet
        assert_eq!(
            r.falta_por_pagar,
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
            r.falta_por_pagar,
            Colones::new(277_000),
            "lo pendiente de antes más el refri"
        );
        assert_eq!(m.consultar(Some(f(13))), Some(r), "quedó guardado así");
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
        let ajena = pollster::block_on(
            ConsultarQuincena::new(&m.perfiles, &m.quincenas, &m.reloj).ejecutar(OTRO, Some(f(13))),
        );
        assert_eq!(ajena, Ok(None), "el otro no ve la quincena de Daniel");
    }
}
