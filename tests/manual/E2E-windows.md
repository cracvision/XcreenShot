# Prueba de punta a punta en Windows

Una sola guía para recorrer XcreenShot de principio a fin, como se usa de verdad, en tres sesiones. Cada bloque dice qué comprobar y enlaza la guía detallada: ábrela solo si algo falla o si te sobra tiempo.

- **Versión:** la versión bajo prueba que indica la Portada del OQ (hoy, 0.2.5) o posterior. Si tienes otra: **Ayuda > Buscar actualizaciones…**, o descarga el instalador de https://github.com/cracvision/xcreenshot-releases.
- **Registro oficial:** el protocolo OQ en Excel [`XcreenShot_OQ_Windows_v1_3.xlsx`](XcreenShot_OQ_Windows_v1_3.xlsx) (mismos casos, con acción, resultado esperado, PASS/FAIL/N/A, evidencia y desviaciones). Se regenera con `python scripts/make-oq.py tests/manual/E2E-windows-steps.json <salida.xlsx>`. Esta guía queda como versión de lectura.
- **Evidencia:** cada casilla dice qué evidencia pide si pasa: *captura* (captura de pantalla hecha con una herramienta ajena a XcreenShot), *archivo* (la ruta de lo que produjo XcreenShot) o *registro* (la salida del comando que lo confirma). Todo fallo lleva captura. Detalle y nombres de archivo en [`docs/jarvis/LEEME-JARVIS.md`](../../docs/jarvis/LEEME-JARVIS.md), sección 3.
- **Cómo marcar:** `[x]` si pasa. Si falla, deja `[ ]` y escribe debajo **FALLA:** qué pasó, con una captura si ayuda.

> **Seguridad: Esc siempre devuelve el ratón y el teclado**
>
> - **Dibujo en vivo** (VID-048) y **Multicaptura** (CAP-067) se quedan con los clics (y Dibujo en vivo, también con el teclado) mientras están activos. Es lo normal.
> - **`Esc` los apaga al instante.** La barra de Dibujo en vivo muestra **Esc** y una **✕**. Si la app dejara de responder, se sueltan solos a los 2 o 3 s.
> - Tu atajo de Dibujo en vivo es **`Ctrl+Shift+F11`** (Jarvis usa las combinaciones con `Ctrl+Alt`). Las guías dicen `Ctrl+Alt+Mayús+D`, que es el de fábrica.
> - Haz esos bloques **al final de la sesión** (1.9 y 3.10), sin nada importante abierto.
> - El Grabador de pasos y la captura con desplazamiento no retienen el ratón: solo escuchan los clics, o `Enter` y `Esc`.

## Antes de empezar

- Ten a mano: una página web larga, un PDF de 5 páginas (imprime cualquier documento con **Microsoft Print to PDF**), un PNG con transparencia (un logo), un SVG si tienes, Word, PowerPoint, Excel y Outlook, y auriculares o altavoces.
- Tu atajo de captura es `Ctrl+Shift+PrtScn`. Las guías lo escriben también `Ctrl+Mayús+ImprPant`.
- `Ctrl+P` ya no es un atajo global de XcreenShot (se quitó el 2026-11-15): imprime como siempre.
- Casi todos los atajos globales de XcreenShot empiezan por `Ctrl+Alt+Mayús`, y Jarvis usa las combinaciones con `Ctrl+Alt`. Si uno no responde, usa la misma función desde el menú del ícono o el panel, y apúntalo: lo cambiamos como hicimos con Dibujo en vivo.
- Al terminar cada sesión, guarda este archivo con las marcas (o mándame una foto).

---

## Sesión 1 · Instalar, ajustes y captura (unos 90 min)

### 1.1 Instalar y primer arranque (10 min)

Que la app instalada arranque bien y viva en la bandeja.
**IDs:** APP-001 a APP-005, APP-013, APP-036 · **Guías:** [APP-001.md](APP-001.md), [APP-002.md](APP-002.md), [APP-004.md](APP-004.md), [APP-013.md](APP-013.md) (APP-005 y APP-036 no tienen guía: bastan los puntos de abajo)

- [x] Desde la 0.2.3: **Ayuda > Buscar actualizaciones… > Instalar y reiniciar**. XcreenShot vuelve solo **con su ventana** y avisa "XcreenShot se actualizó". (PASS el 2026-11-15) · *Evidencia: captura*
- [ ] **Ayuda > Acerca de** dice la versión bajo prueba; **Ayuda > Buscar actualizaciones…** dice que tienes la última. · *Evidencia: captura*
- [ ] Cierra todas las ventanas: el ícono sigue en la bandeja y tu atajo de captura funciona. · *Evidencia: registro*
- [ ] Abre XcreenShot otra vez desde el menú Inicio: no sale un segundo ícono; se muestra la ventana que ya había. · *Evidencia: registro*
- [ ] Clic derecho en el ícono: las capturas con sus atajos, Recientes, Preferencias, Ayuda y Salir. · *Evidencia: captura*
- [ ] Marca **Pausar atajos globales**: el atajo de captura no hace nada y el ícono cambia. Desmárcala: vuelve sin reiniciar. · *Evidencia: captura*
- [ ] **Ayuda > Volver a mostrar la bienvenida**: recorre idioma, atajos, inicio y biblioteca sin cambiar nada. · *Evidencia: registro*
- [ ] Con Preferencias abierta, cambia el idioma de Español a English y vuelve: todo se traduce sin reiniciar, también el menú del ícono. · *Evidencia: captura*

### 1.2 Preferencias, aspecto y atajos (10 min)

Que el aspecto Cristal, los ajustes y los atajos se comporten bien.
**IDs:** APP-006 a APP-012, APP-017 a APP-023 · **Guías:** [APP-012-cristal.md](APP-012-cristal.md), [APP-011.md](APP-011.md), [APP-023.md](APP-023.md), [APP-017.md](APP-017.md), [APP-018.md](APP-018.md), [APP-019.md](APP-019.md), [APP-021.md](APP-021.md)

- [ ] **Preferencias > General > Tema**: Claro, Oscuro y Sistema; todo se lee bien en los tres. · *Evidencia: captura*
- [ ] **Intensidad del cristal** de 0 a 100 (a 0, todo sólido) y los cuatro **Color de acento**. · *Evidencia: captura*
- [ ] **Preferencias > Atajos**: Dibujo en vivo está en `Ctrl+Shift+F11`. El filtro **Con conflicto** muestra solo filas en conflicto. · *Evidencia: registro*
- [ ] `Ctrl+/` abre la hoja de atajos; buscar "flecha" la encuentra; **Guardar como PDF…** da una o dos páginas A4. · *Evidencia: archivo*
- [ ] **Avanzado > Exportar ajustes…**; cambia el tema y un atajo; **Importar ajustes…** sin **Atajos**: vuelve el tema, el atajo no. · *Evidencia: registro*
- [ ] Suspende y reanuda el equipo: el atajo de captura responde a la primera pulsación. · *Evidencia: registro*

### 1.3 Panel de captura: lo que quedó pendiente (10 min)

Cerrar la entrega 3. Casi todo [CAP-panel.md](CAP-panel.md) es PASS desde el 2026-10-18; aquí van solo las casillas sin marcar.
**IDs:** CAP-001 a CAP-007, CAP-010, CAP-048, CAP-065 · **Guía:** [CAP-panel.md](CAP-panel.md)

- [ ] Vuelve a grabar `Ctrl+Shift+PrtScn` en el campo **Atajo** del panel y en Preferencias > Atajos: lo acepta. · *Evidencia: registro*
- [ ] Cambia de pestaña con las flechas: el anillo de foco queda dentro de la pestaña, sin asomar en la vecina. · *Evidencia: captura*
- [ ] Menú **⋯ > Preferencias de captura…** abre Preferencias en la sección Captura. · *Evidencia: registro*
- [ ] Universal con **Retardo de 5 s**: la cruceta aparece a los 5 s. · *Evidencia: registro*
- [ ] Menú del ícono **> Captura universal**: la cruceta aparece enseguida. · *Evidencia: registro*
- [ ] Asigna un atajo a **Repetir última área**. Tras una región, captura el mismo rectángulo sin cruceta; tras una ventana, esa ventana donde esté ahora. · *Evidencia: archivo*

### 1.4 Modos de selección de imagen (15 min)

Probar cada forma de elegir qué se captura.
**IDs:** CAP-009 a CAP-021 (ya PASS el 2026-10-18), CAP-026, CAP-029 a CAP-033, CAP-047, EDT-093 · **Guías:** [CAP-selection.md](CAP-selection.md), [CAP-026.md](CAP-026.md), [CAP-029.md](CAP-029.md), [CAP-030.md](CAP-030.md), [CAP-031.md](CAP-031.md), [CAP-032.md](CAP-032.md), [CAP-033.md](CAP-033.md), [EDT-093.md](EDT-093.md)

- [ ] **Región fija** de 1080 × 1080: el cuadro sigue al puntero y un clic captura. Con **Posición fija**, sale al momento. · *Evidencia: archivo*
- [ ] `Ctrl+Alt+Shift+M` (**Selector de ventanas**): salen todas, también las minimizadas; escribir filtra. Una minimizada se captura entera y vuelve a minimizarse. · *Evidencia: captura*
- [ ] **Menú** con Retardo de 3 s: un menú clásico abierto en la cuenta atrás se captura con su forma. · *Evidencia: archivo*
- [ ] **Mano alzada** (Libre, Polígono y Elipse): fuera de la forma, transparente. · *Evidencia: archivo*
- [ ] **Múltiples áreas**: tres áreas numeradas; en el editor cada una se mueve sola; **Apiladas** y **En fila** las reordenan. · *Evidencia: captura*
- [ ] Pantalla completa con **Cada ventana como un objeto**: un objeto por ventana; al borrar la de delante se ve la de detrás y tu fondo de pantalla. · *Evidencia: captura*
- [ ] Con **Capturar cursor**, el puntero llega como objeto **Cursor**: se mueve sin dejar rastro y cambia a **Mano**. · *Evidencia: registro*
- [ ] (Con dos monitores) Una región que cruza los dos sale como una sola imagen. · *Evidencia: archivo*

### 1.5 Desplazamiento y página web (10 min)

Capturar contenido más largo que la pantalla.
**IDs:** CAP-034 a CAP-037, CAP-064 · **Guías:** [CAP-034-036.md](CAP-034-036.md), [CAP-037.md](CAP-037.md)

- [ ] Universal sobre una página larga, **Desplazamiento** (`S`) y la rueda: el panel muestra la imagen creciendo; **Listo** (`Enter`). · *Evidencia: captura*
- [ ] Con `D` (automático): baja sola desde arriba y al terminar deja la página donde estaba. · *Evidencia: archivo*
- [ ] `Enter` a mitad (**Detener aquí**) conserva lo recorrido; `Esc` no crea nada. · *Evidencia: registro*
- [ ] Una cabecera fija sale una sola vez, arriba. · *Evidencia: archivo*
- [ ] **Página web** con una dirección copiada: la página entera, de la cabecera al pie. Con Ancho de la vista = 375, el diseño de móvil. · *Evidencia: archivo*
- [ ] Sin wifi, **Página web** falla con un aviso y no aparece nada en la biblioteca. · *Evidencia: captura*

### 1.6 Otras fuentes: portapapeles, archivos, PDF y capturas del sistema (10 min)

Que XcreenShot reciba imágenes que no vienen de la pantalla.
**IDs:** CAP-039 a CAP-043, APP-001 · **Guías:** [CAP-039.md](CAP-039.md), [CAP-040-041.md](CAP-040-041.md), [CAP-042.md](CAP-042.md), [CAP-043.md](CAP-043.md), [APP-001.md](APP-001.md)

- [ ] **Portapapeles** con una imagen copiada del navegador: se abre del mismo tamaño. Con texto copiado: un objeto Texto editable. · *Evidencia: registro*
- [ ] **Archivo** con 3 imágenes y un `.txt` renombrado a `.png`: entran 3 y el aviso nombra el dañado. Un SVG sale sin fondo blanco. · *Evidencia: captura*
- [ ] **PDF** con el de 5 páginas: **Importar PDF** con miniaturas; el rango `2-3` da 2 capturas; **Todas las páginas en una imagen** da una. · *Evidencia: captura*
- [ ] Clic derecho en un PDF **> Abrir con > XcreenShot** abre Importar PDF; el doble clic sigue abriendo tu visor de siempre. · *Evidencia: captura*
- [ ] Preferencias > Captura > **Abrir en XcreenShot las capturas del sistema** y `Win+Mayús+S`: se abre una sola vez en XcreenShot. Desactívala después. · *Evidencia: registro*
- [ ] **Carpeta de recepción**: 2 imágenes y un PDF de 3 páginas dan 5 capturas y los archivos pasan a **Importados**. · *Evidencia: registro*

### 1.7 Intervalo y programada (10 min, mientras haces otra cosa)

Capturas que se repiten o que esperan a una hora.
**IDs:** CAP-049, CAP-050 · **Guías:** [CAP-049.md](CAP-049.md), [CAP-050.md](CAP-050.md)

- [ ] **Intervalo** cada 5 s, 60 s con un video en pantalla: unas 12 capturas, y en la biblioteca una tarjeta **Serie · N**. · *Evidencia: registro*
- [ ] Con la pantalla quieta y **Descartar si no hay cambios**: una sola captura. · *Evidencia: registro*
- [ ] Con la serie en marcha, el ícono lleva un reloj; su menú **Detener captura: Intervalo** la para al momento. · *Evidencia: captura*
- [ ] Cada = 1 queda en 3, con aviso. · *Evidencia: registro*
- [ ] **Programada** dentro de 2 min: llega a su hora sin abrir el editor. Programa otra, **Salir** y vuelve a abrir: se hace igual. · *Evidencia: registro*
- [ ] Una hora que ya pasó se rechaza con un aviso. · *Evidencia: registro*

### 1.8 Efectos, destino Archivo, presets y lanzador (15 min)

Que una captura salga ya preparada y que los atajos propios funcionen.
**IDs:** CAP-004, CAP-051 a CAP-053, CAP-057 a CAP-064 · **Guías:** [CAP-051.md](CAP-051.md), [CAP-052-053.md](CAP-052-053.md), [CAP-058-064.md](CAP-058-064.md), [CAP-004.md](CAP-004.md), [CAP-057.md](CAP-057.md)

- [ ] Panel > Imagen > **Efectos**: Borde y Sombra llegan al editor y se pueden quitar. El engranaje (⚙) cambia el grosor del borde. · *Evidencia: registro*
- [ ] **Guardar en un archivo** con `{prefix}_{date}_{counter:3}`: aparecen `_001` y `_002`. En JPG con **Incluir los metadatos**, Propiedades > Detalles muestra título y fecha. · *Evidencia: archivo*
- [ ] Con Abrir en = Editor y Guardar en un archivo: "Pendiente de enviar a: Archivo". **Enviar** escribe el archivo con tu flecha; **No enviar** no escribe nada. · *Evidencia: archivo*
- [ ] Presets: **Región al portapapeles** funciona; **＋ > Desde los ajustes actuales** y su atajo `Ctrl+Alt+Mayús+1` funcionan desde otra app; Eliminar y **Deshacer** lo devuelve. · *Evidencia: registro*
- [ ] **⋯ > Exportar todos los presets…** e **Importar presets…** con **Conservar ambos**. · *Evidencia: archivo*
- [ ] **⋯ > Mostrar el lanzador de borde**: Capturar y Presets funcionan, y no sale en una captura de pantalla completa. · *Evidencia: captura*
- [ ] Preferencias > **Funciones en prueba > Anotación rápida** y `Ctrl+Alt+Shift+Q`: flecha y `Enter` pegan en otra app. Desactívala al terminar. · *Evidencia: captura*

### 1.9 Multicaptura (último bloque de la sesión, 10 min)

Cada clic captura la ventana de debajo. **Toma tus clics hasta que pulses `Esc`.**
**IDs:** CAP-067, EDT-078 · **Guía:** [CAP-067.md](CAP-067.md) (tandas 1 y 2)

- [ ] Ícono **> Multicaptura**: aparece el panel abajo a la derecha y tu ventana sigue activa. · *Evidencia: captura*
- [ ] Un clic en una carpeta del Explorador la abre y añade una miniatura de cómo estaba antes del clic; un doble clic añade una sola. · *Evidencia: captura*
- [ ] Arrastrar un rectángulo lo captura sin que la app seleccione nada; ni el borde ni el panel salen en las miniaturas. · *Evidencia: captura*
- [ ] `Esc`: los clics vuelven a ser normales al instante; el panel conserva las miniaturas. · *Evidencia: captura*
- [ ] Dos miniaturas **> Insertar 2** en una captura abierta: imágenes que se mueven, giran y cambian de **Opacidad**; `Ctrl+Z` quita la inserción entera. · *Evidencia: registro*
- [ ] Sin documentos abiertos, **Insertar** crea uno nuevo con las dos, una debajo de otra. · *Evidencia: registro*

---

## Sesión 2 · Editor, biblioteca y bandeja (unos 90 min)

### 2.1 Ventana del editor, barras y vista (10 min)

Lo básico del editor es PASS desde el 2026-10-25 y el 2026-10-28; aquí van lo nuevo y la tanda 4 pendiente.
**IDs:** EDT-001 a EDT-013, APP-012 · **Guías:** [APP-012-cristal.md](APP-012-cristal.md), [EDT-editor-4c.md](EDT-editor-4c.md) (tanda 4), [EDT-001-barra-estrecha.md](EDT-001-barra-estrecha.md), [EDT-003-004.md](EDT-003-004.md), [EDT-006.md](EDT-006.md), [EDT-008-010.md](EDT-008-010.md)

- [ ] Una captura grande y acercada pasa por debajo de la barra, el panel y la bandeja, borrosa. Las barras de desplazamiento finas se arrastran. · *Evidencia: captura*
- [ ] Tanda 4: **Texto** y luego `A` deja Texto sin marco; Deshacer y `Ctrl+Y` no dejan marco; con `F6`, `Tab` y `Enter` en **Forma**, Forma conserva su marco. · *Evidencia: captura*
- [ ] Achica la ventana al mínimo: nada se monta y lo que no cabe pasa a **Más**. Igual en la Biblioteca. · *Evidencia: captura*
- [ ] **Ver > Barra de herramientas > Personalizar…**: Borrador en segunda posición; sigue así tras reiniciar. **Restaurar valores por defecto**. · *Evidencia: registro*
- [ ] Con una flecha, un texto o un globo elegidos, la barra de edición rápida cambia el color; **Estilos ▾** aplica un estilo. · *Evidencia: registro*
- [ ] Al 800 % se ve la cuadrícula de píxeles; **Ver > Pantalla completa** (`F11`) y `Esc` vuelve. · *Evidencia: captura*

### 2.2 Herramientas de anotación y de píxeles (15 min)

Las herramientas de M2. Flecha, Texto, Forma, Desenfocar y Recortar son PASS desde el 2026-10-28.
**IDs:** EDT-022 a EDT-048, EDT-094 · **Guías:** [EDT-editor-4b.md](EDT-editor-4b.md), [EDT-043-046.md](EDT-043-046.md), [EDT-047-048.md](EDT-047-048.md), [EDT-023-LIB-016-017.md](EDT-023-LIB-016-017.md), [EDT-027.md](EDT-027.md), [EDT-094.md](EDT-094.md)

- [ ] Globo, Paso, Línea, Resaltador, Lápiz, Lupa, Foco y Simplificar se dibujan, y el PNG exportado es igual a lo que ves. · *Evidencia: archivo*
- [ ] **Selección** (`Q`) con **Ajustar al objeto** sobre un botón y `Supr`: el fondo queda limpio. Una elipse copiada se pega redonda en otra app. · *Evidencia: captura*
- [ ] **Varita mágica** (`W`) sobre un fondo blanco, **Invertir selección** y **Copiar**: copia el contenido sin el fondo. · *Evidencia: registro*
- [ ] **Relleno** (`G`) transparente; **Borrador** (`E`) cruzando una flecha no la toca. Guardar y reabrir lo conserva. · *Evidencia: archivo*
- [ ] Con Flecha elegida, **Seleccionar todas las flechas** (bajo el título del panel) elige solo las flechas. · *Evidencia: registro*
- [ ] Un texto con "ejenplo" se subraya y el clic derecho sugiere "ejemplo"; el subrayado no sale al exportar. · *Evidencia: captura*
- [ ] **Más > Imagen > Medir**: longitud, ángulo y, con clic derecho, **Convertir en cota**. · *Evidencia: captura*

### 2.3 Estilos, temas, sellos y emoji (10 min)

Reutilizar estilos y poner sellos propios.
**IDs:** EDT-015 a EDT-021, EDT-031 a EDT-034 · **Guías:** [EDT-015-021.md](EDT-015-021.md), [EDT-032-034.md](EDT-032-034.md)

- [ ] Pestaña **Estilos** con Flecha: **+** guarda un estilo que sigue tras reiniciar; su estrella lo lleva a **Favoritos** (`F`). · *Evidencia: registro*
- [ ] **⋯ > Nuevo tema…** con tres colores y Georgia; exportar e importar el tema pregunta **Reemplazar** o **Conservar ambos**. · *Evidencia: captura*
- [ ] La cuadrícula de **Sombra** pone y quita la sombra; un degradado lineal y uno radial en una forma. · *Evidencia: captura*
- [ ] **Sello** (`S`), busca "pulgar" y "bandera españa": emoji en color. · *Evidencia: captura*
- [ ] Un PNG transparente soltado sobre la cuadrícula de sellos entra en **Mis sellos** sin fondo. **Archivo > Guardar como sello…** con una selección elíptica da un sello redondo. · *Evidencia: registro*
- [ ] **Exportar categoría…** da un `.xstamps`; su doble clic ofrece **Conservar ambos**. Un ZIP con un `.exe` renombrado a `.xstamps` se rechaza. · *Evidencia: archivo*

### 2.4 Imagen, lienzo y efectos (10 min)

El menú **Imagen** y la pestaña Efectos.
**IDs:** EDT-049 a EDT-077, EDT-084 · **Guía:** [EDT-069-077.md](EDT-069-077.md)

- [ ] **Cambiar tamaño de la imagen** al 50 %: imagen y anotaciones se reducen; `Ctrl+Z` lo deshace. · *Evidencia: registro*
- [ ] **Girar** 90° ida y vuelta; **Voltear > Horizontal** deja el texto del globo legible. · *Evidencia: captura*
- [ ] **Tamaño del lienzo**, el rombo de esquinas redondeadas y **Color del lienzo** con degradado. · *Evidencia: captura*
- [ ] **Recortar bordes**; **Quitar franja** (`X`) y **Añadir espacio**. · *Evidencia: registro*
- [ ] Pestaña **Efectos**: los deslizadores se mueven suaves y el lienzo muestra el efecto (arreglado el 2026-11-14; volver a probar). · *Evidencia: registro*
- [ ] **Ajuste magnético**: guías azules al acercar al centro; con `Ctrl` se mueve sin saltos. · *Evidencia: captura*
- [ ] **Edición > Volver al original…** deja la captura limpia y `Ctrl+Z` lo recupera todo. · *Evidencia: registro*

### 2.5 Plantillas (10 min)

Combinar varias capturas en un documento.
**IDs:** EDT-095 a EDT-101 · **Guía:** [EDT-095-101.md](EDT-095-101.md)

- [ ] **Archivo > Nuevo desde plantilla…** (`Ctrl+Shift+T`): las familias Pasos, Línea de tiempo, Comparación y Collage, con los colores del tema. · *Evidencia: captura*
- [ ] Biblioteca: 4 capturas **> Combinar en plantilla… > Pasos en columna**: en orden y numeradas de 1 a 4. · *Evidencia: archivo*
- [ ] Arrastrar la imagen de una zona sobre otra las intercambia; doble clic abre el encuadre. · *Evidencia: captura*
- [ ] Una zona vacía se llena con **Capturar**, con un archivo del Explorador o con una miniatura de la bandeja. · *Evidencia: registro*
- [ ] **Archivo > Guardar como plantilla…** y ábrela desde el selector; **Exportar…** y suelta el `.xtemplate` en la ventana. · *Evidencia: archivo*

### 2.6 Archivos: nueva, exportar, imprimir y reabrir (10 min)

Entrar y sacar imágenes del editor.
**IDs:** EDT-102 a EDT-114, EDT-120 · **Guías:** [EDT-103-104.md](EDT-103-104.md), [EDT-110-112.md](EDT-110-112.md), [EDT-113.md](EDT-113.md), [EDT-114.md](EDT-114.md), [EDT-120.md](EDT-120.md)

- [ ] **Archivo > Nueva imagen…** transparente de 800 × 600; **Nueva desde el portapapeles** con una imagen y con un texto. · *Evidencia: registro*
- [ ] **Exportar…** en JPG (el tamaño estimado cambia con la calidad), GIF, BMP, TIFF, ICO y PSD. PDF en A4 con **Ajustar al ancho** no parte ninguna línea. · *Evidencia: archivo*
- [ ] **Exportar por lotes…** 10 capturas con **Marco fino** y el nombre "guia": `guia_01.jpg`…; con **Omitir** no cambia nada. · *Evidencia: archivo*
- [ ] **Archivo > Imprimir…**: vista previa A4 apaisada; con una selección, **Imprimir: La selección**. Dime si prefieres el diálogo de Windows en lugar del visor de PDF. · *Evidencia: captura*
- [ ] Tres capturas en la tira con la segunda a la vista; **Salir** y abrir: las tres en su orden, con la segunda a la vista. · *Evidencia: registro*

### 2.7 Biblioteca (15 min)

Encontrar, ordenar y sacar capturas.
**IDs:** LIB-001 a LIB-028, LIB-033, LIB-034 · **Guías:** [LIB-004-011.md](LIB-004-011.md), [LIB-003-026.md](LIB-003-026.md), [LIB-008.md](LIB-008.md), [LIB-012.md](LIB-012.md), [LIB-013.md](LIB-013.md), [LIB-018-024-033.md](LIB-018-024-033.md), [LIB-025-APP-052.md](LIB-025-APP-052.md), [EDT-023-LIB-016-017.md](EDT-023-LIB-016-017.md)

- [ ] **Ordenar por** Tamaño y la vista **Lista** con sus columnas. `Ctrl+F` encuentra por nombre, por etiqueta y por el texto de un globo. · *Evidencia: registro*
- [ ] Etiqueta "Cliente A" en 5 capturas: **Etiquetas > Cliente A (5)**. La estrella marca un favorito. **Quitar filtros** vuelve al total. · *Evidencia: registro*
- [ ] Buscar una palabra que solo está dentro de una imagen (por ejemplo "Error 404") la encuentra. · *Evidencia: registro*
- [ ] **Detalles** (`Alt+Intro`) muestra aplicación, sitio web y fecha; cambiar el nombre ahí renombra el archivo. · *Evidencia: registro*
- [ ] `Espacio` abre la vista rápida; `←` y `→` recorren. También reproduce videos y GIF. · *Evidencia: captura*
- [ ] **⋯ > Exportar toda la biblioteca…**: subcarpetas, PNG y `metadatos.csv`. **Cancelar** a mitad deja completo lo ya exportado. · *Evidencia: archivo*
- [ ] Soltar una carpeta "Proyecto X" sobre la biblioteca propone esa etiqueta. · *Evidencia: captura*
- [ ] Con tu biblioteca real, desplazarte rápido no la congela y la búsqueda responde mientras escribes. · *Evidencia: registro*

### 2.8 Bandeja reciente y Portapapeles de XcreenShot (15 min)

La tira de capturas del editor.
**IDs:** TRY-001 a TRY-018 · **Guías:** [TRY-003-007-017.md](TRY-003-007-017.md), [TRY-010-012-016.md](TRY-010-012-016.md), [TRY-014.md](TRY-014.md), [TRY-018.md](TRY-018.md)

- [ ] La chincheta fija un elemento a la izquierda; arrastrar dentro de la bandeja reordena; sigue así tras reiniciar. · *Evidencia: registro*
- [ ] Preferencias > Biblioteca > **Elementos de la bandeja reciente** = 3: los antiguos salen de la bandeja pero siguen en la biblioteca. · *Evidencia: registro*
- [ ] `Ctrl`+clic en varias y clic derecho: el menú actúa sobre todas. **Duplicar** (`Ctrl+Mayús+D`) crea "(copia)" con la flecha editable. · *Evidencia: registro*
- [ ] Arrastrar una miniatura sobre el lienzo de otra captura la añade como imagen flotante; la original no cambia. · *Evidencia: registro*
- [ ] Arrastrar una miniatura al escritorio entrega el archivo. · *Evidencia: archivo*
- [ ] Con Paint delante, `Win+Alt+V`: se abre **Portapapeles de XcreenShot** con las miniaturas de la bandeja, en su orden. · *Evidencia: captura*
- [ ] Elegir una captura con una flecha e **Insertar**: el panel se cierra y en Paint aparece la captura original, sin la flecha. · *Evidencia: captura*
- [ ] Con Word delante, elegir tres e **Insertar (3)**: tres imágenes, en el orden de la bandeja. · *Evidencia: captura*
- [ ] Elegir una, **Copiar** y `Ctrl+V` en Paint: se pega la original. · *Evidencia: captura*
- [ ] `Esc`, y otra vez haciendo clic en otra ventana: el panel se cierra sin pegar nada. · *Evidencia: registro*
- [ ] Con un video en la bandeja: aparece desactivado, con su tooltip. · *Evidencia: captura*
- [ ] `xcreenshot --no-launch tray insert <documentId> --json`: `copied` 1, `pasted` 0 y la imagen original en el portapapeles. · *Evidencia: registro*

---

## Sesión 3 · Inteligentes, video, compartir y automatización (90 a 120 min)

Es la más larga. Si hace falta, pártela en dos por la línea **— pausa —**.

### 3.1 Funciones inteligentes en el editor (15 min)

Leer texto, tapar datos y retocar sin dibujar a mano. Todo ocurre en el equipo.
**IDs:** SMT-001 a SMT-021, SMT-036, SMT-038 · **Guías:** [SMT-003-006.md](SMT-003-006.md), [SMT-M5-2.md](SMT-M5-2.md), [SMT-M5-2b.md](SMT-M5-2b.md), [SMT-M5-2c.md](SMT-M5-2c.md), [SMT-009.md](SMT-009.md), [SMT-018.md](SMT-018.md), [SMT-019.md](SMT-019.md), [SMT-021.md](SMT-021.md), [SMT-020.md](SMT-020.md), [SMT-M5-6.md](SMT-M5-6.md), [SMT-002.md](SMT-002.md)

- [ ] **Extraer texto** (`Ctrl+Alt+Mayús+T`) de un párrafo: ventana **Texto extraído**; al pegar en Word, una palabra roja sigue roja. Sin wifi, igual. · *Evidencia: captura*
- [ ] **Imagen > Redacción automática…** tapa correo, teléfono y tarjeta; **Siguiente** las recorre. Exportado y ampliado al 800 %, no se ve nada debajo. · *Evidencia: archivo*
- [ ] Clic en una palabra no detectada la tapa (fila **Manual**); clic derecho **> Redactar todas las coincidencias**. En la biblioteca, 10 capturas **> Redacción automática…** propone las 10. · *Evidencia: registro*
- [ ] **Selección** sobre el texto de un botón **> Editar texto**: queda editable, parecido al original; **Deshacer** lo devuelve. · *Evidencia: captura*
- [ ] **Separar elementos**, **Simplificar automáticamente** y **Solo el texto** en el Resaltador hacen lo que dice su guía. · *Evidencia: captura*
- [ ] **Imagen > Quitar fondo** sobre un logo: queda el damero; **Invertir** y **Como imagen flotante**. Una foto de una habitación no cambia. · *Evidencia: captura*
- [ ] **Imagen > Insertar código QR…** con `example.com`: el teléfono lo lee. Una captura con un QR muestra **Código QR detectado** y **Abrir enlace** pide confirmación. · *Evidencia: captura*
- [ ] Preferencias > Editor > **Funciones inteligentes**: apagar **Redacción automática** la quita de los menús. Vuelve a encenderla. · *Evidencia: registro*

### 3.2 Grabador de pasos y guías (10 min)

Convertir clics en una guía paso a paso. Escucha los clics, no los retiene; `Esc` pregunta si descartar.
**IDs:** SMT-022 a SMT-035, LIB-019 · **Guías:** [SMT-M5-5a.md](SMT-M5-5a.md), [SMT-M5-5b.md](SMT-M5-5b.md), [SMT-M5-5cd.md](SMT-M5-5cd.md), [LIB-019.md](LIB-019.md)

- [ ] Ícono **> Grabar pasos**, y en el Bloc de notas **Archivo > Guardar como…**: el contador sube; con **Pausar**, los clics no cuentan. · *Evidencia: registro*
- [ ] **Terminar** abre la guía "Pasos en Bloc de notas" con un paso por clic y su frase. · *Evidencia: registro*
- [ ] Pestaña **Pasos**: subir, bajar, **Unir con el siguiente** y **Dividir en dos**. · *Evidencia: registro*
- [ ] **Exportar como** Word, PowerPoint, PDF (con marcadores) y Markdown. · *Evidencia: archivo*
- [ ] Biblioteca, **Guías de pasos**: la guía con la insignia "N pasos"; buscar una palabra de un paso la encuentra. · *Evidencia: registro*
- [ ] **Salir** con una grabación de pasos en curso pregunta **Detener y guardar**. · *Evidencia: captura*

### 3.3 Grabar un video (15 min)

Grabar, pausar, detener y recuperar.
**IDs:** VID-001 a VID-030, VID-044 a VID-047, VID-075, VID-076 · **Guías:** [VID-M4-1.md](VID-M4-1.md), [VID-002.md](VID-002.md), [VID-006.md](VID-006.md), [VID-013.md](VID-013.md), [VID-075.md](VID-075.md), [VID-076.md](VID-076.md), [VID-022.md](VID-022.md), [VID-021.md](VID-021.md)

- [ ] `Ctrl+Alt+Mayús+V` y un área: borde, barra en *Lista* y el resumen **Listo para grabar**, que no sale en el video. · *Evidencia: captura*
- [ ] **Grabar**: cuenta atrás (**Grabar ya** la salta), **Pausar**, **Reanudar** y **Detener**: "Video guardado" y se abre en el editor. · *Evidencia: archivo*
- [ ] Captura universal y `V`: la misma área pasa a *Lista*; **Ancho × Alto** se pueden escribir. · *Evidencia: registro*
- [ ] **Resaltar cursor**, **Animación de clic** y **Mostrar teclas pulsadas** salen en el video; lo escrito en un campo de contraseña no. · *Evidencia: archivo*
- [ ] Grabando el Bloc de notas, lo que escribes llega al Bloc de notas. `Ctrl+Alt+Mayús+B` lleva el foco a la barra y `Esc` lo devuelve. · *Evidencia: captura*
- [ ] **Temporizador** de 30 s: se detiene solo y el aviso dice por qué. **Detener** antes de 1 s avisa "Grabación demasiado corta". · *Evidencia: captura*
- [ ] **Salir** grabando pregunta **Detener y guardar** o **Cancelar**. · *Evidencia: captura*
- [ ] **Finalizar tarea** a los 2 min de grabar y volver a abrir: **Recuperar** devuelve el video. · *Evidencia: archivo*

### 3.4 Micrófono y sonido (10 min)

Antes, en Configuración > Sistema > Sonido > Entrada, comprueba que la barra se mueve al hablar (el micrófono de tu bocina entregaba silencio).
**IDs:** VID-019, VID-039 a VID-043 · **Guías:** [VID-039.md](VID-039.md), [VID-042.md](VID-042.md)

- [ ] En *Lista*, el medidor **Micrófono** se mueve al hablar. · *Evidencia: captura*
- [ ] 20 s de grabación con un video sonando y tu voz encima: se oyen los dos. · *Evidencia: archivo*
- [ ] Con el micrófono silenciado en Windows, la franja de la barra lo avisa. · *Evidencia: captura*
- [ ] **Reducir ruido de fondo** con un ventilador cerca: el ruido baja. · *Evidencia: archivo*
- [ ] Con **Combinar las pistas de audio** desactivada, el MP4 tiene dos pistas (VLC **> Audio > Pista de audio**). Vuelve a activarla. · *Evidencia: archivo*

### 3.5 El video en el editor, GIF y combinar (12 min)

Recortar y reutilizar lo grabado.
**IDs:** VID-054 a VID-070 · **Guía:** [VID-M4-6.md](VID-M4-6.md)

- [ ] `Espacio` reproduce; velocidad 2×; escribir `0:10` en el tiempo lleva al segundo 10; las flechas van fotograma a fotograma. · *Evidencia: captura*
- [ ] Los tiradores y **Cortar** (`Supr`): la reproducción salta el corte; `Ctrl+Z` lo devuelve. **Guardar** muestra su progreso. · *Evidencia: archivo*
- [ ] **Guardar fotograma**, **Guardar selección como video**, **Extraer texto del fotograma** y **Copia sin sonido**. · *Evidencia: archivo*
- [ ] Botón **GIF**: **Archivo pequeño** baja la estimación; **Cancelar** a mitad no deja GIF; el GIF terminado se arrastra a un chat. · *Evidencia: archivo*
- [ ] **Combinar videos** con uno grande y uno pequeño: el pequeño sale centrado con bandas negras. · *Evidencia: archivo*
- [ ] **Copiar todo** con un video abierto y pegar en una carpeta: aparece el MP4. · *Evidencia: archivo*
- [ ] El panel lateral muestra cuándo se grabó, la app que estaba delante y el codificador. · *Evidencia: registro*

### 3.6 Diapositivas narradas (10 min)

Narrar unas capturas y que salga un video.
**IDs:** VID-071 a VID-074 · **Guía:** [VID-071.md](VID-071.md) (puntos 1 a 9; el 10 va en 3.10)

- [ ] Bandeja: tres capturas **> Crear: Diapositivas narradas**: las tres en la tira, en ese orden. · *Evidencia: registro*
- [ ] Fondo blanco y 1280 × 720; grabar narrando da un MP4 de 1280 × 720. · *Evidencia: archivo*
- [ ] Pausar, ir a la 3 y reanudar: el video salta sin enseñar las de en medio. · *Evidencia: archivo*
- [ ] **Anotar > Flecha** grabando: sale en el video; al terminar pregunta **Conservarlas** o quitarlas. · *Evidencia: archivo*
- [ ] Con una imagen abierta, `Ctrl+Alt+V` abre las diapositivas. · *Evidencia: registro*

**— pausa —**

### 3.7 Compartir: destinos estándar (10 min)

Mandar una captura a otra app. Carlos decidió el 2026-11-14 que Compartir podía ir en una segunda ronda: si prefieres, deja este bloque para entonces.
**IDs:** SHR-018 a SHR-022, SHR-024 a SHR-026, SHR-037 · **Guía:** [SHR-020-026.md](SHR-020-026.md)

- [ ] **Compartir > Correo** abre un mensaje de Outlook con la captura adjunta, sin enviarlo. · *Evidencia: captura*
- [ ] **Compartir > Otra aplicación…** con `mspaint.exe`: se abre en Paint; la próxima vez Paint está en el submenú. · *Evidencia: captura*
- [ ] **Word** entra en el cursor; **PowerPoint**, en una diapositiva nueva; **Excel**, en la celda elegida. · *Evidencia: captura*
- [ ] Biblioteca: dos capturas, clic derecho **> Compartir en > Correo**: un mensaje con las dos. · *Evidencia: captura*
- [ ] Un video de más de 20 MB a **Correo** pregunta antes de abrir el mensaje. · *Evidencia: captura*
- [ ] **Compartir de Windows…** abre la ventana del sistema (ya visto el 2026-11-15; repetir solo si hay tiempo). · *Evidencia: captura*

### 3.8 CLI, MCP e IA (10 min)

Usar XcreenShot desde una terminal o desde Claude.
**IDs:** APP-027 a APP-033, SMT-039 a SMT-043 · **Guías:** [APP-029.md](APP-029.md), [APP-033.md](APP-033.md), [APP-028.md](APP-028.md), [APP-032.md](APP-032.md), [SMT-039-043.md](SMT-039-043.md) (tanda B ya PASS el 2026-11-01)

- [ ] En una terminal nueva, `xcreenshot --version` responde la versión bajo prueba. · *Evidencia: registro*
- [ ] `xcreenshot library search --type image --json` lista tus capturas. · *Evidencia: registro*
- [ ] En Claude Desktop: "Haz una captura de pantalla completa con XcreenShot y dime qué ves": la hace y la describe. · *Evidencia: captura*
- [ ] "¿Cuáles son mis últimas capturas en XcreenShot?": las lista. · *Evidencia: captura*
- [ ] Preferencias > Avanzado: sin el control por CLI y agentes, Claude dice que está desactivado. Vuelve a activarlo. · *Evidencia: captura*
- [ ] (Con tu clave) Tanda A: **Preferencias > IA**, **Probar la clave**, **IA > Preguntar a la IA… > Describir** y **IA > Sugerir nombre…**. Los registros no guardan la respuesta ni la clave. · *Evidencia: registro*

### 3.9 Configuración avanzada, instalación y accesibilidad (opcional, 15 min)

Lo que usarán un administrador o un colega con otro equipo.
**IDs:** APP-035, APP-037, APP-038, APP-042 a APP-046, APP-050, APP-054 · **Guías:** [APP-035.md](APP-035.md), [APP-038.md](APP-038.md), [APP-037-042.md](APP-037-042.md), [APP-045.md](APP-045.md), [APP-046.md](APP-046.md), [APP-050.md](APP-050.md), [APP-054.md](APP-054.md)

- [ ] El `managed.json` de la guía quita Video y Funciones en prueba; **Avanzado > Configuración administrada > Ver** lo muestra. Al borrarlo, todo vuelve. · *Evidencia: registro*
- [ ] El `.zip` portable de la versión bajo prueba, en una carpeta (con la instalada cerrada): el título dice "(portable)" y no toca `%APPDATA%\XcreenShot`. · *Evidencia: archivo*
- [ ] Desinstala sin marcar **Borrar también ajustes y datos** y reinstala: ajustes, atajos y biblioteca intactos. · *Evidencia: registro*
- [ ] **Ayuda > Copiar información de diagnóstico** no incluye rutas de la biblioteca. · *Evidencia: registro*
- [ ] Con el Narrador (`Ctrl+Win+Enter`), `Tab` en el lienzo anuncia cada objeto. A 200 % de escala nada se corta. · *Evidencia: captura*
- [ ] Una captura muy larga se desplaza con fluidez; una imagen de más de 32.000 px avisa y la app sigue abierta. · *Evidencia: captura*
- [ ] El Monitor de recursos no muestra conexiones de XcreenShot en una sesión normal (salvo al buscar actualizaciones). · *Evidencia: captura*

### 3.10 Dibujo en vivo (último bloque, 10 min)

Dibujar sobre la pantalla mientras grabas. **Toma el ratón y el teclado hasta que pulses `Esc`.** Lee antes el recuadro de seguridad.
**IDs:** VID-048 a VID-051, VID-073 · **Guías:** [VID-048.md](VID-048.md), [VID-M4-8.md](VID-M4-8.md), [VID-071.md](VID-071.md) (punto 10)

- [ ] Grabando un área, **Dibujar** en la barra (o `Ctrl+Shift+F11`): arriba sale la barra de Dibujo en vivo, con **Esc** a la vista. · *Evidencia: captura*
- [ ] Arrastrar dibuja una flecha roja y la página no recibe el clic; la rueda sigue desplazando. · *Evidencia: captura*
- [ ] `R`, `O`, `P`, `1` (pasos) y los colores `2` a `7`; cada forma desaparece a los 3 s. · *Evidencia: captura*
- [ ] `Esc` apaga el modo al instante: ratón y teclado vuelven y la grabación sigue. · *Evidencia: captura*
- [ ] El video muestra las formas y nunca la barra de Dibujo en vivo. · *Evidencia: archivo*
- [ ] `M` (lupa) sale en el video; `A` la quita. · *Evidencia: archivo*
- [ ] Grabando diapositivas, `Ctrl+Shift+F11`: las formas salen en el video y la diapositiva no cambia. · *Evidencia: archivo*
- [ ] (Opcional, supervisado) Pasos 2 a 6 de [VID-048.md](VID-048.md), con el límite de 15 s: sin pulsar nada, todo vuelve solo. · *Evidencia: captura*

---

## Necesita algo especial

Pruebas que no caben en las sesiones porque piden un aparato, otra persona o más tiempo.

| Necesitas | Qué se prueba | Guía |
|---|---|---|
| Una webcam (o una GoPro como webcam) | Cámara en el video: esquina, forma, borde, espejo, **⇄**, ocupada, tapada y desconectada (VID-031 a VID-036, VID-038); `Ctrl+Alt+Mayús+F` (VID-075) | [VID-031.md](VID-031.md), [VID-075.md](VID-075.md) punto 5 |
| Un segundo monitor | Lanzador en otro monitor (CAP-004), región entre monitores (CAP-021), grabar un monitor y desconectarlo (VID-004, VID-013) | [CAP-004.md](CAP-004.md), [CAP-capture-engine.md](CAP-capture-engine.md), [VID-013.md](VID-013.md) punto 5 |
| Otro PC en la misma red con la misma versión de XcreenShot, y un colega | Red local: verse, emparejar con código, enviar, aceptar, rechazar, pendientes (SHR-042 a SHR-044); Uso compartido en proximidad (SHR-037) | [SHR-042-044.md](SHR-042-044.md), [SHR-020-026.md](SHR-020-026.md) punto 9 |
| Una clave de la API de Anthropic con saldo | IA dentro de la app, tanda A (SMT-039 a SMT-043) | [SMT-039-043.md](SMT-039-043.md) |
| Word, PowerPoint, Excel y Outlook | Compartir en Office y Correo (SHR-020, SHR-024 a SHR-026); guías a Word y PowerPoint (SMT-030) | [SHR-020-026.md](SHR-020-026.md), [SMT-M5-5cd.md](SMT-M5-5cd.md) |
| Una impresora (o Microsoft Print to PDF) | Imprimir desde el editor, la biblioteca, Compartir y la hoja de atajos (EDT-114, LIB-017, SHR-022, APP-023) | [EDT-114.md](EDT-114.md), [APP-023.md](APP-023.md) |
| Altavoces o auriculares **con cable** y 30 min | Sincronía y rendimiento del video (VID-030) | [VID-030.md](VID-030.md) |
| Permisos de administrador | Configuración administrada (APP-035) | [APP-035.md](APP-035.md) |
| Una carpeta de red, OneDrive en dos equipos u otra biblioteca | Biblioteca en red, sincronizada, fusionar y llevarla a otro equipo (LIB-029 a LIB-032, APP-055) | [LIB-030.md](LIB-030.md), [LIB-029.md](LIB-029.md), [LIB-031.md](LIB-031.md), [LIB-032.md](LIB-032.md), [APP-055.md](APP-055.md) |
| Un teléfono | Leer un QR exportado (SMT-036); enviar una foto a la carpeta de recepción (CAP-043) | [SMT-M5-6.md](SMT-M5-6.md) punto 1, [CAP-043.md](CAP-043.md) punto 6 |
| Una segunda cuenta de Windows | Atajos por usuario y automatización aislada (APP-001, APP-032) | [APP-001.md](APP-001.md), [APP-032.md](APP-032.md) |
| Un video H.265 de un teléfono | Importarlo y exportarlo como MP4 H.264 (VID-066) | [VID-M4-6.md](VID-M4-6.md), "Exportar un video" |
| Un monitor HDR | Capturas HDR (CAP-022). **Aún no está construido:** se probará cuando lo esté | — |

## Qué no entra

- **macOS**: todo, incluidos los permisos de macOS (APP-024, APP-025), su paquete (APP-039) y las guías a Pages y Keynote (SMT-031).
- **Compartir en la nube y el Enlace para compartir**: OneNote, Evernote, OneDrive, Teams, Slack, Google Drive, Dropbox, Box, YouTube, FTP/SFTP y los proveedores del Enlace (SHR-001 a SHR-017, SHR-023, SHR-027 a SHR-036, SHR-038 a SHR-041). Son un borrador sin aprobar. Tampoco el QR de un enlace compartido (SMT-037).
- **No construido todavía** (PARITY = —):
  - HDR (CAP-022);
  - la cámara web como imagen (CAP-038);
  - iPhone o iPad (CAP-044);
  - el fondo de la cámara (VID-037);
  - **Solo cámara**, sin pantalla (falta de VID-034);
  - compartir al terminar una grabación (VID-053).
- **H.265** (VID-027): en espera.
- **Windows en ARM** (APP-041): falta el instalador ARM64 y un equipo donde probarlo.
- **Microsoft Store (MSIX)**: lo probaste el 2026-11-15; falta la cuenta de Partner Center.
- **Flecos anotados en PARITY**:
  - soltar un emoji desde `Win+.` (EDT-033);
  - soltar una dirección arrastrada desde el navegador (CAP-037);
  - el texto de las páginas de un PDF (CAP-041);
  - exportar a WebP y HEIC (EDT-111);
  - la entrada **Aplicar tema…** de la pestaña Estilos (EDT-100).
- [APP-012-snap.md](APP-012-snap.md): ya no aplica, porque el aspecto Cristal sustituyó a Snap.
- Rendimiento con tres monitores 4K (CAP-013, APP-053): no hay ese equipo.
