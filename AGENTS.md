# Notas del proyecto

## Preferencias del usuario

- Comunicarse siempre en español.
- Siempre que se complete un cambio exitoso, hacer un commit bien documentado y subirlo.
- **Todos los commits van sólo a nombre de `Daniel Quintana <daniel.bleach1@gmail.com>`**,
  como autor y como committer (`git config user.name`/`user.email` antes del primer
  commit). Sin líneas `Co-Authored-By`, `Claude-Session` ni ninguna otra atribución a
  asistentes en commits ni en pull requests.

## Reglas de arquitectura

La guía completa está en [`docs/arquitectura.md`](docs/arquitectura.md): léela antes de
empezar a trabajar. Resumen:

- Arquitectura hexagonal. Dependencias sólo hacia adentro:
  `apps → composicion → infra-* → aplicacion → dominio`.
- **Toda regla de negocio vive en `crates/dominio`**, sin excepción. Los casos de uso
  buscan los datos y se los entregan al dominio, que decide. Los casos de uso orquestan;
  no deciden.
- El dominio es puro: sin base de datos, red, reloj del sistema ni generación
  aleatoria. Recibe la fecha de hoy y los IDs. `tests/arquitectura.rs` lo vigila.
- **El dinero son colones enteros (`Colones`), nunca `f64`.** Las proporciones (factor
  neto, recargos) van en diezmilésimas y redondean de una sola forma, en el dominio.
- **Multiusuario desde el diseño:** cada dato pertenece a un `UsuarioId` y todo número
  personal (salario, rebajos, tarifas, topes, calendario de cortes) es configuración del
  usuario, no una constante en el código.
- Sin ORM. Las consultas de SurrealDB se escriben a la vista.
- El catálogo de reglas está en `docs/reglas.md`; cualquier cambio de regla se refleja
  ahí en el mismo commit.
- La interfaz sigue `docs/diseno.md` (lenguaje de Lattis/Kash): todo color y medida sale
  de un token, nunca un valor suelto.

## Verificación antes de cada commit

La versión de Rust está fija en `rust-toolchain.toml`; `cargo` la usa sola. Después de
subir, revisar que la CI de GitHub quede en verde.

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```
