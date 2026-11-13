//! Deshacer de punta a punta, con los adaptadores en memoria.

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use finanzas_aplicacion::casos_de_uso::deshacer::{Deshacer, QueDeshacer};
    use finanzas_aplicacion::casos_de_uso::fondo::{
        ConsultarFondo, DepositarEnFondo, Deposito, SacarDelFondo, SolicitudDeposito,
        SolicitudSacar,
    };
    use finanzas_aplicacion::casos_de_uso::registrar_gasto::{RegistrarGasto, SolicitudGasto};
    use finanzas_aplicacion::errores::ErrorAplicacion;
    use finanzas_aplicacion::puertos::{GeneradorIds, RepositorioPerfiles};
    use finanzas_contrato::{DANIEL, OTRO, perfil};
    use finanzas_dominio::base::dinero::Colones;
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

        fn deshacer(
            &self,
            usuario: finanzas_dominio::base::usuario::UsuarioId,
            que: QueDeshacer,
        ) -> Result<(), ErrorAplicacion> {
            pollster::block_on(
                Deshacer::new(&self.gastos, &self.extras, &self.fondo).ejecutar(usuario, que),
            )
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

        let id = QueDeshacer::Gasto(r.gasto.id());
        assert_eq!(
            m.deshacer(OTRO, id),
            Err(ErrorAplicacion::NoEncontrado),
            "otro usuario no puede borrarlo (U1)"
        );
        assert_eq!(m.deshacer(DANIEL, id), Ok(()), "el dueño sí");
        assert_eq!(
            m.deshacer(DANIEL, id),
            Err(ErrorAplicacion::NoEncontrado),
            "ya no existe"
        );
        assert_eq!(
            m.deshacer(
                DANIEL,
                QueDeshacer::Gasto(GastoId::new(Uuid::from_u128(999)))
            ),
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
            m.deshacer(DANIEL, QueDeshacer::MovimientoFondo(retiro)),
            Err(ErrorAplicacion::Fondo(ErrorFondo::SoloElUltimo)),
            "el retiro ya tiene una reposición encima (U2)"
        );
        assert_eq!(
            m.deshacer(OTRO, QueDeshacer::MovimientoFondo(reposicion)),
            Err(ErrorAplicacion::NoEncontrado),
            "lo del fondo de Daniel no lo toca otro"
        );
        assert_eq!(
            m.deshacer(DANIEL, QueDeshacer::MovimientoFondo(reposicion)),
            Ok(()),
            "la reposición es lo último"
        );
        assert_eq!(
            m.deshacer(DANIEL, QueDeshacer::MovimientoFondo(retiro)),
            Ok(()),
            "ahora el retiro es lo último"
        );
        let estado = pollster::block_on(ConsultarFondo::new(&m.fondo).ejecutar(DANIEL)).unwrap();
        assert_eq!(estado.saldo, Colones::new(31_619), "como antes del retiro");
        assert!(!estado.reposicion_pendiente(), "sin nada que reponer");
        assert_eq!(
            m.deshacer(DANIEL, QueDeshacer::MovimientoFondo(aporte)),
            Ok(()),
            "y por último el aporte"
        );
    }
}
