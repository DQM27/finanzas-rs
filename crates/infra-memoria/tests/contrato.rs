//! Los adaptadores en memoria cumplen el contrato de los puertos.

#[cfg(test)]
mod tests {
    use finanzas_infra_memoria::{ExtrasEnMemoria, GastosEnMemoria, PerfilesEnMemoria};

    #[test]
    fn perfiles() {
        pollster::block_on(finanzas_contrato::perfiles(&PerfilesEnMemoria::default()));
    }

    #[test]
    fn gastos() {
        pollster::block_on(finanzas_contrato::gastos(&GastosEnMemoria::default()));
    }

    #[test]
    fn extras() {
        pollster::block_on(finanzas_contrato::extras(&ExtrasEnMemoria::default()));
    }
}
