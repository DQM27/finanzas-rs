# Propuesta: cierre de quincena y de mes

Estado: **propuesta, sin implementar.** Hay que discutirla con Daniel antes de escribir
código. Cuando se apruebe, sus reglas pasan a `reglas.md` con su código y esta página
se borra o queda como historia de la decisión.

## El problema

En la hoja, lo que sobra de una quincena entra a la siguiente como «saldo para operar»
y se suma a los ingresos. Eso esconde un déficit: si la Q2 sólo cuadra gracias a lo que
sobró de la Q1, la Q2 no se sostiene sola, y el día que la Q1 no deje sobra el hueco
aparece de golpe.

Mandar la sobra al fondo de forma automática tampoco sirve: hoy esa sobra está pagando
la quincena siguiente y quitarla rompe el mes.

La meta es que **cada periodo se sostenga solo** y que el sistema muestre con claridad
cuánto falta para llegar ahí, en lugar de taparlo.

## Principios (como un sistema financiero, no como una hoja)

1. **El saldo inicial no es un ingreso.** La plata que ya estaba en la cuenta al empezar
   la quincena es el saldo de apertura. Se ve aparte y no entra en «ingresos».
2. **Un presupuesto se juzga sólo con lo que gana en su periodo.** Si los pagos superan
   los ingresos de la quincena, hay un déficit estructural, aunque el saldo de apertura
   alcance para cubrirlo. La app lo dice: «esta quincena depende de ₡X que no genera».
3. **Cerrar es conciliar.** Al cerrar, la persona escribe cuánto hay de verdad en sus
   cuentas. El sistema calcula lo esperado:
   `apertura + ingresos recibidos − pagos hechos − gastos puntuales`.
   La diferencia es plata que se movió sin anotarse. Se registra como «ajuste de
   conciliación» visible; nunca se esconde.
4. **Nada se mueve solo; el dominio propone y la persona confirma.** Lo que sobra al
   cerrar se reparte por un orden fijo, pero cada traslado queda como un movimiento real
   (por ejemplo, un aporte al fondo) que la persona aprueba.
5. **Lo cerrado no se edita.** El cierre deja una foto inmutable con sus indicadores. Si
   algo estaba mal, se corrige con un ajuste en el periodo abierto, como en contabilidad.
6. **La quincena es el periodo operativo y el mes, el de reporte.** El mes no tiene
   cierre propio: suma sus dos quincenas cerradas.

## Orden de reparto de lo que sobra (propuesto)

Primero se cubre el **colchón de operación**: lo que la cuenta necesita tener al empezar
la siguiente quincena para no caer en negativo entre el día de pago y los cobros
automáticos. Es configuración de cada usuario, por ejemplo los pagos que salen antes de
que llegue el salario.

Mientras la persona tenga déficit estructural, la sobra se queda como colchón y se marca
en amarillo: todavía no hay sobra real. Cuando el déficit es cero, lo que pasa del
colchón se reparte así:

1. Reponer el fondo, si hay una reposición pendiente (cuota del mes, F11).
2. Fondo de emergencia, hasta su meta (configuración de cada usuario).
3. Abono extra a la deuda prioritaria (hoy, Instacredit).
4. Lo que quede pasa a la siguiente quincena como saldo de apertura, sin contarse como
   ingreso.

## Indicadores de cada cierre

- **Autosustentable:** sí o no (ingresos del periodo ≥ pagos del periodo).
- **Dependencia del arrastre:** cuánto del gasto lo cubrió el saldo de apertura.
- **Ajuste de conciliación:** lo que se movió sin anotarse.
- **Ahorro real:** lo que salió hacia el fondo o hacia deudas.
- **Tendencia:** si la dependencia del arrastre baja o sube de una quincena a otra.

## Lo que cambiaría en el modelo

- Quincena: `saldo_apertura`, `estado` (abierta o cerrada), y al cerrar `saldo_real`,
  `ajuste` e indicadores.
- Perfil: `colchon_operacion`, `meta_fondo` y la deuda prioritaria.
- Casos de uso: `CerrarQuincena` (concilia, calcula indicadores, propone el reparto),
  `ConfirmarReparto` (crea los movimientos aprobados) y `AbrirQuincena` (copia las
  líneas de la anterior sin marcar, con la apertura igual al saldo real del cierre).

## Preguntas para Daniel

1. **Cuentas.** ¿Se concilia contra una sola cuenta (planilla) o contra varias
   (planilla, cuenta de gastos, Popular)?
2. **Colchón.** ¿De cuánto es el colchón de operación? ¿Qué pagos salen antes de que
   caiga el salario?
3. **Fuga.** ¿Desde cuánta diferencia en la conciliación conviene una alerta en rojo?
4. **Orden del reparto.** ¿Es correcto el orden: reponer, fondo, deuda y luego arrastre?
