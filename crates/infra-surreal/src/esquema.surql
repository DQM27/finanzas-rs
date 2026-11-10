-- Esquema de finanzas-rs en SurrealDB.
--
-- Se puede aplicar las veces que haga falta: cada definición usa OVERWRITE.
--
-- Aislamiento entre usuarios:
-- * Las tablas no dan permiso a usuarios de registro (PERMISSIONS NONE). Sólo
--   el Worker entra, con un usuario de sistema de la base de datos.
-- * Toda consulta del Worker filtra por `usuario = $usuario`. Las pruebas de
--   contrato comprueban que un usuario nunca ve lo de otro.
--
-- Tipos:
-- * Fechas como texto `AAAA-MM-DD`: se comparan bien como texto y no cargan
--   zona horaria.
-- * Dinero como entero de colones; proporciones en diezmilésimas.
-- * El id del registro es el UUID del dominio (o el del usuario, en `perfil`).

DEFINE TABLE OVERWRITE perfil SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE usuario ON perfil TYPE string;
DEFINE FIELD OVERWRITE salario_base ON perfil TYPE int ASSERT $value > 0;
DEFINE FIELD OVERWRITE factor_neto ON perfil TYPE int ASSERT $value >= 0 AND $value <= 10000;
DEFINE FIELD OVERWRITE rebajos ON perfil TYPE int ASSERT $value >= 0;
DEFINE FIELD OVERWRITE tarifas ON perfil TYPE object;
DEFINE FIELD OVERWRITE tarifas.nocturna ON perfil TYPE int ASSERT $value > 0;
DEFINE FIELD OVERWRITE tarifas.diurna ON perfil TYPE int ASSERT $value > 0;
DEFINE FIELD OVERWRITE tarifas.especial ON perfil TYPE int ASSERT $value > 0;
DEFINE FIELD OVERWRITE tarifas.feriado ON perfil TYPE int ASSERT $value > 0;
DEFINE FIELD OVERWRITE tope_caja_chica ON perfil TYPE int ASSERT $value > 0;
DEFINE FIELD OVERWRITE cortes ON perfil TYPE array<object>;
DEFINE FIELD OVERWRITE cortes[*].inicio ON perfil TYPE string;
DEFINE FIELD OVERWRITE cortes[*].fin ON perfil TYPE string;
DEFINE FIELD OVERWRITE cortes[*].reportar_antes ON perfil TYPE string;
DEFINE FIELD OVERWRITE cortes[*].pago ON perfil TYPE string;
DEFINE INDEX OVERWRITE perfil_usuario ON perfil FIELDS usuario UNIQUE;

DEFINE TABLE OVERWRITE gasto SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE usuario ON gasto TYPE string;
DEFINE FIELD OVERWRITE fecha ON gasto TYPE string;
DEFINE FIELD OVERWRITE concepto ON gasto TYPE string;
DEFINE FIELD OVERWRITE monto ON gasto TYPE int ASSERT $value > 0;
DEFINE FIELD OVERWRITE categoria ON gasto TYPE string
    ASSERT $value IN ["COMIDA", "CASA", "MOTO", "SALUD", "FAMILIA", "ROPA", "TRAMITES", "GUSTO", "OTRO"];
DEFINE FIELD OVERWRITE origen ON gasto TYPE string
    ASSERT $value IN ["CUENTA_GASTOS", "PLANILLA", "SOBRA_DEL_MES", "GUSTO", "FONDO"];
DEFINE INDEX OVERWRITE gasto_usuario_fecha ON gasto FIELDS usuario, fecha;

DEFINE TABLE OVERWRITE extra SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE usuario ON extra TYPE string;
DEFINE FIELD OVERWRITE fecha ON extra TYPE string;
DEFINE FIELD OVERWRITE tipo ON extra TYPE string
    ASSERT $value IN ["NOCTURNA", "DIURNA", "ESPECIAL", "FERIADO", "OTRA"];
DEFINE FIELD OVERWRITE bruto ON extra TYPE int ASSERT $value > 0;
DEFINE INDEX OVERWRITE extra_usuario_fecha ON extra FIELDS usuario, fecha;
