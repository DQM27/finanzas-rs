//! El fondo de emergencia de punta a punta, con los adaptadores en memoria.

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use finanzas_aplicacion::casos_de_uso::fondo::{
        ConsultarFondo, DepositarEnFondo, Deposito, SacarDelFondo, SolicitudDeposito,
        SolicitudSacar,
    };
    use finanzas_aplicacion::errores::ErrorAplicacion;
    use finanzas_dominio::base::dinero::Colones;
    use finanzas_dominio::base::usuario::UsuarioId;
    use finanzas_dominio::fondo::movimientos::{ErrorFondo, EstadoFondo};
    use finanzas_dominio::fondo::{Evaluacion, NivelEmergencia, RechazoRetiro};
    use finanzas_infra_memoria::{FondoEnMemoria, IdsSecuenciales, RelojFijo};
    use uuid::Uuid;

    const DANIEL: UsuarioId = UsuarioId::new(Uuid::from_u128(0xDA));
    const OTRO: UsuarioId = UsuarioId::new(Uuid::from_u128(0x07));
    const TRES_SI: Evaluacion = Evaluacion {
        inesperado: true,
        necesario: true,
        urgente: true,
    };

    struct Mundo {
        fondo: FondoEnMemoria,
        ids: IdsSecuenciales,
        reloj: RelojFijo,
    }

    impl Mundo {
        fn new() -> Self {
            Self {
                fondo: FondoEnMemoria::default(),
                ids: IdsSecuenciales::default(),
                reloj: RelojFijo(NaiveDate::from_ymd_opt(2026, 11, 10).unwrap()),
            }
        }

        fn depositar(
            &self,
            usuario: UsuarioId,
            deposito: Deposito,
            monto: i64,
        ) -> Result<EstadoFondo, ErrorAplicacion> {
            pollster::block_on(
                DepositarEnFondo::new(&self.fondo, &self.ids, &self.reloj).ejecutar(
                    SolicitudDeposito {
                        usuario,
                        deposito,
                        concepto: "  planilla  ".to_owned(),
                        monto,
                    },
                ),
            )
        }

        fn sacar(
            &self,
            monto: i64,
            nivel: NivelEmergencia,
        ) -> Result<EstadoFondo, ErrorAplicacion> {
            pollster::block_on(
                SacarDelFondo::new(&self.fondo, &self.ids, &self.reloj).ejecutar(SolicitudSacar {
                    usuario: DANIEL,
                    concepto: "llanta de la moto".to_owned(),
                    monto,
                    nivel,
                    evaluacion: TRES_SI,
                }),
            )
            .map(|r| r.estado)
        }

        fn estado(&self, usuario: UsuarioId) -> EstadoFondo {
            pollster::block_on(ConsultarFondo::new(&self.fondo).ejecutar(usuario)).unwrap()
        }
    }

    #[test]
    fn sacar_y_reponer_como_en_la_hoja() {
        let m = Mundo::new();
        m.depositar(DANIEL, Deposito::Aporte, 31_619).unwrap();

        let tras_retiro = m.sacar(20_000, NivelEmergencia::Trabajo).unwrap();
        assert_eq!(
            tras_retiro,
            EstadoFondo {
                saldo: Colones::new(11_619),
                por_reponer: Colones::new(21_000),
                cuota_mensual: Colones::new(7_000),
            },
            "el retiro queda guardado con su plan"
        );
        assert_eq!(
            m.sacar(3_000, NivelEmergencia::Descuido),
            Err(ErrorAplicacion::RetiroRechazado(
                RechazoRetiro::ReposicionPendiente
            )),
            "el caso de uso no decide: el dominio rechaza y no se guarda nada"
        );
        assert_eq!(m.estado(DANIEL), tras_retiro, "el rechazo no tocó el fondo");

        assert_eq!(
            m.depositar(DANIEL, Deposito::Reposicion, 25_000),
            Err(ErrorAplicacion::Fondo(ErrorFondo::ReposicionDeMas {
                por_reponer: Colones::new(21_000)
            })),
            "no se repone de más"
        );
        let repuesto = m.depositar(DANIEL, Deposito::Reposicion, 21_000).unwrap();
        assert_eq!(
            repuesto.saldo,
            Colones::new(32_619),
            "el recargo volvió al fondo"
        );
        assert!(!repuesto.reposicion_pendiente(), "nada pendiente");
    }

    #[test]
    fn cada_usuario_tiene_su_fondo() {
        let m = Mundo::new();
        m.depositar(DANIEL, Deposito::Aporte, 31_619).unwrap();
        m.depositar(OTRO, Deposito::Aporte, 100).unwrap();
        assert_eq!(m.estado(DANIEL).saldo, Colones::new(31_619), "Daniel");
        assert_eq!(m.estado(OTRO).saldo, Colones::new(100), "el otro");
        assert_eq!(
            m.depositar(OTRO, Deposito::Reposicion, 50),
            Err(ErrorAplicacion::Fondo(ErrorFondo::SinReposicionPendiente)),
            "el otro no tiene nada que reponer aunque Daniel tuviera"
        );
    }

    #[test]
    fn un_concepto_vacio_no_llega_al_fondo() {
        let m = Mundo::new();
        let r = pollster::block_on(DepositarEnFondo::new(&m.fondo, &m.ids, &m.reloj).ejecutar(
            SolicitudDeposito {
                usuario: DANIEL,
                deposito: Deposito::Aporte,
                concepto: "   ".to_owned(),
                monto: 1_000,
            },
        ));
        assert!(
            matches!(r, Err(ErrorAplicacion::Concepto(_))),
            "concepto vacío: {r:?}"
        );
        assert_eq!(m.estado(DANIEL), EstadoFondo::default(), "no se guardó");
    }
}
