# XcreenShot — instrucciones para Claude Code

XcreenShot es una app de captura de pantalla, anotación y grabación de video para **Windows y macOS**. El objetivo es alcanzar **paridad funcional con Snagit** para uso de Carlos y sus colegas, y después añadir funciones propias.

Toda la definición del producto está en [`spec/`](spec/). Empieza por [`spec/SPEC.md`](spec/SPEC.md).

## Continuidad entre sesiones

- **Al empezar cada sesión, lee [`docs/ESTADO.md`](docs/ESTADO.md) antes de nada.** Es el traspaso de la sesión anterior: qué está hecho, qué falta, qué decisiones se tomaron y cuál es el siguiente paso. Sigue desde ahí sin preguntarle a Carlos "¿qué hacemos ahora?".
- Mantenlo al día. Actualízalo al cerrar una entrega o un bloque de trabajo, y siempre antes de pasar a una sesión nueva. Haz commit junto con el trabajo. Guarda en él solo el resumen: el detalle va en `spec/PARITY.md`, `spec/QUESTIONS.md` y `tests/manual/`.
- Cuando Carlos pida pasar a una sesión nueva ("muévete a un nuevo chat", "genera el handoff"):
  1. Actualiza `docs/ESTADO.md`, incluida la sección "Cambios de la última sesión" (el changelog).
  2. Haz commit y push.
  3. Crea tú la sesión nueva con la herramienta de sesiones de la app: en esta misma carpeta, contexto nuevo, con el mensaje **"Lee docs/ESTADO.md y continúa"**. Dile a Carlos que la abra desde la barra lateral; no tiene que copiar ni pegar nada.
  4. Si prefiere crearla él: botón **+** junto al grupo **XcreenShot** de la barra lateral, con la carpeta `C:\Projects\XcreenShot`, y escribir esa misma frase.
- Las conversaciones anteriores se guardan en este equipo. Búscalas solo si falta un detalle que no está en el repositorio.

---

## Regla 1 — Clean room (no negociable)

Construyes **únicamente** a partir de `spec/`.

**Prohibido:**
- Abrir, listar, leer o analizar la instalación de Snagit o Camtasia: binarios, DLL, recursos, instaladores, carpetas de datos, registro o preferencias.
- Abrir o analizar archivos `.snagx`, `.snag`, `.snagproj`, `.snagtheme`, `.snagpresets`, `.snagarchive` o `.tscplugin`.
- Usar herramientas de ingeniería inversa (REA, Ghidra, Hopper, IDA, dnSpy, ILSpy, Frida, descompiladores) sobre software de TechSmith, o registrarlas como servidor MCP en este proyecto.
- Copiar íconos, sellos, temas, plantillas, sonidos o textos de ayuda de TechSmith, o calcar su interfaz a partir de capturas.
- Usar las marcas "Snagit", "Camtasia", "TechSmith" o "Screencast" en la app. Solo pueden aparecer en `spec/` y en comentarios de seguimiento de paridad.

**Si el spec no alcanza:** no busques cómo lo hace Snagit. Anota la duda en [`spec/QUESTIONS.md`](spec/QUESTIONS.md) y pregúntale a Carlos. Si la decisión es reversible, implementa tu propuesta y márcala como provisional.

**Fuentes permitidas:** `spec/`, la documentación de Rust, Tauri, React y las librerías que uses, la documentación oficial de Microsoft y Apple, y los estándares de formatos.

`spec/research/` es evidencia del proceso. No hace falta leerla para construir.

---

## Estado del spec

| Archivo | Estado |
|---|---|
| `SPEC.md`, `01-captura.md`, `03-video.md`, `05-app.md`, `06-inteligentes.md` | Completos |
| `02-editor.md` | Completo hasta las herramientas y efectos. Las plantillas, la exportación y los atajos del editor se completan antes de M2 |
| `04-biblioteca-compartir.md` | Solo el alcance de M1. Se completa antes de M2 y de M6 |

Si un ID que necesitas no existe todavía, para y pregunta.

---

## Stack

| Capa | Tecnología |
|---|---|
| Shell | Tauri 2 (Rust) |
| UI | React 18 + TypeScript estricto + Vite; Zustand; i18next (EN y ES) |
| Lienzo del editor | Konva (`react-konva`); las operaciones de píxeles se hacen en Rust |
| Captura de imagen | Crate `xcap`, con código propio por plataforma donde falte |
| Video | Crate `scap` o APIs nativas; codificadores nativos (Media Foundation; AVFoundation/VideoToolbox). Sin FFmpeg salvo aprobación |
| OCR | OCR del sistema operativo |
| Biblioteca | Archivos + índice SQLite (`rusqlite`) |
| Automatización | CLI (`clap`) + servidor MCP por stdio |
| Credenciales | Llavero del sistema |

Para cambiar una dependencia importante, proponlo primero en `spec/QUESTIONS.md`.

---

## Estructura (se crea en M0)

```
crates/xs-core      comandos, modelo de documento (.xshot), preferencias
crates/xs-capture   selección y captura de imagen
crates/xs-video     grabación y codificación
crates/xs-image     píxeles y efectos
crates/xs-ocr       reconocimiento de texto
crates/xs-library   biblioteca e índice
crates/xs-share     destinos de compartir
crates/xs-cli       binario `xcreenshot` (CLI + MCP)
app/src-tauri       shell, bandeja / barra de menú, atajos globales
app/src             React
docs/adr/           decisiones de arquitectura
docs/ESTADO.md      estado del proyecto: traspaso entre sesiones
scripts/dev/        ayudas para probar en Windows
tests/manual/       pruebas manuales por ID
AGENTS.md           cómo operan la app los agentes (CLI y MCP)
```

---

## Arquitectura: todo es un comando

- Cada acción es un **comando** con un ID estable (`capture.region`, `editor.object.add`…), parámetros tipados, un resultado y errores definidos.
- Los comandos se registran una sola vez en `xs-core`. La UI, los atajos, el CLI y el servidor MCP usan ese mismo registro.
- Los componentes de React no tienen lógica de negocio.
- Cada comando cita en su documentación los IDs del spec que implementa.

## Multiplataforma

- El comportamiento es el mismo en Windows 11 y en macOS (Apple Silicon). El código de cada plataforma va detrás de traits con `#[cfg(target_os)]`.
- La CI compila y prueba ambas plataformas en cada PR.
- Internamente todo se mide en píxeles físicos; la conversión se hace en el borde con la UI.

---

## Flujo de trabajo

1. Trabaja por hito (`SPEC.md` §4) y por ID.
2. Lee el bloque completo del ID antes de implementarlo.
3. Cada criterio de aceptación lleva una prueba automatizada. Si no se puede automatizar, crea `tests/manual/<ID>.md`.
4. Al terminar un ID, actualiza `spec/PARITY.md`.
5. Los commits van en inglés, con el ID: `feat(CAP-012): window capture`.

### Definición de terminado
- [ ] Los criterios de aceptación se cumplen en Windows **y** macOS.
- [ ] Las pruebas pasan; clippy, fmt, ESLint y Prettier están limpios.
- [ ] Los textos de la UI están en inglés y español.
- [ ] Se puede usar con teclado y tiene etiquetas para lector de pantalla.
- [ ] El comando está en el CLI y documentado en `AGENTS.md`.

## Privacidad

- Sin telemetría. La única red es compartir (cuando el usuario lo elige) y las actualizaciones.
- Las credenciales van en el llavero del sistema, nunca en archivos ni en logs.
- El procesamiento inteligente es siempre local.
- MCP solo por stdio.

## Comandos de desarrollo

Requisitos: Rust estable (rustup), Node 22, y en Windows las Build Tools de Visual Studio (C++).

| Qué | Comando (desde la raíz salvo indicación) |
|---|---|
| Instalar dependencias de la UI | `npm ci` en `app/` |
| App en desarrollo (Vite + Tauri, recarga en caliente) | `npm run tauri dev` en `app/` |
| Compilar app y CLI | `cargo build --workspace` (el CLI queda en `target/debug/xcreenshot`) |
| Probar el CLI contra la app en marcha | `target/debug/xcreenshot app ping --json` |
| Pruebas Rust | `cargo test --workspace` |
| Formato y lint Rust | `cargo fmt --all` · `cargo clippy --workspace --all-targets -- -D warnings` |
| UI: lint, formato, tipos, pruebas | `npm run lint` · `npm run format:check` · `npm run typecheck` · `npm test` en `app/` |
| Sin telemetría (APP-054) | `node scripts/check-no-telemetry.mjs` |
| Regenerar el ícono | `node scripts/make-icon.mjs` y después `npx tauri icon src-tauri/icons/app-icon.png -o src-tauri/icons` en `app/` |
| Paquete instalable | `npm run tauri build` en `app/` |

Dónde vive cada cosa:
- Comandos: especificación en `crates/xs-core/src/catalog.rs`, manejadores en `crates/xs-core/src/core.rs`.
- Textos EN/ES: `crates/xs-core/locales/{en,es}.json` (los comparten Rust y React).
- Datos del usuario: `%APPDATA%\XcreenShot\config` y `%LOCALAPPDATA%\XcreenShot\data` (Windows); `~/Library/Application Support/XcreenShot` (macOS). `XCREENSHOT_DATA_DIR` y `XCREENSHOT_IPC_NAME` aíslan una instancia para pruebas.
- Un comando nuevo exige: entrada en el catálogo, manejador, textos en los dos idiomas y fila en `AGENTS.md` (hay pruebas que fallan si falta alguno).

## Cómo trabajar con Carlos

- Carlos es consultor de tecnología, no programador de profesión. Explícale las decisiones en español claro y directo.
- Señala los riesgos técnicos, legales o de alcance sin rodeos.
- Los cambios de alcance se proponen; no se imponen.
- Conversación y documentación en **español**; código, comentarios y commits en **inglés**.
