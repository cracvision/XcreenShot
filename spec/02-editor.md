# 02 · Editor de imágenes

**Archivo:** `02-editor.md` · **Prefijo de IDs:** `EDT-` · **Versión:** 0.1 (borrador) · **Fecha:** 2026-10-12

Este archivo especifica el editor de imágenes de XcreenShot: la ventana principal en su vista Editor, el sistema de herramientas (propiedades, estilos guardados, favoritos, temas, color y degradados), las herramientas de anotación y de píxeles, los efectos, las operaciones de imagen y lienzo, la gestión de objetos, las plantillas, los archivos (abrir, guardar, exportar, imprimir, lotes), los atajos del editor, sus preferencias y sus comandos para CLI y MCP. La vista Biblioteca (`LIB-`), la Bandeja reciente (`TRY-`), los destinos de compartir y el Enlace para compartir (`SHR-`), la captura (`CAP-`), el video y las Diapositivas narradas (`VID-`), la app en general (`APP-`) y las funciones inteligentes (`SMT-`) se especifican en sus propios archivos; aquí solo se citan. Salvo que una función diga otra cosa, todo vale igual en Windows y macOS (SPEC §2.1): lo que en Snagit existe en una sola plataforma, aquí existe en las dos.

**Convenciones de este archivo**
- **Teclas.** `Ctrl`/`Cmd` significa `Ctrl` en Windows y `Command` (⌘) en macOS; `Alt`/`Option` igual. `Supr`/`Retroceso` son las teclas de borrar. En macOS, `Control` es la tecla Control (⌃), distinta de `Cmd`.
- **(decisión de diseño)** marca un comportamiento que la documentación pública de Snagit no describe, o que XcreenShot define a propósito de otra forma. Lo fija este spec.
- **"Por defecto"** es el valor en una instalación nueva.
- **Píxeles.** Las posiciones y tamaños se expresan en píxeles de la imagen (no de la pantalla), con origen en la esquina superior izquierda del lienzo.
- **Imagen base.** Los píxeles del documento (la captura o el archivo abierto). Los **objetos** (anotaciones e imágenes flotantes) están encima y son editables hasta que se aplanan (glosario de SPEC §6).
- **Herramienta activa / objeto seleccionado.** El panel Propiedades muestra las propiedades de la herramienta activa o, si hay objetos seleccionados, las de esos objetos (EDT-014).
- **Referencias a otros archivos.** `PREFIJO- (nombre de la función)`, o el ID cuando ya está fijado (por ejemplo, CAP-051).
- **Claves de ajuste.** Las preferencias de este archivo usan claves `editor.*` (patrón de APP- (Ventana de Preferencias)).
- **Hitos.** M1: armazón básico, Flecha, Texto, Forma (rectángulo y elipse), Desenfocar, Recortar, deshacer y rehacer, zoom, guardar PNG, JPG y `.xshot`, Copiar todo. Detalles de captura es M3 y la Medición en vivo es M5, como fija SPEC §4. Todo lo demás es M2.

---

## Índice

1. **Armazón del editor** — EDT-001 Ventana principal · EDT-002 Menús del editor · EDT-003 Barra de herramientas y menú Más · EDT-004 Personalizar la barra de herramientas · EDT-005 Panel lateral · EDT-006 Barra de edición rápida · EDT-007 Zoom y vista · EDT-008 Cuadrícula de píxeles · EDT-009 Pantalla completa · EDT-010 Área de trabajo · EDT-011 Barra de estado · EDT-012 Cambios sin guardar y copia de recuperación · EDT-013 Accesibilidad del lienzo
2. **Sistema de herramientas** — EDT-014 Propiedades y estilo actual · EDT-015 Estilos guardados · EDT-016 Favoritos · EDT-017 Temas · EDT-018 Exportar e importar temas `.xtheme` · EDT-019 Selector de color · EDT-020 Degradados · EDT-021 Sombra de los objetos · EDT-022 Selección, tiradores y modificadores · EDT-023 Seleccionar todos los objetos de una herramienta
3. **Herramientas de anotación** — EDT-024 Flecha · EDT-025 Línea · EDT-026 Texto · EDT-027 Corrector ortográfico · EDT-028 Globo · EDT-029 Forma: rectángulo y elipse · EDT-030 Forma: rectángulo redondeado y polígono · EDT-031 Sello · EDT-032 Sellos propios y Guardar como sello · EDT-033 Emoji · EDT-034 Paquetes de sellos `.xstamps` · EDT-035 Paso · EDT-036 Resaltador · EDT-037 Lápiz · EDT-038 Lupa · EDT-039 Foco · EDT-040 Desenfocar · EDT-041 Simplificar
4. **Herramientas de píxeles** — EDT-042 Mover · EDT-043 Selección · EDT-044 Acciones sobre la selección · EDT-045 Relleno del hueco · EDT-046 Varita mágica · EDT-047 Relleno · EDT-048 Borrador · EDT-049 Recortar · EDT-050 Recorte no destructivo · EDT-051 Recortar al lienzo y recortar bordes · EDT-052 Quitar franja y añadir espacio
5. **Efectos** — EDT-053 Panel Efectos · EDT-054 Efectos guardados · EDT-055 Borde · EDT-056 Bordes decorativos · EDT-057 Sombra de la imagen · EDT-058 Perspectiva · EDT-059 Página doblada · EDT-060 Reflejo · EDT-061 Desvanecer bordes · EDT-062 Filtros · EDT-063 Ajuste de color · EDT-064 Reemplazo de color · EDT-065 Efecto Foco y lupa · EDT-066 Marca de agua · EDT-067 Detalles de captura · EDT-068 Efectos de transformación
6. **Imagen y lienzo** — EDT-069 Cambiar tamaño de la imagen · EDT-070 Tamaño del lienzo · EDT-071 Ampliar el lienzo automáticamente · EDT-072 Girar · EDT-073 Voltear · EDT-074 Color del lienzo · EDT-075 Esquinas redondeadas · EDT-076 Volver al original · EDT-077 Resolución y densidad de píxeles
7. **Composición y objetos** — EDT-078 Imágenes flotantes · EDT-079 Selección múltiple · EDT-080 Transformar varios objetos · EDT-081 Orden de apilado · EDT-082 Agrupar y desagrupar · EDT-083 Alinear y distribuir · EDT-084 Ajuste magnético y guías · EDT-085 Bloquear objetos · EDT-086 Posición y tamaño numéricos · EDT-087 Mover con el teclado · EDT-088 Duplicar objetos · EDT-089 Cortar, copiar y pegar · EDT-090 Copiar todo · EDT-091 Aplanar y desaplanar · EDT-092 Deshacer y rehacer · EDT-093 Cursor como objeto · EDT-094 Medición en vivo
8. **Plantillas** — EDT-095 Nuevo desde plantilla · EDT-096 Plantillas incorporadas · EDT-097 Ajustes de la plantilla · EDT-098 Zonas de imagen · EDT-099 Secciones · EDT-100 Aplicar un tema a objetos existentes · EDT-101 Plantillas propias `.xtemplate` · EDT-102 Paginación al imprimir y en PDF
9. **Archivos** — EDT-103 Nueva imagen · EDT-104 Nueva desde el portapapeles · EDT-105 Abrir y arrastrar archivos · EDT-106 Formato `.xshot` · EDT-107 Guardar, Guardar como y Guardar todo · EDT-108 Cerrar y Cerrar todo · EDT-109 Renombrar, duplicar, eliminar y mostrar · EDT-110 Exportar a PNG y JPG · EDT-111 Más formatos de exportación · EDT-112 Opciones de cada formato · EDT-113 Exportación por lotes · EDT-114 Imprimir y configurar página
10. **Atajos de teclado** — EDT-115 Atajos de herramientas · EDT-116 Atajos de edición, vista y archivo · EDT-117 Teclas del lienzo y modificadores
11. **Preferencias y automatización** — EDT-118 Preferencias del editor · EDT-119 Comandos del editor (CLI y MCP)
12. [Relacionado](#relacionado) · [Excluido](#excluido) · [Cobertura](#cobertura)

---

## 1. Armazón del editor

### EDT-001 · Ventana principal
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** ventana del Snagit Editor (pestañas Editor y Library, botón Capture, menú Create, zona de compartir)
- **Comportamiento:**
  - La ventana principal de XcreenShot tiene dos vistas: **Editor** y **Biblioteca**. Un conmutador de dos segmentos en el extremo izquierdo de la barra de herramientas cambia entre ellas; la Biblioteca la define `LIB-`. Las dos vistas comparten ventana, menús y Bandeja reciente.
  - Zonas de la vista Editor, de arriba abajo:
    - barra de menús (EDT-002);
    - barra de herramientas (EDT-003): conmutador Editor/Biblioteca, botón **Capturar**, menú **Crear**, herramientas, y a la derecha la zona de compartir;
    - lienzo dentro del área de trabajo (EDT-010), con el panel lateral a la derecha (EDT-005);
    - barra de estado (EDT-011);
    - Bandeja reciente, que se puede ocultar (TRY- (Bandeja reciente)).
  - **Capturar** abre el Panel de captura (CAP-001).
  - **Crear** contiene: **Nuevo desde plantilla…** (EDT-095), **Nueva imagen…** (EDT-103), **Nueva desde el portapapeles** (EDT-104) y **Diapositivas narradas…** (`VID-`).
  - **Zona de compartir**, en este orden: **Copiar todo** (EDT-090), **Enlace para compartir** (SHR- (Enlace para compartir)) y **Compartir ▾**, con los destinos y **Guardar como…** (EDT-107). Los destinos los define `SHR-`.
  - Sin documento abierto, el lienzo muestra un estado vacío con **Capturar**, **Abrir…**, **Nueva imagen** y **Nueva desde el portapapeles**. También se puede soltar un archivo ahí (EDT-105).
  - La ventana recuerda su tamaño, posición, monitor y vista activa entre sesiones. Si ese monitor ya no existe, se abre centrada en el principal.
  - Tamaño mínimo: 900 × 600 px lógicos (decisión de diseño). Por debajo de 1200 px de ancho, la zona de compartir se reduce a íconos y las herramientas sobrantes pasan al menú Más (EDT-003).
- **Criterios de aceptación:**
  - [ ] Tras una captura de región, la ventana se abre en la vista Editor con la captura en el lienzo y seleccionada en la Bandeja reciente.
  - [ ] El conmutador pasa a Biblioteca y vuelve a Editor sin perder el documento abierto, el zoom ni la herramienta activa.
  - [ ] Con la ventana a 900 px de ancho, todas las herramientas siguen accesibles (en la barra o en Más) y ningún control queda cortado.
  - [ ] Cerrar y volver a abrir la app restaura el tamaño, la posición y la vista de la ventana.
- **Fuentes:** research/02-editor.md §1 (Editor tab vs Library tab, Capture button, Create menu, Share area); research/05-plataformas-versiones.md §1A (2025.3.0)

### EDT-002 · Menús del editor
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** menús File, Edit, Image, View, Help del Editor; menú "?" de ayuda
- **Comportamiento:**
  - En macOS los menús están en la barra de menús del sistema, precedidos del menú **XcreenShot** (Acerca de, Preferencias, Ocultar, Salir). En Windows, la ventana tiene su propia barra de menús bajo la barra de título; `Alt` o `F10` le dan el foco.
  - Menús y entradas (cada entrada es un comando, APP-027; su atajo aparece a la derecha):
    - **Archivo:** Nueva imagen, Nueva desde el portapapeles, Nuevo desde plantilla, Abrir…, Abrir recientes ▸, Guardar, Guardar como…, Guardar todo, Exportar…, Exportar por lotes…, Guardar como sello…, Guardar como plantilla…, Importar ▸ (temas, sellos, plantillas), Renombrar…, Duplicar, Mostrar en la biblioteca, Mostrar en el Explorador/Finder, Mover a la papelera, Configurar página… (macOS), Imprimir…, Cerrar, Cerrar todo.
    - **Edición:** Deshacer, Rehacer, Cortar, Copiar, Copiar todo, Pegar, Pegar en el mismo sitio, Pegar con el mismo estilo, Duplicar, Eliminar, Seleccionar todo, Seleccionar todos los objetos, Deseleccionar, Invertir selección, Volver al original.
    - **Imagen:** Cambiar tamaño de la imagen…, Tamaño del lienzo…, Color del lienzo…, Girar ▸, Voltear ▸, Recortar a la selección, Recortar al lienzo, Ajustar el lienzo al contenido, Recortar bordes, Esquinas redondeadas…, Efectos ▸, Ajuste magnético (casilla).
    - **Objeto:** Traer al frente, Traer adelante, Enviar atrás, Enviar al fondo, Agrupar, Desagrupar, Alinear ▸, Distribuir ▸, Bloquear, Desbloquear todo, Aplanar, Aplanar todo, Desaplanar, Desaplanar todo, Posición y tamaño….
    - **Ver:** Editor, Biblioteca, Acercar, Alejar, Tamaño real, Ajustar a la ventana, Tamaño original en pantalla, Cuadrícula de píxeles, Panel lateral, Bandeja reciente, Barra de herramientas ▸ (Personalizar…, Mostrar etiquetas), Pantalla completa.
    - **Ayuda:** las entradas de APP-015, más **Atajos del editor**, que abre la hoja de atajos (APP-023) filtrada al editor.
  - Las entradas que no se pueden usar en el estado actual aparecen desactivadas; nunca desaparecen (decisión de diseño). Las de un hito aún no implementado no aparecen.
  - El clic derecho (o `Control`+clic en macOS) sobre el lienzo abre un menú contextual con las entradas que se aplican a lo que hay bajo el puntero: un objeto, una selección o la imagen.
- **Criterios de aceptación:**
  - [ ] Cada entrada de menú ejecuta su comando, y una prueba automática comprueba que todas tienen un ID de comando registrado (APP-027).
  - [ ] Sin objetos seleccionados, Agrupar y Aplanar aparecen desactivados.
  - [ ] En Windows, `Alt` y luego las flechas recorren los menús sin usar el ratón.
  - [ ] El clic derecho sobre una flecha muestra, entre otras, Aplanar, Duplicar, Orden ▸ y Eliminar; sobre una zona sin objetos muestra Pegar y Seleccionar todo.
- **Fuentes:** research/02-editor.md §1 (Help / feedback menu), §8, §10.3, §12; research/05-plataformas-versiones.md §1A (2026.4.0), §1B (2025.4.0)

### EDT-003 · Barra de herramientas y menú Más
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** barra de herramientas del Editor y desplegable More
- **Comportamiento:**
  - La barra muestra las herramientas como botones con ícono y, opcionalmente, etiqueta (preferencia **Etiquetas de la barra de herramientas**, `editor.toolbarLabels`; por defecto: sí). Clic derecho en la barra > **Solo íconos** cambia la misma preferencia.
  - Barra por defecto (decisión de diseño): Favoritos, Mover, Flecha, Texto, Globo, Forma, Paso, Desenfocar, Recortar, Selección. El resto está en **Más**.
  - **Más ▾** lista las herramientas y comandos que no están en la barra, agrupados (Anotación, Píxeles, Imagen) y con un campo de búsqueda. Al elegir una herramienta de Más, ocupa durante un rato un hueco temporal al final de la barra, hasta que se elige otra de Más. Para fijarla, se arrastra a la barra o se usa Personalizar (EDT-004).
  - La herramienta activa se marca con fondo y borde de acento, nunca solo con color (APP-045). Cada botón tiene un tooltip con su nombre y atajo (APP-014).
  - Las herramientas solo actúan sobre documentos de imagen. Si el elemento abierto es un video, la barra muestra las herramientas de `VID-`.
  - En M1 la barra solo contiene las herramientas que ya existen (Mover, Flecha, Texto, Forma, Desenfocar, Recortar).
- **Criterios de aceptación:**
  - [ ] Elegir Lupa en Más la pone en el hueco temporal y la activa. Al elegir después Borrador en Más, Borrador ocupa ese hueco y Lupa vuelve a Más.
  - [ ] Con Solo íconos activado, los botones no tienen texto y el tooltip sigue mostrando nombre y atajo.
  - [ ] La búsqueda "bor" en Más muestra Borrador y Borde.
- **Fuentes:** research/02-editor.md §1 (Toolbar), §2.1

### EDT-004 · Personalizar la barra de herramientas
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Customize Toolbar (More > Customize Toolbar), Restore Defaults, Space
- **Comportamiento:**
  - **Ver > Barra de herramientas > Personalizar…** o **Más > Personalizar…** abre un panel con todos los elementos disponibles. Mientras está abierto, en la barra se puede:
    - arrastrar elementos desde el panel para añadirlos;
    - arrastrarlos dentro de la barra para reordenarlos;
    - arrastrarlos fuera de la barra para quitarlos;
    - insertar **Separador** y **Espacio flexible** para agrupar.
  - Elementos disponibles:
    - todas las herramientas;
    - comandos: Guardar, Guardar como, Exportar, Duplicar, Nueva desde el portapapeles, Copiar todo, Volver al original, Recortar bordes, Deshacer, Rehacer, Imprimir;
    - efectos individuales (por ejemplo, Borde o Filtros), que abren directamente ese efecto (EDT-053);
    - destinos de compartir, que define `SHR-`.
  - **Restaurar valores por defecto** vuelve a la barra de EDT-003. **Listo** cierra el panel.
  - La disposición se guarda al momento, sobrevive a reinicios y viaja en la exportación de ajustes (APP-011).
  - Teclado: con un elemento de la barra enfocado en modo personalizar, `Alt`/`Option`+`←`/`→` lo mueve y `Supr` lo quita (APP-043).
- **Criterios de aceptación:**
  - [ ] Arrastrar Borrador a la segunda posición, reiniciar la app y abrir el editor muestra Borrador en la segunda posición.
  - [ ] Arrastrar Flecha fuera de la barra la quita de la barra, y Flecha aparece en Más.
  - [ ] Restaurar valores por defecto devuelve exactamente la barra de EDT-003.
  - [ ] Un botón del efecto Borde añadido a la barra abre la configuración de Borde para el documento actual.
- **Fuentes:** research/02-editor.md §2.1

### EDT-005 · Panel lateral
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** barra lateral con Quick Styles, Tool Properties y Effects
- **Comportamiento:**
  - Panel a la derecha del lienzo con tres pestañas:
    - **Estilos**: los estilos guardados de la herramienta activa (EDT-015);
    - **Propiedades**: las propiedades de la herramienta activa o de los objetos seleccionados (EDT-014), incluida la sección Posición y tamaño (EDT-086);
    - **Efectos**: los efectos del documento o del objeto seleccionado (EDT-053).
  - Sin selección y con Mover activo, Propiedades muestra las del documento: tamaño de la imagen, color del lienzo, resolución y radio de las esquinas, cada uno con su acción (decisión de diseño).
  - Cada sección de una pestaña se puede plegar. El panel se puede ensanchar de 260 a 480 px arrastrando su borde izquierdo.
  - **Ver > Panel lateral** lo muestra u oculta. El panel recuerda pestaña, anchura, secciones plegadas y visibilidad.
  - En M1 existen Propiedades y una pestaña Estilos vacía con el texto "Disponible próximamente"; la pestaña Efectos aparece en M2 (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Al seleccionar un texto, la pestaña Propiedades muestra fuente, tamaño y color de ese texto; al deseleccionar, muestra las de la herramienta activa.
  - [ ] Ocultar el panel amplía el área del lienzo, y el estado se conserva tras reiniciar.
  - [ ] Con el panel en 480 px y la ventana en su tamaño mínimo, el lienzo sigue visible y usable.
- **Fuentes:** research/02-editor.md §1 (Side panel)

### EDT-006 · Barra de edición rápida
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** barra flotante de propiedades junto al objeto seleccionado (Mac), con opción para ocultarla
- **Comportamiento:**
  - Al seleccionar un solo objeto aparece, junto a él, una barra flotante pequeña con sus propiedades más usadas (decisión de diseño):
    - Flecha y Línea: color, grosor, extremos, tipo de línea;
    - Texto: fuente, tamaño, negrita, color;
    - Globo: forma, relleno, contorno, tamaño de letra;
    - Forma: relleno, contorno, grosor;
    - Paso: relleno, forma, valor;
    - Desenfocar: tipo, intensidad.
    Todas las barras terminan con un menú **Estilos ▾** con los estilos guardados de esa herramienta y un botón **⋯**, que lleva a la pestaña Propiedades.
  - Se coloca encima del objeto o, si no cabe, debajo, sin taparlo. Se oculta mientras se arrastra, gira o cambia de tamaño el objeto, y reaparece al soltar.
  - Preferencia **Barra de edición rápida** (`editor.quickEditBar`; por defecto: sí).
  - Nunca aparece en la imagen copiada, exportada ni impresa.
- **Criterios de aceptación:**
  - [ ] Al seleccionar una flecha aparece la barra. Cambiar el color en ella cambia la flecha y lo que muestra Propiedades.
  - [ ] Con un objeto pegado al borde superior del lienzo, la barra aparece debajo del objeto.
  - [ ] Con la preferencia desactivada, la barra no aparece con ningún tipo de objeto.
- **Fuentes:** research/02-editor.md §1 (Quick edit toolbar), §11

### EDT-007 · Zoom y vista
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Actual Size, Zoom to Fit, Zoom In/Out, Always Shrink to Fit (Windows); recordar zoom y desplazamiento (Mac)
- **Comportamiento:**
  - **Tamaño real** (`Ctrl`/`Cmd`+`0`): cada píxel de la imagen ocupa un píxel físico de la pantalla. Una captura vista en el monitor donde se hizo se ve del mismo tamaño que el original.
  - **Tamaño original en pantalla:** muestra la captura con el tamaño que tenía en el monitor de origen, usando la escala que guarda (CAP-021). Por ejemplo, una captura hecha a 200 % y vista en un monitor al 100 % se muestra al 50 % (decisión de diseño).
  - **Ajustar a la ventana:** la imagen entera cabe en el área visible.
  - **Acercar / Alejar** (`Ctrl`/`Cmd`+`=` y `-`) por pasos fijos: 10, 25, 33, 50, 67, 100, 150, 200, 300, 400, 600, 800, 1200, 1600, 2400 y 3200 %. Rango: 10 %–3200 %.
  - Otras formas de hacer zoom: `Ctrl`/`Cmd`+rueda, centrado en el puntero; pellizco en el trackpad; en macOS también `Option`+desplazamiento.
  - **Reducir para ajustar** (`editor.shrinkToFit`; por defecto: sí): un documento que no cabe se abre ajustado a la ventana; uno que cabe se abre a tamaño real, nunca ampliado.
  - **Recordar la vista** (`editor.rememberView`; por defecto: sí): cada documento recuerda su zoom y su posición entre cambios de documento y entre sesiones.
  - Desplazamiento de la vista: barras de desplazamiento, rueda (`Shift`+rueda en horizontal), trackpad, `Espacio`+arrastrar o botón central+arrastrar.
  - La barra de estado muestra el zoom con un campo editable y un menú de valores (EDT-011).
- **Criterios de aceptación:**
  - [ ] Una captura de 6000 × 4000 px se abre ajustada a la ventana; una de 400 × 300 px se abre al 100 %.
  - [ ] Al 100 %, una captura de 200 × 100 px en un monitor al 150 % ocupa 200 × 100 píxeles físicos (se comprueba con una captura del propio editor).
  - [ ] `Ctrl`/`Cmd`+rueda acerca la vista manteniendo bajo el puntero el mismo píxel de la imagen.
  - [ ] Al cambiar a otro documento y volver, el zoom y la posición son los mismos; también tras reiniciar la app.
- **Notas de plataforma:** en macOS, "píxel físico" es el píxel de la pantalla Retina; en Windows, el píxel del monitor a cualquier escala.
- **Fuentes:** research/02-editor.md §1 (Zoom), §12; research/01-captura.md (escala por captura, vía CAP-021)

### EDT-008 · Cuadrícula de píxeles
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Pixel Grid
- **Comportamiento:**
  - **Ver > Cuadrícula de píxeles** (casilla; por defecto: activada) superpone una retícula de un píxel físico de grosor que separa los píxeles de la imagen.
  - Solo se dibuja desde el 600 % de zoom (`editor.pixelGridMinZoom`; valores 400–1600 %).
  - El color de la retícula se adapta al fondo para que siempre se vea (claro sobre oscuro y viceversa).
  - No forma parte de la imagen: no aparece al copiar, exportar ni imprimir.
- **Criterios de aceptación:**
  - [ ] Al 800 % con la casilla marcada se ve la retícula; al 400 % no.
  - [ ] Exportar a PNG al 800 % con la retícula visible produce un archivo sin retícula.
- **Fuentes:** research/02-editor.md §1 (Pixel Grid)

### EDT-009 · Pantalla completa
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** pantalla completa del Editor (Mac)
- **Comportamiento:**
  - **Ver > Pantalla completa** (`F11` en Windows; `Control`+`Cmd`+`F` en macOS, además del atajo del sistema con la tecla Globo) lleva la ventana a pantalla completa en su monitor.
  - En pantalla completa se ocultan la barra de título y la Bandeja reciente. Los menús siguen accesibles (en macOS, al llevar el puntero arriba; en Windows, con `Alt`).
  - `Esc` sale de pantalla completa si no hay ninguna acción que cancelar. Tiene prioridad sobre "`Esc` cierra el editor" (EDT-012).
- **Criterios de aceptación:**
  - [ ] El atajo alterna entre pantalla completa y ventana, y al salir la ventana recupera su tamaño anterior.
  - [ ] En pantalla completa, con "`Esc` cierra el editor" activado, el primer `Esc` solo sale de pantalla completa.
- **Fuentes:** research/02-editor.md §1 (Full screen), §12

### EDT-010 · Área de trabajo
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** color del área de trabajo (preferencia del Editor en Windows)
- **Comportamiento:**
  - El área de trabajo es el fondo que rodea al lienzo. El lienzo tiene una sombra suave para distinguir su borde.
  - Preferencia **Color del área de trabajo** (`editor.workspaceColor`): **Automático** (por defecto; sigue el tema claro u oscuro, APP-012) o un color elegido con el selector (EDT-019).
  - Las zonas transparentes del lienzo se muestran con un damero propio de XcreenShot, de dos grises neutros. El damero mantiene un tamaño fijo en pantalla aunque cambie el zoom (decisión de diseño).
  - Los objetos que quedan fuera del lienzo se ven atenuados en el área de trabajo mientras están seleccionados (EDT-049).
- **Criterios de aceptación:**
  - [ ] Con Automático, cambiar el tema de la app de claro a oscuro cambia el color del área de trabajo.
  - [ ] Un color personalizado se conserva tras reiniciar, y el damero de las zonas transparentes se sigue distinguiendo del área de trabajo.
- **Fuentes:** research/02-editor.md §1 (Workspace / Canvas), §11

### EDT-011 · Barra de estado
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** dimensiones de la imagen bajo el lienzo (clic abre Resize Image); radio de las esquinas junto a ellas (experimento de Mac)
- **Comportamiento:**
  - Contenido, de izquierda a derecha:
    - **Dimensiones** de la imagen (por ejemplo, "1920 × 1080 px"). Un clic abre Cambiar tamaño de la imagen (EDT-069).
    - **Resolución** en PPP. Un clic abre Resolución y densidad de píxeles (EDT-077).
    - **Radio de las esquinas**, si es mayor que 0 (EDT-075).
    - **Posición del puntero** en píxeles de la imagen, como mejora.
    - **Tamaño de la selección** o del objeto seleccionado (An × Al y ángulo).
    - Aviso **N objetos fuera del lienzo**, con la acción Seleccionar (EDT-049).
    - **Zoom**: campo editable, botones − y +, y un menú con Tamaño real, Ajustar a la ventana y los pasos de EDT-007.
  - En M1 aparecen las dimensiones, la posición del puntero y el zoom; el resto llega con sus funciones.
- **Criterios de aceptación:**
  - [ ] Con una captura de 1280 × 720, la barra muestra "1280 × 720 px". Tras recortar a 600 × 400, se actualiza al instante.
  - [ ] Escribir "250" en el campo de zoom y pulsar `Enter` pone la vista al 250 %.
  - [ ] Al pasar el puntero por la esquina inferior derecha de la imagen, la posición mostrada es la del último píxel (An−1, Al−1).
- **Mejora sobre Snagit:** posición del puntero y tamaño de la selección siempre visibles.
- **Fuentes:** research/02-editor.md §1 (Image dimensions readout), §7 (Resize Image)

### EDT-012 · Cambios sin guardar y copia de recuperación
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** indicador de cambios sin guardar en la bandeja (con tooltip en Mac); preferencia Escape closes Editor
- **Comportamiento:**
  - Un documento con cambios que aún no están en su archivo se marca así (nunca solo con color):
    - en la Bandeja reciente, con un punto de acento sobre la miniatura cuyo tooltip dice "Cambios sin guardar" (TRY- (Bandeja reciente));
    - en el título de la ventana, con "— Editado" (en macOS, además, con el punto del botón de cerrar).
  - **Documentos de la biblioteca** (todas las capturas, LIB- (autoguardado)): se guardan solos, sin preguntar, al cambiar de documento, al cerrarlo, al cerrar la ventana y al salir de la app. El indicador desaparece al guardarse.
  - **Documentos de fuera de la biblioteca** (por ejemplo, un PNG abierto desde una carpeta): al cerrarlos con cambios, XcreenShot pregunta **Guardar**, **No guardar** o **Cancelar** (EDT-108, APP-004).
  - **Copia de recuperación:** mientras un documento tenga cambios sin guardar, XcreenShot guarda una copia de recuperación cada 30 s, y solo si hubo cambios desde la anterior. Se usa tras un cierre inesperado (APP-047) y se borra al guardar o descartar los cambios.
  - Preferencia **`Esc` cierra el editor** (`editor.escClosesEditor`; por defecto: no): si está activada, un `Esc` sin nada que cancelar (ni dibujo en curso, ni selección, ni edición de texto, ni pantalla completa) cierra la ventana del editor. No cierra los documentos.
- **Criterios de aceptación:**
  - [ ] Al dibujar una flecha en una captura aparece el indicador en la bandeja y en el título; al cambiar a otra captura, la primera se guarda y su indicador desaparece.
  - [ ] Cerrar un PNG externo con cambios muestra Guardar / No guardar / Cancelar, y Cancelar deja el documento abierto y sin cambios.
  - [ ] Matar el proceso 45 s después de editar un documento externo sin guardar y volver a abrir la app ofrece recuperar ese documento con sus ediciones (APP-047).
  - [ ] Con "`Esc` cierra el editor" activado y una flecha seleccionada, el primer `Esc` deselecciona y el segundo cierra la ventana.
- **Fuentes:** research/02-editor.md §1 (Unsaved-changes indicator, Escape closes Editor), §11; research/04-biblioteca-compartir.md §2 (Unsaved indicator)

### EDT-013 · Accesibilidad del lienzo
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** navegación con Tab y anillo de foco, etiquetas para lectores de pantalla, flechas para ajustar deslizadores y campos numéricos
- **Comportamiento:**
  - `F6` / `Shift`+`F6` mueven el foco entre las zonas de la ventana: barra de herramientas, lienzo, panel lateral, Bandeja reciente y barra de estado.
  - Con el foco en el lienzo:
    - `Tab` / `Shift`+`Tab` seleccionan el objeto siguiente o anterior, por orden de apilado;
    - las flechas mueven el objeto (EDT-087);
    - `Enter` edita el texto del objeto o, si no tiene texto, abre la barra de edición rápida;
    - `Supr` lo elimina;
    - **Insertar** (`Ctrl`/`Cmd`+`Enter`) crea un objeto de la herramienta activa con su tamaño por defecto, en el centro de la vista, para anotar sin ratón (decisión de diseño).
  - Los lectores de pantalla anuncian cada objeto con su tipo, su texto si lo tiene, su posición y su tamaño (por ejemplo, "Globo: Haz clic aquí, en 320, 140, 200 por 80 píxeles"), y anuncian los cambios de herramienta y de zoom.
  - Todos los deslizadores y campos numéricos del editor responden a `↑`/`↓` (un paso) y a `Shift`+`↑`/`↓` (diez pasos).
  - El anillo de foco y los tiradores se ven con contraste suficiente en tema claro, tema oscuro y alto contraste (APP-045).
- **Criterios de aceptación:**
  - [ ] Solo con el teclado se puede capturar, elegir Flecha, insertar una flecha, moverla 10 px, cambiar su color y guardar.
  - [ ] Con un lector de pantalla (Narrador o VoiceOver), `Tab` sobre un texto que dice "Paso 1" anuncia "Texto: Paso 1" y su posición.
  - [ ] Con el foco en el deslizador de opacidad, `↑` sube un 1 % y `Shift`+`↑` un 10 %.
- **Fuentes:** research/02-editor.md §1 (Accessibility); research/05-plataformas-versiones.md §1A (2024.3.5, 2026.0.0)

---

## 2. Sistema de herramientas

### EDT-014 · Propiedades y estilo actual
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Tool Properties; la forma de fijar las propiedades por defecto de una herramienta
- **Comportamiento:**
  - Cada herramienta tiene un **estilo actual**: los valores de sus propiedades con los que se crearán los objetos siguientes. Se ve y se cambia en la pestaña Propiedades (EDT-005).
  - Sin selección, cambiar una propiedad cambia el estilo actual de la herramienta. Con objetos seleccionados, la cambia en esos objetos **y** en el estilo actual de su herramienta (decisión de diseño, coherente con "el último estilo usado").
  - Con varios objetos seleccionados de distintas herramientas, Propiedades muestra solo las propiedades comunes (color, opacidad, sombra…). Una propiedad con valores distintos aparece como "Mixto"; cambiarla aplica el nuevo valor a todos.
  - El estilo actual de cada herramienta se recuerda entre sesiones.
  - Controles comunes: los números aceptan decimales donde tiene sentido (tamaño de letra, grosor) y unidades en píxeles; los deslizadores tienen al lado un campo numérico.
  - **Opacidad** (0–100 %; por defecto 100 %) está en todas las herramientas de anotación salvo Paso y Lupa, que también la tienen pero empiezan en 100 % (decisión de diseño: todas la tienen).
  - En M1, Propiedades cubre Flecha, Texto, Forma y Desenfocar; en M2, el resto.
- **Criterios de aceptación:**
  - [ ] Cambiar el color de Flecha a verde sin selección hace que la siguiente flecha sea verde; tras reiniciar, la siguiente sigue siendo verde.
  - [ ] Con una flecha roja y otra azul seleccionadas, Color muestra "Mixto"; elegir negro pone las dos en negro.
  - [ ] Escribir "12,5" en el tamaño de letra (en español) o "12.5" (en inglés) crea texto de 12,5 pt.
- **Fuentes:** research/02-editor.md §2.2, §2.7 (Opacity, decimal values)

### EDT-015 · Estilos guardados
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Quick Styles (estilos por herramienta, +, reordenar, eliminar, Reset Quick Styles, aplicar a varios objetos); Cursor Quick Style popup (Windows)
- **Comportamiento:**
  - La pestaña **Estilos** muestra una cuadrícula de muestras de la herramienta activa. Cada muestra es una miniatura que representa el estilo (una flecha, un globo…) con el tema activo.
  - Clic en una muestra: la herramienta usa ese estilo. Si hay objetos seleccionados de esa herramienta, se les aplica a todos.
  - Al cambiar una propiedad, aparece al final una muestra **temporal** con borde discontinuo. **+** la guarda como estilo nuevo.
  - Gestión: arrastrar para reordenar; clic derecho > **Eliminar**, **Renombrar**, **Actualizar con el estilo actual** y **Añadir a favoritos**.
  - **Restablecer estilos** (menú ⋯ de la pestaña) vuelve a los estilos generados por el tema activo para esa herramienta, tras confirmar.
  - Los estilos guardados se generan a partir del **tema** activo (EDT-017). Cada tema tiene sus propios estilos.
  - Qué guarda un estilo: todas las propiedades visuales de la herramienta, nunca la posición, el tamaño ni el texto. Excepción: en Desenfocar, un estilo solo guarda tipo, intensidad y color (decisión de diseño, alineada con el comportamiento documentado).
  - Preferencia **Vista previa del estilo junto al puntero** (`editor.cursorStylePreview`; por defecto: no): con una herramienta de dibujo activa, una muestra pequeña del estilo actual acompaña al puntero sobre el lienzo.
- **Criterios de aceptación:**
  - [ ] Cambiar el grosor de la flecha crea una muestra temporal; **+** la convierte en un estilo que sigue ahí tras reiniciar.
  - [ ] Con tres globos seleccionados, un clic en un estilo de globo cambia los tres, y su texto se conserva.
  - [ ] Restablecer estilos de Forma elimina los estilos de Forma añadidos por el usuario y deja intactos los de Flecha.
  - [ ] Un estilo de Desenfocar aplicado a un desenfoque existente cambia su tipo e intensidad, pero no su forma ni su tamaño.
- **Fuentes:** research/02-editor.md §2.2, §1 (Cursor Quick Style popup), §11

### EDT-016 · Favoritos
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Favorites (estrella en los Quick Styles, efectos guardados favoritos en Mac, estilos de Cut Out como favoritos en Mac)
- **Comportamiento:**
  - Cada estilo guardado y cada efecto guardado (EDT-054) tiene una **estrella**. Al marcarla, el estilo aparece en la herramienta **Favoritos**. La estrella marcada se rellena con el color de acento; un segundo clic la quita.
  - La herramienta Favoritos (primera de la barra por defecto) muestra en Estilos una sola cuadrícula con los favoritos de todas las herramientas, cada uno con un pequeño ícono de su herramienta. Clic en un favorito activa su herramienta con ese estilo, o aplica el efecto al documento.
  - Se pueden marcar como favoritos los estilos de cualquier herramienta, incluidos los de Quitar franja y Simplificar.
  - Los favoritos se pueden reordenar arrastrando. Se conservan aunque se cambie de tema: guardan una copia del estilo, no un vínculo (decisión de diseño).
  - Los favoritos son también las herramientas de dibujo de las Diapositivas narradas (`VID-`).
- **Criterios de aceptación:**
  - [ ] Marcar con estrella un estilo de Flecha y uno de Globo los muestra a los dos en Favoritos; clic en el de Globo activa Globo con ese estilo.
  - [ ] Un efecto guardado marcado como favorito, al pulsarlo en Favoritos, se aplica al documento.
  - [ ] Cambiar de tema no quita ni modifica ningún favorito.
- **Fuentes:** research/02-editor.md §2.3

### EDT-017 · Temas
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Themes (crear, editar, colores, fuente, sombra; estilos generados; tema por defecto "Starter"; "Themes 2.0")
- **Comportamiento:**
  - Un **tema** es un conjunto con nombre de **colores** (de 2 a 8), **una fuente** y una **sombra** sí/no, con la posibilidad de desactivarla por herramienta. A partir de él se generan los estilos guardados de Flecha, Línea, Texto, Globo, Forma, Paso, Resaltador, Lápiz, Lupa y Foco.
  - El selector **Tema ▾** está en la cabecera de la pestaña Estilos. Su menú ⋯ ofrece: **Nuevo tema…**, **Editar…**, **Duplicar**, **Renombrar**, **Eliminar**, **Exportar…**, **Importar…** (EDT-018).
  - **Editor de temas:** nombre; lista de colores (añadir, quitar, reordenar, cada uno con el selector de color); fuente; sombra; vista previa en vivo con una flecha, un globo, una forma y un paso.
  - Reglas:
    - un tema tiene al menos dos colores; no se puede quitar el último par;
    - no se permiten dos colores idénticos;
    - un tema sin nombre se guarda como "Tema sin nombre N".
  - **Guardar** un tema editado regenera sus estilos guardados y sustituye los anteriores de ese tema, tras confirmar ("Se sustituirán los estilos de este tema"). Los favoritos no cambian (EDT-016).
  - **Tema por defecto** (decisión de diseño, diseño propio): "Base", con cinco colores (carbón #1F2430, coral #F2545B, azul #2D7FF9, ámbar #F5A524 y blanco #FFFFFF), la fuente de interfaz del sistema y sombra activada. Se incluye un segundo tema, "Sobrio", en escala de grises con un acento azul. Los temas incorporados no se pueden eliminar, pero sí duplicar y editar la copia.
  - Los temas están disponibles en el editor y en las plantillas (EDT-100).
- **Criterios de aceptación:**
  - [ ] Crear un tema con tres colores y la fuente Georgia genera estilos de Globo con esos colores y esa fuente.
  - [ ] Editar el tema activo y guardar sustituye sus estilos tras confirmar; Cancelar deja los estilos como estaban.
  - [ ] Intentar añadir un color ya presente en el tema muestra un aviso y no lo añade.
  - [ ] Eliminar un tema incorporado no es posible: la acción aparece desactivada.
- **Fuentes:** research/02-editor.md §2.4; research/05-plataformas-versiones.md §1A (2024.0.0)

### EDT-018 · Exportar e importar temas `.xtheme`
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** exportar e importar temas para compartirlos (`.snagtheme`)
- **Comportamiento:**
  - **Exportar…** guarda el tema en un archivo `.xtheme` (JSON, SPEC §7) con nombre, colores, fuente, sombra por herramienta y los estilos guardados del tema.
  - **Importar…** (también Archivo > Importar > Tema…, doble clic en el archivo o arrastrarlo a la ventana) añade el tema y lo activa.
  - Si ya existe un tema con ese nombre, se ofrece **Reemplazar** o **Conservar ambos** (el importado recibe el sufijo " (2)").
  - Si la fuente del tema no está instalada, el tema se importa igualmente; los estilos usan la fuente de interfaz del sistema y un aviso nombra la fuente que falta. El archivo conserva la fuente original para cuando esté disponible.
  - Un `.xtheme` creado en Windows se importa en macOS con los mismos valores, y viceversa.
  - La configuración administrada puede instalar temas para todos los usuarios (`policy.themes`, APP-035).
- **Criterios de aceptación:**
  - [ ] Exportar un tema en Windows e importarlo en macOS produce estilos con los mismos colores (hex idénticos), sombras y tamaños.
  - [ ] Importar un tema con el nombre de uno existente y elegir Conservar ambos deja dos temas: "X" y "X (2)".
  - [ ] Un `.xtheme` dañado o de una versión de formato futura se rechaza con un mensaje claro y no cambia ningún tema.
- **Mejora sobre Snagit:** formato JSON abierto y documentado.
- **Fuentes:** research/02-editor.md §2.4

### EDT-019 · Selector de color
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** colores del tema, paleta, Hex/RGB, cuentagotas, transparente, "My Colors" (Mac), selector rediseñado (Mac 2026)
- **Comportamiento:**
  - Cada propiedad de color abre el mismo selector, con:
    - **colores del tema** activo;
    - una **paleta** de 40 colores básicos (decisión de diseño);
    - un **área de matiz y saturación** con barra de luminosidad;
    - campos **Hex**, **RGB** y **HSB**, visibles a la vez (sin cambio de modo);
    - **Opacidad** del color (canal alfa, 0–100 %);
    - **Cuentagotas**, que toma un color de cualquier punto de la pantalla, también fuera de XcreenShot, con una lupa de píxeles;
    - **Transparente** (muestra de damero), solo en las propiedades donde tiene sentido (relleno, contorno de globo, color del lienzo, Relleno);
    - **Mis colores**: hasta 24 colores guardados con **+**, que se comparten entre el editor, la captura y el video;
    - **Recientes**: los últimos 8 colores usados;
    - una pestaña **Degradado** donde se admite (EDT-020).
  - Teclado: `I` activa el cuentagotas del lienzo con la herramienta activa (EDT-117); en el selector, `Enter` acepta y `Esc` cancela.
  - En M1 hay paleta, Hex/RGB, cuentagotas y transparente; Mis colores, Recientes y Degradado llegan en M2.
- **Criterios de aceptación:**
  - [ ] Escribir "#2D7FF9" en Hex muestra RGB 45, 127, 249 y aplica ese color.
  - [ ] Con el cuentagotas sobre otra app, el color elegido coincide con el píxel de la pantalla (diferencia ≤ 1 por canal en una pantalla sin HDR).
  - [ ] Un color guardado en Mis colores desde el editor aparece en el selector del color de la cámara (`VID-`).
  - [ ] La opción Transparente no aparece en el color de una flecha y sí en el relleno de una forma.
- **Fuentes:** research/02-editor.md §2.5, §12 (modifier keys)

### EDT-020 · Degradados
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Gradients (Mac): lineal con ángulo, radial, dos paradas con alfa, "My Gradients"
- **Comportamiento:**
  - Disponibles en: relleno de Forma, relleno de Globo, relleno de Paso, la herramienta Relleno, el color del lienzo y el borde de la cámara (`VID-`).
  - Tipos:
    - **Lineal**, con ángulo de 0 a 359° (campo numérico y rueda);
    - **Radial**, desde el centro del objeto.
  - **Paradas de color:** de 2 a 8, cada una con color, opacidad y posición (0–100 %). Se añaden con clic en la barra del degradado y se quitan arrastrándolas fuera. Es una mejora: Snagit documenta dos paradas.
  - **Invertir** intercambia el orden de las paradas.
  - **Mis degradados** guarda hasta 16 degradados con **+**.
  - Los objetos con degradado conservan el degradado al cambiar de tamaño, girar o aplanar.
- **Criterios de aceptación:**
  - [ ] Un rectángulo con un degradado lineal a 90° de rojo a azul muestra rojo arriba y azul abajo.
  - [ ] Un degradado radial con tres paradas se ve igual en Windows y en macOS al abrir el mismo `.xshot`.
  - [ ] Un degradado guardado en Mis degradados se puede aplicar al color del lienzo.
- **Mejora sobre Snagit:** disponible en ambas plataformas y con más de dos paradas.
- **Fuentes:** research/02-editor.md §2.6, §2.5; research/05-plataformas-versiones.md §3 (Editor: Gradients)

### EDT-021 · Sombra de los objetos
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** sombra por objeto (cuadrícula de direcciones y ajustes avanzados: color, ángulo, distancia, opacidad, desenfoque)
- **Comportamiento:**
  - Herramientas con sombra: Flecha, Línea, Texto, Globo, Forma, Sello, Paso, Lápiz e imágenes flotantes.
  - Control rápido: una cuadrícula de 3 × 3 direcciones. Un clic en una celda activa la sombra en esa dirección; la celda central hace una sombra alrededor, sin desplazamiento. Otro clic en la celda activa la apaga.
  - **Ajustes avanzados** (sección plegable):
    - color (por defecto, negro);
    - ángulo 0–359°, que se sincroniza con la cuadrícula;
    - distancia 0–50 px;
    - opacidad 0–100 %;
    - desenfoque 0–50 px.
  - Valor por defecto: el que fije el tema activo (EDT-017). En el tema Base, sombra activada en Flecha, Globo, Forma y Paso, y desactivada en Texto, Sello y Lápiz.
- **Criterios de aceptación:**
  - [ ] Un clic en la celda inferior derecha añade una sombra abajo a la derecha; un segundo clic la quita.
  - [ ] Cambiar el ángulo a 180° en los ajustes avanzados marca la celda izquierda de la cuadrícula.
  - [ ] Una sombra con desenfoque 20 px y opacidad 30 % se exporta a PNG con esos valores.
- **Mejora sobre Snagit:** los mismos ajustes avanzados, con ángulo, en ambas plataformas.
- **Fuentes:** research/02-editor.md §2.7 (Shadow)

### EDT-022 · Selección, tiradores y modificadores
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** convenciones de tiradores (blanco, verde, amarillo, azul); `Shift` para restringir; `Cmd`+arrastrar desde el centro (Mac); `Option`+arrastrar para dibujar encima (Mac); seleccionar la anotación al terminar de dibujarla
- **Comportamiento:**
  - **Tiradores** (diseño propio, distinguibles por forma y no solo por color):
    - cuadrados blancos con borde de acento: cambiar el tamaño;
    - círculo sobre un tallo, encima del objeto: girar;
    - rombos de acento: puntos de forma (punta y cola de una flecha, curvatura, cola de un globo, radio de las esquinas);
    - círculo con "+": añadir una cola a un globo.
  - Al pasar el puntero por un tirador, el puntero indica la acción.
  - **Modificadores** (iguales en ambas plataformas):
    - `Shift` al dibujar: cuadrado o círculo; líneas, flechas, trazos de Lápiz y de Borrador en horizontal, vertical o a 45°;
    - `Shift` al cambiar el tamaño: mantiene la proporción, también en objetos girados;
    - `Shift` al girar: pasos de 15°;
    - `Alt`/`Option` al cambiar el tamaño: desde el centro;
    - `Alt`/`Option` al empezar a dibujar sobre un objeto existente: dibuja uno nuevo en vez de seleccionar el de debajo;
    - `Alt`/`Option` al arrastrar un objeto con Mover: lo duplica (EDT-088).
  - Al hacer clic sobre un objeto con una herramienta de dibujo activa, el objeto se selecciona (salvo con `Alt`/`Option`). Con una herramienta de píxeles activa, el clic actúa sobre los píxeles.
  - **Seleccionar la anotación al terminar de dibujarla** (`editor.autoSelectAfterDraw`; por defecto: sí): el objeto recién dibujado queda seleccionado para ajustarlo.
  - Doble clic en un objeto con texto entra a editar el texto.
  - En M1 existen los tiradores de tamaño, giro y forma de Flecha, Texto, Forma y Desenfocar, y los modificadores `Shift` y `Alt`/`Option`.
- **Criterios de aceptación:**
  - [ ] Con Forma activa, arrastrar con `Shift` crea un cuadrado exacto (ancho = alto).
  - [ ] Cambiar el tamaño con `Alt`/`Option` de un rectángulo girado 30° mantiene fijo su centro.
  - [ ] Con la preferencia desactivada, tras dibujar una flecha no queda seleccionada y se puede dibujar otra justo encima sin `Alt`/`Option`.
  - [ ] Con Flecha activa, `Alt`/`Option`+arrastrar empezando sobre un rectángulo crea una flecha y no mueve el rectángulo.
- **Fuentes:** research/02-editor.md §2.7 (Handle conventions, Shift, Command-drag, Option-drag, auto-select), §12 (modifier keys), §11

### EDT-023 · Seleccionar todos los objetos de una herramienta
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** botón Select All (<herramienta>) en Tool Properties
- **Comportamiento:**
  - En la cabecera de Propiedades de cada herramienta de anotación hay un botón **Seleccionar todos (<objetos>)**, por ejemplo "Seleccionar todas las flechas" o "Seleccionar todos los pasos".
  - Selecciona todos los objetos de ese tipo en el documento, incluidos los que están dentro de grupos (decisión de diseño: se seleccionan los grupos que los contienen), para editarlos juntos.
  - También con clic derecho sobre un objeto > **Seleccionar todos los de este tipo**.
  - Existe para Flecha, Línea, Texto, Globo, Forma, Sello, Paso, Resaltador, Lápiz (trazos vectoriales), Lupa, Foco, Desenfocar y las formas de Simplificar.
  - Si no hay ninguno, el botón aparece desactivado y su tooltip dice "No hay <objetos> en el documento".
- **Criterios de aceptación:**
  - [ ] En un documento con 5 flechas y 3 textos, Seleccionar todas las flechas selecciona las 5 y ningún texto.
  - [ ] Con las 5 flechas seleccionadas, cambiar el grosor cambia las 5.
  - [ ] Los objetos bloqueados (EDT-085) no se seleccionan.
- **Fuentes:** research/02-editor.md §2.7 (Select All), §3.13; research/05-plataformas-versiones.md §1A (2026.1.0), §1B (2025.3.0)

---

## 3. Herramientas de anotación

Todas las herramientas de esta sección crean **objetos** editables: se pueden seleccionar, mover, girar y cambiar de tamaño (EDT-022), y cambiar de estilo (EDT-014, EDT-015) hasta que se aplanan (EDT-091).

### EDT-024 · Flecha
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Arrow (extremos, estilo de línea, grosor, tamaño de las puntas, curvas)
- **Comportamiento:**
  - Se arrastra desde la cola hasta la punta. `Shift` restringe a 0°, 45° y 90°. Un clic sin arrastrar no crea nada.
  - Tiradores: un rombo en cada extremo (mover la cola o la punta) y uno en el centro, que curva la flecha al arrastrarlo. Tras curvarla, aparecen dos rombos más para afinar la curva (curva Bézier cúbica). **Enderezar**, en Propiedades o en el menú contextual, vuelve a la recta. Decisión de diseño: se adopta el modelo de arrastrar el centro, sin casilla previa.
  - Propiedades:
    - **Color** (EDT-019);
    - **Grosor**: 1–100 px, con decimales (por defecto, el del estilo);
    - **Extremo inicial** y **Extremo final** (decisión de diseño): Ninguno, Punta rellena, Punta abierta, Punta fina, Punta cóncava, Círculo, Círculo vacío, Cuadrado, Rombo y Barra (T);
    - **Tamaño del extremo inicial** y **del final**: 50–400 % del tamaño proporcional al grosor;
    - **Tipo de línea**: Sólida, Discontinua, Punteada, Guion y punto;
    - **Opacidad**; **Sombra** (EDT-021).
  - Los extremos de la flecha se ajustan a los bordes y centros de otros objetos cuando el ajuste magnético está activo (EDT-084).
  - En M1: flecha recta con color, grosor, extremos, tipo de línea y opacidad. En M2: curvas, tamaño de los extremos y sombra.
- **Criterios de aceptación:**
  - [ ] Arrastrar de (100, 100) a (300, 100) crea una flecha horizontal con la punta en (300, 100).
  - [ ] Arrastrar el rombo central 50 px hacia arriba curva la flecha; Enderezar la vuelve recta con los mismos extremos.
  - [ ] Una flecha con Extremo inicial = Círculo y Extremo final = Barra se exporta a PNG con esos extremos, y el `.xshot` abierto en la otra plataforma los muestra igual.
  - [ ] Grosor 2,5 px y Tamaño del extremo final 200 % producen una punta el doble de grande que con 100 %.
- **Fuentes:** research/02-editor.md §3.1, §2.7, §10.2 (diferencias entre plataformas)

### EDT-025 · Línea
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Line (Windows); en Mac, líneas mediante la herramienta Arrow
- **Comportamiento:**
  - Herramienta propia en ambas plataformas (decisión de diseño). Se comporta como Flecha (EDT-024): arrastre, `Shift`, curvas, tiradores y propiedades, con dos diferencias:
    - los extremos son **Ninguno** por defecto;
    - sus estilos guardados son propios (líneas finas, gruesas, discontinuas y punteadas).
  - Tiene **Grosor**, que no consta en la herramienta de Snagit para Windows.
  - Una línea a la que se le pone un extremo distinto de Ninguno sigue siendo un objeto Línea (decisión de diseño). "Seleccionar todas las líneas" (EDT-023) la incluye.
- **Criterios de aceptación:**
  - [ ] Con Línea activa, arrastrar crea un segmento sin puntas con el grosor del estilo actual.
  - [ ] Seleccionar todas las líneas no selecciona ninguna flecha, y Seleccionar todas las flechas no selecciona ninguna línea.
  - [ ] Una línea punteada curvada se exporta igual a como se ve en el lienzo.
- **Mejora sobre Snagit:** herramienta Línea con grosor en ambas plataformas.
- **Fuentes:** research/02-editor.md §3.2, §3.1, §15 (punto 8)

### EDT-026 · Texto
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Text (relleno, contorno, sombra, fuente, tamaño, estilo, alineación, márgenes, interlineado, opacidad, Paste and Match Style)
- **Comportamiento:**
  - **Clic**: crea un cuadro de ancho automático que crece al escribir. **Arrastrar**: crea un cuadro de ancho fijo con ajuste de línea; el alto crece con el texto.
  - `Esc` o un clic fuera termina la edición. Un cuadro vacío al terminar se elimina. Doble clic sobre un texto vuelve a editarlo.
  - Propiedades:
    - **Fuente** (fuentes instaladas, con vista previa y búsqueda) y **Tamaño** (1–1000 pt, con decimales);
    - **Negrita**, **Cursiva**, **Subrayado** y **Tachado**;
    - **Relleno** (color de las letras) y **Contorno** (color y grosor del trazo alrededor de las letras, 0–20 px; 0 = sin contorno);
    - **Alineación horizontal** (izquierda, centro, derecha, justificado) y **vertical** (arriba, centro, abajo);
    - **Interlineado**: 0,8–3,0 (por defecto 1,2);
    - **Margen interior**: 0–100 px;
    - **Opacidad**; **Sombra** (EDT-021).
  - Mientras se edita, el formato (negrita, cursiva, subrayado, tachado, color y tamaño) se aplica a los caracteres seleccionados; si no hay caracteres seleccionados, a todo el cuadro (decisión de diseño). La fuente se aplica siempre a todo el cuadro.
  - Edición estándar: cortar, copiar, pegar, deshacer dentro del cuadro, selección con teclado y ratón, emoji e IME del sistema (acentos, chino, japonés).
  - **Pegar con el mismo estilo** (EDT-089) pega texto sin formato con el estilo del cuadro.
  - Al girar el cuadro, el texto gira con él. Al voltear la imagen, el texto no queda en espejo (EDT-073).
  - Para cambiar texto que ya forma parte de la captura se usa SMT- (Editar texto), no esta herramienta.
  - En M1: clic y arrastre, fuente, tamaño, negrita, cursiva, relleno, alineación y opacidad, aplicados a todo el cuadro. En M2: el resto.
- **Criterios de aceptación:**
  - [ ] Un clic, escribir "Hola" y `Esc` deja un objeto de texto con "Hola" en la posición del clic.
  - [ ] En un cuadro de 200 px de ancho, una frase larga se ajusta en varias líneas y el cuadro crece en alto.
  - [ ] Seleccionar una palabra mientras se edita y pulsar Negrita pone en negrita solo esa palabra.
  - [ ] Un texto con contorno 3 px blanco y relleno negro se lee sobre un fondo oscuro y se exporta igual.
- **Fuentes:** research/02-editor.md §3.3, §2.7 (decimal values)

### EDT-027 · Corrector ortográfico
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** corrector del editor (Windows, con idioma propio; Mac, con el corrector del sistema)
- **Comportamiento:**
  - Mientras se edita un Texto, un Globo o un texto de plantilla, las palabras desconocidas se subrayan con una línea ondulada.
  - Clic derecho sobre una palabra subrayada: hasta 5 sugerencias, **Ignorar**, **Ignorar todo** y **Añadir al diccionario**.
  - Preferencia **Corrector ortográfico** (`editor.spellcheck`; por defecto: sí) e **Idioma de corrección** (`editor.spellcheckLanguage`): **Igual que el sistema** (por defecto) o uno de los idiomas de corrección instalados en el sistema.
  - Ambas plataformas usan el servicio de corrección del sistema, con el mismo comportamiento (decisión de diseño). El diccionario personal de XcreenShot incluye de serie "XcreenShot" y los nombres de las funciones de la app.
  - El subrayado nunca aparece al exportar, copiar ni imprimir.
- **Criterios de aceptación:**
  - [ ] Escribir "ejenplo" con el idioma en español lo subraya y la primera sugerencia es "ejemplo".
  - [ ] Añadir una palabra al diccionario hace que deje de subrayarse en todos los documentos.
  - [ ] Con el corrector desactivado no se subraya ninguna palabra.
- **Fuentes:** research/02-editor.md §3.3 (Spell check), §11

### EDT-028 · Globo
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Callout (forma, colas múltiples, relleno y contorno, texto, estilo de la cola, márgenes, sombreado, radio de las esquinas)
- **Comportamiento:**
  - Se arrastra para dibujar el cuerpo y se escribe. Un globo es un cuadro de texto con fondo, contorno y una o varias **colas** que señalan algo.
  - **Formas** (decisión de diseño): Rectángulo, Rectángulo redondeado, Elipse, Nube, Pensamiento (nube con burbujas en lugar de cola), Explosión, Etiqueta, Flecha (cuerpo con forma de flecha que apunta en la dirección de la cola) y Solo texto (sin fondo ni contorno, con cola).
  - Tiradores:
    - tamaño del cuerpo (cuadrados);
    - giro (círculo);
    - punta de cada cola (rombo). Arrastrar una punta dentro del cuerpo elimina esa cola;
    - **+** en el contorno: añade otra cola, hasta 8;
    - radio de las esquinas (rombo en la esquina superior izquierda, solo en las formas rectangulares).
  - Arrastrar el cuerpo mueve el globo entero. Arrastrar el **ancla** (un punto pequeño en el centro del cuerpo, visible con el globo seleccionado) mueve solo el cuerpo y deja quietas las puntas de las colas (decisión de diseño).
  - Propiedades:
    - **Forma**; **Relleno** (color, degradado o transparente); **Sombreado** (degradado suave automático entre el relleno y un tono más oscuro; sí/no);
    - **Contorno**: color, grosor 0–20 px (0 = sin contorno) y tipo de línea (Sólida, Discontinua, Punteada);
    - **Radio de las esquinas** (0–200 px);
    - **Texto**: fuente, tamaño, color, negrita, cursiva, subrayado, tachado, alineación horizontal y vertical, interlineado y margen interior (0–100 px), como en EDT-026;
    - **Cola**: estilo (Recta, Curva o Flecha, que usa los extremos de EDT-024), **ancho de la base** (2–200 px) y color (por defecto, el del contorno);
    - **Opacidad**; **Sombra** (EDT-021).
  - **Ajustar al texto** (sí/no; por defecto: sí): el cuerpo crece en alto para que quepa todo el texto. Con "no", el texto que no cabe se recorta y aparece un aviso "Texto oculto" en el borde.
  - Al girar el globo, el texto gira con él. Al voltear la imagen, el texto no queda en espejo (EDT-073).
  - Al cambiar el tamaño de la imagen, los globos se escalan en proporción, incluido el texto (EDT-069).
- **Criterios de aceptación:**
  - [ ] Dibujar un globo, escribir dos líneas y añadir una segunda cola deja un globo con dos colas que se mueven juntas al arrastrar el cuerpo.
  - [ ] Arrastrar la punta de una cola dentro del cuerpo la elimina; deshacer la recupera.
  - [ ] Un globo Rectángulo redondeado con radio 24 px, cola curva y contorno punteado se ve igual en Windows y en macOS al abrir el mismo `.xshot`.
  - [ ] Con Ajustar al texto activado, escribir 10 líneas hace crecer el globo hasta mostrarlas todas.
- **Mejora sobre Snagit:** todas las variantes (colas curvas, ancho de cola, esquinas redondeadas, márgenes, sombreado, sin contorno) en ambas plataformas, sin pérdidas al cambiar de plataforma.
- **Fuentes:** research/02-editor.md §3.4, §10.2, §15 (punto 2)

### EDT-029 · Forma: rectángulo y elipse
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Shape (rectángulo y elipse; relleno, contorno, grosor, estilo de línea, opacidad, sombra)
- **Comportamiento:**
  - Se arrastra para dibujar. `Shift` da un cuadrado o un círculo; `Alt`/`Option` dibuja desde el centro.
  - Propiedades:
    - **Tipo de forma** (selector con las formas de EDT-029 y EDT-030);
    - **Relleno**: color, degradado (EDT-020) o Transparente (solo el marco);
    - **Contorno**: color y grosor 0–100 px (0 = sin contorno);
    - **Tipo de línea**: Sólida, Discontinua, Punteada, Guion y punto;
    - **Opacidad**; **Sombra** (EDT-021).
  - El contorno se dibuja hacia dentro del tamaño de la forma, de modo que el tamaño mostrado es el exterior (decisión de diseño).
  - En M1: rectángulo y elipse con relleno de color o transparente, contorno, grosor, tipo de línea y opacidad. En M2: degradado y sombra.
- **Criterios de aceptación:**
  - [ ] Arrastrar con `Shift` crea un círculo; la barra de estado muestra el mismo ancho y alto.
  - [ ] Un rectángulo con relleno Transparente deja ver la imagen a través del marco.
  - [ ] Un rectángulo de 200 × 100 con contorno de 10 px exporta un marco cuyo borde exterior mide 200 × 100.
- **Fuentes:** research/02-editor.md §3.5, §10.2 (líneas discontinuas entre plataformas)

### EDT-030 · Forma: rectángulo redondeado, polígono y otras formas
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Shape (polígono, esquinas redondeadas, radio en varios objetos a la vez)
- **Comportamiento:**
  - Formas adicionales (decisión de diseño): **Rectángulo redondeado**, **Triángulo**, **Rombo** y **Polígono**.
  - **Rectángulo redondeado**: el rombo de la esquina ajusta el radio (0 hasta la mitad del lado menor); también hay un campo **Radio** en Propiedades. Con varias formas seleccionadas, el campo cambia el radio de todas.
  - **Polígono**:
    - cada clic añade un vértice; `Shift` restringe el segmento a múltiplos de 45°;
    - doble clic, `Enter` o un clic sobre el primer vértice cierra la forma;
    - `Retroceso` quita el último vértice y `Esc` cancela;
    - después, cada vértice tiene un rombo para moverlo. `Alt`/`Option`+clic en un lado añade un vértice y `Alt`/`Option`+clic en un vértice lo quita (mínimo 3).
  - Mismas propiedades que EDT-029.
- **Criterios de aceptación:**
  - [ ] Cinco clics y `Enter` crean un pentágono cerrado; el mismo resultado se obtiene con doble clic en el quinto vértice.
  - [ ] Con tres rectángulos redondeados seleccionados, escribir Radio = 12 los deja a los tres con radio 12.
  - [ ] Mover un vértice de un polígono ya creado cambia su forma y se puede deshacer.
- **Fuentes:** research/02-editor.md §3.5, §8 (Multi-object transform)

### EDT-031 · Sello
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Stamp (categorías, búsqueda por palabra clave en varios idiomas, colocar, cambiar de tamaño, sombra, opacidad, conservar la proporción al cambiar de sello)
- **Comportamiento:**
  - La pestaña Estilos muestra la colección de sellos, con un selector de **categoría** y un campo de **búsqueda** que encuentra por nombre y palabras clave en inglés y en español.
  - Un clic en el lienzo coloca el sello elegido a su tamaño por defecto (el menor entre 96 px y un 25 % del lado menor del lienzo). Arrastrar lo coloca con el tamaño arrastrado.
  - Cambiar el tamaño mantiene la proporción; con `Shift` se puede deformar (decisión de diseño).
  - Con un sello seleccionado, un clic en otro sello de la colección lo **reemplaza**, con la misma posición y la misma altura (el ancho sigue la proporción del nuevo).
  - Propiedades: **Sombra** (por defecto, desactivada), **Opacidad** y **Color** para los sellos monocromos, que admiten recolorearse (decisión de diseño).
  - **Colección incorporada** (decisión de diseño, diseño propio, unos 150 sellos vectoriales): Indicadores (marca, cruz, aviso, información, pregunta, prohibido, estrella), Cursores y gestos, Teclas (Ctrl, Cmd, Shift, Enter, Esc, flechas…), Dispositivos, Interfaz (botones, interruptores, casillas), Etiquetas ("Nuevo", "Importante", "Borrador", "Aprobado"…), Flechas decorativas y Emoji (EDT-033).
  - Los sellos vectoriales se ven nítidos a cualquier tamaño. Al colocar un sello, su imagen se incrusta en el documento, de modo que el `.xshot` se abre igual en otro equipo aunque no tenga ese sello.
- **Criterios de aceptación:**
  - [ ] Buscar "warning" o "aviso" muestra el mismo sello de aviso.
  - [ ] Con un sello de 120 px de alto seleccionado, elegir otro sello lo sustituye en el mismo sitio con 120 px de alto.
  - [ ] Un `.xshot` con un sello propio abierto en un equipo que no tiene ese sello lo muestra igual.
- **Fuentes:** research/02-editor.md §3.6; research/05-plataformas-versiones.md §1A (2024.0.0)

### EDT-032 · Sellos propios y Guardar como sello
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Organize Stamps > Add Stamp (Windows), Import Stamps (Mac), File > Save As Stamp (Mac), Edit Search Terms (Mac)
- **Comportamiento:**
  - **Organizar sellos…** (menú ⋯ de la pestaña Estilos con Sello activo) abre un gestor donde se puede:
    - **Añadir sellos…** desde archivos PNG, JPG, GIF (primer fotograma), WebP o SVG, a la categoría **Mis sellos** o a una categoría nueva;
    - crear, renombrar y eliminar categorías propias;
    - renombrar sellos y **editar sus palabras clave**;
    - reordenar sellos y moverlos entre categorías propias;
    - eliminar sellos propios.
    Las categorías incorporadas no se pueden editar.
  - **Archivo > Guardar como sello…** convierte en sello la selección de píxeles activa o, si no la hay, el documento aplanado. Pide nombre, categoría y palabras clave. Se conserva la transparencia.
  - Arrastrar un archivo de imagen sobre la colección de sellos lo añade a Mis sellos.
- **Criterios de aceptación:**
  - [ ] Añadir un PNG con transparencia como sello y colocarlo conserva la transparencia.
  - [ ] Guardar como sello con una selección elíptica crea un sello circular con el exterior transparente.
  - [ ] Editar las palabras clave de un sello propio hace que la búsqueda lo encuentre por la palabra nueva.
- **Fuentes:** research/02-editor.md §3.6

### EDT-033 · Emoji
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** sellos de emoji; arrastrar emoji desde el Visor de caracteres de macOS al lienzo
- **Comportamiento:**
  - La categoría **Emoji** de la colección de sellos contiene un conjunto completo de emoji en color, con licencia libre y el mismo aspecto en ambas plataformas (decisión de diseño), buscables por su nombre en inglés y en español.
  - Arrastrar un emoji desde el selector de emoji del sistema (Windows: `Win`+`.`; macOS: Visor de caracteres) y soltarlo en el lienzo crea un sello de ese emoji, de 96 px.
  - Dentro de un Texto o un Globo, los emoji se escriben como texto, con la fuente de emoji del sistema.
- **Criterios de aceptación:**
  - [ ] Buscar "pulgar" muestra el emoji del pulgar arriba, y al colocarlo se ve igual en Windows y en macOS.
  - [ ] Arrastrar un emoji desde el selector del sistema al lienzo crea un objeto Sello de 96 px.
- **Fuentes:** research/02-editor.md §3.6 (Emoji); research/05-plataformas-versiones.md §1B (2024.2.3, 2025.3.0)

### EDT-034 · Paquetes de sellos `.xstamps`
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** descarga de conjuntos de sellos desde TechSmith Assets (sustituida por paquetes locales; SPEC §3.2)
- **Comportamiento:**
  - Un paquete `.xstamps` es un ZIP con imágenes (PNG o SVG) y un `pack.json` con nombre, versión, autor, licencia, categoría y, por cada imagen, nombre y palabras clave en inglés y español (SPEC §7).
  - **Importar** (Archivo > Importar > Sellos…, doble clic en el archivo, o arrastrarlo a la ventana): añade una categoría con el nombre del paquete. Si ya hay un paquete con el mismo nombre, se ofrece **Actualizar** (si la versión es mayor) o **Conservar ambos**.
  - **Exportar categoría…** (en Organizar sellos) crea un `.xstamps` a partir de una categoría propia, pidiendo nombre, autor y licencia.
  - **Quitar paquete** elimina la categoría importada. Los sellos ya colocados en documentos no cambian, porque están incrustados (EDT-031).
  - La importación rechaza paquetes de más de 100 MB o con archivos que no sean imágenes, con un mensaje que dice el motivo.
- **Criterios de aceptación:**
  - [ ] Importar un `.xstamps` con 20 PNG crea una categoría con 20 sellos, buscables por las palabras clave del `pack.json`.
  - [ ] Exportar una categoría propia en Windows e importarla en macOS deja los mismos sellos con los mismos nombres y palabras clave.
  - [ ] Un paquete que contiene un `.exe` se rechaza y no se añade ningún sello.
- **Fuentes:** research/02-editor.md §3.6 (Download More), §1 (Assets entry)

### EDT-035 · Paso
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Step (secuencia, Edit Value, Restart Sequence, tipo, forma, relleno transparente, texto derecho al girar)
- **Comportamiento:**
  - Cada clic coloca un marcador con el **valor siguiente** de la secuencia. Arrastrar coloca el marcador con el tamaño arrastrado.
  - **Tipo** de secuencia (decisión de diseño): 1, 2, 3… (hasta 9999); A, B, C…; a, b, c…; I, II, III…; i, ii, iii…. Después de Z siguen AA, AB… (y en minúsculas igual).
  - **Valor siguiente:** el último valor colocado más uno. Al abrir un documento se calcula como el mayor valor de ese tipo presente más uno.
  - **Editar valor:** doble clic, o clic derecho > **Editar valor…**, permite escribir cualquier texto de hasta 6 caracteres. Si el valor escrito pertenece a la secuencia (por ejemplo, "7"), el siguiente clic sigue desde él (8).
  - **Reiniciar secuencia** (botón en Propiedades): el siguiente clic vuelve al primer valor del tipo.
  - **Renumerar** (botón en Propiedades y en el menú contextual): vuelve a numerar todos los pasos del documento del mismo tipo, por orden de creación, sin huecos (decisión de diseño).
  - Propiedades:
    - **Forma** (decisión de diseño): Círculo, Cuadrado, Cuadrado redondeado, Rombo, Marcador (gota) y Solo texto;
    - **Relleno** (color, degradado o transparente);
    - **Color del texto**; **Contorno** (color y grosor 0–10 px);
    - **Tamaño**: 16–256 px;
    - **Tipo**; **Opacidad**; **Sombra** (EDT-021).
  - El texto del paso queda siempre derecho, aunque se gire el marcador.
  - Cambiar el estilo o el tamaño de un paso cambia también el estilo actual, de modo que los pasos siguientes salen iguales (EDT-014).
- **Criterios de aceptación:**
  - [ ] Tres clics con el tipo 1, 2, 3 colocan 1, 2 y 3. Reiniciar secuencia y hacer clic coloca otro 1.
  - [ ] Editar el valor de un paso a "B2" muestra "B2" en el marcador.
  - [ ] Con los pasos 1, 2, 4 y 5 (el 3 eliminado), Renumerar los deja en 1, 2, 3 y 4.
  - [ ] Un paso girado 90° muestra su número sin girar.
- **Fuentes:** research/02-editor.md §3.7, §15 (punto 2)

### EDT-036 · Resaltador
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Highlighter (color, opacidad, forma en Mac)
- **Comportamiento:**
  - Se arrastra para crear una zona resaltada translúcida.
  - El resaltado se mezcla con lo que hay debajo como lo haría un rotulador fluorescente: el texto oscuro sigue legible y no se aclara (decisión de diseño).
  - Propiedades:
    - **Color** (por defecto, amarillo del tema);
    - **Opacidad** (por defecto, 50 %);
    - **Forma**: Rectángulo, Rectángulo redondeado, Óvalo y Polígono (este último como en EDT-030).
  - **Aplicar solo al texto** limita el resaltado a las líneas de texto detectadas dentro de la zona; lo define SMT- (Aplicar solo al texto).
- **Criterios de aceptación:**
  - [ ] Un resaltado amarillo sobre texto negro deja el texto negro (no gris) y el fondo blanco amarillo.
  - [ ] Cambiar la forma de un resaltado ya dibujado de Rectángulo a Óvalo conserva su posición y tamaño.
- **Mejora sobre Snagit:** elección de forma en ambas plataformas.
- **Fuentes:** research/02-editor.md §3.8, §5.6; research/05-plataformas-versiones.md §3 (Editor)

### EDT-037 · Lápiz
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Pen (trazo vectorial o de píxeles, punta, suavizado, curvas suaves, `Shift` para línea recta)
- **Comportamiento:**
  - Dibujo a mano alzada. `Shift` traza una recta horizontal, vertical o a 45° desde el punto de inicio.
  - **Crear como trazo editable** (sí/no; por defecto: sí):
    - Sí: cada trazo es un objeto que se puede seleccionar, mover y cambiar de estilo.
    - No: el trazo se pinta directamente en los píxeles de la imagen (o de la imagen flotante seleccionada) y no se puede editar después.
  - Propiedades:
    - **Color**; **Grosor** (1–100 px); **Opacidad**; **Sombra** (solo en trazos editables);
    - **Punta**: Círculo, Cuadrado, Barra inclinada (caligráfica) y Barra horizontal (decisión de diseño);
    - **Suavizado de bordes** (sí/no; por defecto: sí);
    - **Suavizar trazo**: 0–100 % (por defecto 30 %), que reduce el temblor de la mano.
  - Con una tableta digitalizadora, la presión no cambia el grosor en v1 (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con trazo editable, el trazo se puede seleccionar y cambiar a rojo; sin trazo editable, el clic con Mover no selecciona nada.
  - [ ] Un trazo con punta Cuadrado y Barra inclinada se ve igual en Windows y en macOS al abrir el mismo `.xshot`.
  - [ ] Con Suavizar trazo al 100 %, un trazo tembloroso se ve claramente más liso que al 0 %.
- **Mejora sobre Snagit:** todas las puntas en ambas plataformas.
- **Fuentes:** research/02-editor.md §3.9, §10.2

### EDT-038 · Lupa
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Magnify (lente, ampliación desplazada, forma, porcentaje hasta 800 %, contorno)
- **Comportamiento:**
  - Se arrastra sobre una zona: aparece una **lente** que muestra esa zona ampliada.
  - **Ampliación desplazada:** la lente se puede arrastrar a otro sitio. La zona de origen queda marcada con un contorno fino y unida a la lente con dos líneas de conexión (opción **Conectores**: sí/no; por defecto: sí). La zona de origen tiene su propio tirador para moverla sin mover la lente.
  - La lente muestra la imagen base y los objetos que están por debajo de la lente en el orden de apilado.
  - Propiedades:
    - **Forma**: Círculo, Cuadrado, Rectángulo, Rectángulo redondeado;
    - **Aumento**: 110–800 % (por defecto 200 %);
    - **Contorno**: color y grosor 0–20 px;
    - **Opacidad**; **Sombra** (EDT-021).
  - Una lente sin contorno muestra un contorno provisional mientras se mueve.
  - Se ajusta a otros objetos cuando el ajuste magnético está activo (EDT-084).
  - No confundir con el zoom de la vista (EDT-007): la lente forma parte de la imagen.
- **Criterios de aceptación:**
  - [ ] Una lente circular al 300 % sobre una zona de 50 × 50 px muestra esa zona a 150 × 150 px.
  - [ ] Al arrastrar la lente a otro sitio, la zona de origen sigue visible con su contorno y los conectores.
  - [ ] Cambiar el aumento de 200 % a 400 % actualiza la lente al momento.
- **Fuentes:** research/02-editor.md §3.10

### EDT-039 · Foco
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Spotlight (Mac): formas, color y opacidad del exterior, desenfoque del exterior
- **Comportamiento:**
  - Se dibuja una zona; todo lo que queda **fuera** se oscurece, se desenfoca o ambas cosas.
  - Formas: Rectángulo, Rectángulo redondeado, Óvalo y Polígono (como EDT-030).
  - Propiedades: **Color** del exterior (por defecto, negro), **Opacidad** del exterior (0–100 %; por defecto 60 %), **Desenfoque** del exterior (0–100; por defecto 0) y **Borde suave** de la zona (0–50 px).
  - Si hay varios objetos Foco, el exterior es común: queda iluminada la unión de todas las zonas y el resto se oscurece una sola vez, con los ajustes del último Foco modificado, que pasan a todos (decisión de diseño).
  - Se mueve, gira y cambia de tamaño con tiradores.
  - **Aplicar solo al texto**: lo define SMT- (Aplicar solo al texto).
- **Criterios de aceptación:**
  - [ ] Un Foco rectangular con opacidad 60 % oscurece todo menos su interior, que se ve sin cambios.
  - [ ] Dos Focos en zonas distintas dejan iluminadas las dos zonas y oscurecen el resto una sola vez (no hay doble oscurecimiento).
  - [ ] Desenfoque 50 con opacidad 0 % desenfoca el exterior sin oscurecerlo.
- **Mejora sobre Snagit:** herramienta disponible en ambas plataformas.
- **Fuentes:** research/02-editor.md §3.11, §5.6

### EDT-040 · Desenfocar
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Blur (Blur/Pixelate/Solid shapes, color, forma, intensidad, radio de esquinas, Smart Redact, notas de seguridad)
- **Comportamiento:**
  - Se arrastra sobre la zona que se quiere ocultar.
  - **Tipo:** **Desenfoque**, **Pixelado** o **Sólido** (tapa la zona con un color opaco).
  - **Color**: solo con el tipo Sólido (por defecto, negro).
  - **Intensidad**: 1–100 (por defecto 60); en Pixelado, fija el tamaño del bloque.
  - **Forma**: Rectángulo, Rectángulo redondeado (con radio), Elipse y Polígono.
  - Botón **Redacción automática**: abre SMT- (Redacción automática). **Aplicar solo al texto**: SMT- (Aplicar solo al texto).
  - El desenfoque es un objeto y no cambia los píxeles hasta que se aplana: se puede mover, cambiar de tamaño, de tipo o borrar.
  - **Seguridad** (textos de ayuda propios en el panel):
    - El tooltip del tipo Sólido lo recomienda para datos confidenciales, porque el desenfoque y el pixelado a veces se pueden revertir con herramientas externas.
    - Toda exportación a formatos planos (PNG, JPG, PDF, etc.), Copiar todo y los destinos de compartir aplanan siempre: el archivo resultante nunca contiene los píxeles originales de la zona.
    - Un `.xshot` sí conserva los píxeles originales bajo el desenfoque (y los recortados, EDT-050). Al compartir o exportar un `.xshot` que contiene desenfoques sin aplanar, aparece el aviso "Este archivo contiene información oculta recuperable", con **Aplanar desenfoques y continuar**, **Continuar así** y **Cancelar** (decisión de diseño).
    - Aplanar un desenfoque descarta para siempre los píxeles originales de su zona (EDT-091).
  - En M1: los tres tipos, color, intensidad y las formas rectángulo y elipse. En M2: polígono, rectángulo redondeado y el aviso al compartir un `.xshot`.
- **Criterios de aceptación:**
  - [ ] Un desenfoque Sólido negro sobre un texto, exportado a PNG, produce una zona completamente negra cuyos píxeles no contienen el texto.
  - [ ] Antes de aplanar, mover el desenfoque deja ver el texto en su posición original; tras aplanar, ya no se puede recuperar ni con Desaplanar.
  - [ ] Pixelado con intensidad 100 produce bloques más grandes que con intensidad 20.
  - [ ] Compartir un `.xshot` con un desenfoque sin aplanar muestra el aviso, y **Aplanar desenfoques y continuar** envía un archivo sin los píxeles originales.
- **Fuentes:** research/02-editor.md §3.12, §5.3, §5.6, §10.2

### EDT-041 · Simplificar
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Simplify (paletas, objetos de estilo, propiedades por objeto, Dibujar/Mover, Lock Objects, Show Original, Select All)
- **Comportamiento:**
  - Sirve para sustituir texto y detalles de la interfaz por formas planas: para imágenes que no envejecen cuando cambia la interfaz, para traducir sin rehacer capturas o para centrar la atención.
  - **Modo**: **Dibujar** (cada arrastre crea un objeto simplificado) o **Mover** (selecciona varios objetos simplificados y los mueve). `Shift` cambia de modo mientras se mantiene pulsado (decisión de diseño).
  - **Objetos de estilo** (pestaña Estilos): Bloque, Bloque redondeado, Línea de texto (barra con la altura de una línea), Párrafo (varias barras), Círculo, Botón e Ícono genérico (decisión de diseño).
  - **Paleta**: los colores con los que se generan los objetos. Selector **Paleta ▾** con las paletas guardadas; cada color se puede editar; ⋯ > **Guardar como paleta nueva** o **Reemplazar esta paleta**.
  - Propiedades por objeto: **Relleno**, **Forma**, **Fondo** (para líneas de texto), **Altura del texto**, **Interlineado** y, con varios objetos seleccionados, **Alinear** a la izquierda, al centro o a la derecha.
  - **Mostrar original** (sí/no): muestra la imagen original bajo los objetos, con un deslizador de **Opacidad** (0–100 %), para colocar los objetos encima de lo que sustituyen. Al desactivarlo, los objetos se ven sobre su fondo simplificado.
  - **Bloquear al salir** (sí/no; por defecto: no): al cambiar a otra herramienta, los objetos simplificados quedan bloqueados (EDT-085), para no moverlos sin querer.
  - Admite transparencia en los colores de relleno.
  - **Simplificar automáticamente** y su deslizador **Detalle** los define SMT- (Simplificar automáticamente).
  - "Seleccionar todas las formas simplificadas" está en EDT-023.
- **Criterios de aceptación:**
  - [ ] Con Mostrar original al 40 %, se ve la captura atenuada bajo los objetos; al desactivarlo, la captura no se ve.
  - [ ] Guardar una paleta como nueva la añade al selector, y al elegirla los objetos nuevos usan sus colores.
  - [ ] Con tres líneas de texto seleccionadas, Alinear a la izquierda iguala su borde izquierdo.
  - [ ] Con Bloquear al salir activado, tras cambiar a Flecha, un clic sobre un objeto simplificado no lo selecciona.
- **Mejora sobre Snagit:** el bloqueo funciona igual en ambas plataformas y se conserva en el archivo.
- **Fuentes:** research/02-editor.md §3.13, §5.5; research/05-plataformas-versiones.md §2 (Simplify), §3 (Lock Objects in Simplify)

---

## 4. Herramientas de píxeles

Estas herramientas actúan sobre los píxeles de la imagen base o de la imagen flotante seleccionada, no sobre los objetos de anotación (salvo Mover).

### EDT-042 · Mover
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Move (seleccionar, mover, eliminar, doble clic para editar, duplicar con `Ctrl`+clic u `Option`+arrastrar)
- **Comportamiento:**
  - Herramienta de selección por defecto. Clic: selecciona un objeto. Arrastrar un objeto: lo mueve. Arrastrar sobre una zona vacía: selección por recuadro (EDT-079). `Supr`/`Retroceso`: elimina lo seleccionado. Doble clic: edita el texto del objeto, o muestra su barra de edición rápida (EDT-006) si no tiene texto.
  - `Alt`/`Option`+arrastrar duplica el objeto (EDT-088).
  - Una pulsación corta de `V` activa Mover (EDT-115). Con otra herramienta activa, mantener `V` pulsada más de 0,3 s activa Mover temporalmente, y al soltarla vuelve la herramienta anterior (decisión de diseño).
  - El botón **Separar elementos** de esta herramienta lo define SMT- (Separar elementos).
  - Un clic en la imagen base sin objetos debajo no hace nada (no se puede mover la imagen base; decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con Mover, arrastrar una flecha 100 px a la derecha la mueve 100 px y se puede deshacer.
  - [ ] Doble clic en un globo entra a editar su texto, con el cursor al final.
  - [ ] Con Flecha activa, mantener `V`, mover un texto y soltar `V` deja Flecha activa de nuevo.
- **Fuentes:** research/02-editor.md §4.1, §8 (Duplicate object)

### EDT-043 · Selección
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Selection (rectángulo, elipse, polígono, mano alzada; Snap to Object)
- **Comportamiento:**
  - Selecciona píxeles de la imagen base o de una imagen flotante. La selección se muestra con un contorno animado.
  - **Forma**:
    - **Rectángulo** (`Shift` = cuadrado);
    - **Elipse** (`Shift` = círculo);
    - **Polígono**: clic por vértice; doble clic o `Enter` para cerrar;
    - **Mano alzada**: se arrastra un lazo, que se cierra solo al soltar.
  - **Combinar**: con una selección activa, `Shift`+arrastrar añade a la selección y `Alt`/`Option`+arrastrar resta de ella (decisión de diseño).
  - **Ajustar al objeto** (sí/no): al soltar, la selección se ciñe a los bordes del elemento visual (texto, ícono, botón) que hay dentro del recuadro. El deslizador **Nivel** elige entre el elemento más pequeño y el grupo más grande que lo contiene.
  - **Posición y tamaño numéricos**: con una selección rectangular o elíptica, la sección Posición y tamaño (EDT-086) muestra X, Y, An y Al editables.
  - **Bordes**: **Suavizado** (sí/no; por defecto: sí) y **Calado** (0–50 px; por defecto 0) (decisión de diseño).
  - Clic fuera de la selección o `Ctrl`/`Cmd`+`D` la quita. Una selección sigue activa tras cambiar el tamaño de la imagen, ajustada a la nueva escala.
- **Criterios de aceptación:**
  - [ ] Una selección de elipse copiada y pegada en otra app da una imagen elíptica con las esquinas transparentes, no un rectángulo.
  - [ ] Con Ajustar al objeto, un recuadro holgado alrededor de un botón se ciñe al borde del botón.
  - [ ] Escribir An = 300 y Al = 200 en Posición y tamaño deja una selección de exactamente 300 × 200 px.
  - [ ] `Shift`+arrastrar otro rectángulo que se solapa con la selección produce una selección con la unión de ambos.
- **Mejora sobre Snagit:** posición y tamaño numéricos de la selección; añadir y restar.
- **Fuentes:** research/02-editor.md §4.2, §8 (Exact position and size); research/05-plataformas-versiones.md §3 (Fixed-region capture: no numeric size in the Editor)

### EDT-044 · Acciones sobre la selección
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** mover, eliminar, copiar, cortar, pegar, Edit Text, Grab Text, efectos solo en la selección (Windows), Invert Selection, Print Selection (Mac), Save Selection As (Windows), recortar a círculo con elipse
- **Comportamiento:**
  - Con una selección de píxeles activa:
    - **Arrastrar** el interior mueve los píxeles seleccionados. El hueco que dejan se rellena según EDT-045.
    - **`Supr`/`Retroceso`** borra los píxeles y deja el hueco rellenado según EDT-045.
    - **Copiar** (`Ctrl`/`Cmd`+`C`) copia los píxeles seleccionados (con transparencia fuera de la forma). **Cortar** copia y borra.
    - **Pegar** crea una imagen flotante (EDT-078).
    - **Invertir selección** (`Ctrl`/`Cmd`+`Shift`+`I`).
    - **Recortar a la selección**: deja solo el rectángulo que contiene la selección; con una selección no rectangular, lo de fuera de la forma queda transparente (así se recorta en círculo).
    - **Convertir en imagen flotante** (`Ctrl`/`Cmd`+`Shift`+`J`): separa la selección como objeto móvil y deja el hueco según EDT-045 (decisión de diseño).
    - **Aplicar efecto a la selección**: los efectos del panel Efectos (EDT-053) se aplican solo dentro de la selección.
    - **Guardar selección como…**: exporta la selección a un archivo (EDT-110, EDT-111).
    - **Imprimir selección…** (EDT-114).
    - **Guardar como sello…** (EDT-032).
    - **Extraer texto** y **Editar texto**: los define `SMT-`.
  - Todas están en el menú contextual de la selección y en el menú Edición o Imagen.
- **Criterios de aceptación:**
  - [ ] Con una selección rectangular, Cortar y Pegar crea una imagen flotante idéntica y deja el hueco según el relleno elegido.
  - [ ] Invertir selección sobre un círculo central y pulsar `Supr` deja solo el círculo; con relleno Transparente, el exterior queda transparente.
  - [ ] Aplicar Escala de grises a una selección cambia solo los píxeles de dentro.
  - [ ] Guardar selección como PNG produce un archivo del tamaño del rectángulo que contiene la selección.
- **Mejora sobre Snagit:** todas las acciones en ambas plataformas.
- **Fuentes:** research/02-editor.md §4.2, §6 (General behavior: selection only)

### EDT-045 · Relleno del hueco
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Background Fill: Transparent o Auto-Fill (con Optimize for Speed / Photo Quality en Windows)
- **Comportamiento:**
  - Opción **Rellenar el hueco con** de Selección y Varita mágica, que decide qué queda donde se movieron o borraron píxeles:
    - **Transparente**: deja ver el color del lienzo (EDT-074);
    - **Color del lienzo**: rellena con el color sólido del lienzo, aunque luego cambie (decisión de diseño);
    - **Relleno inteligente** (por defecto): reconstruye el fondo a partir de los píxeles de alrededor, para que no se note el hueco.
  - Con Relleno inteligente, **Calidad**: **Rápida** (por defecto; para interfaces y fondos planos) o **Fotográfica** (más lenta; para fotos y texturas).
  - El relleno inteligente se calcula en el dispositivo. Si tarda más de 300 ms, se muestra un indicador de progreso y se puede cancelar con `Esc`.
- **Criterios de aceptación:**
  - [ ] Borrar un botón sobre un fondo liso con Relleno inteligente deja un fondo liso sin marcas visibles.
  - [ ] Con Transparente y lienzo transparente, el hueco exporta como transparente en PNG.
  - [ ] Con Fotográfica sobre una foto de 12 MP, el progreso aparece y `Esc` cancela sin cambiar la imagen.
- **Mejora sobre Snagit:** las dos calidades en ambas plataformas.
- **Fuentes:** research/02-editor.md §4.2, §4.3; research/05-plataformas-versiones.md §3 (Auto-Fill optimize)

### EDT-046 · Varita mágica
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Magic Wand (tolerancia, Global Color Selection, mismas opciones de relleno, eliminar, cortar, copiar, mover)
- **Comportamiento:**
  - Un clic selecciona la zona contigua de color parecido al del píxel pulsado.
  - **Tolerancia**: 0–100 % (por defecto 15 %); 0 = solo el color exacto.
  - **Global**: selecciona todos los píxeles de ese color en la imagen, no solo los contiguos.
  - **Combinar**: `Shift`+clic añade y `Alt`/`Option`+clic resta, como en EDT-043.
  - El resultado es una selección normal: admite todas las acciones de EDT-044 y el relleno de EDT-045.
  - Funciona sobre la imagen base o sobre la imagen flotante seleccionada.
- **Criterios de aceptación:**
  - [ ] Un clic sobre un fondo blanco liso con tolerancia 0 selecciona todo el fondo contiguo, sin el texto.
  - [ ] Con Global, un clic sobre un píxel rojo selecciona también otras zonas rojas no conectadas.
  - [ ] Varita mágica en el fondo, Invertir selección y Copiar copian el objeto sin fondo.
- **Fuentes:** research/02-editor.md §4.3, §5.4 (métodos manuales para quitar el fondo)

### EDT-047 · Relleno
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Fill (color, transparente, degradado en Mac, tolerancia, opacidad, Global Fill en Mac, imágenes flotantes)
- **Comportamiento:**
  - Un clic rellena el píxel pulsado y los píxeles contiguos de color parecido.
  - Propiedades:
    - **Color**: color sólido, degradado (EDT-020) o **Transparente** (la forma principal de dejar transparente un fondo liso);
    - **Tolerancia**: 0–100 % (por defecto 10 %); 0 = solo el color exacto;
    - **Opacidad**;
    - **Global**: rellena todos los píxeles de ese color en la imagen, contiguos o no.
  - Actúa sobre la imagen base o, si el clic cae sobre una imagen flotante, sobre esa imagen. Un clic sobre un objeto de anotación no rellena nada debajo de él.
- **Criterios de aceptación:**
  - [ ] Relleno Transparente sobre un fondo blanco liso deja el fondo transparente al exportar a PNG.
  - [ ] Con Global, rellenar de verde un píxel azul cambia a verde todas las zonas azules de la imagen.
  - [ ] Un clic con Relleno sobre una flecha no cambia la imagen base.
  - [ ] Un clic sobre una imagen flotante rellena solo esa imagen.
- **Mejora sobre Snagit:** Global y degradado en ambas plataformas.
- **Fuentes:** research/02-editor.md §4.4; research/05-plataformas-versiones.md §1A (2026.2.0)

### EDT-048 · Borrador
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Eraser (grosor, opacidad, suavizado, forma redonda o cuadrada, `[` y `]`)
- **Comportamiento:**
  - Al arrastrar, borra píxeles de la imagen base (o de la imagen flotante seleccionada) y deja ver el **color del lienzo** (damero si es transparente). No afecta a los objetos de anotación.
  - `Shift` borra en línea recta.
  - Propiedades: **Grosor** (1–200 px), **Forma** (Redonda o Cuadrada), **Opacidad** (borrado parcial) y **Suavizado de bordes**.
  - `[` y `]` reducen y aumentan el grosor en pasos de 10 % (mínimo 1 px).
  - Mientras está activo, el puntero muestra el contorno del borrador a su tamaño real en el zoom actual.
- **Criterios de aceptación:**
  - [ ] Borrar sobre una captura con el lienzo blanco deja blanco; con el lienzo transparente, deja transparente.
  - [ ] Borrar sobre una flecha no la cambia; la imagen de debajo sí se borra.
  - [ ] `]` tres veces sobre un grosor de 20 px lo deja en unos 27 px, y el contorno del puntero crece.
- **Mejora sobre Snagit:** todas las propiedades en ambas plataformas.
- **Fuentes:** research/02-editor.md §4.5

### EDT-049 · Recortar
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Crop (tiradores, mover el recuadro, aplicar)
- **Comportamiento:**
  - Al activar Recortar aparece un recuadro sobre toda la imagen, con tiradores en las esquinas y los lados. También se puede arrastrar un recuadro nuevo.
  - Arrastrar el interior mueve el recuadro. Fuera del recuadro, la imagen se oscurece.
  - **Proporción** (decisión de diseño): Libre (por defecto), Original, 1:1, 4:3, 3:2, 16:9, 9:16 y Personalizada. `Shift` al arrastrar un tirador fija la proporción actual.
  - **Posición y tamaño numéricos**: X, Y, An y Al en Propiedades (EDT-086).
  - **Aplicar**: botón **Recortar**, `Enter` o doble clic dentro. **Cancelar**: `Esc` o cambiar de herramienta.
  - Los objetos que quedan total o parcialmente fuera del nuevo lienzo **no se borran**: siguen en el documento, recortados a la vista. La barra de estado avisa de cuántos hay fuera y permite seleccionarlos (EDT-011, decisión de diseño).
  - En M1: recuadro, tiradores, mover, aplicar y cancelar, y la proporción Libre. En M2: el resto de proporciones y los campos numéricos.
- **Criterios de aceptación:**
  - [ ] Arrastrar los tiradores y pulsar `Enter` deja la imagen del tamaño del recuadro; la barra de estado lo confirma.
  - [ ] Con la proporción 16:9, el recuadro mantiene 16:9 al arrastrar cualquier tirador.
  - [ ] Una flecha que quedaba medio fuera sigue siendo editable tras recortar, y la barra de estado indica "1 objeto fuera del lienzo".
  - [ ] `Esc` durante el ajuste deja la imagen como estaba.
- **Fuentes:** research/02-editor.md §4.6, §15 (punto 7)

### EDT-050 · Recorte no destructivo
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** el `.snagx` conserva lo recortado; Revert to Original; preferencia Delete cropped content (Windows)
- **Comportamiento:**
  - En un `.xshot`, un recorte no borra píxeles: el archivo conserva la imagen original sin recortar (`base.png`) y el historial de recortes (SPEC §7).
  - **Ampliar el recorte**: volver a activar Recortar y arrastrar un tirador hacia fuera recupera la imagen original que había ahí (decisión de diseño: se puede ampliar hasta los bordes originales sin volver al original).
  - **Volver al original** (EDT-076) deshace todos los recortes.
  - Preferencia **Eliminar el contenido recortado** (`editor.deleteCroppedContent`; por defecto: no): con "sí", al guardar se descartan los píxeles de fuera del recorte. Útil para que un `.xshot` compartido no contenga partes ocultas.
  - Las exportaciones a formatos planos nunca contienen lo recortado.
- **Criterios de aceptación:**
  - [ ] Recortar, guardar, cerrar, abrir de nuevo y arrastrar un tirador hacia fuera recupera la zona recortada.
  - [ ] Con Eliminar el contenido recortado activado, el `base.png` del `.xshot` guardado mide lo mismo que el recorte.
  - [ ] Un PNG exportado tras recortar mide exactamente el tamaño del recorte.
- **Fuentes:** research/02-editor.md §4.6, §10.2, §11

### EDT-051 · Recortar al lienzo y recortar bordes
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Crop to Canvas (Mac), Trim (Mac), Auto Trim (Windows)
- **Comportamiento:**
  - **Recortar al lienzo**: elimina las partes de los objetos que sobresalen del lienzo y aplana esos objetos solo si es imprescindible para recortarlos (las imágenes flotantes se recortan sin aplanar; las anotaciones vectoriales quedan como están, con aviso) (decisión de diseño).
  - **Ajustar el lienzo al contenido**: amplía o reduce el lienzo hasta el rectángulo que contiene la imagen base y todos los objetos, de modo que nada quede fuera.
  - **Recortar bordes**: quita los bordes de color uniforme o transparentes alrededor de la imagen (por ejemplo, el margen blanco de una ventana). Tolerancia ajustable en el diálogo (0–20 %; por defecto 2 %). Se puede añadir a la barra (EDT-004) y aplicar como efecto (EDT-068).
- **Criterios de aceptación:**
  - [ ] Recortar bordes en una captura con 40 px de blanco alrededor de un contenido deja solo el contenido.
  - [ ] Con una imagen flotante que sobresale 100 px por la derecha, Ajustar el lienzo al contenido amplía el lienzo 100 px y la deja dentro.
  - [ ] Recortar bordes sobre una imagen sin bordes uniformes no la cambia y lo indica.
- **Mejora sobre Snagit:** las tres operaciones en ambas plataformas.
- **Fuentes:** research/02-editor.md §4.6, §7 (Trim / Auto Trim / Crop to Canvas), §12

### EDT-052 · Quitar franja y añadir espacio
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Cut Out (franjas horizontales y verticales, estilos de borde, reducir un hueco existente), Add Horizontal/Vertical Space (Mac), tamaño en píxeles al arrastrar (Mac)
- **Comportamiento:**
  - **Quitar franja**: se arrastra sobre la imagen para marcar una franja de **todo el ancho** (horizontal) o de **todo el alto** (vertical). Al soltar, la franja se elimina y las dos partes se juntan.
  - **Dirección**: Horizontal o Vertical (en Propiedades; `Shift` cambia mientras se arrastra).
  - **Estilo de la unión** (decisión de diseño):
    - **Recta**: une las dos partes sin separación;
    - **Rasgada**, **Ondulada** y **Zigzag**: dejan un hueco del color del lienzo de 8–60 px (por defecto 16 px) con ese borde.
    Una mejora: el tamaño del hueco es editable.
  - **Añadir espacio**: la misma herramienta, en modo **Añadir**, inserta una franja del color del lienzo del tamaño arrastrado, en horizontal o en vertical.
  - Mientras se arrastra, una etiqueta muestra el tamaño de la franja en píxeles.
  - **Objetos:** los que están por completo a un lado de la franja se desplazan con su parte; los que la cruzan se mantienen y se avisa de ello (decisión de diseño: no hay que aplanar antes, mejora sobre Snagit).
  - Los efectos del documento se mantienen y se recalculan sobre el resultado.
  - Volver al original (EDT-076) deshace también las franjas.
- **Criterios de aceptación:**
  - [ ] Quitar una franja horizontal de 200 px en una imagen de 1000 px de alto deja 800 px de alto.
  - [ ] Con el estilo Ondulada y hueco de 20 px, la imagen resultante tiene una separación ondulada de 20 px.
  - [ ] Añadir espacio vertical de 150 px ensancha la imagen 150 px y desplaza a la derecha los objetos que había a la derecha.
  - [ ] Una flecha por encima de la franja quitada mantiene su posición; una por debajo sube lo que medía la franja.
- **Mejora sobre Snagit:** sin aplanar los objetos, tamaño de hueco editable, añadir espacio en ambas plataformas.
- **Fuentes:** research/02-editor.md §4.7, §15 (punto 1)

---

## 5. Efectos

### EDT-053 · Panel Efectos
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** panel Effects (Windows: elegir, ajustar, Apply, quitar con Deshacer; Mac: activar con un clic, engranaje, "No Effects"); efectos en una selección (Windows); efectos en imágenes flotantes
- **Comportamiento:**
  - La pestaña **Efectos** del panel lateral (y el menú Imagen > Efectos ▸) lista los efectos de EDT-055 a EDT-068, cada uno con un interruptor y una flecha para desplegar sus opciones.
  - **Modelo** (decisión de diseño; se adopta el de efectos activables y editables):
    - activar un efecto lo aplica al momento con sus últimos valores; desplegarlo permite ajustarlo con vista previa en vivo;
    - los efectos activos forman una **pila** ordenada (por defecto, en el orden de la lista; se puede reordenar arrastrando). Cada uno se puede ajustar, desactivar o quitar en cualquier momento;
    - **Quitar todos** desactiva todos los efectos;
    - los efectos son **no destructivos**: el `.xshot` guarda la pila y la imagen sin efectos. Se aplican al exportar, copiar, compartir o imprimir.
  - **Destino del efecto**, según lo seleccionado:
    - nada seleccionado: el **documento** (imagen base y objetos);
    - una imagen flotante seleccionada: **esa imagen** (cada imagen flotante tiene su propia pila);
    - una selección de píxeles: **Aplicar a la selección** aplica el efecto a esos píxeles y lo fija en ellos (destructivo, se puede deshacer), porque una selección no es un objeto (decisión de diseño). Solo admite los efectos que no cambian el tamaño: filtros, ajuste de color, reemplazo de color y Foco y lupa.
  - Los efectos que amplían la imagen (Sombra, Bordes decorativos, Perspectiva, Página doblada, Reflejo, Detalles de captura en franja) amplían el **resultado exportado**, no el lienzo editable. Las dimensiones de la barra de estado muestran el tamaño final junto al del lienzo (por ejemplo, "1280 × 720 px → 1310 × 750 con efectos").
  - Mientras está activa Quitar franja (EDT-052) o Recortar (EDT-049), los efectos se ocultan para ver la imagen real y vuelven al terminar.
  - Los mismos efectos están disponibles al capturar (CAP-051) y en la exportación por lotes (EDT-113).
- **Criterios de aceptación:**
  - [ ] Activar Borde y Sombra, guardar, cerrar y abrir el `.xshot` muestra los dos efectos activos y editables.
  - [ ] Con una imagen flotante seleccionada, activar Borde pone el borde solo alrededor de esa imagen.
  - [ ] Con una selección rectangular, Aplicar a la selección con Escala de grises solo cambia los píxeles de dentro, y Deshacer lo revierte.
  - [ ] Reordenar la pila (Perspectiva antes o después de Borde) cambia el resultado: con Borde primero, el borde se inclina con la perspectiva.
  - [ ] Un `.xshot` con efectos abierto en la otra plataforma muestra los mismos efectos con los mismos valores.
- **Mejora sobre Snagit:** efectos no destructivos, reordenables y conservados entre plataformas.
- **Fuentes:** research/02-editor.md §6 (General behavior), §10.2 (Mac to Windows: effects lost)

### EDT-054 · Efectos guardados
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Quick Effects (preestablecidos y propios; Add to Quick Effects / +)
- **Comportamiento:**
  - Arriba de la pestaña Efectos, una fila de **efectos guardados**: miniaturas de combinaciones listas (la pila completa con sus valores).
  - Un clic aplica la combinación: sustituye la pila actual del destino (documento o imagen flotante). `Shift`+clic la añade al final de la pila (decisión de diseño).
  - **+** guarda la pila actual como efecto guardado, con un nombre.
  - Gestión: reordenar arrastrando; clic derecho > Renombrar, Actualizar con la pila actual, Eliminar, Añadir a favoritos (EDT-016).
  - Efectos guardados incorporados (decisión de diseño): "Marco fino", "Tarjeta flotante" (esquinas redondeadas 12 px y sombra suave), "Hoja rasgada" (borde decorativo rasgado abajo y sombra), "Perspectiva suave", "Documentación" (borde gris de 1 px y detalles de captura en franja inferior), "Escala de grises".
  - Se pueden elegir al capturar (CAP-051) y en la exportación por lotes (EDT-113). Viajan en la exportación de ajustes (APP-011).
- **Criterios de aceptación:**
  - [ ] Montar Borde + Esquinas redondeadas + Sombra, pulsar + y aplicar ese efecto guardado a otra captura reproduce los tres efectos con los mismos valores.
  - [ ] `Shift`+clic en un efecto guardado con un Borde ya activo añade sus efectos sin quitar el Borde.
  - [ ] Un efecto guardado marcado como favorito aparece en la herramienta Favoritos.
- **Fuentes:** research/02-editor.md §6 (Quick Effects), §2.3

### EDT-055 · Borde
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** efecto Border (color, grosor)
- **Comportamiento:**
  - Dibuja un marco alrededor de la imagen.
  - Opciones: **Color** (EDT-019), **Grosor** (1–100 px; por defecto 2 px), **Posición** (Exterior, que amplía el resultado, o Interior, que pinta encima del borde de la imagen; por defecto Exterior) (decisión de diseño).
  - Sigue las esquinas redondeadas de la imagen (EDT-075).
- **Criterios de aceptación:**
  - [ ] Un borde exterior de 5 px en una imagen de 400 × 300 exporta una imagen de 410 × 310.
  - [ ] Con esquinas redondeadas de 20 px, el borde también es redondeado.
- **Fuentes:** research/02-editor.md §6 (Border), §7 (Rounded image corners)

### EDT-056 · Bordes decorativos
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** efecto Edges (estilos, tamaño, lados, color y posición de la sombra)
- **Comportamiento:**
  - Da forma al borde de la imagen.
  - **Estilo** (decisión de diseño): Rasgado, Ondulado, Dientes de sierra, Biselado, Desvanecido y Festón.
  - **Tamaño**: 2–80 px (por defecto 16 px).
  - **Lados**: casillas Arriba, Abajo, Izquierda y Derecha (por defecto, solo Abajo).
  - **Aleatoriedad** (solo Rasgado): 0–100 %, con un botón para generar otra variante.
  - **Sombra del borde**: sí/no, con color y dirección.
  - Solo puede haber un efecto de bordes decorativos a la vez en un mismo destino.
- **Criterios de aceptación:**
  - [ ] Rasgado en el lado Abajo deja el borde inferior irregular y los otros tres rectos.
  - [ ] El mismo `.xshot` con Rasgado se exporta igual (píxel a píxel) en Windows y en macOS, porque la variante aleatoria se guarda en el archivo.
- **Fuentes:** research/02-editor.md §6 (Edges), §15 (punto 1)

### EDT-057 · Sombra de la imagen
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** efecto Shadow (dirección, color, tamaño; en Mac, opacidad, desplazamiento y desenfoque)
- **Comportamiento:**
  - Añade una sombra detrás de la imagen; amplía el resultado lo necesario para que quepa.
  - Opciones: cuadrícula de **dirección** de 3 × 3 (con el centro = alrededor), **Color**, **Opacidad** (0–100 %), **Distancia** (0–100 px), **Desenfoque** (0–100 px) y **Fondo** (Transparente o el color del lienzo).
  - Sigue la forma de la imagen: esquinas redondeadas, bordes decorativos y zonas transparentes.
- **Criterios de aceptación:**
  - [ ] Una sombra abajo a la derecha con distancia 10 y desenfoque 20, exportada a PNG con Fondo Transparente, tiene la sombra semitransparente.
  - [ ] Con esquinas redondeadas, la sombra también es redondeada.
- **Mejora sobre Snagit:** todos los ajustes en ambas plataformas.
- **Fuentes:** research/02-editor.md §6 (Shadow)

### EDT-058 · Perspectiva
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** efecto Perspective (estilo Perspectiva o Inclinación; horizontal/giro y vertical/inclinación)
- **Comportamiento:**
  - Inclina la imagen como si se viera en 3D, sobre fondo transparente.
  - **Estilo**: Perspectiva (giro en 3D) o Sesgo (inclinación plana).
  - **Horizontal** (−45° a 45°) y **Vertical** (−45° a 45°), con deslizadores y campos.
  - **Profundidad** (solo Perspectiva): 0–100 %.
- **Criterios de aceptación:**
  - [ ] Perspectiva horizontal 20° exporta una imagen trapezoidal sobre fondo transparente en PNG.
  - [ ] Sesgo vertical −10° inclina la imagen sin escorzo.
- **Fuentes:** research/02-editor.md §6 (Perspective)

### EDT-059 · Página doblada
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** efecto Page Curl (Windows)
- **Comportamiento:**
  - Dobla una esquina de la imagen como una hoja de papel, con sombra.
  - Opciones: **Esquina** (cualquiera de las cuatro; por defecto, inferior derecha), **Tamaño** (5–50 % del lado menor) y **Color del reverso** (por defecto, gris claro).
  - Lo que queda detrás de la esquina doblada es transparente.
- **Criterios de aceptación:**
  - [ ] Página doblada en la esquina inferior derecha al 20 % muestra la esquina doblada y transparente detrás.
  - [ ] Cambiar a la esquina superior izquierda mueve el doblez sin dejar rastro en la esquina anterior.
- **Mejora sobre Snagit:** disponible en ambas plataformas.
- **Fuentes:** research/02-editor.md §6 (Page Curl)

### EDT-060 · Reflejo
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** efecto Reflection (Mac)
- **Comportamiento:**
  - Añade bajo la imagen un reflejo invertido que se desvanece.
  - Opciones: **Tamaño** (10–100 % de la altura reflejada), **Opacidad inicial** (0–100 %; por defecto 40 %) y **Separación** (0–50 px).
  - El reflejo amplía el resultado hacia abajo; el fondo es transparente.
- **Criterios de aceptación:**
  - [ ] Reflejo al 30 % en una imagen de 400 px de alto añade 120 px de reflejo bajo la imagen.
  - [ ] La parte inferior del reflejo es totalmente transparente en PNG.
- **Mejora sobre Snagit:** disponible en ambas plataformas.
- **Fuentes:** research/02-editor.md §6 (Reflection)

### EDT-061 · Desvanecer bordes
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** efecto Fade (Mac)
- **Comportamiento:**
  - Hace que los bordes de la imagen se vuelvan transparentes de forma gradual.
  - Opciones: **Tamaño** (2–200 px), **Intensidad** (0–100 %) y **Lados** (los cuatro por defecto; casillas por lado).
- **Criterios de aceptación:**
  - [ ] Desvanecer 40 px en los cuatro lados exporta un PNG cuyos píxeles del borde son transparentes y los del centro opacos.
  - [ ] Desmarcar Arriba deja el borde superior sin desvanecer.
- **Mejora sobre Snagit:** disponible en ambas plataformas.
- **Fuentes:** research/02-editor.md §6 (Fade)

### EDT-062 · Filtros
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Filters (Windows: Soften, Sharpen, Opacity, Monochrome, Grayscale, Invert) y Color Filters (Mac: Grayscale, Sepia, Invert, Sharpen, Opacity)
- **Comportamiento:**
  - Un solo efecto **Filtros** con estos filtros, cada uno con su interruptor y su intensidad:
    - **Suavizar** (1–10);
    - **Enfocar** (1–10);
    - **Escala de grises**;
    - **Sepia** (intensidad 0–100 %);
    - **Monocromo** (blanco y negro puro; umbral 0–255);
    - **Invertir colores**;
    - **Opacidad** de la imagen (0–100 %).
  - Varios filtros pueden estar activos a la vez; se aplican en el orden de la lista.
  - Se aplican a la imagen y a los objetos del destino (con las anotaciones incluidas, decisión de diseño). **Solo la imagen base** (casilla) los limita a la imagen base, sin las anotaciones.
- **Criterios de aceptación:**
  - [ ] Escala de grises sobre una captura con una flecha roja deja la flecha gris; con Solo la imagen base, la flecha sigue roja.
  - [ ] Opacidad 50 % exporta un PNG con alfa 128 en los píxeles opacos.
  - [ ] Monocromo con umbral 128 produce una imagen solo con píxeles negros y blancos.
- **Mejora sobre Snagit:** la unión de los filtros de ambas plataformas, en las dos.
- **Fuentes:** research/02-editor.md §6 (Filters, Color Filters)

### EDT-063 · Ajuste de color
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** efecto Color Adjustment (brillo, contraste, matiz, saturación, gamma, restablecer)
- **Comportamiento:**
  - Deslizadores: **Brillo** (−100 a 100), **Contraste** (−100 a 100), **Matiz** (−180° a 180°), **Saturación** (−100 a 100) y **Gamma** (0,1 a 5,0; por defecto 1,0).
  - **Restablecer** vuelve todos los valores al neutro.
  - Vista previa en vivo mientras se arrastra; mantener pulsado el ojo muestra el antes (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Saturación −100 da una imagen sin color.
  - [ ] Restablecer devuelve los cinco deslizadores al neutro y la imagen se ve como sin el efecto.
- **Fuentes:** research/02-editor.md §6 (Color Adjustment)

### EDT-064 · Reemplazo de color
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** efecto Color Replacement (Windows): cambiar un color por otro o intercambiar dos; muestra transparente
- **Comportamiento:**
  - **Modo**: **Reemplazar** (el color A pasa a ser B) o **Intercambiar** (A y B se cambian entre sí).
  - **Color A** y **Color B**, cada uno con el selector (incluido el cuentagotas sobre el lienzo y Transparente).
  - **Tolerancia**: 0–100 % (por defecto 5 %).
- **Criterios de aceptación:**
  - [ ] Reemplazar blanco por Transparente con tolerancia 0 deja transparentes todos los píxeles blancos.
  - [ ] Intercambiar rojo y azul en una imagen con zonas de ambos colores cambia unas por otras.
- **Mejora sobre Snagit:** disponible en ambas plataformas.
- **Fuentes:** research/02-editor.md §6 (Color Replacement)

### EDT-065 · Efecto Foco y lupa
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** efecto Spotlight and Magnify (Windows; requiere una selección)
- **Comportamiento:**
  - Requiere una selección de píxeles. Resalta la zona seleccionada y atenúa el resto, y opcionalmente la amplía en su sitio.
  - Opciones (decisión de diseño, porque no están documentadas): **Atenuación del exterior** (0–100 %), **Desenfoque del exterior** (0–100), **Aumento** (100–400 %) y **Borde** (color y grosor) alrededor de la zona.
  - El resultado se fija en los píxeles, como todos los efectos aplicados a una selección (EDT-053). Para un resultado editable se usan las herramientas Foco (EDT-039) y Lupa (EDT-038); el panel lo indica con un enlace.
- **Criterios de aceptación:**
  - [ ] Con una selección y Aumento 150 %, la zona aparece ampliada y centrada en su sitio y el exterior atenuado.
  - [ ] Sin selección, el efecto aparece desactivado y su tooltip explica que hace falta una selección.
- **Mejora sobre Snagit:** disponible en ambas plataformas.
- **Fuentes:** research/02-editor.md §6 (Spotlight and Magnify), §15 (punto 6)

### EDT-066 · Marca de agua
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** efecto Watermark (Windows): imagen, posición, escala, sobre o bajo el contenido
- **Comportamiento:**
  - **Origen**: una imagen (PNG con transparencia recomendado) o un **texto** (fuente, tamaño y color) (decisión de diseño).
  - **Posición**: cuadrícula de 3 × 3 o **Mosaico** (repetida por toda la imagen), con **Margen** (0–200 px).
  - **Escala**: 5–200 %. **Opacidad**: 0–100 % (por defecto 30 %). **Giro**: −90° a 90°.
  - **Capa**: **Encima** (sobre todo el contenido) o **Debajo** (integrada en el fondo, en relieve, bajo las anotaciones).
  - La imagen de la marca se guarda dentro del `.xshot`, para que se vea igual en otro equipo.
- **Criterios de aceptación:**
  - [ ] Una marca de agua de imagen en la esquina inferior derecha al 30 % de opacidad se exporta en esa esquina, semitransparente.
  - [ ] Con Debajo, las anotaciones se ven por encima de la marca; con Encima, la marca tapa las anotaciones.
  - [ ] Un texto en Mosaico con giro −30° se repite en diagonal por toda la imagen.
- **Mejora sobre Snagit:** disponible en ambas plataformas y con texto y mosaico.
- **Fuentes:** research/02-editor.md §6 (Watermark)

### EDT-067 · Detalles de captura
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** efecto Capture Info (aplicación y versión o URL, fecha y hora, sistema, compilación, usuario, texto propio; fondo, fuente, color y tamaño)
- **Comportamiento:**
  - Muestra en la imagen los metadatos de la captura (CAP-056).
  - **Campos** (casillas, en este orden): Aplicación y versión; URL (si la hay; sustituye al nombre del navegador); Título de la ventana; Fecha y hora; Sistema operativo y versión; Usuario; Equipo; **Texto propio**.
  - **Formato de fecha**: el del sistema, ISO 8601 o personalizado.
  - **Ubicación**: **Franja** (amplía la imagen con una banda arriba o abajo) o **Superpuesta** (en una esquina, sobre la imagen).
  - **Estilo**: fuente, tamaño, color del texto, color de fondo (con opacidad), alineación y margen interior.
  - **Pedir texto al capturar**: lo define CAP-051.
  - Solo hay datos para capturas de XcreenShot. En imágenes abiertas de archivo, los campos de captura aparecen vacíos y el efecto solo muestra el texto propio y, si se marca, la fecha del archivo (decisión de diseño).
  - Al exportar con este efecto, los datos se convierten en píxeles: el aviso del panel recuerda que se verán en la imagen compartida.
- **Criterios de aceptación:**
  - [ ] En una captura de un navegador, con URL y Fecha marcadas y Franja abajo, la imagen gana una banda inferior con la URL y la fecha.
  - [ ] Desmarcar Usuario lo quita de la imagen al momento.
  - [ ] En un PNG abierto desde una carpeta, el efecto muestra solo el texto propio.
- **Fuentes:** research/02-editor.md §6 (Capture Info); research/01-captura.md §12.2

### EDT-068 · Efectos de transformación
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Trim como efecto (Mac); redondeo de esquinas como efecto de captura (Mac); escalar y girar como efectos de preset (Mac); Scale Image (Windows)
- **Comportamiento:**
  - Efectos de la pila que cambian la geometría del resultado sin tocar el documento:
    - **Recortar bordes**: quita bordes uniformes, como EDT-051, con tolerancia;
    - **Esquinas redondeadas**: radio en px o "círculo", como EDT-075;
    - **Escalar**: por porcentaje, por lado mayor en píxeles o a un ancho o alto fijos, con el remuestreo de EDT-069;
    - **Girar**: 90°, 180° o 270°.
  - Sirven sobre todo para los efectos al capturar (CAP-051), los efectos guardados (EDT-054) y la exportación por lotes (EDT-113). En el editor se pueden usar también y se ven en el resultado.
  - Si el documento ya tiene esquinas redondeadas propias (EDT-075), el efecto Esquinas redondeadas no se suma: el panel indica que el documento ya las tiene y el efecto sustituye ese valor solo en la salida (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Un efecto guardado con Escalar 50 % aplicado en la exportación por lotes a imágenes de 2000 px produce archivos de 1000 px.
  - [ ] Recortar bordes como efecto, desactivado, devuelve la imagen con sus bordes sin haberla modificado.
- **Fuentes:** research/02-editor.md §6 (Trim, Corner rounding); research/01-captura.md §4.3; research/05-plataformas-versiones.md §1B (2025.2.1)

---

## 6. Imagen y lienzo

### EDT-069 · Cambiar tamaño de la imagen
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Resize Image (porcentaje, píxeles, pulgadas, centímetros; proporción; resolución y remuestreo en Windows; densidad de píxeles en Mac)
- **Comportamiento:**
  - **Imagen > Cambiar tamaño de la imagen…** (o clic en las dimensiones de la barra de estado) abre un diálogo con:
    - **Unidades**: Porcentaje, Píxeles, Pulgadas o Centímetros;
    - **Ancho** y **Alto**, con un candado de **proporción** (cerrado por defecto);
    - **Resolución** (PPP), que se usa para convertir pulgadas y centímetros (EDT-077);
    - **Remuestreo**: Automático (por defecto; elige según se amplíe o reduzca), Bicúbico, Bilineal, Lanczos o Vecino más cercano (para conservar los píxeles nítidos de una interfaz al ampliar a 200 %, 300 %…);
    - **Remuestrear la imagen** (por defecto: sí). Si se desmarca, cambiar pulgadas o centímetros solo cambia la resolución, no los píxeles;
    - una línea con el tamaño resultante en píxeles y en disco aproximado.
  - Los objetos vectoriales se **escalan con la imagen** y siguen siendo editables, incluido el tamaño de letra de textos y globos (mejora). Las imágenes flotantes se escalan con el mismo remuestreo.
  - Al ampliar más de un 100 % aparece un aviso: "Ampliar la imagen reduce su nitidez".
  - Se puede deshacer.
- **Criterios de aceptación:**
  - [ ] Reducir una imagen de 2000 × 1000 al 50 % deja 1000 × 500, y un texto de 20 pt pasa a 10 pt y sigue siendo editable.
  - [ ] Con el candado cerrado, escribir Ancho = 800 en una imagen de 1600 × 900 pone Alto = 450.
  - [ ] Ampliar al 200 % con Vecino más cercano duplica cada píxel sin mezclar colores.
  - [ ] Con Remuestrear desmarcado, cambiar el ancho de 10 a 5 pulgadas sube la resolución al doble y deja los mismos píxeles.
- **Mejora sobre Snagit:** los objetos siguen siendo editables tras escalar; todas las opciones en ambas plataformas.
- **Fuentes:** research/02-editor.md §7 (Resize Image), §3.4 (callouts scale proportionally)

### EDT-070 · Tamaño del lienzo
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** ampliar el lienzo arrastrando el borde; Resize Canvas a un tamaño exacto (Mac); `Cmd`+arrastrar desde el centro (Mac)
- **Comportamiento:**
  - **Arrastrando**: con Mover activo y nada seleccionado, el lienzo muestra tiradores en sus bordes y esquinas. Arrastrarlos amplía o reduce el lienzo; la etiqueta junto al puntero muestra el tamaño. `Alt`/`Option` lo hace desde el centro; `Shift` mantiene la proporción.
  - **Imagen > Tamaño del lienzo…**: ancho y alto (en las unidades de EDT-069), un **ancla** de 3 × 3 que fija hacia dónde crece y el color del área nueva (por defecto, el color del lienzo).
  - Ampliar no escala la imagen: añade espacio. Reducir oculta lo que queda fuera, igual que recortar (EDT-049), y es reversible.
- **Criterios de aceptación:**
  - [ ] Arrastrar el borde derecho 200 px amplía el ancho 200 px y el área nueva tiene el color del lienzo.
  - [ ] En el diálogo, 1000 × 1000 con el ancla en el centro añade la misma cantidad de espacio a cada lado.
  - [ ] Reducir el lienzo y volver a ampliarlo devuelve la imagen original sin pérdida.
- **Mejora sobre Snagit:** tamaño exacto y ancla en ambas plataformas.
- **Fuentes:** research/02-editor.md §7 (Resize Canvas / extend canvas), §2.7 (Command-drag)

### EDT-071 · Ampliar el lienzo automáticamente
- **Hito:** M1 (adelantado de M2 por decisión de Carlos, 2026-10-27; las imágenes pegadas o soltadas llegan en M2 con EDT-078) · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** preferencia Auto-expand canvas to fit objects
- **Comportamiento:**
  - Preferencia **Ampliar el lienzo automáticamente** (`editor.autoExpandCanvas`; por defecto: sí): cuando un objeto sobresale del lienzo al dibujarlo, moverlo, redimensionarlo, girarlo o pegarlo, el lienzo crece lo justo para que quepa. El área nueva toma el color del lienzo.
  - Mientras se dibuja o se arrastra, el objeto se ve entero aunque salga del borde; el lienzo crece al soltar.
  - Cuando el objeto vuelve dentro o se elimina, el lienzo se encoge solo. Nunca queda más pequeño que la imagen o que el último recorte (decisión de Carlos, 2026-10-27).
  - Al crecer por la izquierda o por arriba, la imagen y los objetos se quedan en su sitio en pantalla.
  - Lo que un recorte deja fuera se queda fuera aunque se mueva, porque el recorte fue intencionado (EDT-049). Si vuelve entero dentro del recorte, se trata como cualquier otro objeto.
  - Los objetos de Desenfocar no amplían el lienzo: solo actúan sobre la imagen (decisión de diseño).
  - Un objeto que sobresale menos de medio píxel cuenta como dentro, y la cola de una flecha dibujada desde el borde no amplía el lienzo por su grosor (decisión de diseño).
  - Deshacer revierte en un paso el cambio del objeto y el del lienzo.
- **Criterios de aceptación:**
  - [ ] Con la preferencia activada, arrastrar una flecha 100 px fuera del borde derecho amplía el lienzo hasta su punta; volver a meterla lo devuelve a su tamaño.
  - [ ] Una forma que sobresale por la izquierda amplía el lienzo por la izquierda y la imagen no se mueve en pantalla.
  - [ ] Con la preferencia desactivada, el lienzo no cambia y el objeto sobresale.
  - [ ] Una flecha que un recorte dejó medio fuera no amplía el lienzo al moverla.
  - [ ] Un solo Deshacer quita el cambio y devuelve el lienzo a su tamaño anterior.
  - [ ] (M2) Pegar una imagen de 1200 px de ancho en un lienzo de 800 px amplía el lienzo hasta que cabe.
- **Fuentes:** research/02-editor.md §7 (Resize Canvas), §11

### EDT-072 · Girar
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Rotate (90° en ambos sentidos, 180°, ángulo personalizado; diálogo con rueda en Mac)
- **Comportamiento:**
  - **Imagen > Girar ▸**: 90° a la derecha, 90° a la izquierda, 180° y **Ángulo personalizado…**.
  - Ángulo personalizado: campo de −359,9° a 359,9° (con decimales), rueda, vista previa y **Fondo** de las esquinas nuevas (Transparente o color del lienzo). El lienzo crece para que quepa la imagen girada.
  - Los objetos giran con la imagen y siguen siendo editables. Los textos giran con su objeto.
- **Criterios de aceptación:**
  - [ ] Girar 90° a la derecha una imagen de 800 × 600 la deja de 600 × 800 con las flechas en su sitio relativo.
  - [ ] Un giro de 15° deja las esquinas nuevas transparentes con Fondo = Transparente.
  - [ ] Girar 90° y luego 90° a la izquierda devuelve exactamente la imagen original.
- **Fuentes:** research/02-editor.md §7 (Rotate)

### EDT-073 · Voltear
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Flip horizontal y vertical (voltea también las anotaciones y el texto)
- **Comportamiento:**
  - **Imagen > Voltear ▸ Horizontal** y **Vertical** voltean la imagen base y la posición de todos los objetos.
  - Los objetos con texto (Texto, Globo, Paso) se reflejan en posición y forma, pero su texto **sigue legible**, no en espejo (decisión de diseño). Las flechas se reflejan con su dirección.
  - Con objetos seleccionados, **Objeto > Voltear ▸** voltea solo esos objetos (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Voltear en horizontal una captura con una flecha que apunta a la derecha la deja apuntando a la izquierda en la posición reflejada.
  - [ ] Tras voltear, el texto de un globo se lee normal y la cola apunta al lado reflejado.
- **Fuentes:** research/02-editor.md §7 (Flip)

### EDT-074 · Color del lienzo
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Canvas Color (cualquier color, transparente, degradado en Mac)
- **Comportamiento:**
  - **Imagen > Color del lienzo…** abre el selector (EDT-019) con color sólido, **Transparente** y degradado (EDT-020).
  - El color del lienzo se ve en las zonas transparentes de la imagen, en las borradas (EDT-048), en los huecos transparentes (EDT-045) y en las zonas añadidas al ampliar el lienzo.
  - Por defecto: el color de la imagen nueva (EDT-103) o, en una captura, blanco (decisión de diseño). Las capturas con transparencia (ventanas con esquinas redondeadas) tienen el lienzo transparente.
  - Al exportar a formatos sin transparencia (JPG, BMP), las zonas transparentes se rellenan con el color de **Fondo para formatos sin transparencia** (EDT-112).
- **Criterios de aceptación:**
  - [ ] Cambiar el lienzo a Transparente y borrar una zona hace que esa zona sea transparente en el PNG exportado.
  - [ ] Un lienzo con degradado lineal se ve en las zonas añadidas al ampliar el lienzo.
  - [ ] Una imagen con lienzo transparente exportada a JPG tiene las zonas transparentes del color de fondo configurado.
- **Mejora sobre Snagit:** degradados en ambas plataformas.
- **Fuentes:** research/02-editor.md §7 (Canvas color), §2.6, §4.5

### EDT-075 · Esquinas redondeadas
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** esquinas redondeadas de la imagen (rombo en la esquina; un clic = 20 px; arrastrar al centro = círculo); en imágenes flotantes y en varios objetos (Mac)
- **Comportamiento:**
  - Con Mover activo y el puntero sobre el lienzo aparece un **rombo** cerca de la esquina superior derecha. Un clic aplica un radio de 20 px; arrastrarlo ajusta el radio, con la etiqueta del valor junto al puntero.
  - Llevarlo al máximo (la mitad del lado menor) convierte un lienzo cuadrado en un **círculo** y uno rectangular en una cápsula. **Imagen > Esquinas redondeadas…** permite escribir el radio o elegir **Círculo**, que primero recorta la imagen a un cuadrado centrado (decisión de diseño).
  - Las esquinas quedan transparentes; para conservarlas hay que exportar a un formato con transparencia (el diálogo de exportación lo recuerda si se elige JPG).
  - También funciona en **imágenes flotantes** (su propio rombo) y, con varias seleccionadas, el campo **Radio** de Propiedades las cambia todas.
  - Funciona junto con Borde, Sombra, Perspectiva y Detalles de captura, que siguen la forma redondeada.
  - El radio aparece en la barra de estado (EDT-011) y se puede deshacer.
- **Criterios de aceptación:**
  - [ ] Un clic en el rombo deja la imagen con esquinas de 20 px transparentes en PNG.
  - [ ] Elegir Círculo en una imagen de 800 × 600 da una imagen circular de 600 px de diámetro.
  - [ ] Con tres imágenes flotantes seleccionadas, Radio = 16 redondea las tres.
  - [ ] Deshacer devuelve el radio anterior.
- **Fuentes:** research/02-editor.md §7 (Rounded image corners), §1 (Image dimensions readout); research/05-plataformas-versiones.md §1A (2024.3.0), §1B (2024.4.0, 2025.3.1, 2026.1.0)

### EDT-076 · Volver al original
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Revert to Original (descarta todas las ediciones; deshace también Cut Out)
- **Comportamiento:**
  - **Edición > Volver al original** descarta todos los cambios del documento (objetos, efectos, recortes, franjas, cambios de tamaño y de lienzo) y deja la imagen tal como se capturó o abrió. Los metadatos se conservan.
  - Pide confirmación: "Se perderán todas las ediciones de este documento".
  - Se puede deshacer en la misma sesión, como cualquier otra acción (decisión de diseño: más seguro que una acción irreversible).
  - Si el contenido recortado se eliminó al guardar (EDT-050), el diálogo avisa de que el original ya no está completo.
  - Se puede añadir a la barra de herramientas (EDT-004).
- **Criterios de aceptación:**
  - [ ] Tras recortar, quitar una franja y añadir tres flechas, Volver al original deja la captura original sin objetos.
  - [ ] Deshacer justo después recupera todas las ediciones.
- **Fuentes:** research/02-editor.md §7 (Revert to Original), §4.6, §2.1

### EDT-077 · Resolución y densidad de píxeles
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Resolution (DPI) en Windows; Pixel Density y escalado Retina en Mac; 96 PPP por defecto (documentación antigua)
- **Comportamiento:**
  - Cada documento tiene una **resolución** (PPP) que se guarda en el `.xshot` y en los formatos que la admiten (PNG, JPG, TIFF, PDF).
  - Valor inicial de una captura: 96 PPP multiplicado por la escala de la pantalla de origen (por ejemplo, 192 PPP en una pantalla al 200 %) (decisión de diseño, alineada con cómo las apps de documentos interpretan las capturas de alta densidad).
  - **Imagen > Resolución…** (o clic en la resolución de la barra de estado) permite cambiarla sin tocar los píxeles. Los valores comunes 72, 96, 144, 150, 192 y 300 están en un menú.
  - **Densidad** (1×, 2×, 3×) es otra forma de ver el mismo dato: 96 PPP × densidad. Se usa para el Tamaño original en pantalla (EDT-007) y para la opción **Reducir las imágenes de alta densidad al compartir** (SHR- (escalado)).
- **Criterios de aceptación:**
  - [ ] Una captura hecha en una pantalla Retina tiene 192 PPP y, insertada en un procesador de textos, ocupa el mismo tamaño que en pantalla.
  - [ ] Cambiar la resolución a 300 PPP no cambia las dimensiones en píxeles, y el PNG exportado declara 300 PPP.
- **Fuentes:** research/02-editor.md §7 (Image DPI and pixel density, Resize Image), §10.3 (Retina downscaling)

---

## 7. Composición y objetos

### EDT-078 · Imágenes flotantes
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** combinar imágenes arrastrando desde la bandeja, pegando o soltando desde el navegador o el gestor de archivos; transparencia conservada al pegar; preferencia Automatically flatten pasted images (Windows)
- **Comportamiento:**
  - Una **imagen flotante** es un objeto de imagen sobre el lienzo que se puede mover, escalar, girar, recortar, redondear, aplicar efectos y aplanar.
  - Formas de crearla:
    - arrastrar una miniatura de la Bandeja reciente al lienzo (TRY- (Bandeja reciente));
    - pegar una imagen del portapapeles;
    - soltar un archivo de imagen desde el Explorador, el Finder o un navegador;
    - **Archivo > Importar ▸ Imagen como objeto…**;
    - convertir una selección (EDT-044).
  - La imagen se coloca en el punto donde se suelta o, al pegar, centrada en la vista. Si sobresale, se aplica EDT-071.
  - Las imágenes con transparencia (PNG, WebP, GIF, HEIC con alfa) conservan la transparencia.
  - Al pegar o soltar un `.xshot`, se inserta su resultado aplanado (decisión de diseño).
  - **Recortar** una imagen flotante: doble clic sobre ella, o Recortar con la imagen seleccionada, recorta solo esa imagen, de forma no destructiva.
  - Herramientas de píxeles (Relleno, Borrador, Varita mágica, Selección, Lápiz no editable) actúan sobre la imagen flotante seleccionada.
  - Preferencia **Aplanar automáticamente las imágenes pegadas** (`editor.autoFlattenPasted`; por defecto: no).
- **Criterios de aceptación:**
  - [ ] Arrastrar una miniatura de la bandeja al lienzo crea una imagen flotante del tamaño original de esa captura.
  - [ ] Pegar un PNG con fondo transparente lo muestra sin fondo sobre la captura.
  - [ ] Doble clic en una imagen flotante y recortarla reduce solo esa imagen; luego se puede ampliar de nuevo.
  - [ ] Con la preferencia de aplanar activada, la imagen pegada pasa a formar parte de la imagen base y no se puede seleccionar.
- **Fuentes:** research/02-editor.md §8 (Floating images), §1.1 (drag a tray item), §11; research/04-biblioteca-compartir.md §6 (Drag in)

### EDT-079 · Selección múltiple
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Select All, Select All Objects, Deselect All; selección por recuadro y con `Shift` (no documentadas)
- **Comportamiento:**
  - `Shift`+clic o `Ctrl`/`Cmd`+clic añade o quita un objeto de la selección.
  - Con Mover, arrastrar sobre una zona vacía dibuja un **recuadro de selección**: selecciona los objetos que quedan **completamente** dentro. Con `Alt`/`Option`, también los que toca (decisión de diseño).
  - **Seleccionar todos los objetos** (`Ctrl`/`Cmd`+`A`): selecciona todos los objetos no bloqueados. Con Selección o Varita mágica activas, `Ctrl`/`Cmd`+`A` selecciona en cambio todos los píxeles.
  - **Deseleccionar** (`Ctrl`/`Cmd`+`D` o `Esc`).
  - Los objetos seleccionados se muestran con un recuadro común con tiradores (EDT-080) y, cada uno, con un contorno fino.
- **Criterios de aceptación:**
  - [ ] Un recuadro que encierra por completo dos flechas y la mitad de un texto selecciona solo las dos flechas.
  - [ ] Con Mover, `Ctrl`/`Cmd`+`A` selecciona todos los objetos; con Selección, selecciona toda la imagen.
  - [ ] `Shift`+clic sobre un objeto ya seleccionado lo quita de la selección.
- **Fuentes:** research/02-editor.md §8 (Multi-select), §15 (punto 3)

### EDT-080 · Transformar varios objetos
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** mover, cambiar de tamaño y girar varios objetos a la vez, incluidas imágenes flotantes, y radio de esquinas de varios (Mac)
- **Comportamiento:**
  - Con varios objetos seleccionados, el recuadro común tiene tiradores de tamaño y de giro:
    - **mover** desplaza todos;
    - **cambiar de tamaño** los escala en proporción respecto al recuadro. La opción **Escalar grosores y texto** (en Propiedades; por defecto: sí) decide si también se escalan el grosor de las líneas y el tamaño de letra (decisión de diseño);
    - **girar** los gira alrededor del centro del recuadro.
  - Propiedades comunes (color, opacidad, sombra, radio de esquinas) cambian en todos a la vez (EDT-014).
- **Criterios de aceptación:**
  - [ ] Con una flecha y una imagen flotante seleccionadas, ampliar el recuadro un 50 % amplía las dos un 50 % y mantiene sus posiciones relativas.
  - [ ] Girar 90° tres objetos los gira alrededor del centro común.
  - [ ] Con Escalar grosores y texto desactivado, ampliar el recuadro no cambia el grosor de una flecha seleccionada.
- **Mejora sobre Snagit:** disponible en ambas plataformas.
- **Fuentes:** research/02-editor.md §8 (Multi-object transform); research/05-plataformas-versiones.md §1B (2026.1.0)

### EDT-081 · Orden de apilado
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Bring Forward, Send Backward, Bring to Front, Send to Back
- **Comportamiento:**
  - **Objeto > Traer al frente**, **Traer adelante**, **Enviar atrás**, **Enviar al fondo**, también en el menú contextual y con atajos (EDT-116).
  - Un objeto nunca queda por debajo de la imagen base.
  - Con varios objetos seleccionados, mantienen su orden relativo.
- **Criterios de aceptación:**
  - [ ] Con tres objetos solapados, Enviar al fondo sobre el superior lo deja debajo de los otros dos.
  - [ ] Traer adelante sobre el objeto del fondo lo pone justo encima del siguiente, no arriba del todo.
- **Mejora sobre Snagit:** los cuatro comandos con atajo en ambas plataformas.
- **Fuentes:** research/02-editor.md §8 (Z-order), §12

### EDT-082 · Agrupar y desagrupar
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Group / Ungroup
- **Comportamiento:**
  - **Agrupar** (`Ctrl`/`Cmd`+`G`) une los objetos seleccionados en un grupo, que se selecciona, mueve, transforma y apila como un solo objeto. **Desagrupar** (`Ctrl`/`Cmd`+`Shift`+`G`) lo separa.
  - Doble clic en un grupo entra en él para seleccionar y editar un objeto suelto; `Esc` o un clic fuera salen del grupo.
  - Los grupos se pueden anidar.
  - Los grupos se guardan en el `.xshot` y se mantienen en la otra plataforma.
- **Criterios de aceptación:**
  - [ ] Agrupar una flecha y un globo y mover el grupo mueve los dos; Desagrupar los deja como objetos independientes en la misma posición.
  - [ ] Doble clic en el grupo y luego clic en el globo permite editar su texto sin desagrupar.
- **Fuentes:** research/02-editor.md §8 (Group / Ungroup), §12

### EDT-083 · Alinear y distribuir
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Align y distribuir "Evenly" (Mac); alineación de objetos de Simplify (ambas)
- **Comportamiento:**
  - **Objeto > Alinear ▸**: Izquierda, Centro horizontal, Derecha, Arriba, Centro vertical y Abajo.
  - **Objeto > Distribuir ▸**: Horizontalmente y Verticalmente (espacio igual entre objetos), y **Igualar ancho** e **Igualar alto** (decisión de diseño).
  - **Alinear respecto a**: la **selección** (por defecto, con 2 o más objetos) o el **lienzo** (por defecto, con un solo objeto; por ejemplo, para centrarlo).
  - Los botones también están en la pestaña Propiedades cuando hay varios objetos seleccionados.
- **Criterios de aceptación:**
  - [ ] Con tres pasos a distintas alturas, Alinear arriba los deja con el mismo borde superior que el más alto.
  - [ ] Con cuatro objetos, Distribuir horizontalmente deja el mismo espacio entre cada par y no mueve el primero ni el último.
  - [ ] Con un solo objeto, Centro horizontal lo centra en el lienzo.
- **Mejora sobre Snagit:** alineación y distribución generales en ambas plataformas.
- **Fuentes:** research/02-editor.md §8 (Align / distribute), §3.13, §15 (punto 3)

### EDT-084 · Ajuste magnético y guías
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Canvas Snapping (`Ctrl`/`Cmd`+`;`); líneas de ajuste visibles con más contraste
- **Comportamiento:**
  - **Imagen > Ajuste magnético** (casilla; por defecto: activado; `Ctrl`/`Cmd`+`;`).
  - Al mover o cambiar de tamaño un objeto, sus bordes y su centro se ajustan a:
    - los bordes y el centro del lienzo;
    - los bordes y centros de otros objetos;
    - el **mismo espacio** entre objetos, cuando hay tres o más alineados (decisión de diseño).
  - Distancia de captura: 6 px de pantalla, sea cual sea el zoom. Al ajustarse, aparece una **guía** de color de acento y, con el espacio igual, la medida.
  - Mantener `Ctrl`/`Cmd` mientras se arrastra desactiva el ajuste temporalmente (decisión de diseño).
  - Con el contraste aumentado del sistema activo, las guías se dibujan más gruesas (APP-045).
- **Criterios de aceptación:**
  - [ ] Arrastrar un objeto cerca del centro del lienzo lo ajusta al centro y muestra las guías.
  - [ ] Con el ajuste desactivado, el objeto se mueve sin saltos.
  - [ ] Mantener `Ctrl`/`Cmd` al arrastrar cerca de un borde no lo ajusta.
- **Fuentes:** research/02-editor.md §8 (Canvas Snapping), §12

### EDT-085 · Bloquear objetos
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** bloqueo de objetos (Mac; Windows lo ignora); Lock Objects de Simplify (Windows); candado de las zonas de plantilla (Mac)
- **Comportamiento:**
  - **Objeto > Bloquear** (`Ctrl`/`Cmd`+`L`) impide seleccionar, mover y editar el objeto con el ratón. Un candado pequeño aparece al pasar el puntero por encima.
  - Los objetos bloqueados no entran en Seleccionar todo, en el recuadro de selección ni en Seleccionar todos los de una herramienta.
  - Clic derecho sobre un objeto bloqueado > **Desbloquear**; **Objeto > Desbloquear todo** los desbloquea todos.
  - El bloqueo se guarda en el `.xshot` y se respeta en ambas plataformas.
- **Criterios de aceptación:**
  - [ ] Un objeto bloqueado no se mueve al arrastrarlo con Mover; arrastrar sobre él dibuja un recuadro de selección.
  - [ ] Un `.xshot` con un objeto bloqueado guardado en macOS lo mantiene bloqueado al abrirlo en Windows.
  - [ ] Desbloquear todo desbloquea todos los objetos en un solo paso deshacible.
- **Mejora sobre Snagit:** el bloqueo funciona en ambas plataformas y se conserva entre ellas.
- **Fuentes:** research/02-editor.md §8 (Lock objects), §3.13, §9 (Sections), §10.2

### EDT-086 · Posición y tamaño numéricos
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (Snagit no lo permite: posición y tamaño solo arrastrando)
- **Comportamiento:**
  - La sección **Posición y tamaño** de Propiedades (también **Objeto > Posición y tamaño…**) muestra, para el objeto seleccionado, el grupo o la selección múltiple:
    - **X** e **Y** (esquina superior izquierda del recuadro, en píxeles de la imagen);
    - **An** y **Al**, con un candado de proporción;
    - **Ángulo** (−180° a 180°).
  - Para Flecha y Línea muestra en su lugar las coordenadas de **inicio** y **fin** y la **longitud** (decisión de diseño).
  - Los campos aceptan expresiones simples: `+10`, `-5`, `*2`, `/2` (por ejemplo, escribir `+10` en X mueve 10 px a la derecha) (decisión de diseño).
  - También se usa para la selección de píxeles (EDT-043) y para el recuadro de Recortar (EDT-049).
  - `Tab` pasa de un campo a otro; `Enter` aplica; `Esc` descarta lo escrito.
- **Criterios de aceptación:**
  - [ ] Escribir X = 100, Y = 50, An = 300, Al = 200 deja un rectángulo exactamente en esa posición y con ese tamaño.
  - [ ] Escribir `*2` en An con el candado cerrado duplica el ancho y el alto.
  - [ ] Para una flecha, cambiar el fin a (400, 400) mueve solo la punta.
  - [ ] Un valor no válido (texto) se rechaza y el campo vuelve al valor anterior.
- **Mejora sobre Snagit:** función nueva que cubre un límite documentado de Snagit (SPEC §2.6).
- **Fuentes:** research/02-editor.md §8 (Exact position and size); research/05-plataformas-versiones.md §3 (Capture: exact numeric size not possible in the Editor)

### EDT-087 · Mover con el teclado
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** mover objetos con las flechas del teclado (implícito; paso no documentado)
- **Comportamiento:**
  - Con objetos seleccionados, las flechas los mueven **1 px** de la imagen; con `Shift`, **10 px** (decisión de diseño).
  - Con una selección de píxeles activa, las flechas mueven el contorno de la selección; con `Alt`/`Option`+flechas, mueven los píxeles seleccionados.
  - Mantener pulsada una flecha repite el movimiento. Una serie de pulsaciones seguidas cuenta como un solo paso para Deshacer.
  - Un objeto movido con el teclado puede quedar fuera del lienzo; la barra de estado lo avisa (EDT-011) y se puede seleccionar desde ahí.
- **Criterios de aceptación:**
  - [ ] Cinco pulsaciones de `→` mueven el objeto 5 px; `Shift`+`↓` lo mueve 10 px abajo.
  - [ ] Mantener `→` durante 2 s y soltar se deshace con un solo Deshacer.
  - [ ] Un objeto llevado fuera del lienzo con las flechas aparece en el aviso de la barra de estado y se puede volver a seleccionar.
- **Fuentes:** research/02-editor.md §8 (Nudging), §15 (punto 3)

### EDT-088 · Duplicar objetos
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** duplicar con `Ctrl`+clic (Move, Stamp, Step en Windows), `Option`+arrastrar o Control-clic > Duplicate (Mac)
- **Comportamiento:**
  - **Edición > Duplicar** (`Ctrl`/`Cmd`+`J`): crea una copia del objeto seleccionado desplazada 10 px abajo y a la derecha. Repetir el comando después de mover la copia repite el mismo desplazamiento (decisión de diseño).
  - `Alt`/`Option`+arrastrar un objeto con Mover deja la copia donde se suelta.
  - Clic derecho > **Duplicar**.
  - Con Sello o Paso activos, `Ctrl`/`Cmd`+clic sobre un objeto de esa herramienta coloca una copia en el punto del clic; en Paso, la copia recibe el valor siguiente de la secuencia (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] `Alt`/`Option`+arrastrar una flecha deja dos flechas idénticas: la original en su sitio y la copia donde se soltó.
  - [ ] Duplicar un grupo duplica todos sus objetos.
  - [ ] Duplicar, mover la copia 50 px a la derecha y duplicar otra vez coloca la tercera 50 px a la derecha de la segunda.
- **Fuentes:** research/02-editor.md §8 (Duplicate object), §3.6, §3.7, §4.1

### EDT-089 · Cortar, copiar y pegar
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Cut/Copy/Paste, Paste in Place (Mac), Paste and Match Style (Mac)
- **Comportamiento:**
  - **Copiar** (`Ctrl`/`Cmd`+`C`) con objetos seleccionados los copia como objetos editables de XcreenShot **y**, a la vez, como imagen PNG (con transparencia) para otras apps. Con una selección de píxeles, copia los píxeles (EDT-044).
  - **Cortar** (`Ctrl`/`Cmd`+`X`) copia y elimina.
  - **Pegar** (`Ctrl`/`Cmd`+`V`):
    - objetos de XcreenShot: se pegan como objetos editables, desplazados 10 px si el original sigue visible en ese documento; si no, centrados en la vista;
    - imagen de otra app: imagen flotante (EDT-078);
    - texto plano: un objeto Texto nuevo con el estilo actual, en el centro de la vista; si se está editando un texto, se inserta en él;
    - archivos: como EDT-105.
  - **Pegar en el mismo sitio** (`Ctrl`/`Cmd`+`Shift`+`V`): pega objetos en sus coordenadas originales, también en otro documento.
  - **Pegar con el mismo estilo** (`Ctrl`/`Cmd`+`Alt`/`Option`+`Shift`+`V`): mientras se edita un texto, pega el texto sin formato con el estilo del cuadro.
  - En M1: copiar, cortar y pegar objetos y texto plano, e imágenes como imagen base nueva si no hay documento. En M2: el resto.
- **Criterios de aceptación:**
  - [ ] Copiar una flecha y pegarla en otro documento la pega editable y con el mismo estilo.
  - [ ] Copiar una flecha y pegar en un procesador de textos inserta una imagen PNG de la flecha con fondo transparente.
  - [ ] Pegar en el mismo sitio en otro documento deja el objeto en las mismas coordenadas X, Y.
  - [ ] Pegar texto copiado de una web en un globo en edición lo inserta con la fuente y el color del globo.
- **Mejora sobre Snagit:** Pegar en el mismo sitio y Pegar con el mismo estilo en ambas plataformas.
- **Fuentes:** research/02-editor.md §8 (Cut / Copy / Paste), §3.3, §12

### EDT-090 · Copiar todo
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Copy All (copia la imagen aplanada)
- **Comportamiento:**
  - **Copiar todo** (botón de la zona de compartir; `Ctrl`/`Cmd`+`Shift`+`C`) copia al portapapeles la imagen final del documento: imagen base, objetos y efectos, aplanados, sin modificar el documento.
  - Se copia como PNG con transparencia y, a la vez, en el formato de imagen nativo del sistema, para que se pueda pegar en cualquier app. Las zonas transparentes se conservan donde el destino las admite.
  - Respeta la opción **Reducir las imágenes de alta densidad al compartir** (SHR- (escalado)).
  - Una notificación breve dentro de la ventana confirma "Imagen copiada" (decisión de diseño).
  - Las zonas de plantilla vacías no se copian (EDT-098).
- **Criterios de aceptación:**
  - [ ] Copiar todo con una flecha y un borde, y pegar en otra app, produce una imagen con la flecha y el borde.
  - [ ] Tras Copiar todo, el documento sigue con sus objetos editables.
  - [ ] Con un lienzo transparente, pegar en una app que admite transparencia conserva la transparencia.
- **Fuentes:** research/02-editor.md §1 (Share area), §8 (Cut / Copy / Paste), §12

### EDT-091 · Aplanar y desaplanar
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Flatten, Flatten All, Unflatten (Mac)
- **Comportamiento:**
  - **Aplanar** (`Ctrl`/`Cmd`+`E`) fusiona los objetos seleccionados con la imagen base; dejan de ser objetos. **Aplanar todo** (`Ctrl`/`Cmd`+`Shift`+`E`) fusiona todos.
  - **Desaplanar** devuelve a objeto editable algo aplanado en la sesión actual o en una anterior: el `.xshot` guarda, por cada aplanado, los objetos y los píxeles de antes, mientras la imagen base no se modifique por debajo con herramientas de píxeles (decisión de diseño). Clic derecho sobre la zona aplanada > **Desaplanar**, o **Objeto > Desaplanar todo**.
  - Excepciones (siempre irreversibles, porque su fin es eliminar información):
    - aplanar un **Desenfocar** (EDT-040);
    - aplanar con **Aplanar desenfoques y continuar** (EDT-040).
    El aplanado irreversible se avisa en el diálogo de confirmación.
  - Si después de aplanar se pintó, borró o rellenó en esa zona, Desaplanar ya no está disponible para ese aplanado y su tooltip explica el motivo.
- **Criterios de aceptación:**
  - [ ] Aplanar una flecha impide seleccionarla; Desaplanar la vuelve a hacer editable, también tras guardar y reabrir.
  - [ ] Aplanar un Desenfocar y guardar: Desaplanar no está disponible y el `.xshot` ya no contiene los píxeles originales de esa zona.
  - [ ] Tras aplanar una flecha y borrar con el Borrador sobre ella, Desaplanar aparece desactivado.
- **Mejora sobre Snagit:** Desaplanar en ambas plataformas y conservado en el archivo.
- **Fuentes:** research/02-editor.md §8 (Flatten / Flatten All), §3.12, §12

### EDT-092 · Deshacer y rehacer
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Undo/Redo de varios niveles
- **Comportamiento:**
  - **Deshacer** (`Ctrl`/`Cmd`+`Z`) y **Rehacer** (`Ctrl`+`Y` o `Ctrl`+`Shift`+`Z` en Windows; `Cmd`+`Shift`+`Z` en macOS).
  - Historial por documento, de al menos **100 pasos** (decisión de diseño), que se conserva al cambiar de documento en la misma sesión. No se conserva al cerrar el documento ni la app.
  - Todo lo que cambia el documento se puede deshacer: dibujar, mover, cambiar propiedades, efectos, recortes, franjas, cambios de tamaño y de lienzo, esquinas redondeadas, aplanar y Volver al original.
  - Los cambios continuos (arrastrar un deslizador, mover con el teclado) cuentan como un solo paso.
  - Los menús Edición muestran qué se va a deshacer ("Deshacer Mover flecha").
  - **Historial** (Ver > Historial; decisión de diseño, M2): un panel lista los pasos; un clic vuelve a ese punto.
  - En M1: deshacer y rehacer con al menos 100 pasos.
- **Criterios de aceptación:**
  - [ ] Dibujar 5 flechas y deshacer 5 veces deja el documento como estaba; rehacer 5 veces las recupera.
  - [ ] Cambiar de documento y volver mantiene el historial de deshacer del primero.
  - [ ] Arrastrar el deslizador de opacidad de 100 a 20 y deshacer devuelve 100 en un solo paso.
  - [ ] Deshacer 50 pasos en un documento de 10.000 × 30.000 px con 200 objetos tarda < 200 ms por paso (APP-050).
- **Mejora sobre Snagit:** panel de historial.
- **Fuentes:** research/02-editor.md §8 (Undo / Redo), §12

### EDT-093 · Cursor como objeto
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** cursor capturado como objeto vectorial independiente (mover, cambiar de tamaño, eliminar)
- **Comportamiento:**
  - Si la captura incluyó el cursor (CAP-047), este llega al editor como un objeto **Cursor** independiente, en su posición.
  - Se puede mover, cambiar de tamaño (manteniendo la proporción), eliminar o aplanar.
  - Propiedades: **Tamaño** (50–400 %), **Forma** (la capturada, o una de las formas estándar: flecha, mano, texto, cruz) y **Resalte** (ninguno, círculo amarillo, círculo de acento) (decisión de diseño).
  - En M1 el cursor llega aplanado si se capturó (decisión de diseño: el objeto Cursor es M2).
- **Criterios de aceptación:**
  - [ ] Una captura con cursor permite seleccionar el cursor con Mover y llevarlo a otro sitio sin dejar rastro debajo.
  - [ ] Eliminar el cursor deja la captura sin él.
  - [ ] Cambiar la forma a Mano muestra una mano en la misma posición.
- **Fuentes:** research/02-editor.md §8 (Cursor object)

### EDT-094 · Medición en vivo
- **Hito:** M5 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Live Measure (experimento de Mac: medición de distancias en píxeles)
- **Comportamiento:**
  - Con la herramienta **Medir** (en Más) activa:
    - al pasar el puntero sobre la imagen, se muestran las distancias en píxeles desde el puntero hasta los bordes del elemento visual que hay debajo (decisión de diseño);
    - con un objeto seleccionado, mantener `Alt`/`Option` y pasar el puntero por otro objeto muestra la distancia entre ambos;
    - arrastrar dibuja una regla temporal con su longitud, ángulo y ΔX/ΔY.
  - Clic derecho > **Convertir en cota** convierte la medida en un objeto Línea con extremos de barra y una etiqueta de texto con la medida (decisión de diseño).
  - Las medidas temporales nunca aparecen en la imagen exportada.
- **Criterios de aceptación:**
  - [ ] Arrastrar una regla de (10, 10) a (110, 10) muestra 100 px y 0°.
  - [ ] Con un rectángulo seleccionado y `Alt`/`Option` sobre otro rectángulo 40 px a su derecha, se muestra 40 px entre ellos.
  - [ ] Una regla convertida en cota se exporta como línea con su etiqueta; una no convertida no se exporta.
- **Mejora sobre Snagit:** disponible en ambas plataformas, como herramienta estable.
- **Fuentes:** research/02-editor.md §11 (Mac Experiments: Live Measure); research/05-plataformas-versiones.md §3 (Editor: Live Measure), §5

---

## 8. Plantillas

### EDT-095 · Nuevo desde plantilla
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Create > Image from Template; Combine in Template; Apply Template; filtro por número de imágenes; máximo de 10 imágenes
- **Comportamiento:**
  - Entradas:
    - **Crear > Nuevo desde plantilla…** (`Ctrl`/`Cmd`+`Shift`+`T`; EDT-116);
    - con varias capturas seleccionadas en la Bandeja reciente o en la Biblioteca: **Combinar en plantilla…** (el mismo atajo, o el menú contextual);
    - con un documento abierto: **Aplicar plantilla…**, que lo usa como primera imagen.
  - El **selector de plantillas** muestra miniaturas agrupadas por familia (EDT-096), las plantillas propias (EDT-101) y un campo de búsqueda.
  - Con N imágenes seleccionadas, el filtro **Para N imágenes** (activo por defecto) muestra solo las plantillas con al menos N zonas, o con número de zonas variable.
  - Máximo de **20 imágenes** por plantilla (decisión de diseño). Si se seleccionan más, se avisa y se usan las 20 primeras.
  - Al elegir una plantilla y pulsar **Crear**, se abre un documento nuevo con las imágenes colocadas en sus zonas, por el orden de selección, y el panel **Plantilla** (EDT-097) abierto.
  - El resultado es un documento `.xshot` normal, que se guarda en la biblioteca.
- **Criterios de aceptación:**
  - [ ] Con 4 capturas seleccionadas en la bandeja, Combinar en plantilla con una plantilla de pasos crea un documento con las 4 capturas en 4 zonas, en el orden de selección.
  - [ ] Con 6 imágenes seleccionadas, el filtro oculta las plantillas de 2 zonas.
  - [ ] Seleccionar 25 imágenes muestra el aviso y crea la plantilla con las 20 primeras.
- **Mejora sobre Snagit:** hasta 20 imágenes.
- **Fuentes:** research/02-editor.md §9 (Entry points), §1 (Create menu), §1.1 (Multi-select), §12

### EDT-096 · Plantillas incorporadas
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** familias de plantillas (pasos o instrucciones, línea de tiempo, comparación, antes y después; plantilla de pasos personalizada)
- **Comportamiento:**
  - Plantillas incorporadas, de diseño propio (decisión de diseño):
    - **Pasos**: "Pasos en columna" (título, cada imagen con número y descripción debajo; zonas variables), "Pasos en cuadrícula 2×N", "Pasos con texto lateral" (imagen a la izquierda y texto a la derecha), "Pasos personalizados" (una sola columna que crece con cada imagen);
    - **Línea de tiempo**: "Horizontal" (de 3 a 6 hitos sobre una línea, con fecha y texto) y "Vertical";
    - **Comparación**: "Lado a lado" (2 zonas con encabezados), "Antes y después" (2 zonas con etiquetas Antes y Después y una flecha entre ellas), "Tres opciones" (3 columnas);
    - **Collage**: "Destacada y miniaturas" (1 grande y 3 pequeñas), "Cuadrícula" (2×2 o 3×3).
  - Todas usan el tema activo (EDT-017) para sus colores y su fuente.
- **Criterios de aceptación:**
  - [ ] Cada familia aparece en el selector con al menos dos plantillas.
  - [ ] "Antes y después" con 2 imágenes coloca la primera bajo "Antes" y la segunda bajo "Después".
  - [ ] Con el tema Sobrio activo, una plantilla nueva usa sus grises y su fuente.
- **Fuentes:** research/02-editor.md §9 (Template families), §15 (punto 2)

### EDT-097 · Ajustes de la plantilla
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Template Settings (escala de la imagen, pie con autor, título, leyendas, tema, fuente y color, fondo, numerar imágenes con color, invertir el orden)
- **Comportamiento:**
  - El panel **Plantilla** (pestaña adicional del panel lateral mientras el documento es una plantilla) tiene:
    - **Escala de la imagen**: **Llenar la zona** (puede recortar) o **Encajar entera** (puede dejar márgenes);
    - **Título** (texto; sí/no);
    - **Leyendas** de cada imagen (sí/no);
    - **Pie de página** (sí/no) con **Autor**: Nombre del sistema (el nombre visible de la cuenta del usuario) o Personalizado;
    - **Tema**, **Fuente**, **Color del texto** y **Fondo** (color, degradado o transparente);
    - **Numerar imágenes** (sí/no), que añade objetos Paso (EDT-035) a cada zona, con **Color** y **Tipo**;
    - **Invertir el orden** de las imágenes.
  - Los cambios se aplican al momento y se pueden deshacer.
- **Criterios de aceptación:**
  - [ ] Cambiar de Encajar entera a Llenar la zona hace que las imágenes ocupen toda su zona.
  - [ ] Activar Numerar imágenes añade un paso 1, 2, 3… en cada zona con imagen, en orden.
  - [ ] Invertir el orden con 3 imágenes pone la tercera en la primera zona y renumera los pasos.
  - [ ] El pie con Autor = Nombre del sistema muestra el nombre visible de la cuenta.
- **Fuentes:** research/02-editor.md §9 (Template Settings)

### EDT-098 · Zonas de imagen
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** drop zones (arrastrar o pegar, reposicionar, zoom en pasos, ajuste a lados y centro, arrastrar para intercambiar, eliminar imagen o leyenda, botón Capture en la zona vacía, zonas vacías omitidas; formas sobre las zonas)
- **Comportamiento:**
  - Cada **zona** es un marco que recorta la imagen que contiene.
  - Para llenarla: arrastrar una imagen de la bandeja, un archivo o una imagen flotante; pegar con la zona seleccionada; o el botón **Capturar** de la zona vacía, que lanza una captura (CAP-001) y pone el resultado en ella.
  - **Encuadrar**: doble clic en la zona para mover la imagen dentro de ella; se ajusta a los bordes y al centro de la zona. Botones **+** y **−** de zoom en pasos del 10 %, y **Restablecer encuadre**.
  - **Intercambiar**: arrastrar una imagen a otra zona intercambia las dos imágenes con sus leyendas.
  - Clic derecho: **Quitar imagen**, **Quitar leyenda**, **Reemplazar imagen…**.
  - Se pueden dibujar anotaciones encima de las zonas; quedan por encima de las imágenes.
  - Las zonas **vacías** no aparecen al guardar como imagen, copiar, exportar ni imprimir (con su leyenda y su número). El `.xshot` sí las conserva.
- **Criterios de aceptación:**
  - [ ] Arrastrar la imagen de la zona 1 a la zona 3 intercambia ambas imágenes y sus leyendas.
  - [ ] El botón Capturar de una zona vacía, tras capturar una región, la llena con esa captura.
  - [ ] Copiar todo con 2 zonas vacías produce una imagen sin esas zonas ni sus números.
  - [ ] Dos pulsaciones de + en una zona amplían la imagen un 20 %.
- **Fuentes:** research/02-editor.md §9 (Drop zones); research/05-plataformas-versiones.md §1A (2025.0.0), §1B (2025.1.0)

### EDT-099 · Secciones
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** añadir sección con tipo de contenido, redimensionar con el divisor, eliminar con ícono; los objetos se recolocan; los pasos se renumeran
- **Comportamiento:**
  - Una plantilla se compone de **secciones** apiladas (título, pasos, imagen con texto, pie…).
  - **Añadir sección** (botón + entre secciones) muestra los tipos disponibles con su nombre y su miniatura: Imagen, Imagen con texto, Texto, Dos imágenes, Título y Pie.
  - **Cambiar el tamaño**: arrastrar el divisor entre dos secciones; los objetos de la sección se recolocan para seguir en su sitio relativo.
  - **Eliminar** una sección: ícono de papelera al pasar el puntero.
  - **Reordenar** secciones arrastrando su asa (decisión de diseño).
  - Los textos y las zonas se editan directamente, sin desbloquear antes (decisión de diseño: se adopta el modelo sin candado). Las anotaciones que el usuario añade se anclan a la sección donde se dibujaron y se mueven con ella.
  - Al añadir, quitar o reordenar secciones de pasos, los números se actualizan.
- **Criterios de aceptación:**
  - [ ] Añadir una sección Imagen con texto entre los pasos 2 y 3 la numera como 3 y el antiguo 3 pasa a 4.
  - [ ] Arrastrar el divisor hace la sección más alta, y una flecha dibujada en ella se mueve con su sección.
  - [ ] Eliminar una sección se puede deshacer.
- **Fuentes:** research/02-editor.md §9 (Sections), §3.7 (renumber); research/05-plataformas-versiones.md §1A (2025.0.0, 2025.2.0)

### EDT-100 · Aplicar un tema a objetos existentes
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** aplicar los colores del tema a los objetos de la plantilla
- **Comportamiento:**
  - En cualquier documento (no solo en plantillas), **Aplicar tema…** (menú ⋯ de la pestaña Estilos) cambia los colores y la fuente de los objetos seleccionados, o de todos si no hay selección, a los del tema elegido.
  - Cada color se sustituye por el color del nuevo tema que ocupa la misma posición en el tema original. Los colores que no son del tema se sustituyen por el más parecido del nuevo tema (decisión de diseño).
  - También se puede seleccionar un objeto, elegir un tema en la pestaña Estilos y hacer clic en un estilo o en un color del tema.
  - Se puede deshacer en un paso.
- **Criterios de aceptación:**
  - [ ] Aplicar el tema Sobrio a una plantilla hecha con Base cambia todos sus colores a los de Sobrio en un paso.
  - [ ] Con dos globos seleccionados, aplicar un tema solo los cambia a ellos.
- **Fuentes:** research/02-editor.md §9 (Apply theme colors to template objects), §2.4

### EDT-101 · Plantillas propias `.xtemplate`
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** guardar como `.snagx` para reutilizar como plantilla propia; plantillas de TechSmith Assets (sustituidas por paquetes locales)
- **Comportamiento:**
  - **Archivo > Guardar como plantilla…** guarda el documento actual como `.xtemplate` (SPEC §7) en la carpeta de plantillas del usuario. Pide nombre, familia (una de EDT-096 o "Mis plantillas") y si se vacían las zonas (por defecto: sí).
  - Cualquier imagen flotante se puede marcar como **zona** (clic derecho > **Convertir en zona de imagen**), para crear plantillas desde cero (decisión de diseño).
  - Las plantillas propias aparecen en el selector (EDT-095), con **Renombrar**, **Duplicar**, **Eliminar**, **Exportar…** e **Importar…**.
  - Importar un `.xtemplate` (Archivo > Importar > Plantilla…, doble clic o arrastrar) lo añade a Mis plantillas. Las fuentes que falten se sustituyen como en EDT-018.
- **Criterios de aceptación:**
  - [ ] Guardar como plantilla un documento con 3 zonas y abrirla desde el selector con 3 imágenes las coloca en esas zonas.
  - [ ] Una plantilla exportada en macOS e importada en Windows produce el mismo diseño.
  - [ ] Convertir una imagen flotante en zona hace que, en la plantilla guardada, aparezca vacía y con el botón Capturar.
- **Mejora sobre Snagit:** formato de plantilla propio, abierto e intercambiable.
- **Fuentes:** research/02-editor.md §9 (Output), §1 (Assets entry)

### EDT-102 · Paginación al imprimir y en PDF
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** saltos de página o división en varias páginas de las plantillas al imprimir o pasar a PDF; paginación de capturas con desplazamiento a PDF (Mac 2026.4.0)
- **Comportamiento:**
  - Al imprimir (EDT-114) o exportar a PDF (EDT-111) una imagen más alta que la página, XcreenShot la reparte en varias páginas:
    - en una **plantilla**, los saltos van **entre secciones**, nunca a mitad de una sección;
    - en una imagen normal (por ejemplo, una captura con desplazamiento), los saltos se buscan en franjas horizontales de color uniforme cercanas al final de cada página, para no cortar líneas de texto; si no hay, se corta al final de la página (decisión de diseño).
  - Opciones: **Tamaño de página** (A4, Carta, Legal, A3 o Personalizado), **Orientación**, **Márgenes**, **Ajustar al ancho de la página** (por defecto: sí) y **Una sola página** (reduce la imagen para que quepa entera).
  - Las zonas vacías se omiten (EDT-098).
- **Criterios de aceptación:**
  - [ ] Una captura con desplazamiento de 1200 × 9000 px exportada a PDF A4 ajustada al ancho da varias páginas y ninguna línea de texto queda cortada por la mitad en un documento de prueba con interlineado normal.
  - [ ] Una plantilla de 8 pasos impresa en Carta no corta ningún paso entre dos páginas.
  - [ ] Con Una sola página, el PDF tiene exactamente una página.
- **Fuentes:** research/02-editor.md §9 (Output), §10.3 (multi-page PDF), §10.5; research/05-plataformas-versiones.md §1B (2026.4.0); research/01-captura.md (vía CAP-036)

---

## 9. Archivos

### EDT-103 · Nueva imagen
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** New Image (lienzo en blanco); tamaño y color por defecto de una imagen nueva (Mac)
- **Comportamiento:**
  - **Archivo > Nueva imagen…** (`Ctrl`/`Cmd`+`N`) abre un diálogo con:
    - **Ancho** y **Alto** (en las unidades de EDT-069);
    - **Tamaños rápidos** (decisión de diseño): 1920 × 1080, 1280 × 720, 1080 × 1080, 1200 × 630, A4 a 150 PPP, Carta a 150 PPP y **Igual que el portapapeles** (si hay una imagen en él);
    - **Color del lienzo** (color, degradado o Transparente);
    - **Resolución** (PPP).
  - Los valores iniciales salen de la preferencia **Tamaño y color de una imagen nueva** (`editor.newImage.width`, `editor.newImage.height`, `editor.newImage.color`; por defecto: 1280 × 720, blanco). **Recordar como valores por defecto** (casilla del diálogo) los guarda.
  - El documento nuevo se guarda en la biblioteca como cualquier captura, con origen "Imagen nueva" en sus metadatos.
- **Criterios de aceptación:**
  - [ ] Nueva imagen con 800 × 600 y Transparente crea un lienzo de 800 × 600 con damero.
  - [ ] Con Recordar como valores por defecto marcado, el diálogo siguiente propone los mismos valores.
  - [ ] Con una imagen de 640 × 480 en el portapapeles, Igual que el portapapeles rellena 640 × 480.
- **Fuentes:** research/02-editor.md §10.1 (New Image), §11

### EDT-104 · Nueva desde el portapapeles
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** New from Clipboard
- **Comportamiento:**
  - **Archivo > Nueva desde el portapapeles** (`Ctrl`/`Cmd`+`Shift`+`N`) crea un documento con la imagen del portapapeles como imagen base, con su transparencia, y lo guarda en la biblioteca con origen "Portapapeles".
  - Si el portapapeles contiene un archivo de imagen (copiado en el Explorador o el Finder), se usa ese archivo. Si contiene texto, se crea un lienzo ajustado al texto, con un objeto Texto (decisión de diseño).
  - Si el portapapeles no tiene imagen, archivo ni texto, el comando aparece desactivado.
  - También se puede añadir a la barra (EDT-004). Como fuente de captura programable, ver CAP-039.
- **Criterios de aceptación:**
  - [ ] Tras copiar una imagen en un navegador, Nueva desde el portapapeles abre un documento con esa imagen y su tamaño.
  - [ ] Con un PNG transparente copiado desde el Finder o el Explorador, el documento conserva la transparencia.
  - [ ] Con el portapapeles vacío, la entrada del menú aparece desactivada.
- **Fuentes:** research/02-editor.md §10.1 (New from Clipboard), §2.1

### EDT-105 · Abrir, arrastrar y soltar archivos
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Open; arrastrar desde el gestor de archivos o un navegador; SVG no admitido; arrastrar hacia otras apps
- **Comportamiento:**
  - **Archivo > Abrir…** (`Ctrl`/`Cmd`+`O`) admite varios archivos a la vez. **Abrir recientes ▸** lista los 10 últimos.
  - Formatos que se abren como documento:
    - `.xshot` y `.xtemplate` (este último abre el selector de plantillas, EDT-101);
    - PNG, JPG/JPEG, GIF (primer fotograma; un GIF animado se abre en el visor de video, `VID-`), BMP, TIFF (si tiene varias páginas, se pregunta si abrir cada página como documento o solo la primera), WebP, HEIC/HEIF, ICO (el tamaño mayor) y PSD (la imagen compuesta, sin capas);
    - **SVG**: se convierte en imagen al abrir, con un diálogo que pide el tamaño (por defecto, el del archivo, o 1024 px de lado mayor si no tiene tamaño). Mejora sobre Snagit (decisión de diseño);
    - **PDF**: lo importa CAP-041.
  - **Soltar** archivos sobre la ventana:
    - sobre el lienzo de un documento abierto: se añaden como imágenes flotantes (EDT-078);
    - sobre la Bandeja reciente, sobre el estado vacío o con `Shift` pulsado: se abren como documentos nuevos;
    - `.xtheme`, `.xstamps` y `.xtemplate` se importan (EDT-018, EDT-034, EDT-101).
    También se aceptan imágenes arrastradas desde un navegador (se descarga la imagen original si el navegador la ofrece; si no, la vista que se arrastra).
  - **Arrastrar hacia fuera**: la barra de estado tiene un asa **Arrastrar** que, arrastrada a otra app o a una carpeta, entrega la imagen final en el formato configurado en SHR- (arrastrar y soltar) (decisión de diseño). Las miniaturas de la bandeja hacen lo mismo (TRY-).
  - Las imágenes de fuera de la biblioteca se editan en su sitio: se ven en la bandeja y se guardan en su propio archivo (EDT-107). Un archivo que no se puede leer muestra un mensaje con el motivo y no abre nada.
  - En M1: abrir `.xshot`, PNG y JPG con el diálogo y soltándolos. En M2: el resto.
- **Criterios de aceptación:**
  - [ ] Abrir tres PNG a la vez los muestra en la bandeja y deja visible el último.
  - [ ] Soltar un PNG sobre el lienzo con un documento abierto crea una imagen flotante; soltarlo con `Shift` lo abre como documento.
  - [ ] Abrir un SVG con el tamaño 2000 px crea un documento cuyo lado mayor mide 2000 px.
  - [ ] Arrastrar el asa Arrastrar a una carpeta crea un archivo con la imagen final en el formato configurado.
  - [ ] Abrir un archivo dañado muestra "No se puede abrir…" con el motivo y la app sigue funcionando.
- **Mejora sobre Snagit:** apertura de SVG.
- **Fuentes:** research/02-editor.md §10.1, §1.1 (drag a tray item), §10.3 (drag-out file type); research/04-biblioteca-compartir.md §6 (Drag in, Drag out)

### EDT-106 · Formato `.xshot`
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** `.snagx` (formato editable multiplataforma que conserva objetos, efectos, historial de recorte y metadatos)
- **Comportamiento:**
  - Formato nativo de los documentos, definido en SPEC §7: ZIP con `manifest.json`, `document.json`, `base.png`, `images/*.png`, `meta.json` y `thumb.png`.
  - `document.json` guarda: tamaño, color y radio del lienzo; resolución; historial de recortes y franjas; la lista ordenada de objetos con todas sus propiedades (incluidos grupos, bloqueos, curvas, colas, degradados, sombras y datos de desaplanado); la pila de efectos del documento y de cada imagen flotante; los datos de plantilla (secciones y zonas) y el tipo (`image` o `guide`; el tipo `guide` lo define SMT- (Grabador de pasos)).
  - **Sin pérdidas entre plataformas:** un `.xshot` guardado en Windows se abre en macOS con todas sus propiedades, y viceversa. La única diferencia admitida son las fuentes que no estén instaladas: se sustituyen al mostrar, pero el archivo conserva la fuente original y la vuelve a usar donde exista.
  - **Versiones:** `manifest.json` lleva la versión del formato. Una versión anterior se migra al abrir. Una versión posterior con la misma versión mayor se abre y conserva los campos desconocidos al guardar; una versión mayor posterior se rechaza con un mensaje que pide actualizar XcreenShot.
  - **Guardado seguro:** se escribe en un archivo temporal y se sustituye el anterior solo al terminar, para que un fallo no deje un archivo dañado.
  - `thumb.png` (lado mayor de 512 px) permite que la biblioteca y el sistema muestren miniaturas.
  - El formato está documentado en el repositorio (esquema JSON de `document.json`), para que otras herramientas lo puedan leer.
- **Criterios de aceptación:**
  - [ ] Un `.xshot` con un objeto de cada herramienta y tres efectos, guardado en Windows y abierto en macOS, produce una exportación PNG idéntica píxel a píxel en ambas plataformas, salvo el suavizado del texto.
  - [ ] Un `.xshot` se abre con una herramienta ZIP estándar y su `document.json` valida contra el esquema publicado.
  - [ ] Cortar la corriente (o matar el proceso) durante un guardado deja el archivo anterior intacto.
  - [ ] Un `.xshot` con una versión mayor futura muestra el mensaje y no se abre.
- **Mejora sobre Snagit:** sin pérdida de propiedades entre plataformas; formato abierto y documentado.
- **Fuentes:** research/02-editor.md §10.2

### EDT-107 · Guardar, Guardar como y Guardar todo
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Save, Save As, Save All (Mac), File > Export (Mac); formato por defecto (Windows); comportamiento tras guardar en otro formato
- **Comportamiento:**
  - **Guardar** (`Ctrl`/`Cmd`+`S`) escribe el documento en su archivo:
    - documentos de la biblioteca: en su `.xshot` (el autoguardado de EDT-012 hace lo mismo);
    - imágenes externas (PNG, JPG…): en ese archivo, aplanado. La primera vez que un archivo de formato plano tiene objetos, un aviso explica que en ese archivo se aplanarán y que la versión editable se conserva según LIB- (guardar como). El aviso tiene "No volver a preguntar".
  - **Guardar como…** (`Ctrl`/`Cmd`+`Shift`+`S`) elige nombre, carpeta y formato (`.xshot` o cualquiera de EDT-110 y EDT-111), con el botón **Opciones…** del formato (EDT-112). El documento pasa a estar vinculado al archivo nuevo.
  - **Exportar…** (`Ctrl`/`Cmd`+`Alt`/`Option`+`Shift`+`S`) escribe una **copia** en otro formato, con las ediciones sin guardar, sin cambiar el archivo al que está vinculado el documento.
  - **Guardar todo** (`Ctrl`/`Cmd`+`Alt`/`Option`+`S`) guarda todos los documentos abiertos con cambios.
  - Tras guardar como un formato plano, el documento sigue siendo editable durante la sesión. Si se conserva o no una copia `.xshot` editable lo decide LIB- (guardar como) (decisión de diseño: se adopta el comportamiento de "seguir editable", y la copia editable se controla con un ajuste).
  - **Formato por defecto** de Guardar como y Exportar (`editor.defaultSaveFormat`): **Recordar el último** (por defecto) o un formato fijo.
  - Nombre propuesto: el del documento; para capturas, el que fije CAP-053.
  - En M1: Guardar, Guardar como (`.xshot`, PNG, JPG) y Exportar a PNG y JPG. En M2: Guardar todo y el resto de formatos.
- **Criterios de aceptación:**
  - [ ] Guardar como PNG una captura con una flecha crea un PNG con la flecha, y la flecha sigue siendo editable en el editor.
  - [ ] Exportar a JPG no cambia el archivo vinculado: el título de la ventana sigue mostrando el `.xshot`.
  - [ ] Con dos documentos externos modificados, Guardar todo guarda los dos y quita sus indicadores de cambios sin guardar.
  - [ ] Con el formato por defecto en Recordar el último, tras exportar a WebP el siguiente Exportar propone WebP.
- **Fuentes:** research/02-editor.md §10.3, §1.1 (both files in the tray), §11

### EDT-108 · Cerrar y Cerrar todo
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Close, Close All (Mac), Close Other Captures (Mac)
- **Comportamiento:**
  - **Cerrar** (`Ctrl`/`Cmd`+`W`) cierra el documento actual: lo guarda si es de la biblioteca (EDT-012), o pregunta si es externo y tiene cambios. Lo quita de la Bandeja reciente sin borrar el archivo (TRY- (Bandeja reciente)) y muestra el documento siguiente de la bandeja o, si no hay, el estado vacío.
  - **Cerrar todo** (`Ctrl`/`Cmd`+`Alt`/`Option`+`W`) y **Cerrar los demás** (menú contextual de la bandeja) hacen lo mismo con varios documentos. Si hay varios externos con cambios, un solo diálogo los lista con casillas: **Guardar seleccionados**, **No guardar** y **Cancelar**.
  - Cerrar la ventana no cierra los documentos: siguen en la bandeja al volver a abrirla (APP-004).
- **Criterios de aceptación:**
  - [ ] Cerrar una captura la quita de la bandeja y su archivo sigue en la biblioteca.
  - [ ] Cerrar todo con dos PNG externos modificados muestra un solo diálogo con los dos.
  - [ ] Cancelar en ese diálogo no cierra ningún documento.
- **Fuentes:** research/02-editor.md §10.3 (Other file commands), §1.1 (Close)

### EDT-109 · Renombrar, duplicar, eliminar y mostrar
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Rename (F2, File > Rename), Duplicate, Delete / Move to Trash, Reveal in Library (Mac)
- **Comportamiento:**
  - **Renombrar…** (`F2` en Windows; en macOS desde el menú o con el atajo que asigne el usuario) cambia el nombre del archivo en el disco. Conserva la extensión, rechaza caracteres no válidos en cualquiera de las dos plataformas (`\ / : * ? " < > |`) y avisa si ya existe un archivo con ese nombre.
  - **Duplicar** (`Ctrl`/`Cmd`+`Shift`+`D`) crea una copia del documento, con todas sus ediciones, en la misma carpeta, con el sufijo " (copia)", y la abre. Con varias miniaturas seleccionadas en la bandeja, duplica todas (TRY-).
  - **Mover a la papelera** (`Ctrl`/`Cmd`+`Supr`/`Retroceso`) envía el archivo a la papelera del sistema y lo quita de la bandeja y de la biblioteca. La confirmación la define LIB- (eliminar) (decisión de diseño: siempre a la papelera, nunca borrado definitivo desde el editor).
  - **Mostrar en la biblioteca** cambia a la vista Biblioteca con el documento seleccionado (`LIB-`). **Mostrar en el Explorador/Finder** abre la carpeta con el archivo seleccionado.
- **Criterios de aceptación:**
  - [ ] Renombrar a "informe:final" se rechaza indicando el carácter no válido.
  - [ ] Duplicar un documento con una flecha crea "<nombre> (copia).xshot" con la flecha editable.
  - [ ] Mover a la papelera deja el archivo en la papelera del sistema, desde donde se puede restaurar.
- **Fuentes:** research/02-editor.md §10.3 (Other file commands), §1.1 (Rename, Duplicate, Delete)

### EDT-110 · Exportar a PNG y JPG
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Save As PNG y JPG (extensión `.jpg`); transparencia en PNG
- **Comportamiento:**
  - PNG y JPG son los formatos de exportación de M1 y los que se usan por defecto al compartir (SHR- (formato)).
  - Toda exportación produce la **imagen final**: imagen base, objetos, efectos y recortes, aplanados; sin zonas de plantilla vacías (EDT-098) ni elementos de la interfaz (tiradores, guías, cuadrícula).
  - **PNG**: conserva la transparencia; guarda la resolución (EDT-077) y un perfil de color sRGB (o el de la captura, si es de gama amplia; decisión de diseño).
  - **JPG**: extensión `.jpg`; las zonas transparentes se rellenan con el color de fondo de EDT-112; guarda la resolución y el perfil de color.
  - Los metadatos de captura no se escriben en el archivo, salvo con la opción de CAP-053.
  - Si el destino existe, se pregunta **Reemplazar** o **Conservar ambos**.
- **Criterios de aceptación:**
  - [ ] Exportar a PNG una captura con lienzo transparente y una flecha da un PNG con transparencia y la flecha.
  - [ ] Exportar a JPG la misma captura da un `.jpg` con fondo blanco donde era transparente.
  - [ ] El PNG exportado de una captura Retina declara 144 PPP o más (según EDT-077).
- **Fuentes:** research/02-editor.md §10.3 (Save As and export formats, transparency)

### EDT-111 · Más formatos de exportación
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** formatos de Save As de Windows y Mac (BMP, GIF, HEIF/HEIC, ICO, PDF, PSD, TIFF, WebP…); TIFF y PDF de varias páginas
- **Comportamiento:**
  - Formatos (decisión de diseño: la unión útil de ambas plataformas):

    | Formato | Transparencia | Varias páginas | Notas |
    |---|---|---|---|
    | PNG | Sí | No | EDT-110 |
    | JPG | No | No | EDT-110 |
    | GIF | Sí (1 color) | No | Imagen estática, paleta de hasta 256 colores |
    | BMP | No | No | 24 bits |
    | TIFF | Sí | Sí | |
    | WebP | Sí | No | Con o sin pérdida |
    | HEIC | Sí | No | Ver Notas de plataforma |
    | PDF | No (fondo de página) | Sí | Paginación según EDT-102 |
    | ICO | Sí | Varios tamaños en un archivo | 16 a 256 px |
    | PSD | Sí | No | Una sola capa con la imagen final |

  - **Varios documentos en un archivo:** con varias miniaturas seleccionadas en la bandeja o la biblioteca, **Exportar…** a PDF o TIFF ofrece **Un archivo con una página por imagen**.
  - Los formatos se pueden elegir en Guardar como, Exportar, la exportación por lotes (EDT-113), el destino Archivo de la captura (CAP-053) y los destinos de compartir que admiten formato (`SHR-`).
- **Criterios de aceptación:**
  - [ ] Exportar a cada formato de la tabla produce un archivo que abre correctamente el visor de imágenes del sistema (o el lector de PDF) en ambas plataformas, salvo HEIC en Windows sin el códec.
  - [ ] Exportar tres documentos a un solo PDF crea un PDF de al menos tres páginas, en el orden de selección.
  - [ ] Un ICO exportado con 16, 32 y 256 px contiene los tres tamaños.
- **Notas de plataforma:** en Windows, HEIC necesita la extensión de imágenes HEIF del sistema. Si falta, HEIC aparece desactivado y su tooltip explica cómo instalarla.
- **Mejora sobre Snagit:** los mismos formatos en ambas plataformas.
- **Fuentes:** research/02-editor.md §10.3 (Save As and export formats, multi-page TIFF and PDF)

### EDT-112 · Opciones de cada formato
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** opciones de Save As según el formato (Windows); compresión TIFF (Mac); calidad JPG (no documentada)
- **Comportamiento:**
  - El botón **Opciones…** del diálogo de guardar o exportar muestra las opciones del formato elegido. Se recuerdan por formato. Opciones (decisión de diseño):
    - **JPG**: Calidad 1–100 (por defecto 90); Progresivo (por defecto: no);
    - **PNG**: Compresión Rápida o Máxima (por defecto: Rápida); Reducir a 256 colores (por defecto: no);
    - **GIF**: Tramado sí/no; Color transparente (ninguno o el del lienzo);
    - **WebP**: Sin pérdida sí/no (por defecto: sí); Calidad 1–100 si hay pérdida (por defecto 85);
    - **HEIC**: Calidad 1–100 (por defecto 85);
    - **TIFF**: Compresión Ninguna, LZW, ZIP o JPEG (por defecto LZW);
    - **PDF**: tamaño de página, orientación, márgenes, Ajustar al ancho y Una sola página (EDT-102); Calidad de imagen (sin pérdida o JPG 1–100);
    - **ICO**: casillas para 16, 24, 32, 48, 64, 128 y 256 px;
    - **Todos**: **Escala** (25–400 %; por defecto 100 %) e **Incluir el perfil de color**.
  - Preferencia **Fondo para formatos sin transparencia** (`editor.flattenBackground`; por defecto: blanco) para JPG, BMP y PDF.
  - Cada opción muestra el tamaño estimado del archivo, que se recalcula al cambiarla.
- **Criterios de aceptación:**
  - [ ] Un JPG con Calidad 50 pesa claramente menos que con Calidad 95 para la misma imagen.
  - [ ] PNG con Reducir a 256 colores produce un PNG indexado.
  - [ ] Las opciones elegidas para TIFF se conservan la próxima vez que se exporta a TIFF.
- **Fuentes:** research/02-editor.md §10.3 (Per-format options), §15 (punto 4)

### EDT-113 · Exportación por lotes
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Batch Conversion Wizard (Windows: archivos, efectos, carpeta, formato, nombres con prefijo y numeración); Batch Export (Mac: efectos con engranaje, giro, formato, nombre base con sufijo, mostrar en el Finder)
- **Comportamiento:**
  - **Archivo > Exportar por lotes…**, o con varias capturas seleccionadas en la bandeja o la biblioteca (`LIB-`), abre un diálogo de una sola ventana (decisión de diseño) con cinco apartados:
    1. **Archivos**: los seleccionados, más **Añadir archivos…** y **Añadir carpeta…** (`.xshot` y cualquier formato de EDT-105). Se pueden quitar o reordenar.
    2. **Efectos**: ninguno, un efecto guardado (EDT-054) o una pila propia, con las transformaciones de EDT-068 (escalar, girar, recortar bordes, esquinas).
    3. **Formato**: cualquiera de EDT-110 y EDT-111, con sus opciones (EDT-112).
    4. **Destino**: una carpeta, o **Junto a cada original**.
    5. **Nombres**: Conservar el nombre original, o **Nombre base** con numeración (inicio, dígitos, prefijo y sufijo), con una vista previa de los tres primeros nombres. **Si el archivo existe**: Conservar ambos (por defecto), Reemplazar u Omitir.
  - **Exportar** muestra el progreso con **Cancelar**. Al terminar, un resumen dice cuántos archivos se exportaron, cuántos fallaron (con el motivo) y ofrece **Mostrar en el Explorador/Finder**.
  - Los documentos de origen no se modifican.
  - El último ajuste usado se recuerda. Desde el CLI: `xcreenshot document batch-export` (EDT-119).
- **Criterios de aceptación:**
  - [ ] Exportar por lotes 10 capturas a JPG con el efecto guardado "Marco fino" y el nombre base "guia" crea guia_01.jpg … guia_10.jpg con el marco.
  - [ ] Con Si el archivo existe = Omitir, una segunda exportación no crea ni modifica archivos y el resumen lo indica.
  - [ ] Cancelar a mitad deja los archivos ya exportados y no crea archivos incompletos.
  - [ ] Un archivo dañado en la lista aparece en el resumen como fallido y no detiene el resto.
- **Fuentes:** research/02-editor.md §10.4, §6 (effects in batch export); research/04-biblioteca-compartir.md §1 (Batch Export)

### EDT-114 · Imprimir y configurar página
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Print, Page Setup (Mac), Print Selection (Mac), paginación de plantillas, recordar el tamaño de papel (Mac)
- **Comportamiento:**
  - **Imprimir…** (`Ctrl`/`Cmd`+`P`) abre una vista previa propia con:
    - **Tamaño**: Ajustar a la página (por defecto), Tamaño real (según la resolución, EDT-077) o Porcentaje;
    - **Posición**: centrada o arriba a la izquierda;
    - **Páginas**: la paginación de EDT-102 si no cabe en una página;
    - **Imprimir**: Documento completo o Selección (si hay una selección de píxeles; EDT-044).
    El botón **Imprimir** pasa al diálogo de impresión del sistema.
  - **Configurar página…** (macOS, `Cmd`+`Shift`+`P`; en Windows, dentro de la vista previa): tamaño de papel, orientación y márgenes. Se recuerdan entre sesiones.
  - Lo que se imprime es la imagen final, como al exportar (EDT-110).
- **Criterios de aceptación:**
  - [ ] Imprimir una captura de 1920 × 1080 con Ajustar a la página en A4 apaisado la imprime entera en una página.
  - [ ] Con una selección, Imprimir > Selección imprime solo esa zona.
  - [ ] El tamaño de papel elegido se conserva tras reiniciar la app.
- **Mejora sobre Snagit:** imprimir la selección en ambas plataformas.
- **Fuentes:** research/02-editor.md §10.5, §4.2 (Print Selection), §9 (Output)

---

## 10. Atajos de teclado

Reglas comunes a las tres fichas de esta sección:
- Todos los atajos del editor son **de la app** (APP-017): solo funcionan con la ventana principal en primer plano. Por eso no chocan con Snagit aunque las dos apps convivan (D-08), y se pueden reasignar en Preferencias > Atajos (APP-018).
- Ningún atajo de esta sección actúa mientras se escribe en un texto, un globo o un campo; en ese caso las teclas van al texto. Esto evita, en Windows, que `AltGr` (que el sistema trata como `Ctrl`+`Alt`) dispare un atajo al escribir caracteres como `€` o `@`.
- **Perfiles** (APP-021): la columna Windows/macOS es el perfil **Coexistencia** (esquema propio de XcreenShot, por defecto). La columna **Compatibilidad con Snagit** indica solo los valores que cambian en ese perfil, para conservar la memoria muscular; "=" significa que el valor es el mismo. En ese perfil, un atajo propio que coincida con uno de Snagit asignado a otra acción queda sin asignar.
- Atajos reservados para otros archivos, que el editor no usa: mostrar u ocultar la Bandeja reciente, fijar y navegar por la bandeja (`TRY-`); Enlace para compartir y destinos (`SHR-`); Diapositivas narradas (`VID-`); Extraer texto (`SMT-`); Ayuda y hoja de atajos (APP-015, APP-023). En el perfil Compatibilidad con Snagit esos archivos usan los valores de Snagit (por ejemplo, `Option`+`Cmd`+`T` para la bandeja en macOS), y por eso el editor no los ocupa en ninguno de los dos perfiles.

### EDT-115 · Atajos de herramientas
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** `Control`+letra por herramienta (Mac); `Ctrl`+`1`…`9` por posición en la barra (Windows)
- **Comportamiento:**
  - Con el foco en el lienzo, una **tecla sola** activa cada herramienta (decisión de diseño; las letras son las mismas en inglés y en español):

    | Herramienta | Tecla | Compatibilidad con Snagit (macOS) |
    |---|---|---|
    | Favoritos | `F` | `Control`+`R` |
    | Mover | `V` | `Control`+`V` |
    | Flecha | `A` | `Control`+`A` |
    | Línea | `L` | — |
    | Texto | `T` | `Control`+`T` |
    | Globo | `C` | `Control`+`D` |
    | Forma | `U` | `Control`+`U` |
    | Sello | `S` | `Control`+`I` |
    | Paso | `N` | `Control`+`S` |
    | Resaltador | `H` | `Control`+`H` |
    | Lápiz | `P` | `Control`+`P` |
    | Lupa | `M` | `Control`+`M` |
    | Foco | `O` | — |
    | Desenfocar | `B` | `Control`+`B` |
    | Simplificar | `Y` | `Control`+`Y` |
    | Selección | `Q` | `Control`+`L` |
    | Varita mágica | `W` | `Control`+`W` |
    | Relleno | `G` | `Control`+`F` |
    | Borrador | `E` | `Control`+`E` |
    | Recortar | `R` | `Control`+`C` |
    | Quitar franja | `X` | `Control`+`X` |
    | Medir | `K` | — |

  - `Ctrl`/`Cmd`+`1` … `9` activan la herramienta que ocupa esa posición en la barra (contando solo herramientas), en ambas plataformas. En Compatibilidad con Snagit: igual en Windows; en macOS no se asignan, porque Snagit usa ahí otras teclas (EDT-116).
  - En el perfil Compatibilidad con Snagit para Windows, las teclas solas siguen activas (no chocan con nada de Snagit).
  - Pulsar la tecla de la herramienta que ya está activa no hace nada. Las teclas de herramientas que aún no existen en el hito actual no hacen nada.
  - El tooltip de cada herramienta muestra su tecla (APP-014).
- **Criterios de aceptación:**
  - [ ] Con el foco en el lienzo, `A` activa Flecha y `T` activa Texto; mientras se edita un texto, `A` escribe "a".
  - [ ] `Ctrl`/`Cmd`+`3` activa la tercera herramienta de la barra; tras reordenar la barra (EDT-004), activa la nueva tercera.
  - [ ] En macOS con Compatibilidad con Snagit, `Control`+`D` activa Globo.
  - [ ] Reasignar Flecha a `Shift`+`A` en Preferencias hace que `A` deje de activarla.
- **Fuentes:** research/02-editor.md §12 (Mac tool shortcuts, Windows tool shortcuts); research/05-plataformas-versiones.md §3 (Default hotkeys: editor tool selection)

### EDT-116 · Atajos de edición, vista y archivo
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** atajos de edición y vista del editor (Windows y Mac)
- **Comportamiento:**
  - Valores por defecto. `Ctrl`/`Cmd` = `Ctrl` en Windows y `Cmd` en macOS; `Alt`/`Option` igual.

    | Acción | Windows y macOS (Coexistencia) | Compatibilidad con Snagit |
    |---|---|---|
    | **Archivo** | | |
    | Nueva imagen (EDT-103) | `Ctrl`/`Cmd`+`N` | = |
    | Nueva desde el portapapeles (EDT-104) | `Ctrl`/`Cmd`+`Shift`+`N` | = |
    | Nuevo desde plantilla, o Combinar en plantilla con varias capturas seleccionadas (EDT-095) | `Ctrl`/`Cmd`+`Shift`+`T` | Win: `Ctrl`+`Alt`+`T` (Combinar: `Ctrl`+`Alt`+`C`) · Mac: `Option`+`Shift`+`T` (Combinar: `Option`+`Shift`+`C`) |
    | Abrir (EDT-105) | `Ctrl`/`Cmd`+`O` | = |
    | Guardar (EDT-107) | `Ctrl`/`Cmd`+`S` | = |
    | Guardar como | `Ctrl`/`Cmd`+`Shift`+`S` | = |
    | Guardar todo | `Ctrl`/`Cmd`+`Alt`/`Option`+`S` | = |
    | Exportar | `Ctrl`/`Cmd`+`Alt`/`Option`+`Shift`+`S` | = |
    | Guardar como sello (EDT-032) | — | Mac: `Control`+`Option`+`Cmd`+`S` |
    | Imprimir (EDT-114) | `Ctrl`/`Cmd`+`P` | = |
    | Configurar página | macOS: `Cmd`+`Shift`+`P` · Windows: — | = |
    | Cerrar (EDT-108) | `Ctrl`/`Cmd`+`W` | Mac: `Shift`+`Cmd`+`W` |
    | Cerrar todo | `Ctrl`/`Cmd`+`Alt`/`Option`+`W` | = |
    | Renombrar (EDT-109) | Windows: `F2` · macOS: — | = |
    | Duplicar documento | `Ctrl`/`Cmd`+`Shift`+`D` | = |
    | Mover a la papelera | Windows: `Ctrl`+`Supr` · macOS: `Cmd`+`Retroceso` | = |
    | **Edición** | | |
    | Deshacer (EDT-092) | `Ctrl`/`Cmd`+`Z` | = |
    | Rehacer | Windows: `Ctrl`+`Y` y `Ctrl`+`Shift`+`Z` · macOS: `Cmd`+`Shift`+`Z` | = |
    | Cortar / Copiar / Pegar (EDT-089) | `Ctrl`/`Cmd`+`X` / `C` / `V` | = |
    | Pegar en el mismo sitio | `Ctrl`/`Cmd`+`Shift`+`V` | = |
    | Pegar con el mismo estilo | `Ctrl`/`Cmd`+`Alt`/`Option`+`Shift`+`V` | = |
    | Copiar todo (EDT-090) | `Ctrl`/`Cmd`+`Shift`+`C` | Mac: `Option`+`Cmd`+`C` |
    | Duplicar objeto (EDT-088) | `Ctrl`/`Cmd`+`J` | = |
    | Eliminar | `Supr` / `Retroceso` | = |
    | Seleccionar todo (EDT-079) | `Ctrl`/`Cmd`+`A` | = |
    | Deseleccionar | `Ctrl`/`Cmd`+`D` | = |
    | Invertir selección (EDT-044) | `Ctrl`/`Cmd`+`Shift`+`I` | = |
    | Convertir la selección en imagen flotante | `Ctrl`/`Cmd`+`Shift`+`J` | = |
    | **Imagen** | | |
    | Cambiar tamaño de la imagen (EDT-069) | `Ctrl`/`Cmd`+`Shift`+`R` | = |
    | Tamaño del lienzo (EDT-070) | `Ctrl`/`Cmd`+`Alt`/`Option`+`K` | = |
    | Girar 90° a la derecha / a la izquierda (EDT-072) | `Ctrl`/`Cmd`+`R` / `Ctrl`/`Cmd`+`Alt`/`Option`+`R` | Mac: `Cmd`+`R` abre Ángulo personalizado |
    | Recortar a la selección | `Ctrl`/`Cmd`+`Shift`+`K` | = |
    | Recortar al lienzo (EDT-051) | `Ctrl`/`Cmd`+`K` | = |
    | Recortar bordes | `Ctrl`/`Cmd`+`Shift`+`X` | = |
    | Ajuste magnético (EDT-084) | `Ctrl`/`Cmd`+`;` | = |
    | **Objeto** | | |
    | Traer adelante / Enviar atrás (EDT-081) | `Ctrl`/`Cmd`+`↑` / `↓` | Win: `Ctrl`+`Shift`+`F` / `Ctrl`+`Shift`+`B` |
    | Traer al frente / Enviar al fondo | `Ctrl`/`Cmd`+`Shift`+`↑` / `↓` | Mac: `Option`+`Cmd`+`↑` / `↓` |
    | Agrupar / Desagrupar (EDT-082) | `Ctrl`/`Cmd`+`G` / `Ctrl`/`Cmd`+`Shift`+`G` | `Ctrl`/`Cmd`+`Shift`+`G` / `Ctrl`/`Cmd`+`Shift`+`U` |
    | Bloquear (EDT-085) | `Ctrl`/`Cmd`+`L` | = |
    | Aplanar (EDT-091) | `Ctrl`/`Cmd`+`E` | Win: `Ctrl`+`T` · Mac: `Shift`+`Cmd`+`T` |
    | Aplanar todo | `Ctrl`/`Cmd`+`Shift`+`E` | Win: `Ctrl`+`Shift`+`T` · Mac: `Control`+`Shift`+`T` |
    | **Ver** | | |
    | Ver Editor / Ver Biblioteca (EDT-001) | `Ctrl`/`Cmd`+`Shift`+`1` / `Ctrl`/`Cmd`+`Shift`+`2` | = |
    | Acercar / Alejar (EDT-007) | `Ctrl`/`Cmd`+`=` (también `+`) / `Ctrl`/`Cmd`+`-` | = |
    | Tamaño real | `Ctrl`/`Cmd`+`0` | = |
    | Ajustar a la ventana | `Ctrl`/`Cmd`+`Shift`+`0` | = |
    | Tamaño original en pantalla | `Ctrl`/`Cmd`+`Alt`/`Option`+`0` | = |
    | Cuadrícula de píxeles (EDT-008) | `Ctrl`/`Cmd`+`'` | = |
    | Panel lateral (EDT-005) | `Ctrl`/`Cmd`+`Alt`/`Option`+`P` | Mac: `Cmd`+`5` (y `Cmd`+`4` abre la pestaña Efectos) |
    | Pantalla completa (EDT-009) | Windows: `F11` · macOS: `Control`+`Cmd`+`F` | = |

  - Las acciones sin atajo por defecto (Exportar por lotes, Volver al original, Alinear y distribuir, Desbloquear todo, Desaplanar, Posición y tamaño, Historial, Personalizar la barra…) se pueden asignar en Preferencias > Atajos.
  - Los menús muestran el atajo vigente de cada entrada, según el perfil y los cambios del usuario.
- **Criterios de aceptación:**
  - [ ] Con el perfil Coexistencia, cada combinación de la tabla ejecuta su acción en ambas plataformas, y una prueba automática comprueba que no hay dos acciones con la misma combinación.
  - [ ] Con Compatibilidad con Snagit en Windows, `Ctrl`+`T` aplana el objeto seleccionado y `Ctrl`+`Shift`+`T` aplana todo; `Ctrl`+`Shift`+`T` ya no abre las plantillas.
  - [ ] Con Compatibilidad con Snagit en macOS, `Option`+`Cmd`+`C` copia todo.
  - [ ] Mientras se edita un texto en Windows con teclado español, `AltGr`+`E` escribe "€" y no ejecuta ningún atajo.
- **Fuentes:** research/02-editor.md §12 (Editing and view shortcuts), §1 (Side panel, Zoom, Full screen), §10.3 (Other file commands); research/04-biblioteca-compartir.md §2 (Close / Close Others, Delete)

### EDT-117 · Teclas del lienzo y modificadores
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** `Shift` para restringir, `Option`+arrastrar (Mac), `Ctrl` para alternar con el selector de color (Windows), `[` y `]` para el tamaño del borrador (Mac)
- **Comportamiento:**
  - Teclas contextuales del lienzo (no reasignables; aparecen en la hoja de atajos, APP-023):

    | Tecla | Efecto |
    |---|---|
    | `Esc` | Cancela el dibujo, el recorte o el polígono en curso; si no hay nada, deselecciona; si no hay selección, sale de pantalla completa o, con EDT-012, cierra la ventana |
    | `Enter` | Termina un polígono; aplica el recorte; edita el texto del objeto seleccionado |
    | `Ctrl`/`Cmd`+`Enter` | Inserta un objeto de la herramienta activa en el centro de la vista (EDT-013) |
    | `Supr` / `Retroceso` | Elimina lo seleccionado; en un polígono en curso, quita el último vértice |
    | Flechas / `Shift`+flechas | Mueven lo seleccionado 1 / 10 px (EDT-087) |
    | `Alt`/`Option`+flechas | Mueven los píxeles de una selección |
    | `Tab` / `Shift`+`Tab` | Objeto siguiente / anterior (EDT-013) |
    | `Espacio` (mantenido) + arrastrar | Desplaza la vista |
    | `V` (mantenida más de 0,3 s) | Mover temporal; al soltar vuelve la herramienta anterior (EDT-042). Una pulsación corta activa Mover (EDT-115) |
    | `I` | Cuentagotas: toma el color bajo el puntero para la herramienta activa; `Esc` lo cancela |
    | `[` / `]` | Reducen / aumentan el grosor o el tamaño de la herramienta activa (Borrador, Lápiz, Flecha, Línea, Resaltador) |
    | `F6` / `Shift`+`F6` | Zona siguiente / anterior de la ventana (EDT-013) |

  - Modificadores del ratón (EDT-022):

    | Modificador | Al dibujar | Al cambiar de tamaño | Al mover |
    |---|---|---|---|
    | `Shift` | Cuadrado, círculo, líneas a 0/45/90° | Mantiene la proporción | Restringe a horizontal o vertical |
    | `Alt`/`Option` | Dibuja encima de un objeto sin seleccionarlo; desde el centro en Forma | Desde el centro | Duplica (EDT-088) |
    | `Ctrl`/`Cmd` | — | — | Desactiva el ajuste magnético (EDT-084) |
    | `Ctrl`/`Cmd`+clic | Con Sello o Paso: duplica el objeto pulsado en el punto del clic (EDT-088) | — | — |

  - Decisión de diseño: estos modificadores son iguales en ambas plataformas. El "`Ctrl` para alternar con el selector de color" de Snagit para Windows se sustituye por la tecla `I`.
- **Criterios de aceptación:**
  - [ ] Con Flecha activa, `I` sobre un píxel rojo y luego dibujar crea una flecha roja.
  - [ ] Con Borrador activo, `]` aumenta el grosor y el contorno del puntero lo refleja.
  - [ ] Mantener `Espacio` y arrastrar desplaza la vista sin dibujar.
  - [ ] `Esc` durante un polígono a medias lo cancela sin crear objeto.
- **Fuentes:** research/02-editor.md §12 (Modifier keys while drawing), §2.7, §4.5

---

## 11. Preferencias y automatización

### EDT-118 · Preferencias del editor
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** pestañas Editor y Advanced de Editor Preferences (Windows); Advanced de Settings (Mac)
- **Comportamiento:**
  - Ajustes que define este archivo. Aparecen en Preferencias > **Editor** (APP-008), en este orden. Los marcados con "(menú)" se cambian además desde los menús del editor y se recuerdan igual:

    | Ajuste | Clave | Valores (por defecto) | Ficha |
    |---|---|---|---|
    | Color del área de trabajo | `editor.workspaceColor` | **Automático** / color | EDT-010 |
    | Reducir para ajustar al abrir | `editor.shrinkToFit` | **Sí** / No | EDT-007 |
    | Recordar el zoom y la posición de cada documento | `editor.rememberView` | **Sí** / No | EDT-007 |
    | Cuadrícula de píxeles (menú) y zoom mínimo | `editor.pixelGrid`, `editor.pixelGridMinZoom` | **Sí**, **600 %** (400–1600 %) | EDT-008 |
    | Ajuste magnético (menú) | `editor.snapping` | **Sí** / No | EDT-084 |
    | `Esc` cierra el editor | `editor.escClosesEditor` | Sí / **No** | EDT-012 |
    | Etiquetas de la barra de herramientas (menú) | `editor.toolbarLabels` | **Sí** / No | EDT-003 |
    | Vista previa del estilo junto al puntero | `editor.cursorStylePreview` | Sí / **No** | EDT-015 |
    | Seleccionar la anotación al terminar de dibujarla | `editor.autoSelectAfterDraw` | **Sí** / No | EDT-022 |
    | Barra de edición rápida | `editor.quickEditBar` | **Sí** / No | EDT-006 |
    | Ampliar el lienzo automáticamente | `editor.autoExpandCanvas` | **Sí** / No | EDT-071 |
    | Aplanar automáticamente las imágenes pegadas | `editor.autoFlattenPasted` | Sí / **No** | EDT-078 |
    | Eliminar el contenido recortado al guardar | `editor.deleteCroppedContent` | Sí / **No** | EDT-050 |
    | Tamaño y color de una imagen nueva | `editor.newImage.width`, `.height`, `.color` | **1280**, **720**, **blanco** | EDT-103 |
    | Formato por defecto de Guardar como y Exportar | `editor.defaultSaveFormat` | **Recordar el último** / formato | EDT-107 |
    | Fondo para formatos sin transparencia | `editor.flattenBackground` | **Blanco** / color | EDT-112 |
    | Corrector ortográfico e idioma | `editor.spellcheck`, `editor.spellcheckLanguage` | **Sí**, **Igual que el sistema** | EDT-027 |

  - Ajustes que afectan al editor pero pertenecen a otros archivos: mantener el editor cargado en segundo plano (APP-004, APP-009); tema e idioma de la interfaz (APP-012, APP-013); tamaño y número de elementos de la Bandeja reciente y quitar los no fijados al cerrar (`TRY-`); autoguardado y conservar una copia editable al guardar en otro formato (`LIB-`); formato al arrastrar fuera, formato por defecto al compartir y reducir las imágenes de alta densidad (`SHR-`); funciones en prueba (APP-034).
  - Estado del editor que se recuerda sin ser una preferencia visible: estilo actual de cada herramienta (EDT-014), disposición de la barra (EDT-004), pestaña y anchura del panel lateral (EDT-005), opciones de cada formato (EDT-112) y últimos ajustes de la exportación por lotes (EDT-113). Todo ello viaja en la exportación de ajustes (APP-011) y vuelve a sus valores por defecto con Restablecer todos los ajustes (APP-010), salvo los temas, estilos guardados, efectos guardados, sellos y plantillas, que son contenido del usuario.
  - Cada ajuste se aplica al momento, sin reiniciar el editor.
- **Criterios de aceptación:**
  - [ ] Cada ajuste de la tabla aparece una sola vez, en la sección Editor, y al cambiarlo la ficha indicada se comporta como describe.
  - [ ] Desactivar Ampliar el lienzo automáticamente y pegar una imagen grande no amplía el lienzo, sin reiniciar.
  - [ ] `xcreenshot settings get editor.autoSelectAfterDraw --json` devuelve el valor actual (APP-028).
  - [ ] Restablecer esta sección vuelve todos los ajustes de la tabla a su valor por defecto y no borra ningún tema ni estilo guardado.
- **Fuentes:** research/02-editor.md §11, §1 (Workspace, Escape closes Editor, Cursor Quick Style popup, Quick edit toolbar)

### EDT-119 · Comandos del editor (CLI y MCP)
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (la API COM no permitía editar; SPEC §2.2)
- **Comportamiento:**
  - Cada acción del editor es un comando (APP-027) del área `editor` (la ventana y la vista) o `document` (el contenido). Los menús, la barra y los atajos los llaman. Este archivo fija los comandos del catálogo mínimo (APP-030) y sus parámetros principales. Las coordenadas y tamaños siempre van en píxeles de la imagen.

    | Comando | Parámetros principales | Resultado | Hito |
    |---|---|---|---|
    | `document.open` | `path` (o `library_id`), `show` (abrir en la ventana; por defecto `false` desde CLI/MCP) | `document_id`, tamaño, número de objetos | M1 |
    | `document.new` | `width`, `height`, `color`, `dpi`; o `from_clipboard: true` | `document_id` | M2 |
    | `document.info` | `document_id` | tamaño, resolución, radio, lista de objetos (id, tipo, geometría, texto), efectos, metadatos | M1 |
    | `document.annotate` | `document_id`, `objects`: lista de objetos | ids de los objetos creados | M1 |
    | `document.update_object` / `document.delete_object` | `document_id`, `object_id`, propiedades | objeto actualizado / eliminado | M2 |
    | `document.apply_effect` | `document_id`, `effect` (nombre o efecto guardado), `options`, `target` (`document` u `object_id`) | pila de efectos resultante | M2 |
    | `document.crop` | `document_id`, `rect` (`x`,`y`,`w`,`h`) o `trim: true` | tamaño nuevo | M1 |
    | `document.resize` | `document_id`, `width`/`height` o `percent`, `resampling`, `scale_objects` | tamaño nuevo | M2 |
    | `document.rotate` / `document.flip` | `degrees` / `axis` | tamaño nuevo | M2 |
    | `document.canvas` | `width`, `height`, `anchor`, `color`, `corner_radius` | tamaño nuevo | M2 |
    | `document.flatten` | `document_id`, `object_ids` (o todos) | — | M2 |
    | `document.save` | `document_id`, `path` opcional (si no, su archivo) | ruta escrita | M1 |
    | `document.export` | `document_id`, `format`, `path`, `options` (EDT-112) | ruta, tamaño en píxeles y en bytes | M1 |
    | `document.batch_export` | `inputs` (rutas o ids), `effects`, `format`, `options`, `out_dir`, `naming`, `on_conflict` | lista de archivos escritos y fallidos | M2 |
    | `document.from_template` | `template` (nombre o ruta `.xtemplate`), `images`, `settings` (EDT-097) | `document_id` | M2 |
    | `document.copy_all` | `document_id` | — (imagen en el portapapeles) | M1 |
    | `document.print` | `document_id`, `printer`, opciones de EDT-114 | — | M2 |
    | `editor.show` | `view` (`editor`/`library`), `document_id` | — | M1 |
    | `editor.select_tool` | `tool`, `style` (nombre de estilo guardado) | — | M1 |
    | `editor.zoom` | `mode` (`actual`, `fit`, `percent`), `value` | zoom actual | M1 |
    | `editor.theme_import` / `editor.theme_export`, `editor.stamps_import`, `editor.template_import` | `path` | nombre importado o ruta escrita | M2 |

  - **`document.annotate`** recibe objetos en JSON con:
    - `type`: `arrow`, `line`, `text`, `callout`, `shape`, `step`, `stamp`, `blur`, `highlight`, `magnify`, `spotlight`, `image`;
    - geometría según el tipo: `from`/`to` (flecha, línea, cola de un globo), `rect` (`x`,`y`,`w`,`h`), `points` (polígono), `at` (paso, sello), `rotation`;
    - `text` (texto, globo, paso), `path` (imagen), `stamp` (nombre del sello);
    - `style`: nombre de un estilo guardado, o propiedades sueltas con los mismos nombres que la pestaña Propiedades (por ejemplo `color`, `width`, `fill`, `font`, `size`, `blur_type`, `intensity`).
    Las propiedades que faltan toman el estilo actual de la herramienta. Un objeto no válido hace fallar el comando entero con `invalid_params`, indicando su posición en la lista, y no se crea ninguno (operación atómica).
  - Los comandos que modifican un documento crean un único paso de Deshacer, con el nombre del comando, si el documento está abierto en la ventana.
  - Sin `show`, los comandos trabajan sin abrir ventanas, de modo que un agente puede capturar, anotar y exportar en segundo plano (APP-030).
  - Ejemplo de CLI: `xcreenshot document annotate --document-id <id> --params '{"objects":[{"type":"arrow","from":[40,40],"to":[200,120],"style":{"color":"#F2545B","width":6}}]}'`.
- **Criterios de aceptación:**
  - [ ] La secuencia `capture.fullscreen` → `document.annotate` (una flecha y un texto) → `document.export --format png` crea un PNG con ambos objetos en las coordenadas pedidas, sin abrir ventanas (criterio de APP-030).
  - [ ] `document.annotate` con un objeto válido y otro con `type` desconocido falla con `invalid_params`, nombra la posición 2 y no crea ningún objeto.
  - [ ] Un documento anotado por CLI y abierto después en el editor muestra los objetos editables, y un solo Deshacer los quita.
  - [ ] `document.info` devuelve la lista de objetos con el mismo número, tipo y geometría que muestra el editor.
- **Mejora sobre Snagit:** la edición completa es automatizable desde CLI y agentes en ambas plataformas.
- **Fuentes:** research/04-biblioteca-compartir.md §8 (COM Server API); research/01-captura.md §1.9

---
