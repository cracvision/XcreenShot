# Editor básico, parte 4a (M1, entrega 4a) · EDT-001, EDT-003, EDT-005, EDT-007, EDT-011, EDT-012, EDT-024, EDT-042, EDT-090, EDT-092, EDT-105–EDT-108, EDT-110, EDT-119, LIB-020, TRY-002, TRY-011, APP-047

Pruebas automáticas:
- `crates/xs-core/src/document.rs`: `.xshot` de ida y vuelta, campos desconocidos de una versión menor posterior, versión mayor posterior rechazada, archivo que no es `.xshot`, guardado que falla sin dañar el anterior (EDT-106).
- `crates/xs-image`: geometría de la flecha contra `tests/fixtures/arrow-geometry.json`, dibujo de la flecha, transparencia que se mezcla una sola vez, recorte, PNG con resolución y sRGB, JPG con fondo blanco (EDT-024, EDT-110).
- `crates/xs-editor`: captura → anotar → exportar sin ventanas (EDT-119), todo o nada al anotar con la posición del objeto malo, un solo deshacer por comando, 150 pasos de deshacer (EDT-092), guardado automático de documentos de la biblioteca, archivos externos que preguntan antes de cerrar o salir (EDT-012, APP-004), Guardar como (EDT-107, LIB-021), exportar sin sobrescribir, abrir dos veces el mismo archivo, archivos dañados, recuperación (APP-047), esquema publicado de `document.json` (EDT-106).
- `app/src/editor`: la geometría de la flecha de la interfaz coincide con la de Rust; zoom (apertura ajustada o al 100 %, tamaño real en píxeles físicos, ajustar, pasos, tamaño original, zoom bajo el puntero).
- `crates/xs-library`: nombres por app y hora en la carpeta del mes, sin pisar archivos (LIB-020).

Pruebas con la app en marcha (instancia aislada, CLI y entrada simulada en la ventana), 2026-10-24, Windows 11, monitor al 150 %:
- [x] `capture region --copy false` deja un `.xshot` en `<biblioteca>\2026-10\` y devuelve `documentId`.
- [x] `document annotate` (dos flechas) + `document export --format png`: el PNG tiene las flechas, la discontinua, el círculo inicial y la transparencia; declara 144 PPP.
- [x] `editor show --document-id`: el editor muestra la captura con las mismas flechas que el PNG; título "<nombre> — XcreenShot"; miniatura en la tira.
- [x] Con Flecha: arrastrar en una zona vacía crea una flecha; con `Shift` sale horizontal. Un clic sin arrastrar no crea nada.
- [x] Arrastrar una flecha existente la mueve; quedan los rombos en sus extremos y el panel muestra "Flecha seleccionada".
- [x] `Supr` la elimina y `Ctrl+Z` la recupera ("Eliminar flecha" / "Dibujar flecha" en el historial).
- [x] Clic en una muestra de color con una flecha seleccionada: cambia la flecha y el estilo de la herramienta.
- [x] `Ctrl+S` guarda el documento de la biblioteca y quita el punto de cambios sin guardar.
- [x] `Ctrl`+rueda, `Ctrl+0`, `Ctrl+=`, `Ctrl+-`, `Ctrl+Shift+0` y `editor zoom --mode percent --value 250` cambian el zoom de la barra de estado.
- [x] Copiar todo (`Ctrl+Shift+C`) deja una imagen de 900 × 500 en el portapapeles y muestra "Imagen copiada".
- [x] Un PNG de fuera de la biblioteca con una flecha: `Ctrl+W` pregunta Guardar / No guardar / Cancelar; No guardar lo cierra y el PNG queda igual.
- [x] `app quit` con ese PNG modificado devuelve `needs_decision` y la ventana pregunta; Cancelar no sale.
- [x] Matar el proceso 35 s después de editar y volver a abrir: aparece Recuperar trabajo; Recuperar abre el documento con la flecha y sin guardar.
- [x] `Ctrl+S` en ese PNG muestra el aviso de que las anotaciones se integran; Guardar lo escribe con la flecha y borra la copia de recuperación.
- [x] Captura con `--open editor`: el editor la muestra delante y la tira tiene los dos documentos.
- [x] `Ctrl+Alt+P` oculta y muestra el panel lateral; su borde izquierdo lo ensancha de 260 a 480 px.
- [x] Interfaz en español: los textos caben en la barra, el panel y la barra de estado.

## Fallos encontrados por Carlos (2026-10-24)

- Paso 1 de la tanda 1 (instalador `d16fdce`): el editor se abría con el damero en lugar de la captura, y la miniatura de la tira salía rota. Causa: la política de seguridad (CSP) de la app instalada no permitía las imágenes del protocolo `xsdoc`; el servidor de desarrollo no aplica la CSP, por eso no salió en las pruebas. Corregido en `tauri.conf.json`, con una prueba (`app/src/editor/csp.test.ts`). Verificado con una compilación que empaqueta la interfaz como la instalada (`npx tauri build --debug --no-bundle`): la captura, la miniatura y el dibujo funcionan.

- Durante la tanda 1 (instalador `582df5a`): con la ventana acoplada a media pantalla arrastrándola al borde izquierdo o derecho, la captura "temblaba". Causa: con la imagen ajustada a la ventana y un ancho fraccionario (escala 150 %), el área de contenido sobresalía una fracción de píxel; aparecía una barra de desplazamiento, el área se encogía, la barra sobraba y desaparecía, en cada fotograma. Reproducido con la instancia aislada cambiando el tamaño de la ventana (unos 160 cambios de tamaño en 1,5 s); corregido en `app/src/editor/view.ts` (solo hay desplazamiento si la imagen no cabe por más de un píxel, y en píxeles enteros), con prueba en `view.test.ts`. Tras la corrección, ningún ancho probado cambia de tamaño (al 100 % y al 150 %).

- Tanda 2, paso 5 (instalador `2d581d1`): al alternar entre dos documentos, el zoom no se conservaba. Un zoom escrito o elegido en el menú pasaba al otro documento, y uno con + o − se aplicaba otra vez en cada vuelta (150 → 200 → 300 %). Causa: la última orden de zoom quedaba guardada y la volvía a aplicar cada documento que se mostraba. Corregido: la orden lleva el documento al que va y se aplica una sola vez (`takeZoomRequest` en `app/src/editor/state.ts`, con prueba en `state.test.ts`). Verificado en la instancia aislada: cada documento conserva su zoom y su posición al alternar varias veces.

## Tanda 1 · Captura y editor — Carlos

Carlos marcó PASS (2026-10-24, instalador `582df5a`) los siete pasos.

Instalador: `target\release\bundle\nsis\XcreenShot_0.2.0_x64-setup.exe`, compilación `2d581d1` o posterior (aparece junto a la versión con `xcreenshot app ping`).

1. [x] Haz una captura de región con tu atajo (`Ctrl+Shift+PrtScn`). Se abre el editor con la captura, al 100 % si cabe en la ventana.
2. [x] En `Documentos\XcreenShot` hay una carpeta del mes (por ejemplo `2026-10`) con un archivo `.xshot` con el nombre de la app y la hora.
3. [x] Elige **Flecha** (botón o tecla `A`) y arrastra sobre la imagen: aparece la flecha y queda seleccionada con dos rombos en los extremos.
4. [x] Arrastra un rombo: cambia ese extremo. Arrastra la flecha por el medio: se mueve entera.
5. [x] En el panel de la derecha cambia color, grosor, extremos, tipo de línea y opacidad: la flecha cambia. La siguiente flecha que dibujes sale con ese estilo.
6. [x] `Ctrl+Z` deshace y `Ctrl+Y` rehace, paso a paso. Arrastrar el deslizador de opacidad y soltar se deshace de una vez.
7. [x] La pestaña de la tira de abajo y el título de la ventana indican "Editado" mientras hay cambios.

## Tanda 2 · Zoom y vista — Carlos

Carlos marcó PASS (2026-10-24, instalador `2d581d1`) los pasos 1 a 4. El 5 falló (abajo); corregido, falta que lo confirme.

1. [x] `Ctrl`+rueda acerca y aleja manteniendo bajo el puntero el mismo punto de la imagen.
2. [x] `Ctrl+0` = 100 %; `Ctrl+Shift+0` = ajustar a la ventana; el menú ˅ de la barra de estado ofrece los pasos fijos.
3. [x] Escribe `250` en el campo de zoom y pulsa `Enter`: la vista queda al 250 %.
4. [x] Mantén `Espacio` y arrastra (o arrastra con la rueda pulsada): la vista se desplaza sin dibujar.
5. [ ] Con dos documentos en la tira, cambia de uno a otro: cada uno conserva su zoom y su posición.

## Tanda 3 · Guardar, exportar y cerrar — Carlos

1. [ ] **Compartir ▾ > Archivo…**: guarda un PNG; ábrelo con el visor de Windows y tiene las flechas.
2. [ ] Repite con JPG: el archivo se abre en el visor de Windows.
3. [ ] **Copiar todo** y pega en Word o Paint: la imagen lleva las flechas; en el editor siguen siendo editables.
4. [ ] **Abrir…** un PNG de tu escritorio, dibuja una flecha y `Ctrl+W`: pregunta Guardar / No guardar / Cancelar. Cancelar lo deja abierto.
5. [ ] Arrastra dos PNG desde el Explorador a la ventana: se abren los dos y queda visible el último.
6. [ ] Con una captura modificada, cambia a otro documento y vuelve: la captura se guardó sola (sin punto de cambios).
7. [ ] Cierra la ventana del editor y vuelve a abrirla desde el ícono: los documentos siguen en la tira.
