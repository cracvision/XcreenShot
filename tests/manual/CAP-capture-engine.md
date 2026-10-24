# Motor de captura (M1, entrega 1) · CAP-008, CAP-021, CAP-023, CAP-025, CAP-027, CAP-028, CAP-046, CAP-054, CAP-056

Pruebas que necesitan la pantalla real. Las de lógica pura (composición de monitores, nombres de archivo, opciones, metadatos) son automáticas en `crates/xs-capture`. `cargo test -p xs-capture -- --ignored` hace una captura real del monitor principal.

Con la app en marcha (`npm run tauri dev` en `app/`):

## CAP-028 / CAP-021 · Pantalla completa y monitores
- [x] `xcreenshot capture monitors --json` lista cada monitor con posición, tamaño físico, escala y cuál es el principal.
- [x] `xcreenshot capture fullscreen --monitor primary --out <archivo> --json` produce una imagen del tamaño nativo del monitor.
- [ ] Con dos monitores, `--monitor <nombre del secundario>` produce el tamaño nativo del secundario.
- [ ] `--monitor all` cubre el rectángulo envolvente de todos, con transparencia en los huecos.
- [ ] Con un monitor guardado que no existe, se usa el principal y el resultado trae el aviso en `warnings`.

## CAP-025 · Región por rectángulo
- [x] `xcreenshot capture region --rect '{"x":380,"y":360,"width":1500,"height":1060}'` produce exactamente 1500×1060 px.
- [ ] Una región que cruza dos monitores produce una sola imagen continua.
- [x] `capture region` sin `--rect` responde `invalid_params` explicando que la selección en pantalla llega en la entrega 2.

## CAP-027 · Ventana
- [x] `xcreenshot capture window --target active` captura la ventana de aplicación de delante, completa y con su barra de título.
- [ ] Con una ventana tapada a medias, `--target title --title <texto>` la captura completa, sin la que la tapa.
- [ ] Esquinas redondeadas transparentes en Windows 11 (pendiente: hoy salen cuadradas).

## CAP-023 · Ocultar XcreenShot al capturar
- [x] Con la ventana principal abierta, una captura de esa zona muestra lo que hay detrás, no la ventana.
- [ ] Con **Ocultar XcreenShot al capturar** desactivado, la ventana sí aparece.
- [ ] Mientras compartes pantalla en Teams o Zoom, XcreenShot se ve normal en la reunión (la exclusión solo dura lo que dura la captura).

## CAP-046 · Portapapeles
- [x] Botón **Capturar pantalla completa en 3 s** de la ventana principal y pegar en Paint o Word: la imagen pegada tiene el tamaño del monitor. (Carlos, Word, 2026-10-13: se pega completa y sin la ventana de XcreenShot.)

## CAP-054 · Ruta de la captura
- [x] Cada captura queda en `Documentos\XcreenShot` (biblioteca provisional, en PNG) y, con `--out`, también en esa ruta.
- [x] Una segunda captura mientras hay otra en curso responde `busy` (código de salida 5) y no interrumpe la primera.
- [ ] Con Abrir en = Editor (desde la interfaz), la ventana principal pasa al frente con la captura.

## CAP-056 · Metadatos
- [x] El resultado trae `capturedAt` con zona horaria, sistema (Windows 11 25H2, compilación), escala del monitor, versión de XcreenShot, usuario y equipo.
- [x] En una captura de ventana se guardan el nombre de la aplicación, su versión y el título de la ventana (Claude 2.37337.1.0 / "Claude").
- [x] Las capas transparentes de otras apps (overlay de NVIDIA) no se toman como "aplicación" de una región.
- [ ] Con **Guardar el título de la ventana** desmarcado, el título queda vacío.

## Resultados

- Windows 11 (26200), un monitor LG 5120×2160 al 150 %, modo desarrollo, 2026-10-13: lo marcado arriba. Fallos encontrados en la prueba y corregidos antes de entregar: la exclusión de ventanas llegaba tarde justo después de arrancar; la región tomaba el overlay de NVIDIA como aplicación; la descripción del ejecutable traía basura al final; la ventana activa no se encontraba cuando el foco estaba en una ventana que no es de aplicación.
