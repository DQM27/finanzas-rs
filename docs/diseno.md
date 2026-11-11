# Sistema visual

Especificación visual de `apps/web`, la app completa en sus dos niveles (celular y
computadora). Toma el lenguaje de la app móvil de Lattis (identidad
"Kash"): controles que imitan iOS, formularios agrupados al estilo de Ajustes, barra de
secciones de vidrio y avisos que bajan desde arriba. La app de Apps Script (v6) ya usa
estos mismos tokens; sirve de referencia viva mientras la PWA no exista.

Reglas:

- **Todo color, radio, tamaño y espacio sale de un token.** Ningún componente escribe un
  hex o un `px` suelto.
- **Dos temas, Día y Noche**, según `prefers-color-scheme`. Los tokens se llaman igual en
  los dos; sólo cambia su valor.
- **Mobile first.** Se diseña para un iPhone en vertical. El escritorio es la misma columna,
  centrada, con un ancho máximo de 560 px.
- **El color no carga significado solo.** Un monto negativo lleva signo `−` además del rojo,
  y un pago hecho lleva la marca ✓ además del tachado.

---

## 0. Dos niveles: celular y computadora

Es una sola app con dos niveles, no dos apps. Comparten tokens, componentes y datos;
cambia qué se muestra y cómo se organiza.

| | Celular (menos de 900 px) | Computadora (900 px o más) |
|---|---|---|
| Para qué | Lo del día, en segundos | Administrar y planear |
| Navegación | Barra de secciones de vidrio abajo | Barra lateral fija a la izquierda |
| Pantallas | Inicio, Quincena, Anotar | Todo lo del celular, más Proyección, Deudas, Fondo, Calendario de cortes, Historial y Configuración |
| Diseño | Una columna, grupos de iOS | Varias columnas: resumen arriba, tablas y detalle lado a lado |
| Tablas | No hay: listas de renglones | Tablas con orden y filtros (historial, deudas, extras) |

- **Lo del celular también existe en la computadora,** con el mismo comportamiento.
- **Lo exclusivo de la computadora no se esconde en el celular:** queda alcanzable desde
  «Más», pero en modo de consulta (sin editar el calendario ni la configuración).
- **El punto de corte es el ancho, no el dispositivo:** una tableta acostada usa la
  versión de computadora.

---

## 1. Paleta

### Base

| Token | Día (Tokyo Day) | Noche (AMOLED) | Uso |
|---|---|---|---|
| `fondo` | `#D0D5E3` | `#08080A` | Fondo de la pantalla |
| `tarjeta` | `#E1E2E7` | `#1F1F24` | Grupos y tarjetas |
| `variante` | `#C4C8DA` | `#2E2E35` | Pista de barras, selector segmentado, esqueleto |
| `realce` | `#E9E9ED` | `#2E2E35` | Segmento elegido |
| `texto` | `#343B58` | `#F2F2F5` | Texto principal |
| `texto-2` | `#565F89` | `#A1A1AA` | Etiquetas, notas, encabezados de grupo |
| `borde` | `#848CB5` | `#48484F` | Divisores (al 45 %) y marcas vacías |
| `vidrio` | `#848CB5` al 25 % | blanco al 22 % | Borde de la barra de secciones |
| `pildora` | `#343B58` al 7 % | `#F2F2F5` al 12 % | Fondo del renglón presionado y de la pestaña elegida |

### Estados

| Token | Día | Noche | Uso |
|---|---|---|---|
| `acento` | `#2959AA` | `#6EA8FF` | Enlaces, selectores, pestaña elegida |
| `relleno` | `#2959AA` | `#3366CC` | Fondo del botón principal |
| `exito` / `relleno-exito` | `#4A6330` / `#4A6330` | `#4ADE80` / `#1F7A44` | Barras sanas, ✓, aviso correcto |
| `error` / `relleno-error` | `#A93737` / `#A93737` | `#FF6B6B` / `#C53030` | Negativos, pagos pendientes, aviso de error |
| `advertencia` | `#E0A100` (texto `#7F5600`) | `#FBBF24` | Barras al 30 % o menos |
| `info` | `#006D84` | `#6CC5E6` | Notas informativas |

`acento` se usa para texto e íconos. `relleno` va debajo de texto blanco, y por eso es más
oscuro en Noche.

### Tonos de sección

Cada tipo de movimiento tiene un tono propio. Se usa en su ícono (fondo al 22 %), en el
título de su formulario y en su botón Guardar (`relleno`).

| Sección | Acento Día | Acento Noche | Relleno Noche |
|---|---|---|---|
| Gasolina | `#B85C00` | `#FFB547` | `#B85C00` |
| Extra | `#6D32C9` | `#B79CFF` | `#7C45D6` |
| Puntual | `#00806A` | `#5CC8B2` | `#00806A` |
| Instacredit | `#C4161C` | `#FF6B5E` | `#D0312D` |
| Fondo | `#2959AA` | `#6EA8FF` | `#3366CC` |
| Moto | `#006D84` | `#6CC5E6` | `#00708A` |

En Día, el relleno de cada sección es igual a su acento.

---

## 2. Tipografía

Fuente del sistema (`-apple-system`, SF Pro en iPhone, Roboto en Android). Escala de iOS:

| Token | Tamaño / alto de línea | Peso | Uso |
|---|---|---|---|
| `titulo-grande` | 30 / 36 | 700 | Título de pantalla |
| `titulo` | 22 / 28 | 700 | Título de hoja o modal |
| `cifra` | 24 / 30 | 700 | Monto de una tarjeta de resumen |
| `titulo-medio` | 17 / 22 | 600 | Botón principal, encabezado de lista |
| `cuerpo` | 16 / 22 | 400 | Texto y campos (16 evita el zoom de iOS) |
| `cuerpo-chico` | 13 / 18 | 400 | Notas al pie y subtítulos |
| `etiqueta` | 12 / 16 | 500 | Encabezados de grupo (MAYÚSCULA) y claves de tarjeta |
| `etiqueta-chica` | 11 / 14 | 500 | Rótulos de la barra de secciones |

Los montos usan cifras tabulares (`font-variant-numeric: tabular-nums`) para que se alineen.

---

## 3. Medidas

| Token | Valor | Uso |
|---|---|---|
| `radio-grupo` | 14 px | Grupos de formulario y botón principal |
| `radio-tarjeta` | 20 px | Tarjetas de resumen y de sección |
| `radio-control` | 16 px | Avisos |
| `radio-barra` | 22 px | Barra de secciones (la pestaña interna mide 22 − 5 = 17, radio concéntrico) |
| `alto-renglon` | 52 px mínimo | Renglón de un grupo |
| `alto-boton` | 50 px | Botón principal |
| `alto-barra` | 58 px | Barra de secciones |
| `espacio` | 12 px | Separación base; márgenes de pantalla de 16 px |
| `divisor` | 0,5 px, con sangría de 16 px | Entre renglones (58 px si el renglón lleva ícono) |

Toda superficie tocable mide al menos 44 × 44 px.

---

## 4. Componentes

### Grupo de formulario (Ajustes de iOS)

- **Encabezado:** fuera del grupo, en `etiqueta`, MAYÚSCULA, `texto-2`, con 16 px de
  sangría.
- **Grupo:** fondo `tarjeta` y `radio-grupo`, sin borde ni sombra.
- **Renglones:** separados por `divisor`.
- **Renglón de campo:** etiqueta a la izquierda en una columna de 96 px; el valor o campo a
  la derecha, alineado a la derecha y sin marco. El placeholder dice `Requerido` u
  `Opcional`.
- **Selector:** muestra el valor en `acento` con un chevron `›`.
- **Nota al pie:** debajo del grupo, en `cuerpo-chico` y `texto-2`. Si es un error de
  validación, va en `error` y 600.

### Tarjeta de resumen

Clave en `etiqueta`, cifra en `cifra` y, opcional, una barra de 6 px más una nota. La barra
se pone en `exito` arriba del 30 %, en `advertencia` al 30 % o menos y en `error` en 0. El
fondo de emergencia usa `acento`, porque crecer no es un estado de alarma.

### Tarjeta teñida

Fondo suave del tono más una franja de 4 px a la izquierda. Sirve para estados que tienen
que saltar a la vista, por ejemplo una quincena que cierra en negativo.

### Renglón de movimiento

Ícono de 30 px (radio 8, fondo del tono de la sección al 22 %), concepto y fecha, monto a la
derecha, y la acción de deshacer al final. Un deshacer siempre pide confirmación.

### Renglón de pago (lista de verificación)

Marca circular de 24 px. Vacía tiene borde `borde`; hecha tiene fondo `exito` y ✓. Un pago
pendiente muestra el monto en `error`. Uno hecho va tachado y en `texto-2`. Encima de la
lista va el progreso «Pagos · 3 de 9».

### Selector segmentado

Para Q1/Q2. Pista `variante` con radio 9 y padding 2. El segmento elegido va en `realce`
con una sombra leve.

### Botón principal

Ancho completo, `alto-boton`, `radio-grupo`, `titulo-medio` blanco sobre el `relleno` de la
sección. Mientras guarda, queda deshabilitado y dice «Guardando…».

### Barra de secciones (vidrio)

- **Posición y forma:** flota a 12 px del borde inferior, más el área segura. Ocupa el ancho
  de la pantalla menos 16 px por lado. Mide `alto-barra` y usa `radio-barra`.
- **Vidrio:** fondo `tarjeta` al 82 % (78 % en Noche), con `backdrop-filter: blur(18px)
  saturate(1.6)` y un borde de 1 px en `vidrio`.
- **Pestañas:** ícono de 22 px más el rótulo en `etiqueta-chica`. La elegida lleva la
  píldora interna `pildora`, con radio concéntrico, y el ícono y el texto en `acento`.
- **Distintivos:** grises, nunca rojos, para no generar alarma falsa.
- **Respuesta háptica:** al cambiar de pestaña donde el sistema lo permita.

### Aviso

Baja desde arriba bajo el área segura con `radio-control`, texto blanco 600 y fondo
`relleno-exito` o `relleno-error`. Dura 2,4 s. Nunca tapa la barra de secciones.

### Carga

- **Esqueleto:** bloques `variante` con pulso, nunca un spinner a pantalla completa.
- **Datos guardados:** si ya hay datos guardados, se pintan al instante y se refrescan por
  detrás.

---

## 5. Movimiento y accesibilidad

- **Curva:** `cubic-bezier(.2, .8, .2, 1)`, de 200 a 450 ms. Nada rebota.
- **Movimiento reducido:** con `prefers-reduced-motion` se apagan transiciones y
  animaciones.
- **Contraste:** el texto debe cumplir WCAG AA en los dos temas. `texto-2` sobre `tarjeta`
  es el par más justo y no se usa para cifras.
- **Íconos solos:** todo ícono sin texto lleva `aria-label`.

---

## 6. En Angular

- **Tokens:** viven en `apps/web/src/styles/tokens.scss` como propiedades CSS, `:root` para
  Día y `@media (prefers-color-scheme: dark)` para Noche. Los tonos de sección se exponen
  como `--seccion` y `--seccion-relleno`; cada componente de sección los fija con
  `[attr.data-seccion]`.
- **Componentes autónomos:** `fz-grupo`, `fz-renglon`, `fz-tarjeta`, `fz-segmentado`,
  `fz-barra-secciones`, `fz-barra-lateral`, `fz-tabla` y `fz-aviso`. No llevan
  dependencias de UI de terceros. El nivel se elige con un solo `BreakpointObserver` a
  900 px; los componentes no preguntan por el dispositivo.
- **Montos:** el formato sale de una sola tubería, `colones`, que recibe enteros. La
  aritmética de dinero nunca se hace en TypeScript: viene del dominio (WASM o worker).

### Estado

Hechos, en `apps/web/src/app/ui/`: `fz-grupo`, `fz-renglon`, `fz-tarjeta` (de resumen
y teñida), `fz-segmentado`, `fz-barra-secciones`, `fz-aviso` (con el servicio
`Avisos`), `fz-esqueleto`, `button[fz-boton]` y `fz-icono` (trazos propios, sin
librería). La página **Más → Muestrario** los muestra todos con datos de ejemplo.

Las medidas internas de los componentes (tamaños de ícono, divisor, columna de
etiqueta, ancho máximo, duraciones) también son tokens, en `tokens.scss`.

Faltan `fz-barra-lateral`, `fz-tabla` y el cambio de nivel a 900 px, que llegan con
la versión de computadora.
