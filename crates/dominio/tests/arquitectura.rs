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

    /// Todos los `.rs` de una carpeta y de las de adentro: el dominio está
    /// partido en módulos con subcarpetas y ninguno queda sin vigilar.
    fn archivos_rust(carpeta: &Path) -> Vec<std::path::PathBuf> {
        let mut encontrados = Vec::new();
        for entrada in fs::read_dir(carpeta).expect("carpeta legible") {
            let ruta = entrada.expect("entrada legible").path();
            if ruta.is_dir() {
                encontrados.extend(archivos_rust(&ruta));
            } else if ruta.extension().is_some_and(|e| e == "rs") {
                encontrados.push(ruta);
            }
        }
        encontrados
    }

    #[test]
    fn el_codigo_del_dominio_no_toca_el_mundo_exterior() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let archivos = archivos_rust(&src);
        assert!(
            archivos.len() >= 12,
            "se esperaban todos los módulos del dominio y se vieron {}: \
             ¿la búsqueda dejó de entrar en las subcarpetas?",
            archivos.len()
        );
        for ruta in archivos {
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
