# Estado del proyecto

Este archivo es el traspaso entre sesiones de Claude Code. Cada sesión nueva lo lee al empezar (lo pide `CLAUDE.md`). Se actualiza al cerrar una entrega o un bloque de trabajo, y siempre antes de pasar a una sesión nueva. Es un resumen: el detalle está en las fuentes de abajo.

**Última actualización:** 2026-10-24, al cerrar la entrega 3 de M1.

## Fuentes

| Qué | Dónde |
|---|---|
| Definición del producto | [`spec/SPEC.md`](../spec/SPEC.md) y los archivos `spec/0x-*.md` |
| Avance por ID | [`spec/PARITY.md`](../spec/PARITY.md) |
| Decisiones y dudas abiertas | [`spec/QUESTIONS.md`](../spec/QUESTIONS.md) |
| Pruebas manuales y sus resultados | [`tests/manual/`](../tests/manual/) |
| Decisiones de arquitectura | [`docs/adr/`](adr/) |
| Comandos para agentes (CLI y MCP) | [`AGENTS.md`](../AGENTS.md) |
| Historial de cambios | `git log` y los PR en [GitHub](https://github.com/cracvision/XcreenShot/pulls) |
| Ayudas para probar en Windows | [`scripts/dev/`](../scripts/dev/README.md) |
| Conversaciones anteriores | Se guardan en este equipo y se pueden buscar cuando falta un detalle que no está en el repositorio |

## Dónde estamos

- **Hito actual:** M1 · Captura y edición básicas. Versión 0.2.0.
- **M0** está terminado y verificado en Windows ([PR #1](https://github.com/cracvision/XcreenShot/pull/1), fusionado).
- **M1** se construye por entregas en la rama `m1-capture` ([PR #2](https://github.com/cracvision/XcreenShot/pull/2), abierto, CI en verde). Último commit: `c97ce05`.

| Entrega de M1 | Contenido | Estado |
|---|---|---|
| 1 · Motor de captura | Pantalla completa, región y ventana; portapapeles; metadatos; comandos `capture.*` en el CLI | Hecha y probada por Carlos |
| 2 · Selección en pantalla | Pantallas congeladas, cruceta, lupa, detección de ventanas, barra de captura con **Capturar** | Hecha y probada por Carlos |
| 3 · Panel de captura | Pestañas Universal e Imagen, retardo con cuenta atrás, menú del ícono, Repetir última captura y Repetir última área, restablecer ajustes | Hecha; faltan 3 pruebas de Carlos (abajo) |
| 4 · Editor básico | Flecha, Texto, Forma, Desenfocar, Recortar, deshacer y rehacer; guardar PNG, JPG y `.xshot` | Siguiente |
| 5 · Biblioteca | Autoguardado y bandeja reciente básica | Pendiente |

En `spec/QUESTIONS.md` el editor aparece como "parte 2 de M1" y la biblioteca como "parte 3": son las entregas 4 y 5 de esta tabla.

macOS: el código compila en la CI, pero ese trabajo no bloquea los PR (`continue-on-error`). Las pruebas en Mac empiezan cuando Carlos lo diga. Hasta entonces, ningún ID pasa de `Win` a `Hecho` en `PARITY.md`.

## Cambios de la última sesión

Sesión "M0 startup según spec" (2026-10-12 a 2026-10-24). El detalle está en `git log`.

- **M0 completo:**
  - Registro de comandos, CLI, ícono en la bandeja, atajos globales, preferencias, EN/ES y CI.
  - Fusionado en el PR #1.
- **M1, entrega 1:** motor de captura.
- **M1, entrega 2:**
  - Selección en pantalla rápida (~105 ms en 5120×2160).
  - Lupa con copia del color y botón **Capturar**.
  - Selector EN | ES.
- **M1, entrega 3:**
  - Panel de captura y cuenta atrás que no roba el foco.
  - Entradas de captura en el menú del ícono.
  - Repetir última captura y Repetir última área.
  - Corregidos: el menú ⋯ cortado en la vista compacta, la grabación de `PrtScn` en los atajos y el anillo de foco de las pestañas.
- **Continuidad:**
  - Se creó este archivo y la regla en `CLAUDE.md`.
  - Los scripts de prueba pasaron a `scripts/dev/`.

## Pendiente inmediato

1. **Pruebas de Carlos con el instalador `c97ce05`** ([`tests/manual/CAP-panel.md`](../tests/manual/CAP-panel.md)):
   - grabar otra vez `Ctrl+Shift+PrtScn` en el campo Atajo del panel (antes no lo aceptaba);
   - cambiar de pestaña con las flechas sin que asome el turquesa en la pestaña vecina;
   - **Repetir última área**: asignarle un atajo en Preferencias > Atajos y comprobar que repite el mismo rectángulo sin cruceta.
2. Otras casillas sin marcar en ese archivo: Preferencias de captura desde el menú ⋯, Universal con Retardo de 5 s, y Captura universal desde el menú del ícono.
3. **Carlos no ha respondido:** qué hacer con 16 capturas de prueba que Claude guardó el 2026-10-18 (hacia las 15:20) en `OneDrive\Documents\XcreenShot`. No borrarlas sin su permiso.
4. **Propuesta, pendiente de aprobar:** cuando Carlos confirme la entrega 3, fusionar el PR #2. Antes hay que cambiarle el título, porque dice "capture engine" y ya abarca las entregas 1 a 3. El editor iría en una rama nueva (`m1-editor`).

## Siguiente paso

Entrega 4, el editor básico:
1. Leer [`spec/02-editor.md`](../spec/02-editor.md) completo y sacar los IDs `EDT-` de M1.
2. Proponer a Carlos el plan de la entrega antes de programar.

El stack ya está fijado en `CLAUDE.md`: Konva para el lienzo y las operaciones de píxeles en Rust. Mientras no exista el editor:
- "Abrir en = Editor" muestra la ventana principal con la última captura;
- "Capturar cursor" está desactivado, porque el cursor será un objeto del documento.

## Decisiones vigentes que conviene recordar

El detalle y el motivo de cada una están en `spec/QUESTIONS.md`.

- **Windows primero.** macOS compila pero no bloquea.
- **Versión:** un número menor por hito (0.1 = M0, 0.2 = M1) más el commit de la compilación.
- **Atajo de prueba:** Carlos cambió el de `app.ping` a `Ctrl+P` a propósito. No tocarlo.
- **Barra de captura:** el botón se llama **Capturar** (`Enter`; `I` también funciona). El selector rápido **EN | ES** está en la ventana principal y en Preferencias.
- **Dónde se guardan las capturas:** como PNG en `Documentos\XcreenShot` hasta que llegue la biblioteca. En el equipo de Carlos esa carpeta está en OneDrive.
- **Valores por defecto según quién lanza la captura:**
  - Desde atajos y menús, las acciones de un solo modo (ventana, ventana activa, región, pantalla completa) toman de la pestaña Imagen lo que no es el modo.
  - Desde el CLI y MCP usan valores fijos y no abren ventanas salvo `--open editor`.
- **Ocultar al capturar:** el panel se aparta durante la captura solo si "Ocultar XcreenShot al capturar" está activado. Las ventanas de XcreenShot se excluyen de la imagen solo mientras dura la captura.
- **`Esc` en la cuenta atrás:** queda reservado para XcreenShot mientras corre el contador.

## Cómo trabajamos con Carlos

- **Idioma:** siempre en español, también las instrucciones de prueba. El código, los comentarios y los commits van en inglés.
- **Perfil:** Carlos es consultor de tecnología, no programador. Las pruebas manuales van guiadas paso a paso, numeradas y en tandas cortas. Él responde PASS o fallo, con capturas.
- **Instalador para sus pruebas:** `npm run tauri build` en `app/`. El archivo queda en `target\release\bundle\nsis\XcreenShot_0.2.0_x64-setup.exe` y lo ejecuta él.
- **Antes de `npx tauri dev`:** cerrar la app instalada con `target/debug/xcreenshot.exe app quit --no-launch`, porque la instancia única ocupa el canal.
- **Probar sin tocar su app:** usar una instancia aislada de **depuración** con `XCREENSHOT_IPC_NAME`, `XCREENSHOT_DATA_DIR` y `XCREENSHOT_LIBRARY_DIR`.
  - Nunca la versión de release aislada: al primer arranque se registra para iniciar con Windows y pisa su entrada.
  - Sus atajos los tiene la app instalada, así que la copia aislada avisa de que no pudo activarlos.
- **Herramientas:**
  - `gh` está en `C:\Program Files\GitHub CLI\gh.exe` (no está en el PATH de Git Bash).
  - Rust está en `~/.cargo/bin` (en Git Bash: `export PATH="$HOME/.cargo/bin:$PATH"`).
  - Los scripts largos de Node o PowerShell van en archivos, para evitar problemas de comillas.
- **Comandos que debe ejecutar Carlos:** decirle en qué ventana (PowerShell o cmd) y darle una sola línea.
