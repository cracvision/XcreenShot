# ADR 0001 · Registro de comandos y canal local

**Estado:** aceptada · **Fecha:** 2026-10-12 · **IDs:** APP-001, APP-027, APP-028, APP-032

## Contexto

SPEC §2.2 exige que toda acción sea un comando que llaman por igual la interfaz, los atajos, el menú del ícono, el CLI y (desde M2) el servidor MCP. APP-032 prohíbe abrir puertos de red para la automatización.

## Decisión

1. **Catálogo estático en `xs-core`** (`catalog.rs`): ID, esquema JSON de parámetros y de resultado, ámbitos, banderas (`mutates`, `network`, `destructive`, `interactive`, `assignable`) y los IDs del spec. El CLI lo lee directamente para construir su ayuda y su análisis de argumentos, aunque la app no esté en marcha.
2. **Manejadores en `xs-core::Core`.** Lo que necesita el sistema de ventanas (abrir una ventana, el portapapeles, el ícono) pasa por el trait `Host`, que implementa el shell de Tauri. Así la lógica es probable sin interfaz.
3. **Una sola puerta de entrada:** `Registry::call` valida el ámbito y los parámetros (subconjunto de JSON Schema propio, sin dependencia externa), aplica las *gates* (interruptor de automatización) y avisa a los observadores (registro local y eventos para la interfaz).
4. **Menús como datos** (`menus.rs`). Cada entrada nombra un comando; una prueba comprueba que todos existen (criterio de APP-027).
5. **Canal local con `interprocess`:** tubería con nombre por usuario y sesión en Windows, con una ACL que solo permite al propietario y a SYSTEM; socket Unix con permisos 0600 en el `$TMPDIR` del usuario en macOS. Protocolo: una línea JSON de petición y una de respuesta por conexión.
6. **Instancia única** con el mismo canal: si al arrancar ya hay alguien escuchando, la segunda copia le pide `app.show` y termina.
7. **El origen declarado no es de fiar.** La app trata toda petición del canal como `cli` (o `mcp`), salvo `app.show` de una segunda copia de la app. La interfaz solo puede declararse `ui` o `shortcut`.

## Consecuencias

- Añadir una función es: añadir su `CommandSpec`, su manejador, sus textos en `en.json`/`es.json` y su fila en `AGENTS.md`. Las pruebas fallan si falta cualquiera de las tres últimas.
- Dependencias nuevas dentro del stack aprobado: `interprocess` (canal local), `clap` (CLI), `tracing` (registros) y los plugins oficiales de Tauri `global-shortcut`, `autostart`, `notification`, `opener`, `clipboard-manager` y `window-state`.
