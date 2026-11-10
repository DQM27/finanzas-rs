//! Vigila que el dominio siga puro: sólo crates puras en el manifiesto y nada
//! de reloj, aleatoriedad, red, archivos ni variables de entorno en el código.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    const PERMITIDAS: [&str; 3] = ["chrono", "thiserror", "uuid"];
    const PROHIBIDO: [&str; 9] = [
        "SystemTime",
        "Instant::now",
        "Utc::now",
        "Local::now",
        "Uuid::new_v",
        "rand::",
        "std::fs",
        "std::net",
        "std::env",
    ];

    #[test]
    fn el_dominio_solo_depende_de_crates_puras() {
        let manifiesto = include_str!("../Cargo.toml");
        let dependencias = manifiesto
            .split("[dependencies]")
            .nth(1)
            .expect("el manifiesto tiene [dependencies]")
            .split("\n[")
            .next()
            .unwrap_or_default();
        for linea in dependencias.lines() {
            let linea = linea.trim();
            if linea.is_empty() || linea.starts_with('#') {
                continue;
            }
            let nombre = linea.split(['.', '=', ' ']).next().unwrap_or_default();
            assert!(
                PERMITIDAS.contains(&nombre),
                "el dominio no puede depender de `{nombre}`: sólo de {PERMITIDAS:?}"
            );
        }
    }

    #[test]
    fn el_codigo_del_dominio_no_toca_el_mundo_exterior() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for entrada in fs::read_dir(&src).expect("existe src") {
            let ruta = entrada.expect("entrada legible").path();
            let codigo = fs::read_to_string(&ruta).expect("archivo legible");
            for prohibido in PROHIBIDO {
                assert!(
                    !codigo.contains(prohibido),
                    "{} usa `{prohibido}`: el dominio recibe la fecha y los IDs, no los busca",
                    ruta.display()
                );
            }
        }
    }
}
