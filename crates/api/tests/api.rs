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
        assert_eq!(estado, 200, "sin presupuesto igual se ve: {r}");
        assert_eq!(r["presupuestada"], false, "{r}");
        assert_eq!(
            r["automaticos"]["salario"], 232_225,
            "el salario entra solo"
        );
        let entra_solo = r["sobra"].as_i64().unwrap();
        assert_eq!(
            entra_solo,
            232_225 + r["automaticos"]["extras"].as_i64().unwrap(),
            "salario + extras del pago"
        );

        let (estado, q) = pedir(
            &m,
            DANIEL,
            Metodo::Put,
            "/quincenas/2026-11-13",
            Some(json!({
                "ingresos": [{ "concepto": "papá", "monto": 65_000 }],
                "pagos": [
                    { "concepto": "moto", "monto": 104_000 },
                    { "concepto": "comida", "monto": 100_000 }
                ]
            })),
        );
        assert_eq!(estado, 200, "{q}");
        assert_eq!(q["presupuestada"], true, "{q}");
        assert_eq!(q["otros_ingresos"], 65_000, "{q}");
        assert_eq!(
            q["sobra"],
            entra_solo + 65_000 - 204_000,
            "lo automático + papá − pagos"
        );

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
    fn el_mes_por_la_api() {
        let m = mundo();
        let (estado, mes) = pedir(&m, DANIEL, Metodo::Get, "/meses/2026-11", None);
        assert_eq!(estado, 200, "{mes}");
        assert_eq!(mes["mes"], "2026-11", "{mes}");
        let pagos: Vec<&str> = mes["quincenas"]
            .as_array()
            .unwrap()
            .iter()
            .map(|q| q["pago"].as_str().unwrap())
            .collect();
        assert_eq!(pagos, ["2026-11-13", "2026-11-30"], "Q1 y Q2");
        assert_eq!(mes["balance"]["salario"], 464_450, "dos salarios");

        let (_, actual) = pedir(&m, DANIEL, Metodo::Get, "/meses/actual", None);
        assert_eq!(actual, mes, "hoy es noviembre");
        for malo in ["/meses/2026-13", "/meses/2026-1", "/meses/noviembre"] {
            let (estado, r) = pedir(&m, DANIEL, Metodo::Get, malo, None);
            assert_eq!(
                (estado, codigo(&r)),
                (404, "RUTA_INEXISTENTE"),
                "{malo}: {r}"
            );
        }
        let (estado, r) = pedir(&m, DANIEL, Metodo::Delete, "/meses/actual", None);
        assert_eq!((estado, codigo(&r)), (405, "METODO_NO_PERMITIDO"), "{r}");
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

    /// El JSON de `GET /perfil` sin lo calculado sirve como cuerpo de `PUT`.
    fn como_pedido(mut perfil: Value) -> Value {
        let o = perfil.as_object_mut().unwrap();
        o.remove("salario_neto");
        o.remove("cubre_hasta");
        perfil
    }

    #[test]
    fn el_perfil_se_lee_y_se_reemplaza() {
        let m = mundo();
        let (estado, perfil) = pedir(&m, DANIEL, Metodo::Get, "/perfil", None);
        assert_eq!(estado, 200, "{perfil}");
        assert_eq!(
            perfil["salario_neto"], 232_225,
            "P2 calculado por el dominio"
        );
        assert_eq!(perfil["factor_neto"], 8617, "diezmilésimas");
        assert_eq!(perfil["tarifas"]["nocturna"], 25_800, "tarifa");
        assert_eq!(
            perfil["cubre_hasta"], "2026-11-30",
            "último pago del calendario"
        );

        let mut pedido = como_pedido(perfil.clone());
        pedido["tope_caja_chica"] = json!(6_000);
        let (estado, nuevo) = pedir(&m, DANIEL, Metodo::Put, "/perfil", Some(pedido));
        assert_eq!(estado, 200, "{nuevo}");
        assert_eq!(nuevo["tope_caja_chica"], 6_000, "reemplazado");
        let (_, caja) = pedir(&m, DANIEL, Metodo::Get, "/caja-chica", None);
        assert_eq!(caja["queda"], 6_000, "la caja chica usa el tope nuevo");

        let (estado, r) = pedir(&m, DANIEL, Metodo::Put, "/perfil", Some(perfil));
        assert_eq!(
            (estado, codigo(&r)),
            (400, "JSON_INVALIDO"),
            "lo calculado no se puede mandar: {r}"
        );
    }

    #[test]
    fn el_dominio_valida_el_perfil() {
        let m = mundo();
        let (_, perfil) = pedir(&m, DANIEL, Metodo::Get, "/perfil", None);
        let base = como_pedido(perfil);

        let mut factor = base.clone();
        factor["factor_neto"] = json!(10_001);
        let mut tarifa = base.clone();
        tarifa["tarifas"]["feriado"] = json!(0);
        let mut caja = base.clone();
        caja["tope_caja_chica"] = json!(0);
        let mut hueco = base.clone();
        hueco["cortes"][1]["inicio"] = json!("2026-11-05");
        let mut vacio = base.clone();
        vacio["cortes"] = json!([]);
        let mut invertido = base;
        invertido["cortes"][0]["pago"] = json!("2026-11-01");

        for (nombre, malo) in [
            ("factor mayor a 100 %", factor),
            ("tarifa en cero", tarifa),
            ("caja chica en cero", caja),
            ("cortes con hueco", hueco),
            ("calendario vacío", vacio),
            ("pago antes del reporte", invertido),
        ] {
            let (estado, r) = pedir(&m, DANIEL, Metodo::Put, "/perfil", Some(malo));
            assert_eq!(
                (estado, codigo(&r)),
                (422, "DATOS_INVALIDOS"),
                "{nombre}: {r}"
            );
        }
        let (_, sigue) = pedir(&m, DANIEL, Metodo::Get, "/perfil", None);
        assert_eq!(
            sigue["tope_caja_chica"], 5_000,
            "ningún intento malo lo cambió"
        );
    }

    #[test]
    fn cada_quien_guarda_su_perfil() {
        let m = mundo();
        let (_, perfil) = pedir(&m, DANIEL, Metodo::Get, "/perfil", None);
        let mut ajeno = como_pedido(perfil);
        ajeno["salario_base"] = json!(400_000);
        let (estado, _) = pedir(&m, OTRO, Metodo::Put, "/perfil", Some(ajeno));
        assert_eq!(estado, 200, "el otro crea el suyo");
        let (_, mio) = pedir(&m, DANIEL, Metodo::Get, "/perfil", None);
        assert_eq!(mio["salario_base"], 293_000, "el de Daniel no cambió (P5)");
        let (_, suyo) = pedir(&m, OTRO, Metodo::Get, "/perfil", None);
        assert_eq!(suyo["salario_base"], 400_000, "el del otro");
    }

    #[test]
    fn las_listas_para_la_interfaz() {
        let m = mundo();
        pedir(
            &m,
            DANIEL,
            Metodo::Post,
            "/gastos",
            Some(json!({
                "fecha": "2026-11-09", "concepto": "jugo nieto", "monto": 2_950,
                "categoria": "FAMILIA", "origen": "CUENTA_GASTOS"
            })),
        );
        pedir(
            &m,
            DANIEL,
            Metodo::Post,
            "/extras",
            Some(json!({ "fecha": "2026-11-14", "tipo": "NOCTURNA" })),
        );
        pedir(
            &m,
            DANIEL,
            Metodo::Post,
            "/fondo/depositos",
            Some(json!({ "tipo": "APORTE", "concepto": "inicial", "monto": 31_619 })),
        );

        let (estado, gastos) = pedir(
            &m,
            DANIEL,
            Metodo::Get,
            "/gastos/2026-11-01/2026-11-12",
            None,
        );
        assert_eq!(estado, 200, "{gastos}");
        assert_eq!(gastos[0]["concepto"], "JUGO NIETO", "gasto listado");
        assert_eq!(gastos.as_array().unwrap().len(), 1, "sólo los del rango");

        let (estado, extras) = pedir(&m, DANIEL, Metodo::Get, "/extras", None);
        assert_eq!(estado, 200, "{extras}");
        assert_eq!(extras[0]["neto"], 22_232, "neto calculado por el dominio");
        // En el calendario del perfil de prueba, el corte del pago del 30
        // llega hasta el 16/11, así que la extra del 14 cae ahí.
        assert_eq!(
            extras[0]["fecha_pago"], "2026-11-30",
            "pago del corte que la contiene"
        );

        let (estado, movs) = pedir(&m, DANIEL, Metodo::Get, "/fondo/movimientos", None);
        assert_eq!(estado, 200, "{movs}");
        assert_eq!(movs[0]["tipo"], "APORTE", "movimiento listado");
        assert_eq!(movs[0]["plan"], Value::Null, "un aporte no lleva plan");

        let (estado, ajenos) = pedir(&m, OTRO, Metodo::Get, "/gastos/2026-11-01/2026-11-12", None);
        assert_eq!((estado, &ajenos), (200, &json!([])), "el otro no ve nada");
        let (estado, r) = pedir(
            &m,
            DANIEL,
            Metodo::Get,
            "/gastos/2026-13-01/2026-11-12",
            None,
        );
        assert_eq!(
            (estado, codigo(&r)),
            (404, "RUTA_INEXISTENTE"),
            "fecha mala"
        );
    }

    #[test]
    fn el_calendario_visto_desde_hoy() {
        // Domingo 15/11: cae en el corte del 2 al 16, que se paga el 30.
        let mut m = mundo();
        let (estado, c) = pedir(&m, DANIEL, Metodo::Get, "/cortes/hoy", None);
        assert_eq!(estado, 200, "{c}");
        assert_eq!(c["en_curso"]["inicio"], "2026-11-02", "corte de hoy");
        assert_eq!(c["en_curso"]["reportar_antes"], "2026-11-17", "reportar");
        assert_eq!(
            c["ultimo_pago"]["pago"], "2026-11-13",
            "se gasta el pago del 13 (Q6)"
        );
        assert_eq!(c["proximo_pago"]["pago"], "2026-11-30", "próximo pago");
        assert_eq!(c["dias_para_pago"], 15, "días que faltan");

        // El día de pago cuenta como próximo pago, con 0 días.
        m.reloj = RelojFijo(NaiveDate::from_ymd_opt(2026, 11, 13).unwrap());
        let (_, c) = pedir(&m, DANIEL, Metodo::Get, "/cortes/hoy", None);
        assert_eq!(c["proximo_pago"]["pago"], "2026-11-13", "hoy pagan");
        assert_eq!(c["dias_para_pago"], 0, "es hoy");
        assert_eq!(
            c["ultimo_pago"]["pago"], "2026-11-13",
            "el de hoy ya cuenta como recibido"
        );

        // Fuera del calendario: nada en curso ni por cobrar.
        m.reloj = RelojFijo(NaiveDate::from_ymd_opt(2027, 1, 10).unwrap());
        let (estado, c) = pedir(&m, DANIEL, Metodo::Get, "/cortes/hoy", None);
        assert_eq!(estado, 200, "{c}");
        assert_eq!(
            c,
            json!({ "en_curso": null, "ultimo_pago": {
                    "inicio": "2026-11-02",
                    "fin": "2026-11-16",
                    "reportar_antes": "2026-11-17",
                    "pago": "2026-11-30"
                }, "proximo_pago": null, "dias_para_pago": null }),
            "fuera del calendario: queda el último pago, nada en curso ni por cobrar"
        );

        let (estado, r) = pedir(&m, DANIEL, Metodo::Post, "/cortes/hoy", None);
        assert_eq!(
            (estado, codigo(&r)),
            (405, "METODO_NO_PERMITIDO"),
            "sólo se lee"
        );
        let sin_perfil = mundo_con(PerfilesEnMemoria::default());
        let (estado, r) = pedir(&sin_perfil, DANIEL, Metodo::Get, "/cortes/hoy", None);
        assert_eq!((estado, codigo(&r)), (404, "PERFIL_NO_ENCONTRADO"), "{r}");
    }
}
