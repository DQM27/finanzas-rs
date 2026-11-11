# Reglas de negocio

Catálogo de las reglas acordadas. Cada una tiene un código, dónde vive en el dominio y
su estado. Cualquier cambio de regla se refleja aquí en el mismo commit.

Estados: ✅ implementada y probada · 🕓 acordada, falta implementar.

## D · Dinero

| Código | Regla | Dónde | Estado |
|---|---|---|---|
| D1 | El dinero son colones enteros; nunca decimales binarios. | `dinero::Colones` | ✅ |
| D2 | Las proporciones van en diezmilésimas (`0.8617` = `8617`). | `proporcion::Proporcion` | ✅ |
| D3 | Al aplicar una proporción se redondea al colón más cercano; el empate se aleja del cero. | `Proporcion::aplicar` | ✅ |

## P · Perfil del usuario

| Código | Regla | Dónde | Estado |
|---|---|---|---|
| P1 | Neto de un monto variable = bruto × factor neto. | `PerfilSalarial::neto_de` | ✅ |
| P2 | Salario neto de la quincena = salario base bruto × factor neto − rebajos fijos. | `PerfilSalarial::salario_neto_quincena` | ✅ |
| P3 | Salario base > 0, factor neto ≤ 100 %, rebajos ≥ 0, tarifas > 0. | `PerfilSalarial::new`, `TarifasExtra::new` | ✅ |
| P4 | Ningún número personal está fijo en el código: es configuración de cada usuario. | `perfil::Perfil` | ✅ |
| P5 | Cada quien sólo lee y guarda su propio perfil: el dueño es siempre el usuario autenticado, nunca un dato que llegue en la petición. | `GuardarPerfil`, `ConsultarPerfil`, API `/perfil` | ✅ |
| P6 | El perfil se guarda completo y entero válido (salario, tarifas, caja chica y calendario); si una pieza no cumple, no se guarda nada. | `GuardarPerfil` | ✅ |

## C · Calendario de cortes

| Código | Regla | Dónde | Estado |
|---|---|---|---|
| C1 | Lo trabajado entre el inicio y el fin de un corte se paga en la fecha de pago de ese corte. | `CalendarioCortes::corte_de`, API `/cortes/hoy` | ✅ |
| C2 | Un corte válido: fin ≥ inicio, reportar antes ≥ fin, pago ≥ reportar antes. | `Corte::new` | ✅ |
| C3 | Los cortes son seguidos: cada uno empieza el día después del anterior, sin huecos ni traslapes. | `CalendarioCortes::new` | ✅ |
| C4 | El periodo de gasto va del último día de pago (incluido) al día antes del siguiente. | `CalendarioCortes::periodo_de_gasto` | ✅ |
| C5 | Pagos estimados: el 15 y el 30 (o el último día del mes si es antes); si cae en fin de semana o feriado, el día hábil anterior. | — | 🕓 |

## E · Extras

| Código | Regla | Dónde | Estado |
|---|---|---|---|
| E1 | El bruto de una extra sale de la tarifa del usuario según su tipo. | `Extra::new` | ✅ |
| E2 | Una extra `Otra` necesita el monto escrito; si se escribe un monto en otro tipo, manda lo escrito. | `Extra::new` | ✅ |
| E3 | La fecha de pago de una extra es la del corte del día trabajado (C1). | `Extra::fecha_pago` | ✅ |
| E4 | Extras de un pago = suma de los netos de las extras cuyo corte paga ese día. | `extra::neto_en_pago` | ✅ |

## K · Caja chica

| Código | Regla | Dónde | Estado |
|---|---|---|---|
| K1 | Tope por periodo de gasto (C4), mayor que cero. | `CajaChica::new` | ✅ |
| K2 | Cuentan todos los gastos puntuales del periodo, salvo los que salen del fondo. | `gasto::gastado_en_caja_chica` | ✅ |
| K3 | Baja con el 30 % del tope o menos; agotada en cero. | `CajaChica::estado` | ✅ |
| K4 | Lo que pasa del tope es exceso y sale de la sobra de la quincena. | `EstadoCaja::exceso` | ✅ |

## F · Fondo de emergencia

| Código | Regla | Dónde | Estado |
|---|---|---|---|
| F1 | Para sacar tienen que ser tres «sí»: inesperado, necesario y urgente. | `Evaluacion::es_emergencia` | ✅ |
| F2 | Recargo y plazo por nivel: vital 0 % / 6 meses, trabajo 5 % / 3, descuido 10 % / 2, falta 20 % / 2. | `NivelEmergencia` | ✅ |
| F3 | Una «falta» no es emergencia; si se saca igual, queda anotada con el 20 %. | `decidir_retiro` | ✅ |
| F4 | Una a la vez: con una reposición pendiente no se saca otra, salvo lo vital. | `decidir_retiro` | ✅ |
| F5 | No se saca más de lo que hay ni montos de cero. | `decidir_retiro` | ✅ |
| F6 | La cuota mensual de reposición se redondea hacia arriba. | `decidir_retiro` | ✅ |
| F7 | El recargo vuelve al fondo; nadie se lo queda: cada retiro se repone con su recargo. | `MovimientoFondo::reposicion`, `EstadoFondo::de` | ✅ |
| F8 | El saldo no se guarda: sale de sumar aportes y reposiciones y restar retiros. | `EstadoFondo::de` | ✅ |
| F9 | Las reposiciones pagan primero el retiro más viejo. | `EstadoFondo::de` | ✅ |
| F10 | Sólo se repone si hay algo pendiente y nunca más de lo que falta; lo que sobre se anota como aporte. | `MovimientoFondo::reposicion` | ✅ |
| F11 | Cuota del mes = la cuota de cada retiro no repuesto, sin pasarse de lo que le falta. | `EstadoFondo::de` | ✅ |

## Q · Quincena

| Código | Regla | Dónde | Estado |
|---|---|---|---|
| Q1 | Sobra = total de ingresos − total de pagos. | `Quincena::sobra` | ✅ |
| Q2 | Falta por pagar = suma de los pagos no marcados. | `Quincena::falta_por_pagar` | ✅ |
| Q3 | Ningún monto presupuestado es negativo. | `Linea::new` | ✅ |
| Q4 | Rehacer el presupuesto no borra lo marcado: una línea nueva hereda la marca de la anterior del mismo lado y concepto (en orden si se repite). | `Quincena::con_presupuesto` | ✅ |
| Q5 | Cada quincena es de un usuario y de un día de pago de su calendario; un día que no es de pago no lleva presupuesto. | `CalendarioCortes::es_dia_de_pago` | ✅ |
| Q6 | La quincena en curso es la del último pago recibido (C4). | `CalendarioCortes::ultimo_pago`, API `/cortes/hoy` | ✅ |

## G · Gastos puntuales

| Código | Regla | Dónde | Estado |
|---|---|---|---|
| G1 | Un gasto tiene monto mayor que cero. | `Gasto::new` | ✅ |
| G2 | El concepto se guarda sin espacios sobrantes, en mayúscula y con 80 caracteres como máximo. | `texto::Concepto` | ✅ |

## U · Deshacer

| Código | Regla | Dónde | Estado |
|---|---|---|---|
| U1 | Sólo se deshace lo propio: un id de otro usuario es como si no existiera. | `Deshacer`, repositorios (`borrar` filtra por usuario) | ✅ |
| U2 | En el fondo sólo se deshace el último movimiento; si no, las reposiciones posteriores quedarían pagando un retiro que ya no existe. | `movimientos_fondo::deshacible` | ✅ |
