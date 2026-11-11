# Despliegue

Cómo poner en marcha el Worker con SurrealDB Earth y Cloudflare Access. Los pasos que
piden iniciar sesión o crear claves los hace Daniel; nada de eso va al repositorio.

## 1. SurrealDB Earth

Instancia: `cfinanzas` (`06gqbrtn9dv671ekf9ecbetue4.surrealdb.earth`), residencia
mundial, SurrealDB 4.0.0.

En SurrealDB Studio, con la sesión de la cuenta dueña:

```surql
DEFINE NAMESPACE IF NOT EXISTS finanzas;
USE NS finanzas;
DEFINE DATABASE IF NOT EXISTS produccion;
USE NS finanzas DB produccion;

-- Quien aplica el esquema (sólo para migraciones).
DEFINE USER esquema ON DATABASE PASSWORD "<clave larga generada>" ROLES OWNER;
-- Con quien entra el Worker: lee y escribe datos, no cambia el esquema.
DEFINE USER worker ON DATABASE PASSWORD "<otra clave larga generada>" ROLES EDITOR;
```

Después, con el usuario `esquema`, se corre el contenido de
`crates/infra-surreal/src/esquema.surql`. Se puede repetir sin miedo: usa `OVERWRITE`.

## 2. Cloudflare Access (Google)

En Zero Trust:

1. **Ajustes → Autenticación → Métodos de inicio de sesión:** agregar Google. No hace
   falta el código por correo.
2. **Acceso → Aplicaciones → Agregar → Autoalojada:** dominio del Worker, política
   «Permitir» con los correos que pueden entrar y Google como único método.
3. Copiar el **nombre del equipo** (`https://EQUIPO.cloudflareaccess.com`) y la
   **etiqueta AUD** de la aplicación a `ACCESO_EMISOR` y `ACCESO_AUDIENCIA` en
   `apps/worker/wrangler.toml`.

Access corta antes de que llegue la petición, pero el Worker igual verifica el token
(`crates/acceso`). Un correo que pase Access y no esté en `USUARIOS` recibe 401.

## 3. Secretos del Worker

Desde `apps/worker`:

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

`wrangler` compila con `worker-build`, que necesita el objetivo
`wasm32-unknown-unknown` instalado. La CI ya comprueba que todo compile para ese
objetivo en cada commit.

## Qué queda pendiente

- Cargar el perfil (salario, tarifas, calendario de cortes) del primer usuario.
  Mientras no exista, la API responde `PERFIL_NO_ENCONTRADO`.
- Servir la PWA de Angular desde el mismo Worker, para que el inicio de sesión sea uno
  solo.
