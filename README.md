# finanzas-rs

Sistema de finanzas personales por quincena: presupuesto, extras con su fecha de pago
real, caja chica, fondo de emergencia con reglas de reposición, gastos puntuales y
deudas.

Hecho en Rust con arquitectura hexagonal. Pensado para varios usuarios, cada uno con
su configuración y sus datos.

- Arquitectura: [`docs/arquitectura.md`](docs/arquitectura.md)
- Reglas de negocio: [`docs/reglas.md`](docs/reglas.md)

## Verificar

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```
