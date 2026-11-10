//! Los adaptadores en memoria cumplen el contrato de los puertos.

#[cfg(test)]
mod tests {
    use finanzas_infra_memoria::{
        ExtrasEnMemoria, FondoEnMemoria, GastosEnMemoria, PerfilesEnMemoria, QuincenasEnMemoria,
    };

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

    #[test]
    fn fondo() {
        pollster::block_on(finanzas_contrato::fondo(&FondoEnMemoria::default()));
    }

    #[test]
    fn quincenas() {
        pollster::block_on(finanzas_contrato::quincenas(&QuincenasEnMemoria::default()));
    }
}
