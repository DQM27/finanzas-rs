//! Deshacer de punta a punta, con los adaptadores en memoria.

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use finanzas_aplicacion::casos_de_uso::extras::{
        DeshacerExtra, RegistrarExtra, SolicitudExtra,
    };
    use finanzas_aplicacion::casos_de_uso::fondo::{
        ConsultarFondo, DepositarEnFondo, Deposito, DeshacerMovimientoFondo, SacarDelFondo,
        SolicitudDeposito, SolicitudSacar,
    };
    use finanzas_aplicacion::casos_de_uso::gastos::{
        DeshacerGasto, RegistrarGasto, SolicitudGasto,
    };
    use finanzas_aplicacion::errores::ErrorAplicacion;
    use finanzas_aplicacion::puertos::{GeneradorIds, RepositorioPerfiles};
    use finanzas_contrato::{DANIEL, OTRO, perfil};
    use finanzas_dominio::base::dinero::Colones;
    use finanzas_dominio::base::usuario::UsuarioId;
    use finanzas_dominio::extras::{ExtraId, TipoExtra};
    use finanzas_dominio::fondo::movimientos::{ErrorFondo, MovimientoFondoId};
    use finanzas_dominio::fondo::{Evaluacion, NivelEmergencia};
    use finanzas_dominio::gastos::{Categoria, GastoId, Origen};
    use finanzas_infra_memoria::{
        ExtrasEnMemoria, FondoEnMemoria, GastosEnMemoria, IdsSecuenciales, PerfilesEnMemoria,
        RelojFijo,
    };
    use uuid::Uuid;

    struct Mundo {
        perfiles: PerfilesEnMemoria,
        gastos: GastosEnMemoria,
        extras: ExtrasEnMemoria,
        fondo: FondoEnMemoria,
        ids: IdsSecuenciales,
        reloj: RelojFijo,
    }

    impl Mundo {
        fn new() -> Self {
            let m = Self {
                perfiles: PerfilesEnMemoria::default(),
                gastos: GastosEnMemoria::default(),
                extras: ExtrasEnMemoria::default(),
                fondo: FondoEnMemoria::default(),
                ids: IdsSecuenciales::default(),
                reloj: RelojFijo(NaiveDate::from_ymd_opt(2026, 11, 15).unwrap()),
            };
            pollster::block_on(m.perfiles.guardar(&perfil(DANIEL, 5_000))).unwrap();
            m
        }

        fn deshacer_gasto(&self, usuario: UsuarioId, id: GastoId) -> Result<(), ErrorAplicacion> {
            pollster::block_on(DeshacerGasto::new(&self.gastos).ejecutar(usuario, id))
        }

        fn deshacer_movimiento(
            &self,
            usuario: UsuarioId,
            id: MovimientoFondoId,
        ) -> Result<(), ErrorAplicacion> {
            pollster::block_on(DeshacerMovimientoFondo::new(&self.fondo).ejecutar(usuario, id))
        }

        /// Deposita y devuelve el id que le tocó al movimiento.
        fn depositar(&self, deposito: Deposito, monto: i64) -> MovimientoFondoId {
            pollster::block_on(
                DepositarEnFondo::new(&self.fondo, &self.ids, &self.reloj).ejecutar(
                    SolicitudDeposito {
                        usuario: DANIEL,
                        deposito,
                        concepto: "planilla".to_owned(),
                        monto,
                    },
                ),
            )
            .unwrap();
            self.ultimo_id()
        }

        fn deshacer_extra(&self, usuario: UsuarioId, id: ExtraId) -> Result<(), ErrorAplicacion> {
            pollster::block_on(DeshacerExtra::new(&self.extras).ejecutar(usuario, id))
        }

        fn ultimo_id(&self) -> MovimientoFondoId {
            // Los ids son secuenciales: el último dado es el siguiente menos uno.
            let siguiente = self.ids.nuevo().as_u128();
            MovimientoFondoId::new(Uuid::from_u128(siguiente - 1))
        }
    }

    #[test]
    fn deshacer_un_gasto_devuelve_la_caja_chica() {
        let m = Mundo::new();
        let r = pollster::block_on(
            RegistrarGasto::new(&m.perfiles, &m.gastos, &m.ids, &m.reloj).ejecutar(
                SolicitudGasto {
                    usuario: DANIEL,
                    fecha: NaiveDate::from_ymd_opt(2026, 11, 14).unwrap(),
                    concepto: "jugo nieto".to_owned(),
                    monto: 2_950,
                    categoria: Categoria::Familia,
                    origen: Origen::CuentaGastos,
                },
            ),
        )
        .unwrap();
        assert_eq!(
            r.caja.estado.queda,
            Colones::new(2_050),
            "antes de deshacer"
        );

        let id = r.gasto.id();
        assert_eq!(
            m.deshacer_gasto(OTRO, id),
            Err(ErrorAplicacion::NoEncontrado),
            "otro usuario no puede borrarlo (U1)"
        );
        assert_eq!(m.deshacer_gasto(DANIEL, id), Ok(()), "el dueño sí");
        assert_eq!(
            m.deshacer_gasto(DANIEL, id),
            Err(ErrorAplicacion::NoEncontrado),
            "ya no existe"
        );
        assert_eq!(
            m.deshacer_gasto(DANIEL, GastoId::new(Uuid::from_u128(999))),
            Err(ErrorAplicacion::NoEncontrado),
            "un id inventado"
        );
    }

    #[test]
    fn en_el_fondo_solo_se_deshace_lo_ultimo() {
        let m = Mundo::new();
        let aporte = m.depositar(Deposito::Aporte, 31_619);
        pollster::block_on(SacarDelFondo::new(&m.fondo, &m.ids, &m.reloj).ejecutar(
            SolicitudSacar {
                usuario: DANIEL,
                concepto: "llanta".to_owned(),
                monto: 20_000,
                nivel: NivelEmergencia::Trabajo,
                evaluacion: Evaluacion {
                    inesperado: true,
                    necesario: true,
                    urgente: true,
                },
            },
        ))
        .unwrap();
        let retiro = m.ultimo_id();
        let reposicion = m.depositar(Deposito::Reposicion, 7_000);

        assert_eq!(
            m.deshacer_movimiento(DANIEL, retiro),
            Err(ErrorAplicacion::Fondo(ErrorFondo::SoloElUltimo)),
            "el retiro ya tiene una reposición encima (U2)"
        );
        assert_eq!(
            m.deshacer_movimiento(OTRO, reposicion),
            Err(ErrorAplicacion::NoEncontrado),
            "lo del fondo de Daniel no lo toca otro"
        );
        assert_eq!(
            m.deshacer_movimiento(DANIEL, reposicion),
            Ok(()),
            "la reposición es lo último"
        );
        assert_eq!(
            m.deshacer_movimiento(DANIEL, retiro),
            Ok(()),
            "ahora el retiro es lo último"
        );
        let estado = pollster::block_on(ConsultarFondo::new(&m.fondo).ejecutar(DANIEL)).unwrap();
        assert_eq!(estado.saldo, Colones::new(31_619), "como antes del retiro");
        assert!(!estado.reposicion_pendiente(), "sin nada que reponer");
        assert_eq!(
            m.deshacer_movimiento(DANIEL, aporte),
            Ok(()),
            "y por último el aporte"
        );
    }

    #[test]
    fn deshacer_una_extra_la_saca_del_pago() {
        let m = Mundo::new();
        let r = pollster::block_on(
            RegistrarExtra::new(&m.perfiles, &m.extras, &m.ids).ejecutar(SolicitudExtra {
                usuario: DANIEL,
                fecha: NaiveDate::from_ymd_opt(2026, 11, 14).unwrap(),
                tipo: TipoExtra::Nocturna,
                monto: None,
            }),
        )
        .unwrap();
        let id = r.extra.id();

        assert_eq!(
            m.deshacer_extra(OTRO, id),
            Err(ErrorAplicacion::NoEncontrado),
            "una extra ajena no se toca (U1)"
        );
        assert_eq!(m.deshacer_extra(DANIEL, id), Ok(()), "la suya sí");
        assert_eq!(
            m.deshacer_extra(DANIEL, id),
            Err(ErrorAplicacion::NoEncontrado),
            "ya no existe"
        );
    }
}
