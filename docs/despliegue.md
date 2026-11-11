# Despliegue

Cómo poner en marcha el Worker con SurrealDB Earth y Cloudflare Access. Los pasos que
piden iniciar sesión o crear claves los hace Daniel; nada de eso va al repositorio.

Estado actual: el Worker `finanzas-rs` atiende en `https://cf.megabrisas.com`, detrás
de Access, contra la instancia de SurrealDB Earth de abajo.

## 1. SurrealDB Earth

Instancia: `cfinanzas` (`06gqbrtn9dv671ekf9ecbetue4.surrealdb.earth`), residencia
mundial, SurrealDB 4.

### Entrar desde la terminal

El menú **Connect → CLI** de Studio muestra un comando con `--username root
--password root`: es sólo un ejemplo, la instancia no trae usuarios. Para trabajar
desde la terminal:

1. En Studio, **Authentication → Level: Root → Add system user**: un usuario
   temporal (por ejemplo `instalador`) con rol **Owner** y una clave larga generada.
2. Pasarle usuario y clave a `surreal sql` por las variables `SURREAL_USER` y
   `SURREAL_PASS` (nunca como argumentos), con `--auth-level root`.
3. Al terminar, `REMOVE USER instalador ON ROOT;` y borrar la variable.

**Cuidado:** `surreal sql` guarda todo lo que recibe, también por stdin, en un
`history.txt` dentro de la carpeta desde donde se corre. Un `DEFINE USER … PASSWORD`
deja la clave ahí en texto plano. Correrlo desde una carpeta temporal y borrar ese
archivo al terminar; nunca desde la raíz del repo.

### Base y usuarios

Con ese usuario root:

```surql
DEFINE NAMESPACE IF NOT EXISTS finanzas;
USE NS finanzas;
DEFINE DATABASE IF NOT EXISTS produccion;
USE NS finanzas DB produccion;

-- Quien aplica el esquema (sólo para migraciones).
DEFINE USER OVERWRITE esquema ON DATABASE PASSWORD "<clave larga generada>" ROLES OWNER;
-- Con quien entra el Worker.
DEFINE USER OVERWRITE worker ON DATABASE PASSWORD "<otra clave larga generada>" ROLES EDITOR;
```

Ojo: en SurrealDB, **EDITOR también puede cambiar el esquema** (`DEFINE TABLE`,
`FIELD`, `INDEX`…); lo único que no toca son usuarios y accesos. Es el rol más chico
que puede escribir datos (VIEWER sólo lee), así que el aislamiento del esquema depende
de que la clave de `worker` sólo la tenga el Worker.

Después, con el usuario `esquema` (`--auth-level database --ns finanzas --db
produccion`), se corre el contenido de `crates/infra-surreal/src/esquema.surql`. Se
puede repetir sin miedo: usa `OVERWRITE`. Se verifica con `INFO FOR DB;`: tablas
`perfil`, `gasto`, `extra`, `movimiento_fondo` y `quincena`; usuarios `esquema`
(OWNER) y `worker` (EDITOR).

La clave de `esquema` no se guarda en ningún lado. Para la próxima migración se
redefine (`DEFINE USER OVERWRITE esquema …`) con un root temporal como el de arriba.

### Consultas desde Studio: siempre con `USE`

El editor de consultas de Studio abre en `main` / `main` o en el último namespace
elegido, no en el del Worker. Una consulta corrida ahí no da error (si la tabla no
existe con esquema, SurrealDB la crea) y simplemente no llega a la app. Toda consulta
para producción empieza con:

```surql
USE NS finanzas DB produccion;
```

Así pasó con la primera carga del perfil: quedó en un namespace viejo de prueba
(`cfinanzas`), que después se borró.

### Cómo entra el Worker

El cliente (`crates/infra-surreal`) hace `signin` por RPC con usuario, clave,
namespace y base, y consulta con el token que le devuelven. Earth no acepta
`Authorization: Basic` para un usuario de base de datos: responde 401 aunque la
clave sea correcta.

## 2. Cloudflare Access

Se usa el equipo de Zero Trust que ya existía en la cuenta (`icy-field-7df6`, el
mismo de `panel.megabrisas.com`), con el **código por correo** como método de
inicio de sesión. No hace falta configurar Google; se puede agregar después sin
tocar el código.

1. **Controles de Access → Aplicaciones → Agregar → Autoalojada y privada → DNS
   público:** subdominio `cf`, dominio `megabrisas.com`, sin ruta.
2. Política `solo-daniel`: «Permitir», selector **Correos electrónicos** con el
   correo de Daniel. Una aplicación propia, no la del panel: ésa permite a otras
   personas.
3. Duración de la sesión a gusto (24 horas por defecto).
4. `ACCESO_EMISOR` es `https://<equipo>.cloudflareaccess.com`. La **etiqueta AUD**
   sale del panel o del parámetro `kid` de la redirección al login:
   `curl -sI https://cf.megabrisas.com/api/caja-chica`.

El Worker sólo se publica en el dominio propio: en `wrangler.toml`, `workers_dev` y
`preview_urls` están en `false`, porque `*.workers.dev` sería una entrada sin Access.

Access corta antes de que llegue la petición, pero el Worker igual verifica el token
(`crates/acceso`). Un correo que pase Access y no esté en `USUARIOS` recibe 401.

## 3. Secretos del Worker

Desde `apps/worker`, siempre por stdin (por ejemplo
`printf '%s' "$CLAVE" | npx wrangler secret put SURREAL_CLAVE`), nunca escritos en la
terminal ni en un archivo:

```text
npx wrangler secret put SURREAL_USUARIO   # worker
npx wrangler secret put SURREAL_CLAVE     # la clave del usuario worker
npx wrangler secret put USUARIOS          # correo@gmail.com=<uuid de Daniel>
```

El UUID de cada persona se genera una vez y no cambia: es la llave de todos sus datos.

## 4. Publicar

```text
cd apps/worker
npx wrangler deploy
```

El build de `wrangler` hace dos cosas, en este orden:

1. Compila la PWA (`apps/web`: `npm ci` y `ng build`, con Node 24) en
   `apps/web/dist/web/browser`.
2. Compila el Worker con `worker-build`, que necesita el objetivo
   `wasm32-unknown-unknown` instalado.

La PWA se publica como archivos estáticos del mismo Worker (`[assets]` en
`wrangler.toml`). Sólo `/api/*` llega al código Rust; cualquier otra ruta que no sea
un archivo devuelve `index.html`, y la resuelve el router de Angular. Access protege
el dominio entero, así que la página y la API comparten una sola sesión.

La CI comprueba en cada commit que el Rust compile a WebAssembly y que la PWA
compile y pase sus pruebas.

En Windows, `worker-build` y las pruebas necesitan un enlazador para el host. Sin las
Build Tools de Visual Studio, la toolchain `msvc` termina usando el `link.exe` de
msys/Git y falla. Alternativa sin instalar nada más: la misma versión con la toolchain
GNU, que trae su propio enlazador:

```text
rustup toolchain install 1.97.0-x86_64-pc-windows-gnu --profile minimal -c clippy,rustfmt -t wasm32-unknown-unknown
RUSTUP_TOOLCHAIN=1.97.0-x86_64-pc-windows-gnu npx wrangler deploy
```

## 5. Comprobar

- Sin sesión, `curl -s -o /dev/null -w '%{http_code}' https://cf.megabrisas.com/api/caja-chica`
  da `302` hacia el login de Access. Lo mismo `/` y cualquier archivo de la PWA.
- Con sesión, `https://cf.megabrisas.com/` abre la PWA.
- Con sesión, en el navegador, `/api/caja-chica` responde 404
  `PERFIL_NO_ENCONTRADO` mientras no se cargue el perfil.
- `npx wrangler tail` no muestra errores de configuración ni de SurrealDB.

## Qué queda pendiente

- La pantalla de configuración que use `GET`/`PUT /api/perfil` (la ruta ya existe).
  El perfil de Daniel ya está cargado (11/11/2026), con el calendario oficial hasta
  febrero de 2027 y el estimado de marzo a diciembre de 2027.
- Las pantallas de la PWA que faltan: Quincena, Anotar y Configuración. Inicio ya
  está.
- Hacerla instalable: manifiesto e íconos.
