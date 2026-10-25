# Estado del proyecto

Este archivo es el traspaso entre sesiones de Claude Code. Cada sesión nueva lo lee al empezar (lo pide `CLAUDE.md`). Se actualiza al cerrar una entrega o un bloque de trabajo, y siempre antes de pasar a una sesión nueva. Es un resumen: el detalle está en las fuentes de abajo.

**Última actualización:** 2026-10-25, con la entrega 4b terminada y pendiente de las pruebas de Carlos.

## Fuentes

| Qué | Dónde |
|---|---|
| Definición del producto | [`spec/SPEC.md`](../spec/SPEC.md) y los archivos `spec/0x-*.md` |
| Avance por ID | [`spec/PARITY.md`](../spec/PARITY.md) |
| Decisiones y dudas abiertas | [`spec/QUESTIONS.md`](../spec/QUESTIONS.md) |
| Pruebas manuales y sus resultados | [`tests/manual/`](../tests/manual/) |
| Decisiones de arquitectura | [`docs/adr/`](adr/) (la 0004 es la del editor) |
| Formato `.xshot` | [`docs/formats/`](formats/) |
| Comandos para agentes (CLI y MCP) | [`AGENTS.md`](../AGENTS.md) |
| Historial de cambios | `git log` y los PR en [GitHub](https://github.com/cracvision/XcreenShot/pulls) |
| Ayudas para probar en Windows | [`scripts/dev/`](../scripts/dev/README.md) |
| Conversaciones anteriores | Se guardan en este equipo y se pueden buscar cuando falta un detalle que no está en el repositorio |

## Dónde estamos

- **Hito actual:** M1 · Captura y edición básicas. Versión 0.2.0.
- **M0** está terminado y verificado en Windows ([PR #1](https://github.com/cracvision/XcreenShot/pull/1), fusionado).
- **M1, entregas 1 a 3:** rama `m1-capture` ([PR #2](https://github.com/cracvision/XcreenShot/pull/2), abierto, CI en verde).
- **M1, entrega 4 (editor):** rama `m1-editor`, que sale de `m1-capture`. Su PR apunta a `m1-capture` hasta que se fusione el PR #2; después, a `main`.

| Entrega de M1 | Contenido | Estado |
|---|---|---|
| 1 · Motor de captura | Pantalla completa, región y ventana; portapapeles; metadatos; comandos `capture.*` en el CLI | Hecha y probada por Carlos |
| 2 · Selección en pantalla | Pantallas congeladas, cruceta, lupa, detección de ventanas, barra de captura con **Capturar** | Hecha y probada por Carlos |
| 3 · Panel de captura | Pestañas Universal e Imagen, retardo con cuenta atrás, menú del ícono, Repetir última captura y Repetir última área, restablecer ajustes | Hecha; faltan 3 pruebas de Carlos (abajo) |
| 4a · De la captura al archivo | Ventana Editor con lienzo, zoom y barra de estado; Mover y Flecha; deshacer y rehacer; guardar `.xshot`, exportar PNG y JPG, Copiar todo, abrir y soltar archivos, cerrar; aviso de cambios sin guardar y recuperación; cada captura se guarda como `.xshot` en la biblioteca; tira de documentos abiertos; comandos `document.*` y `editor.*` | Hecha y probada por Carlos (tres tandas, 2026-10-25) |
| 4b · Las demás herramientas | Texto, Forma, Desenfocar, Recortar sin perder lo recortado; tiradores con `Shift`/`Alt`; selector de color completo; cortar, copiar y pegar objetos; teclas de herramienta | Hecha; faltan las pruebas de Carlos (cuatro tandas) |
| 4c · Menús, teclado y accesibilidad | Barra de menús del editor y menú contextual, tabla completa de atajos (asignables), accesibilidad del lienzo, Preferencias > Editor | Siguiente |
| 5 · Biblioteca | Vista Biblioteca, índice, bandeja reciente completa y persistente, copia editable al guardar en otro formato | Pendiente |

Plan aprobado por Carlos el 2026-10-24: la entrega 4 en tres partes (A: adelantar el `.xshot` de cada captura y una tira simple; B: un único motor de dibujo en Rust, ADR 0004; C: rama `m1-editor`).

macOS: el código compila en la CI, pero ese trabajo no bloquea los PR (`continue-on-error`). Las pruebas en Mac empiezan cuando Carlos lo diga. Hasta entonces, ningún ID pasa de `Win` a `Hecho` en `PARITY.md`.

## Cambios de la última sesión

Sesión "Lee docs/ESTADO.md y continúa" (2026-10-24 y 25). El detalle está en `git log`.

- **Plan de la entrega 4** propuesto y aprobado (tres partes, decisiones A, B y C). ADR 0004.
- **Entrega 4a:**
  - Modelo de documento y formato `.xshot` con esquema publicado; guardado seguro; versiones.
  - Motor de dibujo en Rust (`xs-image`): flecha con 10 extremos y 4 tipos de línea, opacidad, PNG con resolución y sRGB, JPG.
  - Crate nuevo `xs-editor`: documentos abiertos, 200 pasos de deshacer, guardado automático de la biblioteca, preguntar antes de perder cambios (cerrar y salir), copias de recuperación cada 30 s.
  - Las capturas se guardan como `.xshot` en `Documentos\XcreenShot\AAAA-MM\` y se abren en el editor.
  - Ventana Editor (Konva): lienzo con zoom y desplazamiento, Mover y Flecha, panel Propiedades, barra de estado, tira de documentos, diálogos.
  - 20 comandos nuevos (`document.*`, `editor.*`); `app.quit` pregunta por cambios sin guardar.
- **Herramientas de prueba:** `scripts/dev/cdp.mjs` prueba la ventana de una instancia aislada sin mover el ratón real.
- **Fallos que encontró Carlos en sus pruebas, corregidos:**
  - el editor instalado no mostraba la captura (la política de seguridad bloqueaba las imágenes del protocolo `xsdoc`; ahora hay una prueba);
  - la captura temblaba con la ventana acoplada a media pantalla (barra de desplazamiento que aparecía y desaparecía por fracciones de píxel);
  - al alternar documentos el zoom se copiaba o se volvía a aplicar.
- **Lección:** antes de dar un instalador, probar con `npx tauri build --debug --no-bundle`, que empaqueta la interfaz como la app instalada (el servidor de desarrollo no aplica la política de seguridad).
- **Entrega 4b** (2026-10-25):
  - Texto con `cosmic-text` y fuentes del sistema: Rust lo dibuja también en pantalla; mientras se escribe, un campo encima con la misma medida.
  - Forma (rectángulo y elipse) y Desenfocar (desenfoque, pixelado y sólido), calculados en Rust sobre lo que hay debajo.
  - Tiradores para redimensionar y girar, con `Shift` y `Alt`.
  - Recortar sin perder lo recortado (se recupera volviendo a recortar) y aviso de objetos fuera del lienzo. La preferencia `editor.deleteCroppedContent` recorta de verdad al guardar.
  - Selector de color completo, con cuentagotas de pantalla (Windows) y `I` sobre el lienzo.
  - Cortar, copiar y pegar objetos: editables dentro de la app, PNG con transparencia para las demás; texto e imágenes de otras apps.
  - Teclas de herramienta, `Tab`, `Ctrl+Enter`, `[` y `]`. 2 comandos nuevos (`document.copy_objects`, `document.paste`).
  - Decisiones provisionales nuevas en `spec/QUESTIONS.md` (tamaño del texto en puntos, intensidad del desenfoque, cuentagotas en macOS, huella del portapapeles, pegar imágenes con un documento abierto, tamaño de `Ctrl+Enter`).

## Pendiente inmediato

1. **Pruebas de Carlos con el instalador `c97ce05`** (entrega 3, [`tests/manual/CAP-panel.md`](../tests/manual/CAP-panel.md)):
   - grabar otra vez `Ctrl+Shift+PrtScn` en el campo Atajo del panel;
   - cambiar de pestaña con las flechas sin que asome el turquesa en la pestaña vecina;
   - **Repetir última área** con un atajo asignado en Preferencias > Atajos.
   - Otras casillas sin marcar: Preferencias de captura desde el menú ⋯, Universal con Retardo de 5 s, y Captura universal desde el menú del ícono.
   - El instalador de la 4a (`9ddb1ba`) sustituye al de la entrega 3: las pruebas que faltan se pueden hacer con él.
2. **Carlos no ha respondido:** qué hacer con 16 capturas de prueba que Claude guardó el 2026-10-18 (hacia las 15:20) en `OneDrive\Documents\XcreenShot`. No borrarlas sin su permiso.
3. **Pruebas de Carlos de la entrega 4b:** cuatro tandas en [`tests/manual/EDT-editor-4b.md`](../tests/manual/EDT-editor-4b.md) (texto; formas y desenfoque; recortar y color; copiar, pegar y teclado), con el instalador nuevo.
4. **Propuesta, pendiente de aprobar:** cuando Carlos confirme la entrega 3, fusionar el PR #2 (antes, cambiarle el título, que dice "capture engine"). Después, el [PR #3](https://github.com/cracvision/XcreenShot/pull/3) (entrega 4a, probada y con la CI en verde) pasa a apuntar a `main` y se puede fusionar también. La 4b ya está en el mismo PR #3 (rama `m1-editor`).

## Siguiente paso

Mientras Carlos prueba la 4b, corregir lo que reporte y empezar la entrega 4c, menús, teclado y accesibilidad:
1. Leer los bloques de EDT-002, EDT-012, EDT-013, EDT-116 y EDT-118 en `spec/02-editor.md`.
2. Barra de menús del editor y menú contextual (EDT-002); sustituyen al menú de M0 y a los botones provisionales de Deshacer y Rehacer.
3. Tabla completa de atajos del editor, asignables en Preferencias > Atajos (EDT-116). Hoy están fijos en `app/src/editor/keys.ts`.
4. Accesibilidad del lienzo (EDT-013): etiquetas para el lector de pantalla y anuncio del objeto seleccionado.
5. Preferencias > Editor (EDT-118) y `Esc` para cerrar el editor (EDT-012).

## Decisiones vigentes que conviene recordar

El detalle y el motivo de cada una están en `spec/QUESTIONS.md`.

- **Windows primero.** macOS compila pero no bloquea.
- **Versión:** un número menor por hito (0.1 = M0, 0.2 = M1) más el commit de la compilación.
- **Atajo de prueba:** Carlos cambió el de `app.ping` a `Ctrl+P` a propósito. No tocarlo.
- **Barra de captura:** el botón se llama **Capturar** (`Enter`; `I` también funciona). El selector rápido **EN | ES** está en la ventana principal y en Preferencias.
- **Dónde se guardan las capturas:** como `.xshot` en `Documentos\XcreenShot\AAAA-MM\`, con el nombre `<App> AAAA-MM-DD HH.MM.SS` (provisional hasta SHR-006). En el equipo de Carlos esa carpeta está en OneDrive. Las PNG de antes se quedan donde estaban.
- **Editor (ADR 0004):** el documento vive en Rust; la interfaz es una vista que manda comandos. Cada gesto es un solo comando al soltar. La imagen final la dibuja solo Rust; la geometría está duplicada en `geometry.ts` y `boxes.ts` y la vigilan `tests/fixtures/arrow-geometry.json` y `shape-geometry.json`. El texto y el desenfoque los dibuja Rust también en pantalla (protocolo `xsdoc`).
- **Atajos del editor:** fijos hasta la 4c. En la ventana del editor `Ctrl+W` cierra el documento, no la ventana.
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
  - Para probar la ventana sin mover su ratón: arrancarla además con `WEBVIEW2_USER_DATA_FOLDER` propio y la depuración remota, y usar `scripts/dev/cdp.mjs` (instrucciones en `scripts/dev/README.md`).
  - No usar Copiar todo, copiar o cortar objetos en las pruebas automáticas: pisan el portapapeles de Carlos.
- **Herramientas:**
  - `gh` está en `C:\Program Files\GitHub CLI\gh.exe` (no está en el PATH de Git Bash).
  - Rust está en `~/.cargo/bin` (en Git Bash: `export PATH="$HOME/.cargo/bin:$PATH"`). Mínimo 1.88.
  - Los scripts largos de Node, Python o PowerShell van en archivos, para evitar problemas de comillas. Al reescribir archivos con Python en Windows, usar `newline=''` para no cambiar los finales de línea.
- **Comandos que debe ejecutar Carlos:** decirle en qué ventana (PowerShell o cmd) y darle una sola línea.
