//! La API completa, en JSON, sobre los adaptadores en memoria. Es lo mismo que
//! verá Angular, salvo el Worker y SurrealDB.

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use finanzas_api::{Contexto, Metodo, Peticion, Respuesta, atender};
    use finanzas_aplicacion::errores::ErrorRepositorio;
    use finanzas_aplicacion::puertos::RepositorioPerfiles;
    use finanzas_contrato::{DANIEL, OTRO, perfil};
    use finanzas_dominio::perfil::Perfil;
    use finanzas_dominio::usuario::UsuarioId;
    use finanzas_infra_memoria::{
        ExtrasEnMemoria, FondoEnMemoria, GastosEnMemoria, IdsSecuenciales, PerfilesEnMemoria,
        QuincenasEnMemoria, RelojFijo,
    };
    use serde_json::{Value, json};

    struct Mundo<P = PerfilesEnMemoria> {
        perfiles: P,
        gastos: GastosEnMemoria,
        extras: ExtrasEnMemoria,
        fondo: FondoEnMemoria,
        quincenas: QuincenasEnMemoria,
        ids: IdsSecuenciales,
        reloj: RelojFijo,
    }

    impl<P: RepositorioPerfiles> Contexto for Mundo<P> {
        type Perfiles = P;
        type Gastos = GastosEnMemoria;
        type Extras = ExtrasEnMemoria;
        type Fondo = FondoEnMemoria;
        type Quincenas = QuincenasEnMemoria;
        type Ids = IdsSecuenciales;
        type Reloj = RelojFijo;

        fn perfiles(&self) -> &P {
            &self.perfiles
        }
        fn gastos(&self) -> &GastosEnMemoria {
            &self.gastos
        }
        fn extras(&self) -> &ExtrasEnMemoria {
            &self.extras
        }
        fn fondo(&self) -> &FondoEnMemoria {
            &self.fondo
        }
        fn quincenas(&self) -> &QuincenasEnMemoria {
            &self.quincenas
        }
        fn ids(&self) -> &IdsSecuenciales {
            &self.ids
        }
        fn reloj(&self) -> &RelojFijo {
            &self.reloj
        }
    }

    fn mundo_con<P>(perfiles: P) -> Mundo<P> {
        Mundo {
            perfiles,
            gastos: GastosEnMemoria::default(),
            extras: ExtrasEnMemoria::default(),
            fondo: FondoEnMemoria::default(),
            quincenas: QuincenasEnMemoria::default(),
            ids: IdsSecuenciales::default(),
            // Domingo 15 de noviembre: se gasta el pago del 13.
            reloj: RelojFijo(NaiveDate::from_ymd_opt(2026, 11, 15).unwrap()),
        }
    }

    /// Daniel con su perfil configurado.
    fn mundo() -> Mundo {
        let m = mundo_con(PerfilesEnMemoria::default());
        pollster::block_on(m.perfiles.guardar(&perfil(DANIEL, 5_000))).unwrap();
        m
    }

    fn pedir<C: Contexto>(
        ctx: &C,
        usuario: UsuarioId,
        metodo: Metodo,
        ruta: &str,
        cuerpo: Option<Value>,
    ) -> (u16, Value) {
        let bytes = cuerpo.map(|v| v.to_string()).unwrap_or_default();
        let Respuesta { estado, cuerpo, .. } = pollster::block_on(atender(
            ctx,
            usuario,
            Peticion {
                metodo,
                ruta,
                cuerpo: bytes.as_bytes(),
            },
        ));
        let valor = if cuerpo.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&cuerpo).unwrap()
        };
        (estado, valor)
    }

    fn codigo(v: &Value) -> &str {
        v["codigo"].as_str().unwrap()
    }

    #[test]
    fn anotar_un_gasto_y_ver_la_caja_chica() {
        let m = mundo();
        let (estado, caja) = pedir(&m, DANIEL, Metodo::Get, "/caja-chica", None);
        assert_eq!(estado, 200, "{caja}");
        assert_eq!(caja["queda"], 5_000, "caja llena");
        assert_eq!(caja["nivel"], "HOLGADA", "nivel");
        assert_eq!(caja["desde"], "2026-11-13", "periodo desde el pago del 13");

        let (estado, r) = pedir(
            &m,
            DANIEL,
            Metodo::Post,
            "/gastos",
            Some(json!({
                "fecha": "2026-11-14",
                "concepto": "jugo nieto",
                "monto": 2950,
                "categoria": "FAMILIA",
                "origen": "CUENTA_GASTOS"
            })),
        );
        assert_eq!(estado, 201, "{r}");
        assert_eq!(r["gasto"]["concepto"], "JUGO NIETO", "normalizado");
        assert_eq!(r["caja"]["queda"], 2_050, "la caja baja");
        assert_eq!(r["caja"]["nivel"], "HOLGADA", "41 % todavía");

        let id = r["gasto"]["id"].as_str().unwrap();
        let (estado, _) = pedir(&m, OTRO, Metodo::Delete, &format!("/gastos/{id}"), None);
        assert_eq!(estado, 404, "otro usuario no lo ve");
        let (estado, cuerpo) = pedir(&m, DANIEL, Metodo::Delete, &format!("/gastos/{id}"), None);
        assert_eq!((estado, cuerpo), (204, Value::Null), "deshecho");
        let (_, caja) = pedir(&m, DANIEL, Metodo::Get, "/caja-chica", None);
        assert_eq!(caja["queda"], 5_000, "la caja vuelve a estar llena");
    }

    #[test]
    fn una_extra_dice_cuanto_y_cuando_se_cobra() {
        let m = mundo();
        let (estado, r) = pedir(
            &m,
            DANIEL,
            Metodo::Post,
            "/extras",
            Some(json!({ "fecha": "2026-11-14", "tipo": "NOCTURNA" })),
        );
        assert_eq!(estado, 201, "{r}");
        assert_eq!(r["bruto"], 25_800, "tarifa del perfil");
        assert_eq!(
            r["fecha_pago"], "2026-11-30",
            "se cobra con el corte del 30"
        );
        let (estado, r) = pedir(
            &m,
            DANIEL,
            Metodo::Post,
            "/extras",
            Some(json!({ "fecha": "2026-11-14", "tipo": "OTRA" })),
        );
        assert_eq!(
            (estado, codigo(&r)),
            (422, "DATOS_INVALIDOS"),
            "OTRA sin monto: {r}"
        );
    }

    #[test]
    fn el_fondo_por_la_api() {
        let m = mundo();
        let deposito = |tipo: &str, monto: i64| {
            pedir(
                &m,
                DANIEL,
                Metodo::Post,
                "/fondo/depositos",
                Some(json!({ "tipo": tipo, "concepto": "planilla", "monto": monto })),
            )
        };
        assert_eq!(deposito("APORTE", 31_619).0, 201, "aporte");
        let (estado, r) = pedir(
            &m,
            DANIEL,
            Metodo::Post,
            "/fondo/retiros",
            Some(json!({
                "concepto": "llanta",
                "monto": 20_000,
                "nivel": "TRABAJO",
                "inesperado": true,
                "necesario": true,
                "urgente": true
            })),
        );
        assert_eq!(estado, 201, "{r}");
        assert_eq!(
            r,
            json!({
                "plan": { "nivel": "TRABAJO", "sacado": 20_000, "a_reponer": 21_000, "meses": 3, "cuota_mensual": 7000 },
                "fondo": { "saldo": 11_619, "por_reponer": 21_000, "cuota_mensual": 7000 }
            }),
            "plan y fondo"
        );
        let (estado, r) = pedir(
            &m,
            DANIEL,
            Metodo::Post,
            "/fondo/retiros",
            Some(json!({
                "concepto": "antojo",
                "monto": 1000,
                "nivel": "DESCUIDO",
                "inesperado": true,
                "necesario": true,
                "urgente": true
            })),
        );
        assert_eq!((estado, codigo(&r)), (409, "RETIRO_RECHAZADO"), "{r}");
        assert!(
            r["mensaje"]
                .as_str()
                .unwrap()
                .contains("reposición pendiente"),
            "el mensaje explica por qué: {r}"
        );
        let (estado, r) = deposito("REPOSICION", 30_000);
        assert_eq!(
            (estado, codigo(&r)),
            (422, "DATOS_INVALIDOS"),
            "de más: {r}"
        );
        let (_, fondo) = pedir(&m, DANIEL, Metodo::Get, "/fondo", None);
        assert_eq!(fondo["saldo"], 11_619, "los rechazos no tocaron nada");
    }

    #[test]
    fn la_quincena_por_la_api() {
        let m = mundo();
        let (estado, r) = pedir(&m, DANIEL, Metodo::Get, "/quincenas/actual", None);
        assert_eq!((estado, codigo(&r)), (404, "SIN_PRESUPUESTO"), "{r}");

        let (estado, q) = pedir(
            &m,
            DANIEL,
            Metodo::Put,
            "/quincenas/2026-11-13",
            Some(json!({
                "ingresos": [{ "concepto": "salario base", "monto": 232_225 }],
                "pagos": [
                    { "concepto": "moto", "monto": 104_000 },
                    { "concepto": "comida", "monto": 100_000 }
                ]
            })),
        );
        assert_eq!(estado, 200, "{q}");
        assert_eq!(q["sobra"], 28_225, "232 225 − 204 000");

        let (estado, q) = pedir(
            &m,
            DANIEL,
            Metodo::Put,
            "/quincenas/2026-11-13/marcas",
            Some(json!({ "lado": "PAGO", "indice": 0, "hecho": true })),
        );
        assert_eq!(estado, 200, "{q}");
        assert_eq!(q["falta_por_pagar"], 100_000, "sólo falta la comida");
        assert_eq!(q["avance"], json!({ "hechos": 1, "total": 2 }), "avance");

        let (estado, actual) = pedir(&m, DANIEL, Metodo::Get, "/quincenas/actual", None);
        assert_eq!((estado, &actual), (200, &q), "la actual es la del 13");

        let (estado, r) = pedir(
            &m,
            DANIEL,
            Metodo::Put,
            "/quincenas/2026-11-15",
            Some(json!({ "ingresos": [], "pagos": [] })),
        );
        assert_eq!(
            (estado, codigo(&r)),
            (422, "DATOS_INVALIDOS"),
            "no es pago: {r}"
        );
    }

    #[test]
    fn errores_de_forma() {
        let m = mundo();
        let (estado, r) = pedir(&m, DANIEL, Metodo::Get, "/no-existe", None);
        assert_eq!((estado, codigo(&r)), (404, "RUTA_INEXISTENTE"), "ruta");
        let (estado, r) = pedir(&m, DANIEL, Metodo::Delete, "/fondo", None);
        assert_eq!((estado, codigo(&r)), (405, "METODO_NO_PERMITIDO"), "método");
        let (estado, r) = pedir(&m, DANIEL, Metodo::Delete, "/gastos/no-es-uuid", None);
        assert_eq!((estado, codigo(&r)), (404, "RUTA_INEXISTENTE"), "id malo");
        let (estado, r) = pedir(&m, DANIEL, Metodo::Get, "/quincenas/13-11-2026", None);
        assert_eq!(
            (estado, codigo(&r)),
            (404, "RUTA_INEXISTENTE"),
            "fecha mala"
        );

        let malo = pollster::block_on(atender(
            &m,
            DANIEL,
            Peticion {
                metodo: Metodo::Post,
                ruta: "/gastos",
                cuerpo: b"{no es json",
            },
        ));
        assert_eq!(malo.estado, 400, "JSON roto");
        let (estado, r) = pedir(
            &m,
            DANIEL,
            Metodo::Post,
            "/fondo/depositos",
            Some(json!({ "tipo": "APORTE", "concepto": "x", "monto": 1, "colado": true })),
        );
        assert_eq!(
            (estado, codigo(&r)),
            (400, "JSON_INVALIDO"),
            "campo de más: {r}"
        );
        let (estado, r) = pedir(
            &m,
            DANIEL,
            Metodo::Post,
            "/gastos",
            Some(json!({
                "fecha": "2026-11-14", "concepto": "x", "monto": 1,
                "categoria": "INVENTADA", "origen": "PLANILLA"
            })),
        );
        assert_eq!(
            (estado, codigo(&r)),
            (400, "JSON_INVALIDO"),
            "enum desconocido: {r}"
        );
    }

    #[test]
    fn sin_perfil_se_avisa() {
        let m = mundo();
        let (estado, r) = pedir(&m, OTRO, Metodo::Get, "/caja-chica", None);
        assert_eq!((estado, codigo(&r)), (404, "PERFIL_NO_ENCONTRADO"), "{r}");
    }

    /// Un almacenamiento que siempre falla.
    #[derive(Debug)]
    struct Caido;

    impl RepositorioPerfiles for Caido {
        async fn perfil(&self, _: UsuarioId) -> Result<Option<Perfil>, ErrorRepositorio> {
            Err(ErrorRepositorio(
                "conexión rechazada a db.interna:8000".to_owned(),
            ))
        }
        async fn guardar(&self, _: &Perfil) -> Result<(), ErrorRepositorio> {
            Err(ErrorRepositorio("conexión rechazada".to_owned()))
        }
    }

    #[test]
    fn un_error_interno_no_se_le_muestra_al_cliente() {
        let m = mundo_con(Caido);
        let r = pollster::block_on(atender(
            &m,
            DANIEL,
            Peticion {
                metodo: Metodo::Get,
                ruta: "/caja-chica",
                cuerpo: b"",
            },
        ));
        assert_eq!(r.estado, 500, "error interno");
        assert!(r.cuerpo.contains("ERROR_INTERNO"), "{}", r.cuerpo);
        assert!(
            !r.cuerpo.contains("db.interna"),
            "no filtra detalles: {}",
            r.cuerpo
        );
        assert_eq!(
            r.detalle_interno.as_deref(),
            Some("conexión rechazada a db.interna:8000"),
            "el detalle queda para el registro del servidor"
        );
    }
}
