# Editor, parte 4b (M1, entrega 4b) · EDT-013, EDT-014, EDT-019, EDT-022, EDT-026, EDT-029, EDT-040, EDT-049, EDT-050, EDT-089, EDT-115, EDT-117

Pruebas automáticas:
- `crates/xs-image`: texto de ancho automático y de ancho fijo con salto de línea, tamaño en puntos según la resolución del documento, glifos del color del texto (EDT-026); geometría de las formas contra `tests/fixtures/shape-geometry.json`, contorno dentro del tamaño, límites de una forma girada (EDT-029); desenfoque y pixelado que borran el detalle, bloques más grandes con más intensidad, Sólido que no deja rastro de lo que tapa (EDT-040); recorte que muestra solo su ventana (EDT-049).
- `crates/xs-editor`: anotar texto, formas y desenfoque por comando; recorte no destructivo que conserva los objetos sobre la imagen y se recupera volviendo a recortar (EDT-050); "Eliminar el contenido recortado" guarda solo lo recortado; lista de fuentes instaladas; esquema publicado de `document.json` con los tipos nuevos.
- `app/src/editor`: la geometría de las formas de la interfaz coincide con la de Rust; tiradores (redimensionar desde el lado opuesto, `Shift` mantiene la proporción, `Alt` desde el centro, giro en pasos de 15° con `Shift`); qué hay bajo el puntero (formas sin relleno por el contorno, con relleno por dentro, texto por su tamaño real); objetos fuera del lienzo; selector de color (Hex ↔ RGB ↔ HSB, paleta de 40 colores distintos).

Pruebas con la app en marcha (instancia aislada y entrada simulada en la ventana), 2026-10-25, Windows 11, monitor al 150 %:
- [x] Forma, desenfoque (pixelado) y texto se crean arrastrando; la imagen exportada coincide con lo que se ve en el editor.
- [x] Redimensionar desde una esquina de una forma girada, moverla y girarla con `Shift` hasta 90°.
- [x] Doble clic en un texto abre el campo de escritura alineado con el texto; el cambio queda como "Cambiar texto" en el historial.
- [x] Recortar: oscurece lo que queda fuera, tiene tiradores y panel; `Enter` recorta; los objetos se quedan sobre la misma parte de la imagen; la barra de estado avisa de los objetos que quedan fuera, con **Seleccionar**.
- [x] El selector de color se abre entero aunque el panel tenga poco alto; elegir un color de la paleta y **Aceptar** cambia el estilo de la herramienta.
- [x] Con la ventana estrecha, la barra muestra solo los íconos de las herramientas (nombre y tecla en la información emergente) y no se corta nada.
- [x] Con la interfaz empaquetada como en la app instalada (`npx tauri build --debug --no-bundle`, compilación `01bab3e`): pixelado, texto, elipse y flecha se ven en el editor igual que en el PNG exportado; `Enter` sobre el texto abre el campo de escritura y `Esc` lo cierra; `I` sobre una forma azul toma #4682B4.
- [x] `I` fuera de la imagen tomaba negro (el lienzo está vacío ahí). Corregido: fuera de la imagen no hace nada.

Copiar y pegar no se probó en la instancia aislada para no tocar el portapapeles de Carlos: se prueba en la tanda 4.

## Tanda 1 · Texto — Carlos

Instalador: `target\release\bundle\nsis\XcreenShot_0.2.0_x64-setup.exe`, compilación `46cbb30` o posterior (aparece junto a la versión con `xcreenshot app ping`).

1. [ ] Haz una captura de región. En el editor elige **Texto** (botón o tecla `T`), haz clic sobre la imagen y escribe "Hola mundo". `Enter` añade una línea.
2. [ ] Haz clic fuera del texto: queda en la imagen, seleccionado, con el mismo aspecto que mientras escribías.
3. [ ] Con el texto seleccionado, cambia en el panel de la derecha la fuente, el tamaño, negrita, cursiva, color y alineación: el texto cambia. El siguiente texto que escribas sale con ese estilo.
4. [ ] Doble clic sobre el texto (o `Enter` con él seleccionado): vuelves a escribir en él, con el cursor al final.
5. [ ] Con **Texto**, arrastra un recuadro ancho y escribe una frase larga: salta de línea al ancho del recuadro. Arrastra el tirador del lado derecho: el texto se reacomoda.
6. [ ] Arrastra el tirador redondo de arriba: el texto gira. Con `Shift`, en pasos de 15°.
7. [ ] **Compartir ▾ > Archivo…** como PNG y ábrelo con el visor de Windows: el texto se ve igual que en el editor.

## Tanda 2 · Formas y desenfoque — Carlos

1. [ ] Elige **Forma** (`U`) y arrastra: sale un rectángulo. Con `Shift` sale un cuadrado; con `Alt`, se dibuja desde el centro.
2. [ ] En el panel cambia Forma a **Elipse**, pon un **Relleno** y cambia el contorno, su grosor y el tipo de línea: la forma cambia.
3. [ ] Arrastra una esquina, un lado y el tirador redondo: cambia el tamaño y gira. `Shift` en una esquina mantiene la proporción.
4. [ ] Elige **Desenfocar** (`B`) y arrastra sobre un texto de la captura: queda ilegible. Sube y baja la **Intensidad** en el panel.
5. [ ] Cambia el Tipo a **Pixelado** y después a **Sólido** (con color): las tres variantes tapan el texto.
6. [ ] Exporta un PNG y ábrelo: el desenfoque tapa lo mismo que en el editor y no se puede quitar.
7. [ ] `Ctrl+Z` deshace cada cambio, paso a paso, con su nombre en el botón Deshacer.

## Tanda 3 · Recortar y color — Carlos

1. [ ] Dibuja una flecha cerca de un borde. Elige **Recortar** (`R`): lo que queda fuera del recuadro se oscurece.
2. [ ] Arrastra los bordes hacia dentro para dejar la flecha fuera, y pulsa `Enter` (o doble clic dentro): la imagen queda recortada. La barra de estado dice "1 objeto fuera del lienzo".
3. [ ] Vuelve a **Recortar**: se ve otra vez la imagen entera. Arrastra los bordes hacia fuera hasta el original y pulsa `Enter`: vuelve todo, flecha incluida. `Esc` durante el recorte lo deja como estaba.
4. [ ] Abre un color del panel: el selector se ve entero. Escribe `#2D7FF9` en Hex: muestra R 45, G 127, B 249. **Aceptar** lo aplica.
5. [ ] En el selector, pulsa **Cuentagotas** y haz clic en cualquier punto de la pantalla, también fuera de XcreenShot: el color pasa al selector.
6. [ ] **Transparente** aparece en el Relleno de una forma, pero no en el color de una flecha.
7. [ ] Con **Flecha** activa, pon el puntero sobre un color de la imagen y pulsa `I`: la herramienta toma ese color.

## Tanda 4 · Copiar, pegar y teclado — Carlos

Estos pasos usan tu portapapeles.

1. [ ] Selecciona una flecha, `Ctrl+C` y `Ctrl+V`: aparece una copia 10 px más abajo y a la derecha, editable.
2. [ ] `Ctrl+X` sobre otra flecha la quita; abre otro documento de la tira y `Ctrl+V`: la flecha aparece en el centro de la vista. `Ctrl+Shift+V` la pega en su mismo sitio.
3. [ ] Copia la flecha y pega en Paint o Word: llega solo la flecha, sin la captura de fondo.
4. [ ] Copia una frase en el Bloc de notas y pega en el editor: aparece como texto.
5. [ ] Pulsa `V`, `A`, `T`, `U`, `B` y `R`: cambia la herramienta. Mantener `V` y soltar vuelve a la herramienta de antes.
6. [ ] Haz clic en la imagen y pulsa `Tab` varias veces: se seleccionan los objetos uno a uno. Las flechas del teclado mueven el seleccionado 1 px (10 px con `Shift`).
7. [ ] Con **Forma** activa, `Ctrl+Enter` inserta un rectángulo en el centro de la vista. Con una flecha seleccionada, `[` y `]` cambian su grosor.
