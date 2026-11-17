# ADR 0004 · Documento del editor y un único motor de dibujo para la imagen final

**Estado:** aceptada (Carlos, 2026-10-24) · **Fecha:** 2026-10-24 · **IDs:** EDT-007, EDT-012, EDT-024, EDT-090, EDT-092, EDT-106, EDT-107, EDT-110, EDT-119, LIB-020

## Contexto

El editor de M1 tiene que cumplir a la vez:

- **EDT-106:** un `.xshot` guardado en Windows y abierto en macOS da una exportación PNG idéntica píxel a píxel (salvo el suavizado del texto).
- **EDT-119 y APP-030:** un agente captura, anota y exporta **sin abrir ninguna ventana**. Los comandos que cambian un documento abierto en el editor crean un solo paso de Deshacer.
- **EDT-092:** deshacer y rehacer con al menos 100 pasos, por documento, también para los cambios hechos desde el CLI.
- `CLAUDE.md` fija Konva para el lienzo y Rust para las operaciones de píxeles, y que los componentes de React no tengan lógica de negocio.

## Decisión

1. **El documento vive en Rust.** El modelo (`xs_core::document`), los documentos abiertos, el historial de deshacer y el guardado (`xs-editor`) están en el proceso de la app. La interfaz es una vista: dibuja el documento que le da el núcleo y le manda comandos (`document.*`, `editor.*`). Un arrastre se previsualiza en la interfaz y se confirma con **un** comando al soltar; así cada gesto es un solo paso de Deshacer.
2. **Dos dibujantes con un mismo modelo:**
   - **En pantalla** dibuja Konva, que responde al instante al arrastrar y hacer zoom.
   - **La imagen final** (exportar, Copiar todo, guardar en formato plano, la miniatura y todo lo que hace el CLI) la dibuja **un único motor en Rust** (`xs-image`, con `tiny-skia`). Es el mismo código en Windows y en macOS, así que el resultado es idéntico en ambas.
3. **La geometría se define una vez.** Las puntas de flecha, los tipos de línea y demás formas se calculan con fórmulas fijadas en `xs-image` y repetidas en `app/src/editor/geometry.ts`. Un archivo de casos (`tests/fixtures/arrow-geometry.json`) que genera Rust lo comprueba también Vitest: si una de las dos copias cambia, las pruebas fallan.
4. **El texto (entrega 4b) lo dibuja siempre Rust**, también en pantalla, salvo mientras se escribe. La interfaz muestra el resultado de Rust como imagen. Así el salto de línea y la fuente coinciden en pantalla y en el archivo. Se usará `cosmic-text`, que lee las fuentes del sistema.
5. **Formato `.xshot`:** ZIP con `manifest.json`, `document.json`, `base.png`, `meta.json` y `thumb.png` (SPEC §7), escrito con el crate `zip`. Las coordenadas de `document.json` van en píxeles de la imagen con origen en la esquina superior izquierda del lienzo. Se escribe en un temporal y se renombra al final (guardado seguro).
6. **Biblioteca adelantada (decisión A del plan):** desde la entrega 4a cada captura se guarda como `.xshot` en `<biblioteca>\AAAA-MM\` (LIB-020). La ubicación y los nombres viven en `xs-library`; el índice y la vista Biblioteca siguen en la entrega 5.

## Dependencias nuevas

No cambian el stack de `CLAUDE.md`; lo completan.

| Dónde | Librería | Para qué |
|---|---|---|
| Rust | `tiny-skia` | Dibujar la imagen final (trazos, rellenos, discontinuas, suavizado) |
| Rust | `zip` | Leer y escribir `.xshot` |
| Rust | `png` | Escribir PNG con resolución (pHYs) y sRGB |
| Rust | `image` con JPEG | Escribir JPG |
| Rust (4b) | `cosmic-text` | Componer y dibujar el texto con las fuentes del sistema |
| UI | `konva`, `react-konva` 18 | Lienzo interactivo |
| UI | `@tauri-apps/plugin-dialog` | Diálogos del sistema para Abrir y Guardar como |

## Consecuencias

- **Riesgo:** que la pantalla y el archivo no coincidan. Se controla con la geometría compartida y sus pruebas, y con el texto dibujado por Rust.
- El CLI puede anotar y exportar documentos que no están abiertos en la ventana: el núcleo los carga desde su archivo.
- Las imágenes grandes ocupan memoria en el proceso de la app (unos 44 MB por una captura de 5120×2160). Los documentos que no están a la vista y no tienen cambios liberan sus píxeles y los vuelven a leer del archivo cuando hace falta.
- Los píxeles base llegan a la interfaz por un protocolo propio (`xsdoc`), como en la selección en pantalla (ADR 0003), sin pasar por JSON.
