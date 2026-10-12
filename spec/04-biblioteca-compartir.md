# 04 · Biblioteca, bandeja reciente y compartir

**Archivo:** `04-biblioteca-compartir.md` · **Prefijos de IDs:** `LIB-`, `TRY-`, `SHR-` · **Versión:** 0.1 (borrador) · **Fecha:** 2026-10-12

Este archivo especifica dónde viven las capturas de XcreenShot y cómo salen de la app. Cubre tres áreas. La **biblioteca** (`LIB-`) es la carpeta con todas las capturas, más un índice propio que permite buscar, filtrar, etiquetar y ordenar sin depender del indexador del sistema operativo; aquí se define también su ubicación, su mantenimiento y su traslado. La **bandeja reciente** (`TRY-`) es la tira de miniaturas del editor con lo último que se ha capturado o abierto. **Compartir** (`SHR-`) agrupa el marco común de envío (menú, opciones, progreso, errores, historial, credenciales), cada destino concreto y el **Enlace para compartir** con proveedor configurable (D-02). La captura (`CAP-`), el editor (`EDT-`), el video (`VID-`), la app (`APP-`) y las funciones inteligentes (`SMT-`) se definen en sus archivos y aquí solo se citan. Salvo que una función diga otra cosa, todo vale igual en Windows y macOS (SPEC §2.1).

**Convenciones de este archivo**
- **Teclas.** `Ctrl`/`Cmd` significa `Ctrl` en Windows y `Command` (⌘) en macOS; `Alt`/`Option` igual. `Supr` es la tecla Suprimir de Windows y `Retroceso` la tecla ⌫ de macOS.
- **(decisión de diseño)** marca un comportamiento que la documentación pública de Snagit no describe, o que XcreenShot define a propósito de otra forma. Lo fija este spec.
- **"Por defecto"** es el valor en una instalación nueva, sin configuración administrada (APP-035).
- **Elemento** es cualquier cosa que muestra la biblioteca: una imagen (`.xshot` o una imagen suelta), un video (`.mp4`), un GIF o una guía de pasos (`.xshot` de tipo guía).
- **Clave de ajuste.** Se proponen claves con el patrón de APP- (`área.nombre`). El equipo de construcción puede ajustarlas manteniendo el patrón.
- **Medios** de un destino: Imagen, Video, GIF y Guía. "Varios" indica si el destino acepta varios elementos en un mismo envío.
- **Destino en línea** es el que sube a un servicio de Internet. **Destino local** es el que entrega a algo del propio equipo (carpeta, portapapeles, otra app, impresora).
- **Referencias a otros archivos.** `PREFIJO-NNN (nombre)` cuando el ID existe en el índice de ese archivo; `PREFIJO- (nombre)` cuando todavía no.

---

## Índice

1. **Biblioteca: vista y navegación** — LIB-001 Vista Biblioteca y cambio con el editor · LIB-002 Cuadrícula, selección y menú contextual · LIB-003 Tamaño de las miniaturas · LIB-004 Ordenar y vista de lista
2. **Biblioteca: buscar y filtrar** — LIB-005 Panel de filtros: tipo y fecha · LIB-006 Filtros por aplicación y sitio web · LIB-007 Campo de búsqueda · LIB-008 Buscar el texto de las imágenes
3. **Biblioteca: organizar** — LIB-009 Etiquetas: añadir y quitar · LIB-010 Gestionar etiquetas · LIB-011 Favoritos · LIB-012 Detalles del elemento · LIB-013 Vista rápida · LIB-014 Mostrar en la biblioteca · LIB-015 Eliminar
4. **Biblioteca: crear, imprimir y exportar** — LIB-016 Crear desde la biblioteca · LIB-017 Imprimir desde la biblioteca · LIB-018 Exportación por lotes desde la biblioteca · LIB-019 Guías de pasos en la biblioteca
5. **Biblioteca: guardado, índice y rendimiento** — LIB-020 Autoguardado de capturas · LIB-021 Guardar como y copia editable · LIB-022 Índice propio de la biblioteca · LIB-023 Reconstruir y comprobar la biblioteca · LIB-024 Archivos añadidos a la carpeta e importación · LIB-025 Bibliotecas grandes · LIB-026 Barra de estado de la biblioteca
6. **Biblioteca: ubicación y migración** — LIB-027 Ubicación por defecto · LIB-028 Cambiar o mover la biblioteca · LIB-029 Biblioteca en una carpeta sincronizada · LIB-030 Biblioteca en una ubicación de red · LIB-031 Fusionar bibliotecas · LIB-032 Llevar la biblioteca a otro equipo · LIB-033 Exportar toda la biblioteca a archivos normales
7. **Biblioteca: automatización** — LIB-034 Comandos de la biblioteca (CLI y MCP)
8. **Bandeja reciente** — TRY-001 Mostrar y ocultar la bandeja · TRY-002 Contenido y orden · TRY-003 Capacidad · TRY-004 Fijar y desfijar · TRY-005 Reordenar · TRY-006 Renombrar · TRY-007 Tamaño de las miniaturas · TRY-008 Cerrar elementos · TRY-009 Eliminar desde la bandeja · TRY-010 Duplicar · TRY-011 Indicador de cambios sin guardar · TRY-012 Selección múltiple y acciones · TRY-013 Navegación con teclado · TRY-014 Combinar arrastrando al lienzo · TRY-015 Arrastrar a otras aplicaciones · TRY-016 Detalles, etiquetas y otras acciones · TRY-017 Comandos de la bandeja (CLI y MCP)
9. **Compartir: marco común** — SHR-001 Zona de compartir · SHR-002 Menú Compartir · SHR-003 Gestión de destinos · SHR-004 Instancias con nombre y atajos por destino · SHR-005 Opciones comunes de los destinos · SHR-006 Nombres de archivo y títulos · SHR-007 Formato al compartir y al guardar como · SHR-008 Reducir imágenes de alta densidad · SHR-009 Compartir varios elementos y a varios destinos · SHR-010 Progreso, estado y cancelación · SHR-011 Resultado: avisos, errores y reintento · SHR-012 Historial de envíos · SHR-013 Destinos en la barra de herramientas · SHR-014 Cuentas conectadas y credenciales · SHR-015 Copiar todo desde la biblioteca y la bandeja · SHR-016 Arrastrar y soltar · SHR-017 Comandos de compartir (CLI y MCP)
10. **Compartir: destinos** — SHR-018 Archivo · SHR-019 Portapapeles · SHR-020 Correo · SHR-021 Aplicación · SHR-022 Impresora · SHR-023 FTP y SFTP · SHR-024 Word · SHR-025 PowerPoint · SHR-026 Excel · SHR-027 OneNote · SHR-028 Evernote · SHR-029 OneDrive y SharePoint · SHR-030 Microsoft Teams · SHR-031 Slack · SHR-032 Google Drive · SHR-033 Dropbox · SHR-034 Box · SHR-035 YouTube · SHR-036 Pages, Keynote y Numbers · SHR-037 AirDrop y Compartir del sistema
11. **Enlace para compartir** — SHR-038 Enlace para compartir · SHR-039 Proveedores del Enlace para compartir · SHR-040 Proveedor: carpeta sincronizada · SHR-041 Proveedor: almacenamiento compatible con S3
12. [Atajos por defecto](#12-atajos-por-defecto-de-este-archivo) · [Relacionado](#relacionado) · [Excluido](#excluido) · [Cobertura](#cobertura)

---

## 1. Biblioteca: vista y navegación

### LIB-001 · Vista Biblioteca y cambio con el editor
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Library tab/view; Editor/Library toggle on the main toolbar; Cmd+1 / Cmd+2 (Mac)
- **Comportamiento:**
  - La ventana principal tiene dos vistas, **Editor** y **Biblioteca**. Un selector de dos botones en la barra superior, siempre visible, cambia entre ellas. También están en el menú Ver.
  - Atajos: `Cmd+1` / `Cmd+2` en macOS; `Ctrl+Shift+1` / `Ctrl+Shift+2` en Windows, porque allí `Ctrl+1…9` eligen herramientas del editor (`EDT-`) (decisión de diseño).
  - La biblioteca también se abre con **Abrir biblioteca** del menú rápido del ícono (APP-002), con Mostrar en la biblioteca (LIB-014) y por comando (LIB-034).
  - Disposición de la vista Biblioteca: panel de filtros a la izquierda (LIB-005), arriba el campo de búsqueda y los controles de orden y zoom, en el centro la cuadrícula (LIB-002) y abajo la barra de estado (LIB-026). La zona de compartir (SHR-001) actúa sobre la selección.
  - Doble clic o `Enter` sobre un elemento lo abre en la vista Editor: las imágenes en el editor (`EDT-`), los videos en el editor de video (`VID-`) y las guías en su editor (`SMT-`). El elemento entra en la bandeja reciente (TRY-002).
  - Cambiar de vista no pierde nada. Mientras la app está abierta, la biblioteca conserva búsqueda, filtros, selección y posición de desplazamiento. Al volver a abrir la app se restauran la vista activa, los filtros, el orden y el zoom, pero no el texto de búsqueda (decisión de diseño).
  - La bandeja reciente solo aparece en la vista Editor (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] `Cmd+2` (macOS) o `Ctrl+Shift+2` (Windows) muestran la biblioteca; `Cmd+1` o `Ctrl+Shift+1` vuelven al editor con el mismo documento y el mismo zoom.
  - [ ] Doble clic en un video de la biblioteca lo abre en el editor de video y lo añade a la bandeja.
  - [ ] Tras filtrar por una etiqueta, pasar al editor y volver, el filtro y la selección siguen igual.
  - [ ] Con todas las ventanas cerradas, Abrir biblioteca desde el menú del ícono abre la ventana principal directamente en la vista Biblioteca.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Library (tab/view)); research/02-editor.md §1 (Editor tab vs Library tab)

### LIB-002 · Cuadrícula, selección y menú contextual
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Library thumbnails; multi-select and Shift-click range; Library keyboard navigation (Mac: Cmd+A, Cmd+Up/Down, Control-click); Library context menu
- **Comportamiento:**
  - Cada elemento muestra miniatura, nombre y fecha, y estas insignias: tipo (duración en videos, "GIF", número de pasos en guías, LIB-019), favorito (LIB-011), "en la nube" o "no disponible" (LIB-029, LIB-030) y error de guardado (LIB-020).
  - **Selección:** clic; `Ctrl`/`Cmd`+clic añade o quita; `Shift`+clic selecciona un rango; arrastrar un rectángulo desde una zona vacía (decisión de diseño); `Ctrl+A`/`Cmd+A` selecciona todo lo visible, respetando filtros y búsqueda.
  - **Teclado:** flechas mueven el foco y la selección; `Shift`+flechas amplían; `Inicio`/`Fin` y `Ctrl`/`Cmd`+`↑`/`↓` van al principio o al final; `RePág`/`AvPág` pasan una pantalla; `Enter` abre; `Espacio` abre la vista rápida (LIB-013); `Supr` (Windows) o `Cmd+Retroceso` (macOS) eliminan (LIB-015). El menú contextual se abre con la tecla Menú o `Shift+F10` en Windows y con `Control`+clic en macOS. Mejora: esta navegación existe en las dos plataformas.
  - **Menú contextual**, en este orden: Abrir · Vista rápida · Compartir ▸ · Enlace para compartir · Copiar todo · Etiquetas… · Marcar como favorito / Quitar de favoritos · Renombrar · Duplicar · Crear ▸ · Exportar… · Imprimir… · Mostrar en el Explorador (Windows) / Mostrar en el Finder (macOS) · Copiar ruta · Detalles · Eliminar.
  - Las entradas que no aplican a la selección aparecen desactivadas, con el motivo en el tooltip y en la descripción accesible.
  - Renombrar y Duplicar se comportan como en la bandeja (TRY-006, TRY-010).
  - Cada miniatura tiene un nombre accesible con nombre, tipo, fecha y etiquetas, y un foco visible (APP-043, APP-044).
- **Criterios de aceptación:**
  - [ ] Clic en el 3.º elemento y `Shift`+clic en el 10.º seleccionan 8 elementos, y la barra de estado dice "8 seleccionados".
  - [ ] Con un filtro activo, `Ctrl+A`/`Cmd+A` selecciona solo los elementos visibles.
  - [ ] Solo con el teclado se puede llegar a un elemento, abrir su menú contextual y abrirlo en el editor.
  - [ ] Con una imagen y un video seleccionados, Imprimir aparece desactivado y su tooltip explica que solo se imprimen imágenes.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Library (tab/view), Library keyboard navigation, Delete / Move to Trash)

### LIB-003 · Tamaño de las miniaturas
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Library thumbnail size / zoom slider (percentage on Windows); zoom level persists
- **Comportamiento:**
  - Deslizador en la barra superior de la biblioteca, de 50 % a 300 % sobre una miniatura base de 160 px (decisión de diseño), con el porcentaje visible.
  - Con el foco en la cuadrícula, `Ctrl`/`Cmd` + rueda del ratón y `Ctrl`/`Cmd` + `+`/`-` cambian el zoom.
  - Al cambiar el zoom, el elemento seleccionado sigue visible.
  - Por debajo de 80 % se ocultan los nombres y solo queda la miniatura con su tooltip (decisión de diseño).
  - El valor se guarda y se restaura en el siguiente arranque.
- **Criterios de aceptación:**
  - [ ] Poner 200 % y reiniciar la app deja el zoom en 200 %.
  - [ ] Con un elemento seleccionado al final de la lista, cambiar el zoom lo mantiene a la vista.
  - [ ] `Ctrl`/`Cmd` + rueda sobre la cuadrícula cambia el zoom y el porcentaje mostrado.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Thumbnail size / zoom)

### LIB-004 · Ordenar y vista de lista
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Library sort (date modified, date created, name, size; click a column header, click again to reverse)
- **Comportamiento:**
  - Criterios de orden: **Fecha de modificación** (por defecto, más reciente primero), **Fecha de captura**, **Nombre**, **Tamaño**, y además **Tipo** y **Aplicación** (decisión de diseño).
  - El nombre se ordena en orden natural ("Captura 2" antes que "Captura 10") y sin distinguir mayúsculas ni acentos.
  - Un selector de orden y un botón de dirección (ascendente o descendente) están en la barra superior.
  - **Vista Lista** además de la cuadrícula (decisión de diseño). Columnas: Nombre, Fecha de captura, Fecha de modificación, Tamaño, Tipo, Dimensiones o duración, Aplicación, Sitio web y Etiquetas. Un clic en la cabecera ordena por esa columna y otro clic invierte el orden. Las columnas se pueden redimensionar y ocultar.
  - El orden y la vista elegidos se recuerdan.
  - Etiquetar o marcar como favorito no cambia la fecha de modificación, así que no reordena la lista (LIB-009, LIB-011).
- **Criterios de aceptación:**
  - [ ] Ordenar por Tamaño ascendente pone primero el elemento más pequeño; un segundo clic en la cabecera Tamaño invierte el orden.
  - [ ] Con orden por fecha de modificación, añadir una etiqueta a un elemento no lo mueve de sitio.
  - [ ] "Captura 2" aparece antes que "Captura 10" al ordenar por nombre ascendente.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Sort, Favorites (capture-level))

---

## 2. Biblioteca: buscar y filtrar

### LIB-005 · Panel de filtros: tipo y fecha
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Library search pane categories (file type; date by year, month, day); Animated GIFs category (Mac); Step Capture category (2026.3.0)
- **Comportamiento:**
  - Panel lateral izquierdo, plegable, con secciones también plegables: **Tipo**, **Fecha**, **Aplicaciones** y **Sitios web** (LIB-006), **Etiquetas** (LIB-009) y **Favoritos** (LIB-011). Cada valor muestra su número de elementos.
  - **Tipo:** Imágenes (con el subfiltro "Con desplazamiento" para las capturas largas, CAP-036), Videos, GIF animados y Guías de pasos.
  - **Fecha:** árbol Año ▸ Mes ▸ Día, según la fecha de captura (o la fecha de creación del archivo si no hay metadatos). Arriba, accesos rápidos: Hoy, Ayer, Últimos 7 días y Este mes (decisión de diseño).
  - **Combinación** (decisión de diseño): los valores elegidos dentro de una misma sección se suman (O); las secciones distintas se cruzan (Y); el texto de búsqueda (LIB-007) se cruza con todo.
  - Los conteos de cada sección se calculan sobre el resultado de los filtros de las demás secciones. Los valores con 0 elementos se ocultan, salvo que estén elegidos (decisión de diseño).
  - Los filtros activos aparecen como fichas encima de la cuadrícula, cada una con ✕. **Quitar filtros** los quita todos.
  - Todo funciona igual en bibliotecas locales, sincronizadas y de red (LIB-029, LIB-030).
- **Criterios de aceptación:**
  - [ ] Elegir Videos y el año 2025 muestra solo videos de 2025, y el conteo coincide con lo que se ve.
  - [ ] Elegir Imágenes y GIF animados muestra los dos tipos a la vez.
  - [ ] **Quitar filtros** vuelve a mostrar el total de la biblioteca.
  - [ ] Con la biblioteca en una carpeta de red, el panel muestra los mismos filtros y conteos que en local.
- **Mejora sobre Snagit:** categoría GIF en las dos plataformas, accesos rápidos de fecha y filtros completos en bibliotecas de red.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Search pane categories, Step Capture items in Library), §1.2 (Network location)

### LIB-006 · Filtros por aplicación y sitio web
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Applications and Websites categories in the search pane; video metadata included (2023.2.0)
- **Comportamiento:**
  - **Aplicaciones:** lista de aplicaciones de origen, con su ícono si existe, ordenada por número de elementos. Las versiones de una misma aplicación se agrupan por nombre.
  - **Sitios web:** lista de dominios sin "www.", sacados de la URL de los metadatos de captura (CAP-056).
  - Con más de 10 valores, la sección tiene su propio campo para filtrar la lista (decisión de diseño).
  - Incluye imágenes, videos, GIF y guías. Los elementos sin dato aparecen bajo "Desconocida" o "Sin sitio web".
  - Si el usuario desactivó los metadatos de captura (CAP-056), las capturas nuevas no aportan valores a estas secciones.
- **Criterios de aceptación:**
  - [ ] Tras capturar Chrome en github.com y Visual Studio Code, Aplicaciones muestra las dos y Sitios web muestra github.com.
  - [ ] Un video grabado sobre una página web aparece bajo su dominio.
  - [ ] Elegir una aplicación muestra solo los elementos capturados en ella.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Search pane categories)

### LIB-007 · Campo de búsqueda
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Library search field (file name, tag, application, website; annotation text on Mac (?)); result counts for accessibility (2025.1.0); Find Capture (Cmd+F)
- **Comportamiento:**
  - Campo en la barra superior de la biblioteca. `Ctrl+F`/`Cmd+F` le da el foco y `Esc` lo vacía.
  - Busca mientras se escribe, en: nombre, etiquetas, aplicación, sitio web (dominio y URL), texto de las anotaciones (texto, globos, pasos) y textos de las guías de pasos.
  - Reglas (decisión de diseño):
    - sin distinguir mayúsculas ni acentos ("camion" encuentra "Camión");
    - varias palabras: tienen que aparecer todas, en cualquier campo;
    - cada palabra coincide por prefijo ("factura" encuentra "facturas");
    - entre comillas, frase exacta;
    - prefijos opcionales: `tag:`, `app:`, `site:` y `type:` (`image`, `video`, `gif`, `guide`).
  - Se combina con los filtros (LIB-005).
  - Muestra "N resultados" y lo anuncia al lector de pantalla. Sin resultados, muestra un mensaje con el botón **Quitar filtros**.
- **Criterios de aceptación:**
  - [ ] "factura" encuentra un elemento llamado "Factura cliente" y otro etiquetado "facturas".
  - [ ] Un texto escrito dentro de un globo de una captura la encuentra.
  - [ ] `app:chrome tag:cliente` devuelve solo las capturas de Chrome con la etiqueta "cliente".
  - [ ] El lector de pantalla anuncia el número de resultados al terminar de escribir.
- **Mejora sobre Snagit:** búsqueda sin acentos, texto de anotaciones en las dos plataformas y prefijos por campo.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Search field, Library keyboard navigation)

### LIB-008 · Buscar el texto de las imágenes
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (mejora; Snagit no busca el texto que aparece dentro de las imágenes)
- **Comportamiento:**
  - Preferencias > Biblioteca > **Reconocer el texto de las imágenes para la búsqueda** (clave `library.ocrIndex`; por defecto: sí) (decisión de diseño).
  - XcreenShot reconoce en segundo plano, en el dispositivo, el texto de las imágenes nuevas y de las ya existentes, con el motor de SMT- (Extraer texto) y en su idioma.
  - Trabaja con prioridad baja: se pausa durante capturas y grabaciones y con la batería baja. Nunca retrasa una captura (APP-048, APP-049).
  - El texto reconocido entra en la búsqueda de LIB-007. Los resultados que solo coinciden por este texto llevan la marca "en la imagen".
  - Se reconocen imágenes y pasos de guías, no videos (decisión de diseño).
  - El texto se guarda en el índice y con los metadatos duraderos (LIB-022), para no repetir el trabajo al reconstruir.
  - Desactivar la opción borra el texto reconocido del índice. El progreso aparece en la barra de estado (LIB-026).
- **Criterios de aceptación:**
  - [ ] Una captura sin anotaciones y con nombre genérico que muestra "Error 404" aparece al buscar "404".
  - [ ] Con la opción desactivada, la misma búsqueda no la encuentra.
  - [ ] Mientras se reconoce el texto de 5.000 imágenes, una captura de región cumple los tiempos de APP-049.
  - [ ] Durante el reconocimiento no hay conexiones de red (APP-054).
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Search field); SPEC §2.6

---

## 3. Biblioteca: organizar

### LIB-009 · Etiquetas: añadir y quitar
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Library tags (multi-select and Tag button; Delete Tag on Windows; Delete key on Mac; Image > Edit Tags; videos taggable)
- **Comportamiento:**
  - Se abre con el botón **Etiquetas** de la barra de la biblioteca, con Etiquetas… del menú contextual, con Imagen > Editar etiquetas en el editor y desde Detalles (LIB-012).
  - Editor de etiquetas: un campo con autocompletado de las etiquetas existentes. `Enter` o la coma añaden la etiqueta. Cada etiqueta es una ficha con ✕. `Retroceso` en el campo vacío quita la última, y `Supr`/`Retroceso` sobre una ficha con foco la quitan.
  - Con varios elementos seleccionados, las etiquetas comunes a todos aparecen como fichas normales; las que solo tienen algunos, como fichas punteadas con "en 3 de 5". Añadir aplica a todos; quitar quita de todos.
  - Se pueden etiquetar imágenes, videos, GIF y guías.
  - Reglas (decisión de diseño): de 1 a 50 caracteres; se recortan los espacios de los extremos; hasta 100 etiquetas por elemento; dos etiquetas que solo difieren en mayúsculas son la misma (LIB-010).
  - Las etiquetas viven en XcreenShot (índice y metadatos duraderos, LIB-022). No se escriben en las etiquetas del sistema operativo (decisión de diseño).
  - Etiquetar no cambia la fecha de modificación.
- **Criterios de aceptación:**
  - [ ] Seleccionar 5 elementos y añadir "Cliente A" hace que el filtro Cliente A muestre esos 5.
  - [ ] Quitar "Cliente A" de 2 de ellos deja el conteo de la etiqueta en 3.
  - [ ] Un video se etiqueta igual que una imagen.
  - [ ] Añadir "cliente a" a un elemento que ya tiene "Cliente A" no crea una etiqueta duplicada.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Tags); research/05-plataformas-versiones.md §1A (2024.0.3)

### LIB-010 · Gestionar etiquetas
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** tags that differ only by capitalization merge (Mac 2025.4.0); Delete Tag (Windows)
- **Comportamiento:**
  - En la sección Etiquetas del panel, el menú contextual de una etiqueta ofrece: **Renombrar…**, **Fusionar con…**, **Eliminar etiqueta** y **Seleccionar sus elementos**.
  - Eliminar quita la etiqueta de todos sus elementos, tras confirmar con el número afectado.
  - Renombrar una etiqueta con un nombre que ya existe (sin distinguir mayúsculas) ofrece fusionarlas.
  - La identidad no distingue mayúsculas: "Cliente" y "cliente" son la misma etiqueta y se muestra con la grafía de su primera aparición. Renombrar puede cambiar solo las mayúsculas. Los acentos sí distinguen ("camion" y "camión" son distintas) (decisión de diseño).
  - Las etiquetas que llegan con mayúsculas distintas (por ejemplo, al fusionar bibliotecas, LIB-031) se unifican al indexar.
  - La lista de etiquetas se ordena por nombre o por número de elementos.
  - Todos los cambios se escriben en los metadatos duraderos (LIB-022).
- **Criterios de aceptación:**
  - [ ] Renombrar "proj-x" a "Proyecto X" cambia la etiqueta en todos sus elementos.
  - [ ] Fusionar "Bug" con "Error" deja solo "Error" en todos los elementos que tenían una de las dos; el conteo no cuenta dos veces los que tenían ambas.
  - [ ] Eliminar una etiqueta con 12 elementos pide confirmación indicando "12 elementos".
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Tags)

### LIB-011 · Favoritos
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** capture-level Favorites and Favorites filter (Mac only); marking a favorite does not change the modified date (Mac 2023.2.6)
- **Comportamiento:**
  - Una estrella aparece al pasar el ratón sobre la miniatura y queda fija cuando el elemento es favorito. También se marca desde el menú contextual, desde Detalles (LIB-012) y desde la bandeja (TRY-016).
  - Con varios seleccionados: si hay mezcla, la acción los marca todos; si todos son favoritos, los desmarca.
  - La sección **Favoritos** del panel de filtros muestra cuántos hay.
  - Marcar o desmarcar no cambia la fecha de modificación.
  - La marca se guarda con los metadatos duraderos (LIB-022) y sobrevive a una reconstrucción del índice.
- **Criterios de aceptación:**
  - [ ] Marcar 3 elementos hace que el filtro Favoritos muestre esos 3, también en Windows.
  - [ ] La fecha de modificación de un elemento no cambia al marcarlo.
  - [ ] Tras reconstruir el índice (LIB-023), los 3 siguen siendo favoritos.
- **Mejora sobre Snagit:** disponible también en Windows.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Favorites (capture-level), Search pane categories)

### LIB-012 · Detalles del elemento
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Details (Windows right-click) / Get Info (Mac, not for multi-selection); original capture time (2022.0.1); thumbnail tooltips with file details (Mac 2024.0.0)
- **Comportamiento:**
  - Se abre con **Detalles** del menú contextual, `Alt+Enter` (Windows) o `Cmd+I` (macOS). Además, la biblioteca tiene un panel lateral derecho opcional, **Detalles**, que sigue a la selección (decisión de diseño).
  - Campos: Nombre (editable), Tipo, Dimensiones en píxeles y escala de pantalla, Duración y fotogramas por segundo (videos), Pasos (guías), Tamaño del archivo, Ubicación (con Mostrar en la carpeta), Fecha de captura original, Fecha de creación, Fecha de modificación, Aplicación de origen y versión, Sitio web (URL completa; un clic la abre en el navegador), Sistema operativo, Usuario, Etiquetas (editables), Favorito y Último envío (del historial, SHR-012).
  - Qué metadatos existen lo define CAP-056 (Metadatos de captura). Un dato que no existe se muestra como "—".
  - Cada campo se puede copiar desde su menú contextual.
  - **Varios elementos** (mejora): número de elementos, tamaño total, intervalo de fechas y etiquetas comunes, editables como en LIB-009.
  - Tooltip de la miniatura (tras 600 ms): nombre, tipo, dimensiones o duración, fecha de captura, aplicación y tamaño.
- **Criterios de aceptación:**
  - [ ] Los detalles de una captura de navegador muestran la URL, la aplicación y la hora de captura original, aunque la imagen se haya editado después.
  - [ ] Un clic en la URL la abre en el navegador predeterminado.
  - [ ] Con 4 elementos seleccionados, Detalles muestra "4 elementos" y el tamaño total.
  - [ ] Al dejar el ratón sobre una miniatura aparece el tooltip con sus datos.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Details / Get Info, Thumbnail tooltips with file details); research/04-biblioteca-compartir.md §10 (Windows Capture Info / Details)

### LIB-013 · Vista rápida
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Quick Look preview in the Library (Space; arrow keys) (Mac 2024.0.1)
- **Comportamiento:**
  - `Espacio` con un elemento seleccionado, en la biblioteca o en la bandeja, abre una vista previa grande sobre la ventana, sin abrir el editor.
  - Contenido según el tipo:
    - imagen: ajustada a la ventana, con sus anotaciones;
    - video: se reproduce con controles de reproducción, posición y volumen;
    - GIF: animado;
    - guía: muestra un paso cada vez, con "Paso 3 de 12"; `RePág`/`AvPág` cambian de paso (decisión de diseño).
  - `←`/`→` pasan al elemento anterior o siguiente sin cerrar la vista previa. Con varios seleccionados, recorre solo la selección (decisión de diseño).
  - `Espacio` o `Esc` cierran; `Enter` abre el elemento en el editor. Al cerrar, queda seleccionado el último elemento visto.
  - Pie de la vista previa: nombre, dimensiones o duración y fecha.
  - Se abre en menos de 200 ms para imágenes de hasta 20 megapíxeles (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] `Espacio` sobre una imagen muestra la vista previa en menos de 200 ms.
  - [ ] `→` muestra el siguiente elemento; `Esc` cierra y deja seleccionado ese elemento.
  - [ ] Un video se reproduce con sonido dentro de la vista previa.
- **Mejora sobre Snagit:** disponible también en Windows y en la bandeja.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Quick Look preview)

### LIB-014 · Mostrar en la biblioteca
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Reveal in Library (File menu; Shift+Cmd+L) (Mac 2022.2.0)
- **Comportamiento:**
  - Desde el editor: Archivo > **Mostrar en la biblioteca**, el menú contextual de la bandeja (TRY-016) o el atajo `Shift+Cmd+L` en macOS. En Windows no tiene atajo por defecto, porque `Ctrl+Shift+L` es el Enlace para compartir; se puede asignar uno (APP-018).
  - Cambia a la vista Biblioteca, desplaza la cuadrícula hasta el elemento y lo selecciona.
  - Si los filtros o la búsqueda lo ocultan, se quitan y aparece un aviso breve, "Se quitaron los filtros para mostrar el elemento", con **Deshacer** (decisión de diseño).
  - Si el documento no está en la biblioteca (archivo externo o autoguardado desactivado), la entrada aparece desactivada con el motivo, y se ofrece **Mostrar en el Explorador / Finder**.
- **Criterios de aceptación:**
  - [ ] Con una imagen abierta en el editor, el comando muestra la biblioteca con esa imagen seleccionada y visible.
  - [ ] Con el filtro Videos activo, mostrar una imagen quita el filtro y muestra el aviso.
  - [ ] Con un PNG externo abierto, la entrada está desactivada y su tooltip explica por qué.
- **Mejora sobre Snagit:** disponible también en Windows.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Reveal in Library); research/02-editor.md §10.3

### LIB-015 · Eliminar
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Library Delete / Move to Trash (single, multiple, Shift-click range); "Prompt before deleting files" (Windows)
- **Comportamiento:**
  - Se elimina con `Supr` (Windows) o `Cmd+Retroceso` (macOS) en la biblioteca, con Eliminar del menú contextual o con el menú Edición. Desde la bandeja: TRY-009.
  - Los archivos van a la Papelera de reciclaje de Windows o a la Papelera de macOS, de donde se pueden recuperar (decisión de diseño).
  - Si la ubicación no tiene papelera (carpeta de red, algunas carpetas sincronizadas), el diálogo avisa de que el borrado es definitivo y siempre pide confirmación, aunque la confirmación esté desactivada.
  - Preferencia **Confirmar antes de eliminar** (clave `library.confirmDelete`; por defecto: sí). El diálogo indica cuántos elementos son, nombra hasta 5 e incluye "No volver a preguntar".
  - Si alguno tiene cambios sin guardar, el diálogo lo dice ("2 tienen cambios sin guardar"). Esos cambios se pierden.
  - Tras eliminar aparece "N elementos eliminados" con **Deshacer** durante 10 s. Deshacer recupera los archivos desde la papelera con sus etiquetas (decisión de diseño).
  - Los elementos eliminados se cierran en el editor y en la bandeja.
  - Eliminar no borra las copias ya compartidas (enlaces, destinos). Sus entradas del historial siguen allí, con la nota "archivo local eliminado" (SHR-012).
- **Criterios de aceptación:**
  - [ ] Eliminar 3 elementos con confirmación los quita de la biblioteca y quedan en la papelera del sistema.
  - [ ] **Deshacer** los devuelve a la biblioteca con sus etiquetas.
  - [ ] Con la confirmación desactivada no aparece ningún diálogo.
  - [ ] En una biblioteca de red, el diálogo avisa de que el borrado es definitivo aunque la confirmación esté desactivada.
- **Mejora sobre Snagit:** borrado recuperable con Deshacer en las dos plataformas.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Delete / Move to Trash); research/04-biblioteca-compartir.md §5.2 (Library)

---

## 4. Biblioteca: crear, imprimir y exportar

### LIB-016 · Crear desde la biblioteca
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Create button in the Library (Image from Template, Video from Images) using the selected captures
- **Comportamiento:**
  - Botón **Crear ▾** en la barra de la biblioteca y submenú Crear ▸ del menú contextual. Actúa sobre la selección, en el orden en que se ve:
    - **Nuevo desde plantilla…**: abre el selector de plantillas con las imágenes elegidas ya colocadas en sus zonas. Lo define EDT- (Nuevo desde plantilla).
    - **Diapositivas narradas…**: abre la grabación de narración sobre las imágenes elegidas. Lo define VID- (Diapositivas narradas).
    - **Combinar videos…** (decisión de diseño): abre la combinación de videos con los videos elegidos. Lo define VID- (Combinar videos).
  - Cada entrada se activa solo con elementos compatibles (imágenes para las dos primeras, videos para la tercera). Si no, aparece desactivada con el motivo.
  - Si la selección mezcla tipos, se usan solo los compatibles y un aviso dice cuántos se omitieron.
  - El resultado es un elemento nuevo de la biblioteca. Los originales no cambian.
- **Criterios de aceptación:**
  - [ ] Con 3 imágenes seleccionadas, Nuevo desde plantilla abre el selector con las 3 imágenes listas para colocar.
  - [ ] Con 2 imágenes y 1 video seleccionados, Diapositivas narradas usa las 2 imágenes y avisa de que omitió 1 elemento.
  - [ ] Con solo videos seleccionados, Nuevo desde plantilla aparece desactivado y su tooltip explica el motivo.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Create from Library); research/02-editor.md §1 (Create menu)

### LIB-017 · Imprimir desde la biblioteca
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Print from the Library context menu (Mac)
- **Comportamiento:**
  - **Imprimir…** (menú contextual o `Ctrl+P`/`Cmd+P` con el foco en la biblioteca) imprime las imágenes seleccionadas con el diálogo de impresión de EDT- (Imprimir), sin abrirlas en el editor.
  - Con varias imágenes: una por página, ajustada al papel y con la orientación que mejor encaje en cada una (decisión de diseño). La vista previa muestra todas las páginas.
  - Las guías se imprimen con la salida de impresión de SMT- (Grabador de pasos). Videos y GIF: la entrada aparece desactivada con el motivo.
  - Se imprime la imagen con sus anotaciones, sin modificar el archivo.
- **Criterios de aceptación:**
  - [ ] Seleccionar 3 imágenes e imprimir muestra una vista previa de 3 páginas.
  - [ ] Imprimir no cambia la fecha de modificación de los elementos.
  - [ ] Con un video en la selección, Imprimir aparece desactivado y el tooltip dice por qué.
- **Mejora sobre Snagit:** disponible también en Windows, con varias imágenes a la vez.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Print from Library); research/02-editor.md §10.5

### LIB-018 · Exportación por lotes desde la biblioteca
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Batch Export / Batch Conversion Wizard started from the Library right-click (Windows) or File > Export (Mac)
- **Comportamiento:**
  - **Exportar…** del menú contextual (o del menú Archivo con la biblioteca activa):
    - con un elemento, abre Guardar como (LIB-021, SHR-007);
    - con varios, abre la exportación por lotes de EDT- (Exportación por lotes), con la selección ya cargada. Ese diálogo define formato, efectos, giro, carpeta y nombres con prefijo y numeración.
  - Lo que añade la biblioteca:
    - **Videos y GIF:** el motor de imágenes no los convierte. Opción **Copiar tal cual** (por defecto) u **Omitir**.
    - **Guías:** opción **Una imagen por paso** (por defecto) u **Omitir**. Las demás salidas de las guías están en LIB-019.
    - Si la carpeta de destino está dentro de la biblioteca, se avisa de que los archivos exportados aparecerán como elementos sueltos (LIB-024).
  - Corre en segundo plano, con progreso en la barra de estado (LIB-026) y **Cancelar**. Cancelar conserva lo ya exportado.
  - Al terminar, un resumen dice cuántos se exportaron, cuántos se omitieron y cuáles fallaron (con el motivo), con **Mostrar carpeta**.
- **Criterios de aceptación:**
  - [ ] Seleccionar 20 imágenes y 2 videos y exportar a JPG produce 20 JPG y 2 MP4 copiados, y el resumen lo indica.
  - [ ] Con Omitir para los videos, la carpeta solo contiene los 20 JPG.
  - [ ] Cancelar a mitad deja en la carpeta los archivos ya exportados, completos y sin archivos a medias.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Batch Export / Batch Conversion Wizard); research/02-editor.md §10.4

### LIB-019 · Guías de pasos en la biblioteca
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Step Capture category in the Library and image-count badge on thumbnails (2026.3.0); export from the Library (Mac)
- **Comportamiento:**
  - Las guías de pasos (SMT- (Grabador de pasos)) son elementos de tipo **Guía**, con su categoría en el filtro Tipo (LIB-005).
  - Su miniatura es el primer paso y lleva una insignia con el número de pasos ("12 pasos").
  - Doble clic abre la guía en su editor (`SMT-`). La vista rápida la recorre paso a paso (LIB-013).
  - Menú contextual > **Exportar guía ▸**: las salidas de SMT- (Word, PowerPoint, Markdown, HTML, PDF e imágenes separadas), sin abrir la guía.
  - El texto de los pasos entra en la búsqueda (LIB-007).
  - Se pueden etiquetar, marcar como favoritas, compartir y eliminar como cualquier elemento.
- **Criterios de aceptación:**
  - [ ] Tras grabar una guía de 8 pasos, aparece en la categoría Guías con la insignia "8 pasos".
  - [ ] Exportar guía ▸ Markdown desde la biblioteca crea el `.md` y sus imágenes sin abrir el editor.
  - [ ] Buscar una palabra que solo aparece en el texto del paso 5 encuentra la guía.
- **Mejora sobre Snagit:** exportación desde la biblioteca en las dos plataformas.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Step Capture items in Library, Search pane categories)

---

## 5. Biblioteca: guardado, índice y rendimiento

### LIB-020 · Autoguardado de capturas
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Automatic Library save (image captures as editable files; unsaved videos as MP4); separate toggles for images and videos (Windows)
- **Comportamiento:**
  - Cada captura de imagen se guarda sola en la biblioteca como `.xshot` (SPEC §7) al terminar la captura, antes de abrirse en el editor. Cada grabación se guarda como `.mp4` al detenerla (`VID-`).
  - Preferencias > Biblioteca: **Guardar automáticamente las imágenes** (clave `library.autosaveImages`) y **Guardar automáticamente los videos** (clave `library.autosaveVideos`). Las dos están activadas por defecto y existen en ambas plataformas.
  - **Con el autoguardado desactivado:**
    - las capturas nuevas viven en una carpeta temporal y aparecen en la bandeja con el indicador de cambios sin guardar (TRY-011);
    - solo entran en la biblioteca si el usuario las guarda;
    - al cerrarlas o al salir, XcreenShot pide **Guardar**, **No guardar** o **Cancelar** (APP-004);
    - la recuperación tras un fallo sigue funcionando (APP-047).
  - **Organización en disco** (decisión de diseño): subcarpetas `AAAA-MM` por mes de captura dentro de la biblioteca. El nombre sigue SHR-006, por ejemplo `Chrome 2026-10-12 14.03.22.xshot`. Si el nombre ya existe, se añade " (2)", " (3)", etc.
  - Cuándo se guardan las ediciones de un documento que ya está en la biblioteca lo define EDT- (Guardar).
  - **Si el autoguardado falla** (disco lleno, carpeta sin permiso o inaccesible):
    - aviso inmediato de la categoría Problemas (APP-016);
    - el contenido se queda en la carpeta de recuperación (APP-047) y el elemento muestra la insignia de error en la bandeja;
    - XcreenShot reintenta cuando la ubicación vuelve a estar disponible, o cuando el usuario pulsa **Reintentar** o elige **Guardar en otra carpeta**.
  - Ninguna captura se pierde en silencio.
- **Criterios de aceptación:**
  - [ ] Tras una captura de región, el `.xshot` existe en la subcarpeta del mes antes de que el editor termine de abrirse.
  - [ ] Con el autoguardado de videos desactivado, una grabación queda en la bandeja sin guardar y, al salir, la app pide Guardar o No guardar.
  - [ ] Con la carpeta de la biblioteca sin permiso de escritura, una captura muestra el aviso, se puede editar y aparece en la biblioteca al pulsar Reintentar después de restaurar el permiso.
  - [ ] Dos capturas en el mismo segundo y la misma aplicación producen dos archivos distintos.
- **Mejora sobre Snagit:** interruptores separados para imágenes y videos también en macOS; un fallo de guardado nunca pierde la captura.
- **Fuentes:** research/04-biblioteca-compartir.md §1.2 (Automatic Library save, SNAGX format), §5.2 (Library); research/02-editor.md §10.2

### LIB-021 · Guardar como y copia editable
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Save As vs Library (a copy in another format or place, the editable file stays in the Library); "Keep a duplicate editable capture when saving" (Mac); flattening of non-editable copies
- **Comportamiento:**
  - **Guardar como…** (`Ctrl+Shift+S`/`Shift+Cmd+S`; diálogo y formatos de EDT- (Guardar como)) escribe una **copia** en el formato y la carpeta elegidos. El `.xshot` de la biblioteca no cambia y sigue siendo el elemento editable de la bandeja (decisión de diseño). El formato propuesto sigue SHR-007.
  - Si la carpeta elegida está dentro de la biblioteca, la copia aparece como un elemento suelto más (LIB-024).
  - **Documentos abiertos desde fuera de la biblioteca** (por ejemplo, un PNG del escritorio): **Guardar** escribe en ese archivo, aplanando las anotaciones, porque el formato no admite objetos editables.
  - Preferencia **Conservar una copia editable al guardar en otro formato** (clave `library.keepEditableCopy`; por defecto: sí). Con ella activada, al guardar un archivo externo con anotaciones se guarda también un `.xshot` editable en la biblioteca, enlazado al original (decisión de diseño). Detalles (LIB-012) muestra ese enlace.
  - Antes de aplanar un archivo externo, un aviso explica que las anotaciones dejarán de ser editables en ese archivo. Incluye "No volver a mostrar".
- **Criterios de aceptación:**
  - [ ] Guardar como PNG una captura de la biblioteca crea el PNG, y el `.xshot` sigue en la biblioteca con sus objetos editables.
  - [ ] Abrir un PNG del escritorio, añadir una flecha y guardar deja el PNG con la flecha aplanada y, con la copia editable activada, un `.xshot` en la biblioteca con la flecha editable.
  - [ ] Con la copia editable desactivada, el mismo flujo no crea ningún `.xshot`.
- **Fuentes:** research/04-biblioteca-compartir.md §1.2 (Save As vs Library); research/02-editor.md §10.3

### LIB-022 · Índice propio de la biblioteca
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Library listing and search (Snagit for Windows depends on Windows Search indexing; the Library looks empty when the folder is not indexed)
- **Comportamiento:**
  - XcreenShot mantiene su propio índice (`library.db`, SPEC §7), con nombre, tipo, fechas, dimensiones, duración, metadatos de captura, etiquetas, favoritos, texto de anotaciones y texto reconocido. **No usa** el indexador del sistema (Windows Search, Spotlight) ni depende de él: la biblioteca funciona igual con el indexador desactivado o con la carpeta excluida.
  - **Los datos duraderos viven junto a los archivos**, no solo en el índice (decisión de diseño):
    - un `.xshot` lleva sus metadatos de captura en su `meta.json`;
    - las etiquetas, el favorito y el texto reconocido de cada elemento van en un archivo oculto al lado, `<archivo>.xmeta` (JSON pequeño). También lo tienen los `.mp4`, `.gif`, `.png` y `.jpg`;
    - al guardar un `.xshot`, sus etiquetas se copian también en su `meta.json`, para que viajen con el archivo aunque se copie solo.
  - Así, el índice se puede borrar y reconstruir sin perder nada (LIB-023).
  - **Dónde está el índice:**
    - en una biblioteca local, dentro de la carpeta de la biblioteca (SPEC §7);
    - en una carpeta sincronizada o de red (LIB-029, LIB-030), cada equipo guarda su índice en la carpeta de datos de la app, porque dos equipos que escriben la misma base de datos a través de un servicio de sincronización la pueden corromper (decisión de diseño; propuesta de ajuste a SPEC §7).
  - **Cambios hechos fuera de XcreenShot** (copiar, renombrar, mover o borrar archivos con el Explorador o el Finder, o los que llegan por sincronización) se reflejan solos: en menos de 5 s en carpetas locales y sincronizadas, y en menos de 60 s en carpetas de red (decisión de diseño).
  - Los archivos ocultos, los temporales de los servicios de sincronización y los `.xmeta` no aparecen como elementos.
- **Criterios de aceptación:**
  - [ ] (Windows) Con el servicio de búsqueda de Windows detenido, la biblioteca muestra todos los elementos y la búsqueda funciona.
  - [ ] Renombrar un `.xshot` en el Explorador o el Finder actualiza su nombre en la biblioteca en menos de 5 s y conserva sus etiquetas.
  - [ ] Borrar `library.db` y abrir la app reconstruye el índice con las mismas etiquetas y favoritos.
  - [ ] Copiar solo un `.xshot` etiquetado a otra biblioteca lo muestra allí con sus etiquetas.
- **Mejora sobre Snagit:** no depende del indexador del sistema operativo (SPEC §2.6).
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Search depends on OS indexing (Win)), §1.2 (Cloud Library, Network location); SPEC §2.6, §7

### LIB-023 · Reconstruir y comprobar la biblioteca
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** fallback for bad indexing (Windows 2022.0.1); Support Tool that repoints the Library and fixes permissions (Mac 2026.0.1)
- **Comportamiento:**
  - Preferencias > Biblioteca > **Reconstruir el índice…** y la entrada equivalente del menú Biblioteca. Pide confirmación y explica que no se pierde nada.
  - La reconstrucción ocurre sola si `library.db` falta, está dañado o es de una versión que no se puede migrar.
  - Corre en segundo plano, con progreso en la barra de estado (LIB-026). La biblioteca se puede usar mientras tanto: los elementos aparecen a medida que se indexan, y las capturas nuevas se guardan con normalidad.
  - Tiempo objetivo: el de APP-052 (50.000 elementos en menos de 10 min).
  - **Comprobar la biblioteca…** (decisión de diseño) revisa la carpeta y presenta un informe con acciones:
    - archivos que no se pueden leer o están dañados (**Mostrar** / **Mover a una carpeta aparte**);
    - `.xmeta` sin su archivo (**Limpiar**);
    - permisos insuficientes: explica qué falta y ofrece **Elegir otra ubicación** (LIB-028);
    - espacio libre en disco bajo (menos de 1 GB).
- **Criterios de aceptación:**
  - [ ] Con un `library.db` dañado a propósito, la app arranca, avisa de que está reconstruyendo y, al terminar, muestra todos los elementos con sus etiquetas.
  - [ ] Durante una reconstrucción se puede hacer una captura, y aparece en la biblioteca.
  - [ ] Un `.xshot` truncado aparece en el informe de Comprobar como dañado y no bloquea la reconstrucción del resto.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Search depends on OS indexing (Win)), §1.3 (Support Tool repoint)

### LIB-024 · Archivos añadidos a la carpeta e importación
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** PNG and JPG files saved into the Library folder appear in the Library (Windows 2024.1.0)
- **Comportamiento:**
  - Cualquier archivo compatible que esté en la carpeta de la biblioteca o en sus subcarpetas aparece como elemento, venga de donde venga.
  - Tipos admitidos (decisión de diseño): `.xshot`, PNG, JPG/JPEG, GIF, WebP, BMP, TIFF, HEIC (si el sistema lo puede leer), MP4 y MOV.
  - Los archivos sin metadatos de captura usan como fecha de captura la del archivo. Si contienen fecha EXIF, se usa esa.
  - **Importar a la biblioteca…** (menú Biblioteca) y **soltar archivos o carpetas sobre la vista Biblioteca** (SHR-016) **copian** los archivos a la biblioteca, en la subcarpeta del mes de su fecha. El original no cambia.
  - Al importar una carpeta, su nombre se puede añadir como etiqueta (casilla, activada por defecto) (decisión de diseño).
  - Las imágenes sueltas se abren en el editor como cualquier imagen. Al guardarlas se aplica LIB-021: guardar en el mismo archivo aplana, y la copia editable va a la biblioteca.
- **Criterios de aceptación:**
  - [ ] Copiar 10 PNG a la carpeta de la biblioteca con el Explorador o el Finder los muestra en la biblioteca en menos de 5 s.
  - [ ] Soltar una carpeta "Proyecto X" con 5 imágenes sobre la biblioteca las importa con la etiqueta "Proyecto X" y deja intacta la carpeta original.
  - [ ] Un JPG con fecha EXIF de 2019 aparece en el filtro Fecha bajo 2019.
- **Mejora sobre Snagit:** más formatos, importación explícita y disponible en las dos plataformas.
- **Fuentes:** research/04-biblioteca-compartir.md §1.2 (Loose PNG/JPG shown); research/04-biblioteca-compartir.md §6 (Drag in)

### LIB-025 · Bibliotecas grandes
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** faster and more reliable thumbnails with more than 5,000 items (Windows 2026.1.0); better load and responsiveness for large or cloud libraries (Mac 2024–2026); Mac search up to 3x faster (2025.2.0)
- **Comportamiento:**
  - Objetivo de diseño: **50.000 elementos** con los tiempos de APP-052 (primeras miniaturas en menos de 1 s; búsqueda y filtros en menos de 300 ms; desplazamiento fluido) (decisión de diseño).
  - La cuadrícula solo dibuja lo visible. Las miniaturas pendientes aparecen como marcadores con el ícono del tipo y se completan en segundo plano, de lo visible hacia fuera.
  - Las miniaturas se guardan en una caché local de la carpeta de datos de la app, con un máximo de 2 GB (decisión de diseño). Si se borra, se regenera.
  - Las tareas pesadas (indexar, reconocer texto, generar miniaturas de video) nunca bloquean la interfaz ni retrasan una captura.
  - Por encima de 100.000 elementos sigue funcionando, aunque puede ir más lenta. La barra de estado lo indica una vez (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con una biblioteca generada de 50.000 elementos se cumplen los criterios de APP-052.
  - [ ] Borrar la caché de miniaturas y abrir la biblioteca muestra marcadores que se completan sin bloquear el desplazamiento.
  - [ ] Durante la generación inicial de miniaturas de 50.000 elementos, una captura de región cumple los tiempos de APP-049.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Large-library handling, Search field)

### LIB-026 · Barra de estado de la biblioteca
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Library status message (banner stays expanded by default and remembers its state, Windows 2023.1.0); result counts (Windows 2025.1.0)
- **Comportamiento:**
  - Barra al pie de la vista Biblioteca con:
    - número de elementos totales, visibles con los filtros y seleccionados, y el tamaño de la selección;
    - tareas en segundo plano con su progreso: indexar, reconstruir, reconocer texto, mover, importar, exportar;
    - estado de la ubicación: local, sincronizada (con el nombre del servicio), de red, o **no disponible**;
    - avisos (por ejemplo, autoguardado fallido), con un enlace a la acción que los resuelve.
  - Se puede expandir para ver el detalle de cada tarea, cada una con **Cancelar** si se puede cancelar. Por defecto está expandida y recuerda su estado.
  - El lector de pantalla anuncia los cambios importantes (fin de tarea, error), sin anunciar cada porcentaje.
- **Criterios de aceptación:**
  - [ ] Con 8 elementos seleccionados, la barra dice "8 seleccionados" y su tamaño total.
  - [ ] Durante una reconstrucción, la barra muestra el progreso y, al terminar, el lector de pantalla anuncia "Índice reconstruido".
  - [ ] Plegar la barra y reiniciar la app la deja plegada.
- **Fuentes:** research/04-biblioteca-compartir.md §1.2 (Library status message); §1.1 (Search field)

---

## 6. Biblioteca: ubicación y migración

### LIB-027 · Ubicación por defecto
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** default Library location (Documents\Snagit on Windows; Pictures/Snagit on Mac, formerly "Autosave Location"); no first-run location prompt
- **Comportamiento:**
  - Ubicación por defecto:
    - Windows: `Documentos\XcreenShot` (la carpeta Documentos real del usuario, aunque esté redirigida);
    - macOS: `~/Pictures/XcreenShot` (Imágenes);
    - modo portable: la carpeta `library` junto al ejecutable (APP-038).
  - La primera captura no pregunta nada: la biblioteca se crea sola. La bienvenida (APP-005) muestra la ubicación y permite cambiarla o elegir una biblioteca existente, sin obligar.
  - En Windows, si Documentos está sincronizada con OneDrive, la bienvenida y Preferencias > Biblioteca lo indican ("tu biblioteca se sincroniza con OneDrive") y se aplica LIB-029 (decisión de diseño).
  - La configuración administrada puede fijar la ubicación inicial o bloquearla (clave `library.location`, APP-035). Admite variables de entorno o la forma `~` (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] En una instalación nueva de Windows, la primera captura crea `Documentos\XcreenShot` y se guarda allí, sin ningún diálogo.
  - [ ] En macOS, la primera captura se guarda en `~/Pictures/XcreenShot`.
  - [ ] Con `"locked": {"library.location": "D:\\Capturas"}`, la biblioteca se crea en `D:\Capturas` y el control de ubicación aparece bloqueado.
- **Fuentes:** research/04-biblioteca-compartir.md §1.2 (Default Library location, No first-run location prompt, Change Library location)

### LIB-028 · Cambiar o mover la biblioteca
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Change Library location (Editor Preferences > Library > Choose location / Browse other; Mac Settings > Advanced > Choose); files are moved; admins can lock it; Support Tool repoint
- **Comportamiento:**
  - Preferencias > Biblioteca > **Ubicación**: la ruta (completa en el tooltip), **Mostrar** (abre la carpeta) y **Cambiar…**.
  - **Cambiar…** abre el selector de carpetas, con accesos a las carpetas de los servicios sincronizados detectados (LIB-029). Según la carpeta elegida:
    - **Vacía o sin biblioteca:**
      - **Mover la biblioteca aquí** (por defecto): mueve todos los elementos y sus `.xmeta`;
      - **Empezar una biblioteca vacía aquí**: deja la anterior donde está.
    - **Con una biblioteca de XcreenShot:**
      - **Usar esa biblioteca**: la actual queda donde está;
      - **Fusionar** (LIB-031).
  - **Mover:**
    - antes de empezar, se comprueba el espacio libre y se avisa si no basta;
    - una ventana muestra el progreso ("1.240 de 8.000 elementos") y el tiempo restante estimado;
    - **Cancelar** devuelve lo movido a la ubicación original;
    - durante el movimiento, la biblioteca es de solo lectura, pero se puede capturar: las capturas nuevas esperan y entran en la ubicación nueva al terminar;
    - la bandeja reciente conserva sus elementos (decisión de diseño).
  - Si un archivo no se puede mover (en uso, sin permiso), se mueve el resto y un informe lista los que quedaron, con **Reintentar**.
  - Si la ubicación está bloqueada por la configuración administrada, **Cambiar…** aparece desactivado con un candado y "Fijado por la configuración administrada".
  - La automatización no puede cambiar la ubicación (APP-032).
- **Criterios de aceptación:**
  - [ ] Mover una biblioteca de 1.000 elementos a otro disco deja los 1.000 en la ubicación nueva con sus etiquetas, y la carpeta anterior queda sin elementos.
  - [ ] Cancelar a mitad de un movimiento deja todos los elementos en la ubicación original.
  - [ ] Una captura hecha durante el movimiento aparece en la ubicación nueva al terminar.
  - [ ] Elegir una carpeta que ya tiene una biblioteca ofrece Usar esa biblioteca y Fusionar.
- **Mejora sobre Snagit:** la bandeja no se vacía, se puede capturar durante el movimiento, el movimiento se puede cancelar y hay informe de errores.
- **Fuentes:** research/04-biblioteca-compartir.md §1.2 (Change Library location), §1.3 (Support Tool repoint); research/05-plataformas-versiones.md §1A (2024.0.1)

### LIB-029 · Biblioteca en una carpeta sincronizada
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Cloud Library in a synced folder (Box, Dropbox, Google Drive, iCloud, OneDrive Personal and Business); auto-detection of installed clients; existing Library suggested; large-library size warning (Mac)
- **Comportamiento:**
  - **Servicios:** OneDrive (personal y de trabajo o escuela), Google Drive para escritorio, Dropbox, Box Drive e iCloud Drive (en Windows, con iCloud para Windows).
  - **Detección:** el selector de ubicación lista los clientes de sincronización instalados y sus carpetas, una por cuenta, aunque haya varias cuentas del mismo servicio. Si en alguna ya hay una biblioteca de XcreenShot, la propone.
  - **Archivos solo en la nube:** XcreenShot funciona aunque el servicio deje archivos disponibles solo en línea; no exige descargarlo todo.
    - Esos elementos llevan la insignia "en la nube".
    - Su miniatura y sus datos salen del índice si el equipo ya los vio. Si no, se muestran nombre y tipo.
    - Al abrirlos, previsualizarlos o compartirlos se descargan, con progreso.
    - XcreenShot nunca fuerza la descarga de toda la biblioteca.
  - **Búsqueda y filtros completos** (LIB-005 a LIB-011), también con elementos solo en la nube.
  - **Conflictos de sincronización:** las copias en conflicto que crea el servicio aparecen como elementos aparte, con la insignia "conflicto", y la acción **Resolver**: quedarse con una, o conservar las dos (decisión de diseño).
  - **Varios equipos con la misma biblioteca:** se permite. Los cambios de otro equipo aparecen cuando el servicio los entrega. Si dos equipos cambian las etiquetas del mismo elemento a la vez, gana el último cambio (decisión de diseño).
  - **Aviso de tamaño:** al mover a una carpeta sincronizada una biblioteca de más de 5 GB, se avisa del espacio que ocupará en el servicio (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con OneDrive instalado, el selector de ubicación muestra su carpeta, y elegirla mueve allí la biblioteca.
  - [ ] Con los archivos en modo "solo en línea", la biblioteca lista los elementos y la búsqueda por etiqueta funciona; abrir uno lo descarga y lo muestra.
  - [ ] Una etiqueta añadida en el equipo A aparece en el equipo B cuando el servicio termina de sincronizar.
  - [ ] Una copia en conflicto creada por el servicio aparece con la insignia "conflicto" y Resolver la elimina tras elegir una versión.
- **Mejora sobre Snagit:** funciona con archivos solo en la nube y con varias cuentas, con búsqueda y filtros completos y gestión de conflictos.
- **Fuentes:** research/04-biblioteca-compartir.md §1.2 (Cloud Library); §1.1 (Large-library handling)

### LIB-030 · Biblioteca en una ubicación de red
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Network location (Windows; limited: no counts, no date/tag/app/website filters, file-name search only)
- **Comportamiento:**
  - La biblioteca puede estar en una carpeta compartida de red: ruta UNC o unidad asignada en Windows; volumen SMB montado en macOS.
  - **Funciones completas:** conteos, todos los filtros, etiquetas y búsqueda (LIB-005 a LIB-011), con el índice local de cada equipo (LIB-022).
  - **Sin conexión:**
    - la biblioteca muestra "no disponible" en la barra de estado, pero el índice local permite seguir buscando y viendo miniaturas;
    - las capturas nuevas se guardan en una carpeta local de espera y se mueven a la red cuando vuelve la conexión, con aviso (decisión de diseño).
  - Eliminar en una ubicación de red es definitivo y siempre se confirma (LIB-015).
- **Criterios de aceptación:**
  - [ ] Con la biblioteca en un recurso compartido de red, el filtro por etiqueta y la búsqueda por aplicación funcionan igual que en local.
  - [ ] Desconectar la red, capturar y volver a conectar deja la captura en la carpeta de red, sin intervención.
  - [ ] Con la red desconectada, la búsqueda sigue devolviendo resultados.
- **Mejora sobre Snagit:** filtros, conteos y búsqueda completos en bibliotecas de red, y también en macOS.
- **Fuentes:** research/04-biblioteca-compartir.md §1.2 (Network location)

### LIB-031 · Fusionar bibliotecas
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Merge libraries (pointing at another Library folder merges both)
- **Comportamiento:**
  - Se ofrece al elegir una carpeta que ya tiene una biblioteca (LIB-028) y desde Biblioteca > **Fusionar otra biblioteca…**.
  - **Fusionar** copia los elementos de la otra biblioteca a la actual, con sus `.xmeta`. La otra carpeta no se toca (decisión de diseño).
  - **Duplicados:** un archivo con el mismo contenido que uno existente no se copia; si el duplicado trae etiquetas que el existente no tiene, se le añaden. Los nombres repetidos con distinto contenido reciben " (2)".
  - Las etiquetas se unifican sin distinguir mayúsculas (LIB-010). Los favoritos se conservan.
  - Progreso, cancelación e informe final como en LIB-028.
- **Criterios de aceptación:**
  - [ ] Fusionar una biblioteca de 300 elementos, 20 de ellos idénticos a otros ya existentes, añade 280 elementos.
  - [ ] Una etiqueta "cliente" de la otra biblioteca se une con la etiqueta "Cliente" existente.
  - [ ] Tras fusionar, la carpeta de la otra biblioteca conserva todos sus archivos.
- **Fuentes:** research/04-biblioteca-compartir.md §1.3 (Merge libraries)

### LIB-032 · Llevar la biblioteca a otro equipo
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Move Library to a new computer (copy the folder, then point the new install to it)
- **Comportamiento:**
  - El asistente Mover XcreenShot a otro equipo (APP-055) guía el traslado. La parte de la biblioteca es:
    - en el equipo de origen, **Mostrar la carpeta** para copiarla (disco externo, red) o confirmar que ya está sincronizada;
    - en el equipo nuevo, elegir esa carpeta en la bienvenida (APP-005) o en **Cambiar…** > **Usar esa biblioteca** (LIB-028).
  - La biblioteca es portátil (decisión de diseño):
    - no guarda rutas absolutas;
    - funciona igual entre Windows y macOS;
    - el equipo nuevo construye su índice a partir de los archivos y los `.xmeta` (LIB-023), así que conserva etiquetas, favoritos y texto reconocido.
- **Criterios de aceptación:**
  - [ ] Copiar la carpeta de una biblioteca de Windows a un Mac y elegirla allí muestra todos los elementos con sus etiquetas y favoritos.
  - [ ] Usar la biblioteca en el equipo nuevo no renombra ni duplica ningún archivo.
- **Fuentes:** research/04-biblioteca-compartir.md §1.3 (Move Library to a new computer)

### LIB-033 · Exportar toda la biblioteca a archivos normales
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Export Library on exit (batch-convert editable files to PNG/JPG; videos and GIFs are already plain files)
- **Comportamiento:**
  - Biblioteca > **Exportar toda la biblioteca…** crea en una carpeta elegida una copia de toda la biblioteca, o de lo que muestren los filtros, en formatos estándar:
    - imágenes `.xshot` como PNG (por defecto) o JPG, con las anotaciones aplanadas;
    - videos y GIF copiados tal cual;
    - guías como PDF y una imagen por paso (salidas de `SMT-`).
  - Mantiene la estructura de subcarpetas.
  - Añade un `metadatos.csv` con nombre, fechas, aplicación, sitio web y etiquetas de cada elemento (decisión de diseño).
  - Usa el motor de LIB-018, con su progreso, cancelación y resumen.
  - Pensada para dejar XcreenShot sin perder nada.
- **Criterios de aceptación:**
  - [ ] Exportar una biblioteca con 50 imágenes, 5 videos y 2 guías produce 50 PNG, 5 MP4, 2 PDF con sus imágenes de pasos y el CSV.
  - [ ] El CSV se abre en una hoja de cálculo y tiene una fila por elemento con sus etiquetas.
  - [ ] Los PNG exportados muestran las anotaciones tal como se ven en el editor.
- **Fuentes:** research/04-biblioteca-compartir.md §1.3 (Export Library on exit)

---

## 7. Biblioteca: automatización

### LIB-034 · Comandos de la biblioteca (CLI y MCP)
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (la API COM de Snagit no tenía acceso a la biblioteca; APP-030 lo exige)
- **Comportamiento:**
  - Comandos del área `library` (APP-027). Los nombres de parámetros y los valores van en inglés (convención de APP-):
    - `library.list`: lista elementos con orden y paginación.
    - `library.search`: texto, `tags`, `app`, `site`, `type`, `from`/`to` (fechas), `favorite`; mismas reglas que LIB-005 y LIB-007.
    - `library.get`: devuelve los metadatos de un elemento (los de LIB-012) y su ruta.
    - `library.tag`: añade, quita, renombra o fusiona etiquetas.
    - `library.favorite`: marca o desmarca favoritos.
    - `library.reveal`: muestra un elemento en la biblioteca (LIB-014).
    - `library.delete`: destructivo; exige `--yes` en el CLI.
    - `library.export`: exporta elementos o toda la biblioteca (LIB-018, LIB-033).
    - `library.rebuild`: reconstruye el índice (LIB-023).
    - `library.import`: copia archivos a la biblioteca (LIB-024).
  - Cada elemento se identifica con un **ID estable** que no cambia al renombrar ni al mover el archivo dentro de la biblioteca. El ID se guarda en su `.xmeta` (decisión de diseño).
  - `library.location` es de solo lectura por automatización (APP-032).
  - `library.list` y `library.get` existen desde M1; los demás llegan con sus funciones.
- **Criterios de aceptación:**
  - [ ] `xcreenshot library search --tag cliente --type image --json` devuelve solo imágenes con esa etiqueta, con su ID, nombre y ruta.
  - [ ] El ID de un elemento no cambia tras renombrarlo en la bandeja.
  - [ ] `xcreenshot library delete <id>` sin `--yes` no borra nada y explica por qué.
- **Fuentes:** research/04-biblioteca-compartir.md §8 (COM Server API) (ver 05-app.md, APP-030)

---

## 8. Bandeja reciente

### TRY-001 · Mostrar y ocultar la bandeja
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Recent Captures Tray; Show Recent / Hide Recent button (Windows); Recent button and Option+Cmd+T (Mac)
- **Comportamiento:**
  - La bandeja reciente es una tira horizontal de miniaturas al pie de la vista Editor.
  - Se muestra u oculta con el botón **Recientes** del pie del editor, con Ver > Bandeja reciente o con `Ctrl+Alt+R` (Windows) / `Option+Cmd+T` (macOS).
  - Su estado (visible u oculta) y su altura se recuerdan entre sesiones.
  - Oculta, la bandeja sigue funcionando: las capturas nuevas entran en ella y la navegación por teclado (TRY-013) recorre sus elementos.
  - Cuando una captura llega con la bandeja oculta, el botón **Recientes** muestra un punto discreto hasta que se abre (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] El atajo oculta y vuelve a mostrar la bandeja; tras reiniciar la app, la bandeja conserva el último estado.
  - [ ] Con la bandeja oculta, una captura nueva se abre en el editor y aparece en la bandeja al mostrarla.
  - [ ] El botón Recientes se puede usar con el teclado y su estado (pulsado o no) lo anuncia el lector de pantalla.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Show/Hide tray); research/02-editor.md §1.1

### TRY-002 · Contenido y orden
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Recent Captures Tray content (recent captures and opened files; click to load on the canvas); closed items remain in the Library
- **Comportamiento:**
  - La bandeja contiene los documentos abiertos en el editor: las capturas nuevas, los elementos abiertos desde la biblioteca y los archivos abiertos desde fuera (doble clic, Abrir, soltar).
  - **Orden:** primero los elementos fijados (TRY-004), en el orden que les dio el usuario; luego los no fijados, con el más reciente pegado a los fijados. Abrir de nuevo un elemento que ya está en la bandeja no lo duplica: lo selecciona.
  - Un clic carga el elemento en el lienzo o en el editor de video. El elemento activo se distingue con un borde y su nombre accesible incluye "abierto".
  - **Cada miniatura muestra:** la imagen; la insignia de tipo (duración en videos, "GIF", pasos en guías); el indicador de cambios sin guardar (TRY-011); el ícono de fijado y la insignia de error de guardado (LIB-020). El tooltip muestra nombre, dimensiones o duración y fecha.
  - **Persistencia:** la bandeja se conserva entre sesiones de la app.
  - **Elementos cuyo archivo desapareció** (borrado o movido fuera de XcreenShot): se muestran atenuados con "No encontrado" y ofrecen **Buscar…** (localizar el archivo) o **Quitar de la bandeja**.
  - Quitar un elemento de la bandeja no lo borra: sigue en la biblioteca (TRY-008).
- **Criterios de aceptación:**
  - [ ] Tras tres capturas seguidas, la más reciente aparece justo a la derecha de los elementos fijados.
  - [ ] Abrir desde la biblioteca un elemento que ya está en la bandeja lo selecciona sin crear otra miniatura.
  - [ ] Cerrar y volver a abrir la app conserva los elementos de la bandeja en el mismo orden.
  - [ ] Borrar el archivo de un elemento con el Explorador o el Finder lo muestra como "No encontrado", sin errores al hacer clic en él.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Pin / unpin, Move to Library view); research/02-editor.md §1 (Recent Captures Tray), §1.1

### TRY-003 · Capacidad
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** user-set tray capacity of 1–200 items
- **Comportamiento:**
  - Preferencias > Biblioteca > **Número de elementos de la bandeja reciente**: de 1 a 200 (clave `tray.capacity`; por defecto: 50) (decisión de diseño sobre el valor por defecto).
  - Al superarse la capacidad, sale de la bandeja el elemento no fijado más antiguo. No se borra: sigue en la biblioteca.
  - **Nunca salen solos** los elementos fijados ni los que tienen cambios sin guardar. Si son tantos que superan la capacidad, la bandeja la supera temporalmente y la barra de estado lo indica (decisión de diseño).
  - Reducir la capacidad quita de inmediato los no fijados sobrantes, empezando por los más antiguos.
- **Criterios de aceptación:**
  - [ ] Con capacidad 5 y 5 elementos no fijados, una captura nueva entra y el más antiguo sale de la bandeja, pero sigue en la biblioteca.
  - [ ] Con capacidad 3 y 3 elementos fijados, una captura nueva entra igualmente y la bandeja muestra 4 elementos.
  - [ ] El campo no acepta 0 ni 201 y explica el intervalo válido.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Capacity), §5.2 (Library), §5.3 (General)

### TRY-004 · Fijar y desfijar
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Pin / unpin in the tray (anchored left; new captures to their right; Ctrl+Shift+P / Control+Shift+P; multi-pin); option to remove unpinned items when the Editor closes
- **Comportamiento:**
  - **Fijar** y **Desfijar** se hacen desde el menú contextual, con el ícono de chincheta que aparece al pasar el ratón, o con `Ctrl+Shift+P` (Windows) / `Control+Shift+P` (macOS).
  - Funcionan con varios elementos seleccionados a la vez. Si la selección mezcla fijados y no fijados, la acción los fija todos.
  - Los fijados quedan a la izquierda, separados del resto por una línea fina, y no salen de la bandeja por capacidad (TRY-003) ni por la limpieza al salir (TRY-008).
  - Desfijar devuelve el elemento a la zona de no fijados, en su posición cronológica.
  - Los fijados se conservan entre sesiones.
- **Criterios de aceptación:**
  - [ ] Fijar dos elementos los coloca a la izquierda; la siguiente captura aparece a su derecha.
  - [ ] Con 3 elementos seleccionados, el atajo fija los 3.
  - [ ] Con la limpieza al salir activada, tras reiniciar la app solo quedan los elementos fijados.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Pin / unpin, Close / Close Others / auto-clear); research/02-editor.md §1.1

### TRY-005 · Reordenar
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** reorder tray items (pinned or unpinned) by dragging
- **Comportamiento:**
  - Arrastrar uno o varios elementos dentro de la bandeja los cambia de sitio. Una línea vertical marca dónde caerán.
  - Soltar un elemento no fijado entre los fijados lo fija; soltar uno fijado entre los no fijados lo desfija (decisión de diseño).
  - Con el teclado: `Ctrl+Shift+←`/`→` (Windows) y `Cmd+Shift+←`/`→` (macOS) mueven el elemento con foco una posición (decisión de diseño). El lector de pantalla anuncia la nueva posición.
  - El orden manual se conserva entre sesiones. Las capturas nuevas siguen entrando junto a los fijados.
  - Arrastrar fuera de la bandeja no reordena: combina en el lienzo (TRY-014) o lleva el archivo a otra app (TRY-015).
- **Criterios de aceptación:**
  - [ ] Arrastrar el 5.º elemento a la primera posición de los no fijados lo deja allí, y se mantiene tras reiniciar.
  - [ ] Soltar un elemento no fijado entre los fijados muestra el ícono de fijado.
  - [ ] Mover un elemento con el atajo de teclado cambia su posición y el lector de pantalla la anuncia.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Reorder by drag)

### TRY-006 · Renombrar
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Rename via right-click, F2 or File menu (Windows); File > Rename (Mac)
- **Comportamiento:**
  - **Renombrar** desde el menú contextual de la bandeja o de la biblioteca, desde Archivo > Renombrar o con `F2` (Windows; en macOS no tiene atajo por defecto, se puede asignar).
  - El nombre se edita en el sitio, bajo la miniatura, o en un diálogo si la bandeja está oculta. `Enter` confirma y `Esc` cancela.
  - Cambia el nombre del archivo en disco, sin la extensión, y conserva etiquetas, favorito, historial e ID (LIB-034).
  - Se rechazan los caracteres no válidos en cualquiera de los dos sistemas (`\ / : * ? " < > |`) y los nombres reservados de Windows, con el motivo, para que la biblioteca siga funcionando en ambos (decisión de diseño).
  - Si ya existe otro archivo con ese nombre en la misma carpeta, se avisa y no se renombra.
  - Si el archivo está fuera de la biblioteca, se renombra en su ubicación.
- **Criterios de aceptación:**
  - [ ] `F2` sobre un elemento de la bandeja permite escribir un nombre nuevo, y el archivo en disco cambia de nombre.
  - [ ] Un nombre con `:` se rechaza con un mensaje, también en macOS.
  - [ ] Tras renombrar, el elemento conserva sus etiquetas.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Rename); research/02-editor.md §1.1

### TRY-007 · Tamaño de las miniaturas
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** tray thumbnail size Small / Medium / Large via right-click (Windows)
- **Comportamiento:**
  - Menú contextual de una zona vacía de la bandeja > **Tamaño de las miniaturas**: Pequeño (48 px), **Mediano** (72 px, por defecto) o Grande (112 px), medidos en altura (decisión de diseño). También está en Preferencias > Biblioteca (clave `tray.thumbnailSize`).
  - La altura de la bandeja se ajusta al tamaño elegido.
- **Criterios de aceptación:**
  - [ ] Elegir Grande agranda las miniaturas y la bandeja, y el ajuste se mantiene tras reiniciar.
  - [ ] El ajuste existe y funciona igual en Windows y en macOS.
- **Mejora sobre Snagit:** disponible también en macOS.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Thumbnail size), §5.2 (Editor); research/02-editor.md §1.1, §11

### TRY-008 · Cerrar elementos
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Close (removes from the tray without deleting); Close Other Captures and Close All (Mac); remove unpinned items when the Editor closes (Windows)
- **Comportamiento:**
  - Acciones del menú contextual y de Archivo:
    - **Cerrar**: `Ctrl+W` (Windows) / `Shift+Cmd+W` (macOS). En Windows, con la bandeja vacía, `Ctrl+W` cierra la ventana (APP-004).
    - **Cerrar los demás**.
    - **Cerrar todos**: `Ctrl+Alt+W` / `Option+Cmd+W`.
    - **Cerrar los no fijados** (decisión de diseño).
  - Cerrar quita el elemento de la bandeja y del editor, **sin borrar el archivo**; sigue en la biblioteca.
  - Si hay cambios sin guardar en un elemento que no está en la biblioteca (archivo externo o autoguardado desactivado), se pregunta **Guardar**, **No guardar** o **Cancelar**. Los cambios de un elemento de la biblioteca se guardan solos (EDT- (Guardar)).
  - Preferencia **Quitar de la bandeja los elementos no fijados al salir** (clave `tray.clearUnpinnedOnExit`; por defecto: no). Se aplica al salir de la app; cerrar la ventana del editor no la dispara, porque la app sigue en segundo plano (APP-004) (decisión de diseño).
  - Las acciones con varios elementos se pueden deshacer durante 10 s desde un aviso con **Deshacer**.
- **Criterios de aceptación:**
  - [ ] Cerrar un elemento lo quita de la bandeja y sigue apareciendo en la biblioteca.
  - [ ] Cerrar los demás deja solo el elemento elegido.
  - [ ] Con la limpieza al salir activada, salir y volver a abrir la app deja solo los fijados.
  - [ ] Cerrar un PNG externo con una flecha añadida pregunta si se quiere guardar.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Close / Close Others / auto-clear, Move to Library view), §5.2 (Library)

### TRY-009 · Eliminar desde la bandeja
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Delete (Windows) / Move to Trash (Mac) from the tray (Ctrl+Delete / Cmd+Delete)
- **Comportamiento:**
  - **Eliminar** desde el menú contextual, desde Archivo o con `Ctrl+Supr` (Windows) / `Cmd+Retroceso` (macOS). Usa el modificador para no chocar con borrar objetos del lienzo.
  - Borra el archivo con las mismas reglas que LIB-015: papelera del sistema, confirmación según preferencia, aviso si el borrado es definitivo y **Deshacer**.
  - Funciona con varios elementos seleccionados.
  - Tras eliminar el elemento activo, se carga el siguiente de la bandeja o, si no hay, el editor queda vacío.
- **Criterios de aceptación:**
  - [ ] Eliminar un elemento lo quita de la bandeja y de la biblioteca, y el archivo queda en la papelera del sistema.
  - [ ] Con un objeto seleccionado en el lienzo, `Supr` sin modificador borra el objeto y no el elemento.
  - [ ] **Deshacer** devuelve el elemento a la bandeja y a la biblioteca.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Delete); research/02-editor.md §1.1

### TRY-010 · Duplicar
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Duplicate (Ctrl+Shift+D / Shift+Cmd+D); several items at once (Mac 2026.3.0)
- **Comportamiento:**
  - **Duplicar**, desde el menú contextual de la bandeja o de la biblioteca, desde Archivo o con `Ctrl+Shift+D` / `Shift+Cmd+D`, crea una copia completa e independiente de cada elemento seleccionado.
  - La copia conserva objetos editables, etiquetas y metadatos de captura. No es favorita, no tiene historial de envíos y recibe un ID nuevo.
  - Nombre: el original más " (copia)", " (copia 2)", etc. Se guarda en la misma carpeta que el original.
  - Las copias aparecen en la bandeja junto a cada original. Con un solo elemento, la copia queda activa (decisión de diseño).
  - Los cambios sin guardar del original se incluyen en la copia.
- **Criterios de aceptación:**
  - [ ] Duplicar una captura con una flecha crea "<nombre> (copia)" con la flecha editable.
  - [ ] Con 3 elementos seleccionados, Duplicar crea 3 copias.
  - [ ] Editar la copia no cambia el original.
- **Mejora sobre Snagit:** duplicado múltiple en las dos plataformas.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Duplicate); research/02-editor.md §1.1

### TRY-011 · Indicador de cambios sin guardar
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** unsaved-changes badge on tray items, with a tooltip (Mac)
- **Comportamiento:**
  - Un punto de color de acento en la esquina de la miniatura marca un elemento con cambios que todavía no están en su archivo. El color no es la única señal: tiene contorno y forma propios (APP-045).
  - El tooltip del indicador dice "Cambios sin guardar" y, si el elemento no está en la biblioteca, "Este archivo no está en la biblioteca".
  - El nombre accesible del elemento incluye "sin guardar".
  - El indicador desaparece al guardar el elemento, incluido el guardado automático de la biblioteca (EDT- (Guardar)).
  - Guardar como en otro formato no lo quita, porque el elemento en sí sigue sin guardar (LIB-021).
- **Criterios de aceptación:**
  - [ ] Añadir una flecha a un PNG externo muestra el indicador; guardar lo quita.
  - [ ] Con el autoguardado de imágenes desactivado, una captura nueva aparece con el indicador.
  - [ ] El lector de pantalla anuncia "sin guardar" al llegar a ese elemento.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Unsaved indicator); research/02-editor.md §1 (Unsaved-changes indicator)

### TRY-012 · Selección múltiple y acciones
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** tray multi-select (Ctrl/Cmd+click) with Create Image from Template / Combine in Template and Create Video from Images; Combine Videos from the tray context menu
- **Comportamiento:**
  - Selección en la bandeja: `Ctrl`/`Cmd`+clic añade o quita; `Shift`+clic selecciona un rango; `Ctrl+A`/`Cmd+A` con el foco en la bandeja selecciona todo.
  - Con varios seleccionados, el menú contextual ofrece las acciones que valen para todos, con las mismas reglas de LIB-016:
    - Fijar / Desfijar, Cerrar, Eliminar, Duplicar;
    - Etiquetas…, Marcar como favorito;
    - Compartir ▸, Enlace para compartir (SHR-009, SHR-038), Exportar… (LIB-018);
    - Crear ▸ **Nuevo desde plantilla** (imágenes), **Diapositivas narradas** (imágenes) y **Combinar videos** (videos).
  - El lienzo sigue mostrando el elemento activo. La selección múltiple no cambia lo que se edita.
- **Criterios de aceptación:**
  - [ ] Con 4 imágenes seleccionadas en la bandeja, Crear ▸ Nuevo desde plantilla abre el selector con las 4.
  - [ ] Con 3 videos seleccionados, Crear ▸ Combinar videos los abre en la combinación de videos, en el orden de la bandeja.
  - [ ] Con 2 imágenes y 1 video seleccionados, Diapositivas narradas usa solo las imágenes y avisa del elemento omitido.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Multi-select actions, Combine); research/02-editor.md §1.1

### TRY-013 · Navegación con teclado
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** next / previous capture with Left/Right arrows (Windows) or Cmd+[ / Cmd+] (Mac); trackpad and mouse scrolling of the tray (Mac 2025.1.0)
- **Comportamiento:**
  - **Elemento anterior / siguiente**, desde cualquier parte del editor: `Ctrl+RePág` / `Ctrl+AvPág` (Windows) y `Cmd+[` / `Cmd+]` (macOS). Al llegar al final no da la vuelta (decisión de diseño).
  - **En el perfil Compatibilidad con Snagit, en Windows**, `←` / `→` también cambian de elemento cuando no hay ningún objeto seleccionado en el lienzo (APP-021).
  - **Con el foco en la bandeja** (con `F6` se llega desde el lienzo):
    - `←` / `→` mueven el foco;
    - `Inicio` / `Fin` van a los extremos;
    - `Enter` abre;
    - `Espacio` abre la vista rápida (LIB-013);
    - la tecla Menú o `Shift+F10` abre el menú contextual.
  - La rueda del ratón y el desplazamiento horizontal del trackpad mueven la bandeja en las dos plataformas.
  - El elemento al que se llega se carga en el editor.
- **Criterios de aceptación:**
  - [ ] Con 5 elementos en la bandeja, `Ctrl+AvPág` (Windows) o `Cmd+]` (macOS) carga el siguiente cada vez, hasta el último.
  - [ ] Con el foco en la bandeja, `←` y `Enter` abren el elemento anterior.
  - [ ] La rueda del ratón sobre la bandeja la desplaza en horizontal, también en Windows.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Navigation); research/02-editor.md §1.1

### TRY-014 · Combinar arrastrando al lienzo
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** drag a tray image onto the canvas to combine images
- **Comportamiento:**
  - Arrastrar una imagen de la bandeja sobre el lienzo de otra imagen la añade como **imagen flotante** en el punto donde se suelta. La imagen flotante y sus opciones las define EDT- (Imágenes flotantes).
  - Varias imágenes arrastradas a la vez se colocan en cascada (decisión de diseño).
  - Al pasar por encima del lienzo se muestra una vista previa del lugar donde caerá. Soltar fuera del lienzo no hace nada.
  - La imagen de origen se usa aplanada, tal como se ve, y no cambia.
  - Arrastrar un video de la bandeja al editor de video lo añade al final de la combinación de videos (VID- (Combinar videos)) (decisión de diseño).
  - Arrastrar un elemento sobre sí mismo no hace nada.
- **Criterios de aceptación:**
  - [ ] Arrastrar la imagen B de la bandeja sobre el lienzo de la imagen A crea en A un objeto imagen que se puede mover y redimensionar.
  - [ ] La imagen B queda igual en la bandeja y en la biblioteca.
  - [ ] Arrastrar 3 imágenes a la vez crea 3 objetos en cascada.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Combine); research/02-editor.md §1.1

### TRY-015 · Arrastrar a otras aplicaciones
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** drag a tray item into another app with a default file type for drags (Windows 2024.1.2); video drags keep edits (Mac 2024.2.1)
- **Comportamiento:**
  - Arrastrar uno o varios elementos de la bandeja (o de la biblioteca) a otra aplicación, al escritorio o a una carpeta entrega archivos.
  - Lo que se entrega:
    - imágenes: en el formato de arrastre (SHR-016), con las anotaciones aplanadas y la reducción de alta densidad (SHR-007);
    - videos: con sus recortes y ediciones aplicados;
    - guías: como PDF (decisión de diseño).
  - El nombre del archivo entregado sigue SHR-006.
  - Si hay que preparar el archivo (aplanar o renderizar un video), el puntero muestra que está en curso. Si tarda más de 1 s, aparece un indicador de progreso; si la otra app no espera, se avisa y se ofrece **Copiar todo** (SHR-015).
  - El elemento original no cambia y sigue en la bandeja.
- **Criterios de aceptación:**
  - [ ] Arrastrar una captura anotada a un chat de Teams o Slack entrega un PNG con las anotaciones visibles.
  - [ ] Arrastrar un video recortado al escritorio crea un `.mp4` con la duración recortada.
  - [ ] Arrastrar 3 imágenes a una carpeta crea 3 archivos.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Drag out to other apps), §6 (Drag out of tray/Library)

### TRY-016 · Detalles, etiquetas y otras acciones
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Details / Get Info and tags on tray items (same as Library); tags on videos in the tray (Windows 2024.0.3)
- **Comportamiento:**
  - El menú contextual de un elemento de la bandeja ofrece, además de lo descrito en TRY-004 a TRY-012:
    - Detalles (LIB-012);
    - Etiquetas… (LIB-009) y Marcar como favorito (LIB-011);
    - Vista rápida (LIB-013);
    - Mostrar en la biblioteca (LIB-014);
    - Mostrar en el Explorador / Finder y Copiar ruta;
    - Compartir ▸, Enlace para compartir, Copiar todo (SHR-015) y Exportar….
  - Todas funcionan igual para imágenes, videos, GIF y guías.
  - Los cambios de etiquetas o favorito hechos en la bandeja se ven al momento en la biblioteca, y al revés.
- **Criterios de aceptación:**
  - [ ] Etiquetar un video desde la bandeja hace que aparezca en el filtro de esa etiqueta en la biblioteca.
  - [ ] Detalles desde la bandeja muestra los mismos datos que desde la biblioteca.
  - [ ] Mostrar en la biblioteca desde la bandeja selecciona el elemento en la biblioteca.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Details/Get Info, tags on tray items); research/05-plataformas-versiones.md §1A (2024.0.3)

### TRY-017 · Comandos de la bandeja (CLI y MCP)
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (mejora; APP-027 asigna el área `tray` a este archivo)
- **Comportamiento:**
  - Comandos del área `tray`:
    - `tray.list`: elementos en orden, con su ID de biblioteca, fijado, sin guardar y activo;
    - `tray.open`: abre un elemento de la biblioteca o un archivo y lo pone en la bandeja;
    - `tray.activate`;
    - `tray.pin` / `tray.unpin`;
    - `tray.close` (uno, los demás, todos, los no fijados);
    - `tray.move`.
  - No hay `tray.delete`: para borrar se usa `library.delete` (LIB-034), que exige confirmación.
- **Criterios de aceptación:**
  - [ ] `xcreenshot tray list --json` devuelve los elementos en el mismo orden que la bandeja.
  - [ ] `tray pin <id>` fija el elemento, y la interfaz lo muestra al momento.
  - [ ] `tray close --all` vacía la bandeja sin borrar ningún archivo.
- **Fuentes:** research/04-biblioteca-compartir.md §2; §8 (COM Server API) (ver 05-app.md, APP-027)


## 9. Compartir: marco común

> **Pendiente.** SHR-001 a SHR-017 se redactan antes de M2. Para M1 solo hacen falta los dos destinos de la sección 10.

## 10. Compartir: destinos (alcance de M1)

### SHR-018 · Archivo
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** File share destination
- **Comportamiento:**
  - Guarda la captura en una carpeta elegida, en un formato elegido (PNG por defecto; también JPG y `.xshot`).
  - Opciones: carpeta de destino, formato, nombre automático con los tokens de CAP-053, o preguntar el nombre cada vez.
  - Si el archivo ya existe, se añade un sufijo numérico y nunca se sobrescribe sin preguntar (decisión de diseño).
  - Se puede usar como destino al capturar (CAP-) o desde el editor.
- **Criterios de aceptación:**
  - [ ] Con el destino Archivo activo al capturar, aparece un PNG en la carpeta elegida, con el nombre según el patrón configurado.
  - [ ] Si se elige JPG, el archivo resultante es un JPG válido que abre en el visor del sistema.
  - [ ] Si se capturan dos veces con el mismo nombre, el segundo archivo recibe un sufijo y el primero queda intacto.
  - [ ] Si la carpeta no existe o no tiene permiso de escritura, aparece un error claro y la captura sigue en la biblioteca.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (File)

### SHR-019 · Portapapeles
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Clipboard share destination
- **Comportamiento:**
  - Copia la imagen, con las anotaciones aplanadas, al portapapeles del sistema.
  - Conserva la transparencia cuando la aplicación de destino la admite.
  - Es independiente de la opción "Copiar al portapapeles" al capturar (CAP-), que copia la captura sin editar.
- **Criterios de aceptación:**
  - [ ] Tras compartir al portapapeles, pegar en un editor de documentos inserta la imagen con sus anotaciones.
  - [ ] Una imagen con lienzo transparente pegada en una app compatible conserva la transparencia.
  - [ ] Compartir un video al portapapeles no está disponible: el destino aparece deshabilitado.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Clipboard), §6

> **Pendiente.** SHR-020 a SHR-041 (correo, aplicaciones, nube, Office y Enlace para compartir) se redactan antes de M6, siguiendo el índice de arriba.
