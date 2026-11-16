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
12. **Compartir con otros usuarios de XcreenShot en la red local** — SHR-042 Red local: activar y ver quién está · SHR-043 Red local: emparejar y contactos · SHR-044 Red local: enviar, pendientes y recibir
13. [Atajos por defecto](#12-atajos-por-defecto-de-este-archivo) · [Relacionado](#relacionado) · [Excluido](#excluido) · [Cobertura](#cobertura)

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

### TRY-018 · Portapapeles de XcreenShot
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (función propia: IDEA-006, aprobada por Carlos el 2026-11-16)
- **Comportamiento:**
  - El atajo global **Portapapeles de XcreenShot** abre un panel con las miniaturas de las capturas de la bandeja reciente (TRY-002), en su mismo orden. Por defecto `Win+Alt+V` (Windows) y `Ctrl+Option+Cmd+V` (macOS) (decisión de diseño, provisional). Se puede cambiar en Preferencias > Atajos (APP-017).
  - Panel:
    - pequeño, delante de todo, en el monitor del puntero;
    - sale en las capturas de pantalla, como el portapapeles del sistema (decisión de diseño: así queda como evidencia de una prueba);
    - cada miniatura lleva el nombre de la captura;
    - un campo filtra por nombre.
  - Se eligen una o varias capturas:
    - clic, o `Espacio`, para marcar o desmarcar;
    - `Ctrl`/`Mayús` + clic para varias;
    - las flechas para moverse.
  - El botón **Insertar** (o `Enter`) cierra el panel y pega las capturas elegidas en la app que estaba delante al abrirlo, en el orden de la bandeja.
    - Se pega la **imagen original**, la de la captura, sin anotaciones ni efectos (decisión de Carlos).
    - Varias capturas se pegan **una tras otra**, cada una en su propio pegado (decisión de diseño, provisional; Carlos lo decide "en la marcha").
  - **Copiar** (o `Ctrl+C`) deja en el portapapeles la primera elegida, sin pegar.
  - `Esc`, o hacer clic fuera, cierra sin hacer nada.
  - Los videos y los GIF no se pueden pegar como imagen: aparecen desactivados, con un tooltip que lo explica.
  - Sin capturas en la bandeja, el panel lo dice y ofrece **Capturar**.
  - Pegar es una pulsación simulada de `Ctrl+V` (`Cmd+V` en macOS) en esa app. XcreenShot no se queda con el teclado ni con el ratón.
  - Comandos:
    - `tray.clipboard` abre o cierra el panel;
    - `tray.insert` recibe los documentos (`documentIds`) y `paste`:
      - con `paste: false`, solo copia la primera, sin pegar (por defecto en automatización, para no pegar en una terminal);
      - con `paste: true`, pega todas en la ventana que estaba delante al abrir el panel.
- **Criterios de aceptación:**
  - [ ] Con Paint delante, el atajo, una captura e **Insertar** pegan esa imagen en el lienzo.
  - [ ] Tres capturas elegidas e **Insertar** en Word dejan tres imágenes, en el orden de la bandeja.
  - [ ] Una captura con una flecha se pega sin la flecha.
  - [ ] `Esc` cierra el panel sin cambiar el portapapeles.
  - [ ] Un video aparece desactivado.
  - [ ] `xcreenshot tray insert <documentId> --json` deja la imagen original en el portapapeles y no pega.
- **Fuentes:** IDEA-006 (`spec/IDEAS.md`)


## 9. Compartir: marco común

> **Borrador del 2026-11-15, pendiente de aprobación.** Las fichas SHR-001 a SHR-017 (esta sección) y SHR-020 a SHR-041 (secciones 10 y 11) se redactaron el 2026-11-15 a partir de research/04-biblioteca-compartir.md y del resto del spec. Carlos las revisa antes de construirlas: hasta entonces solo se construye lo que ya piden otras fichas (por ejemplo, el nombre de LIB-020 o el arrastre de TRY-015). Lo que decidieron los redactores va marcado como "(decisión de diseño)". Cada destino lleva una línea **Destino** con su identificador estable (el que usan los presets, el CLI y `policy.disabledDestinations`, APP-035), si es local o en línea, sus medios y si acepta varios elementos.

### SHR-001 · Zona de compartir
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** share area at the right of the Editor toolbar (Copy All, Share Link, Share dropdown); acts on the Library selection
- **Comportamiento:**
  - La zona de compartir está a la derecha de la barra de herramientas del editor (EDT-002), en este orden: **Copiar todo** (EDT-090; en la biblioteca, SHR-015), **Enlace para compartir** (SHR-038) y **Compartir ▾** (SHR-002).
  - Actúa sobre el documento activo en la vista Editor, y sobre la selección en la vista Biblioteca (LIB-001) y en la bandeja.
  - Un botón desactivado dice por qué en su tooltip y en su nombre accesible: "Elige un elemento", "Elige un solo elemento", "Ya se está enviando", "Este tipo de elemento no se puede enviar así" (decisión de diseño).
  - Por debajo de 1200 px de ancho se reduce a íconos (EDT-002); el nombre sigue en el tooltip y en el lector de pantalla.
  - Solo aparece lo que existe (como en APP-008). M1 ya tiene la zona mínima: **Copiar todo** y **Compartir ▾** con Archivo, Portapapeles y Guardar como…; el Enlace para compartir aparece con M6.
  - Si la configuración administrada quita el Enlace para compartir (SHR-039), su botón no aparece.
- **Criterios de aceptación:**
  - [ ] Con dos elementos elegidos en la biblioteca, Enlace para compartir está desactivado y su tooltip pide elegir uno solo; Copiar todo y Compartir siguen activos.
  - [ ] Sin nada elegido en la biblioteca, los tres botones están desactivados y el lector de pantalla anuncia el motivo.
  - [ ] Los tres botones se alcanzan con `Tab` y se activan con `Enter` o `Espacio`.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Share Link button, Share dropdown), §1.1 (Copy All from Library)

### SHR-002 · Menú Compartir
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Share dropdown (enabled destinations; unsupported file types greyed out; Preferences, Save As Preferences, Share History; screen-reader accessible); Share menu in the Windows menu bar
- **Comportamiento:**
  - **Compartir ▾** abre un menú (`Menu`) con los destinos activados, en el orden de Preferencias (SHR-003), incluidas las instancias con nombre (SHR-004) con su atajo a la derecha.
  - Un destino que no acepta lo elegido aparece desactivado, con el motivo en una línea secundaria: "Solo video", "Solo imágenes", "Un elemento cada vez".
  - Los destinos que necesitan una app que no está instalada (Word, Keynote…) no aparecen en el menú; en Preferencias se ven con "No instalada" (decisión de diseño, como SMT-031).
  - Un destino en línea sin sesión iniciada dice "Conectar…": al elegirlo, primero se inicia la sesión (SHR-014) y después se envía.
  - Debajo de los destinos, separadas: **Guardar como…** (EDT-107), **Historial de envíos…** (SHR-012) y **Preferencias de compartir…**.
  - El mismo contenido está en el menú **Compartir** de la barra de menús (la que dibuja la app en Windows; la del sistema en macOS), en Compartir ▸ del menú contextual de la bandeja y de la biblioteca, y en el menú Compartir de la guía (SMT-030).
  - M1 ya tiene un menú mínimo con Archivo (SHR-018), Portapapeles (SHR-019) y Guardar como…; lo demás llega con su ficha.
- **Criterios de aceptación:**
  - [ ] Con un video elegido, Portapapeles aparece desactivado con el motivo "Solo imágenes".
  - [ ] Desactivar un destino en Preferencias lo quita del menú al momento, sin reiniciar.
  - [ ] El menú se recorre con las flechas y el lector de pantalla lee el nombre, el estado y el motivo de cada entrada desactivada.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Share dropdown, Enable/disable destinations)

### SHR-003 · Gestión de destinos
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** enable/disable destinations without restart (Windows); +/– buttons and drag ordering (Mac); admins can disable individual destinations
- **Comportamiento:**
  - Preferencias > Compartir > **Destinos** lista todos los destinos de la plataforma, cada uno con su casilla de activado, ícono, nombre, tipo (local o en línea), cuenta conectada si la tiene y botón **Opciones** (SHR-005).
  - Se ordenan arrastrando o con `Alt`/`Option`+`↑`/`↓`; ese orden es el del menú Compartir. **+** añade una instancia (SHR-004) y **–** la quita. Los cambios se aplican al momento.
  - **Por defecto** están activados los destinos locales disponibles (Archivo, Portapapeles, Correo, Aplicación, Impresora, Compartir del sistema y las apps de oficina instaladas). Los destinos en línea están en la lista, desactivados, hasta que el usuario los activa (decisión de diseño: nada en línea a la vista sin elegirlo).
  - **Identificadores** estables de cada destino:
    - locales: `file` (SHR-018), `clipboard` (SHR-019), `email` (SHR-020), `app` (SHR-021), `printer` (SHR-022), `word` (SHR-024), `powerpoint` (SHR-025), `excel` (SHR-026), `pages`, `keynote` y `numbers` (SHR-036), `airdrop` y `system` (SHR-037);
    - en línea: `ftp` (SHR-023), `evernote` (SHR-028), `onedrive` (SHR-029), `teams` (SHR-030), `slack` (SHR-031), `gdrive` (SHR-032), `dropbox` (SHR-033), `box` (SHR-034), `youtube` (SHR-035), `link` (Enlace para compartir, SHR-038);
    - `onenote` (SHR-027) es local o en línea según cómo envía; `lan` (SHR-042 a SHR-044) tiene sus propias reglas.
  - **Configuración administrada:** un ID de `policy.disabledDestinations` (APP-035) quita ese destino de todas partes: menú Compartir, Preferencias, barra de herramientas (SHR-013), listas Compartir del panel de captura y de video (CAP-052, VID-053) y `share.destinations`. `share.send` a él falla con `disabled_by_policy`. `link` quita también el botón Enlace para compartir.
  - Un preset que usa un destino que ya no existe o está oculto muestra el aviso de ajustes que no existen (CAP-058) y envía al resto.
  - Destinos, orden, instancias y opciones se guardan en `share.destinations` y viajan en la exportación de ajustes (APP-011), **sin** cuentas ni credenciales (SHR-014).
- **Criterios de aceptación:**
  - [ ] Desactivar Correo y mover Archivo al final se refleja en el menú Compartir al momento y se conserva tras reiniciar.
  - [ ] Con `policy.disabledDestinations: ["slack", "link"]`, Slack no aparece en Preferencias, en el menú ni en `share.destinations`, el botón Enlace para compartir no aparece y `xcreenshot share send --to slack` falla con código 6.
  - [ ] En una instalación nueva, ningún destino en línea aparece en el menú Compartir.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Enable/disable destinations, Share dropdown), §5.2 (Share), §5.3 (Share), §9 (Admin deployment controls)

### SHR-004 · Instancias con nombre y atajos por destino
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** add the same destination several times, each with its own Share Name (Mac); keyboard shortcut per destination (Mac)
- **Comportamiento:**
  - Un mismo destino se puede añadir varias veces: dos carpetas con Archivo, dos espacios de Slack, dos servidores FTP. Cada **instancia** tiene su **nombre visible** (único; por defecto, el del destino y lo que la distingue: "Slack — Marketing"), sus opciones (SHR-005) y su cuenta.
  - Cada instancia tiene un identificador `<destino>:<sufijo>` (por ejemplo `slack:a3f9`) que no cambia al renombrarla (decisión de diseño). Presets, barra de herramientas y CLI la guardan por ese identificador.
  - **Atajo** en las opciones de la instancia: atajo de la app (APP-022), sin valor por defecto, con detección de conflictos (APP-019). Al quitar la instancia, se libera.
  - El atajo envía al momento el documento activo o la selección con las opciones de la instancia; si hay que elegir algo (canal, "preguntar cada vez"), abre su diálogo.
  - **Duplicar** una instancia copia sus opciones, sin la cuenta (decisión de diseño).
  - Antes de quitar una instancia que usan presets o botones de la barra, se avisa de cuántos la usan.
- **Criterios de aceptación:**
  - [ ] Con dos instancias de Archivo ("Informes" y "Soporte") con carpetas distintas, el menú muestra dos entradas y cada una guarda en su carpeta.
  - [ ] Renombrar una instancia que usa un preset no rompe el preset.
  - [ ] El atajo de una instancia envía el documento activo y aparece en Preferencias > Atajos, en el grupo Compartir.
- **Mejora sobre Snagit:** instancias y atajos en las dos plataformas.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Multiple instances per destination, Per-destination options), §5.3 (Share)

### SHR-005 · Opciones comunes de los destinos
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** per-destination options (account sign-in/out, privacy level, default folder, ask for file name each time; custom display name on Mac)
- **Comportamiento:**
  - **Opciones** de cada destino o instancia abre su panel. Estas opciones son comunes, cuando el destino las admite:

    | Opción | Valores (por defecto) | Destinos |
    |---|---|---|
    | Nombre visible | texto (el del destino) | todos |
    | Cuenta | **Conectar**, o la cuenta con **Cerrar sesión** (SHR-014) | en línea |
    | Carpeta o ubicación | según el destino | Archivo, FTP, nube |
    | Preguntar nombre (y ubicación) cada vez | Sí / **No** | Archivo, FTP, nube |
    | Formato de imagen | **El formato por defecto al compartir** (SHR-007) / PNG / JPG / `.xshot` | los que reciben un archivo |
    | Quién puede abrir el enlace | según el servicio (su ficha) | los que devuelven enlace |
    | Copiar el enlace al terminar | **Sí** / No | los que devuelven enlace |
    | Avisar al terminar | **Sí** / No | todos (respeta APP-016) |
    | Atajo | ninguno (SHR-004) | todos |

  - Las opciones propias (asunto del correo, canal de Slack, visibilidad de YouTube…) están en la ficha de cada destino.
  - Las opciones son de la instancia. Un preset guarda la instancia, no una copia de sus opciones: cambiarlas cambia lo que hacen los presets que la usan (decisión de diseño).
  - Un destino a medio configurar (por ejemplo, FTP sin servidor) aparece en el menú con "Configurar…" y abre sus opciones.
  - Enviar un `.xshot` con desenfoques o redacciones sin aplanar muestra el aviso de EDT-040 en cualquier destino.
- **Criterios de aceptación:**
  - [ ] Con "Preguntar nombre cada vez" en Archivo, cada envío abre el diálogo de guardar con el nombre de SHR-006 ya puesto.
  - [ ] Con "Copiar el enlace al terminar" desactivado en Dropbox, el envío no cambia el portapapeles y el enlace queda en el historial.
  - [ ] Cambiar la carpeta de una instancia cambia también dónde guarda un preset que la usa.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Per-destination options), §5.2 (Share), §5.3 (Share)

### SHR-006 · Nombres de archivo y títulos
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** option to leave the capture source (app or website) out of shared file names and titles, using date/time instead; descriptive share titles
- **Comportamiento:**
  - **Nombre de lo que sale:** el del elemento en la biblioteca, con la extensión del formato enviado. Las capturas nuevas se llaman `<App> AAAA-MM-DD HH.MM.SS` (LIB-020), o `XcreenShot AAAA-MM-DD HH.MM.SS` si no se conoce la app; si el usuario lo renombró (TRY-006), se usa su nombre.
  - El destino Archivo usa su propio patrón (CAP-053). Los demás usan este nombre, salvo que su ficha diga otra cosa o se elija "Preguntar nombre cada vez" (SHR-005).
  - **Título** en los servicios que lo piden (YouTube, Slack, Teams…): el mismo nombre sin extensión, que se puede cambiar en el diálogo del destino.
  - Preferencia **No incluir el origen de la captura en nombres y títulos** (Preferencias > Compartir; clave `share.excludeSource`; por defecto: desactivada). Activada, solo cambia lo que sale, no la biblioteca:
    - en un nombre automático, el nombre de la app se sustituye por `XcreenShot`;
    - los marcadores `{app}`, `{title}` y `{site}` de los patrones (CAP-053) quedan vacíos;
    - los nombres escritos por el usuario no cambian.
  - **Metadatos:** lo que sale a cualquier destino que no sea Archivo no lleva dentro los metadatos de captura (app, título de ventana, dirección web; CAP-056) (decisión de diseño, por privacidad). Archivo sigue su opción "Incluir los metadatos" (CAP-053).
  - Caracteres no válidos y longitud: las reglas de CAP-053, iguales en las dos plataformas. Si el nombre ya existe en el destino, se añade " (2)", " (3)"…, salvo que el destino diga otra cosa.
  - En M1 basta con el nombre de la biblioteca y el de lo que se arrastra (TRY-015); títulos y preferencia llegan con los destinos (M6).
- **Criterios de aceptación:**
  - [ ] Con la preferencia activada, una captura de Excel enviada a Slack se titula `XcreenShot 2026-11-15 10.22.03`, sin "Excel" en el nombre ni en el título, y en la biblioteca sigue llamándose `Excel 2026-11-15 10.22.03`.
  - [ ] Una captura renombrada a "Error de login" llega a Dropbox como `Error de login.png`.
  - [ ] Un PNG enviado por Correo no lleva los metadatos de captura aunque el destino Archivo los tenga activados.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Capture source in names/titles), §5.2 (Share), §5.3 (Share)

### SHR-007 · Formato al compartir y al guardar como
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Save As Preferences (remember the last format or always use a chosen one); default sharing format (Mac)
- **Comportamiento:**
  - Preferencia **Formato por defecto al compartir** (Preferencias > Compartir; clave `share.imageFormat`; **PNG** / JPG). JPG usa la calidad de EDT-112 y deja blanco lo transparente (CAP-053).
  - Qué se envía, salvo que el destino fije otra cosa (su ficha) o se elija otro formato en sus opciones (SHR-005):

    | Elemento | Se envía como |
    |---|---|
    | Imagen | el formato por defecto al compartir, con objetos y efectos aplanados |
    | Video | MP4 con los cortes aplicados (VID-066) |
    | GIF | GIF, tal cual |
    | Guía | PDF (SMT-033); Word, PowerPoint, Pages y Keynote reciben su propio documento (SMT-030, SMT-031) |

  - Un PNG o JPG sin cambios se envía tal cual, sin volver a codificarlo (decisión de diseño, como TRY-015).
  - El `.xshot` solo sale si se elige en las opciones de un destino que guarda archivos (Archivo, FTP, OneDrive, Google Drive, Dropbox, Box, Red local), para que quien lo recibe pueda seguir editando (decisión de diseño).
  - Preferencia **Formato de Guardar como** (Preferencias > Compartir; clave `share.saveAsFormat`): **Recordar el último usado** (por defecto) o **Usar siempre** un formato de EDT-111.
  - En M1, Guardar como propone el último formato usado (LIB-021); la preferencia de compartir llega con los destinos (M6) y la usa también el destino Archivo (CAP-053).
- **Criterios de aceptación:**
  - [ ] Con JPG como formato por defecto, una captura enviada a Correo llega como JPG, y una con transparencia tiene el fondo blanco.
  - [ ] Un video con un corte enviado a Dropbox llega como MP4 con la duración cortada.
  - [ ] Con "Usar siempre: PNG", Guardar como propone PNG aunque el último guardado fuera JPG.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Save As Preferences), §5.3 (Advanced/Editor: default sharing format), §6 (Drag out of tray/Library)

### SHR-008 · Reducir imágenes de alta densidad
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Mac scales down Retina images when sharing (setting on by default); hold Shift while dragging to skip it
- **Comportamiento:**
  - Preferencia **Reducir las imágenes de alta densidad al compartir** (Preferencias > Compartir; clave `share.downscaleHighDensity`; por defecto: **No**; decisión de diseño: nunca se pierden píxeles sin pedirlo).
  - Activada, una imagen capturada con densidad mayor que 1× (EDT-077: 2× en un Mac Retina, 1,5× con la escala al 150 % en Windows) sale a su tamaño lógico: dividida por su densidad, con un filtro de buena calidad. Una captura de 2880 × 1800 a 2× sale de 1440 × 900.
  - Se aplica a los destinos, a Copiar todo (EDT-090, SHR-015) y a lo que se arrastra fuera (SHR-016).
  - No se aplica a Guardar como, Exportar ni a la exportación por lotes (tienen su propia escala), al `.xshot` ni a los videos.
  - Las imágenes sin densidad conocida (importadas, pegadas) no cambian.
  - Para no reducir una vez: `Shift` al empezar a arrastrar, o `Shift`+clic en el destino (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con la preferencia activada, una captura de 2880 × 1800 a 2× enviada a Archivo como PNG mide 1440 × 900; desactivada, 2880 × 1800.
  - [ ] Con la preferencia activada, arrastrar esa captura con `Shift` entrega 2880 × 1800.
  - [ ] Un PNG importado sin densidad sale con su tamaño original.
- **Mejora sobre Snagit:** también en Windows, con cualquier escala de pantalla.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Retina downscale on share), §5.3 (Advanced/Editor), §6 (Drag out of tray/Library)

### SHR-009 · Compartir varios elementos y a varios destinos
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** share several selected items in one action; presets and the Capture Window can send to several destinations (Windows)
- **Comportamiento:**
  - Con varios elementos elegidos en la biblioteca o la bandeja (TRY-012), Compartir los envía en una sola acción a los destinos con "Varios: sí": un correo con todos los adjuntos, un mensaje de Slack con todos los archivos, N archivos en la carpeta.
  - El Enlace para compartir ("Varios: no") aparece desactivado con "Un elemento cada vez".
  - Si parte de lo elegido no es del tipo que acepta el destino (por ejemplo, videos a Excel), se ofrece **Enviar solo los N compatibles** o **Cancelar** (decisión de diseño).
  - **Varios destinos a la vez:** Compartir ▾ > **Enviar a varios…** abre la lista de destinos e instancias con casillas; se marcan y se pulsa **Enviar** (decisión de diseño). Los presets ya pueden tener varios (CAP-052, VID-053).
  - Cada destino se envía por separado, hasta 3 a la vez (decisión de diseño): si uno falla, los demás siguen y el resultado dice cuál falló (SHR-011).
  - Cada destino de cada envío es una fila del historial (SHR-012).
- **Criterios de aceptación:**
  - [ ] Tres capturas enviadas a Correo abren un solo mensaje con tres adjuntos.
  - [ ] Con 2 imágenes y 1 video elegidos, enviar a Excel ofrece enviar solo las 2 imágenes.
  - [ ] Enviar a la vez a Archivo y a un servidor FTP apagado escribe el archivo y deja el envío FTP como Fallido con Reintentar.
- **Mejora sobre Snagit:** varios destinos también desde el menú Compartir, no solo al capturar.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Multi-file share, Share from Capture / presets)

### SHR-010 · Progreso, estado y cancelación
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** "Sending..." for local targets and "Uploading..." for cloud targets; Share Link progress bar with cancel; video optimization before upload can be cancelled (Mac)
- **Comportamiento:**
  - Los envíos van en segundo plano: se puede seguir capturando, editando y cerrar el documento.
  - Estados: **Preparando** (aplanar, convertir, aplicar los cortes de un video), **Enviando** (destinos locales) o **Subiendo** (en línea) con porcentaje y tamaño, y al final **Terminado**, **Fallido** o **Cancelado**. La Red local añade los suyos (SHR-044).
  - Dónde se ve: la barra de estado del editor (envío del documento activo, con **Cancelar**), un anillo de progreso en la miniatura de la bandeja, y el Historial de envíos (SHR-012) con todos.
  - **Cancelar** detiene el envío y borra lo subido a medias cuando el servicio lo permite; nunca deja un archivo a medias en una carpeta local.
  - Errores de red pasajeros: se reintenta solo hasta 3 veces, con espera creciente (decisión de diseño); después, Fallido (SHR-011). Las subidas grandes se reanudan donde se quedaron si el servicio lo permite.
  - Salir de la app con envíos en curso pregunta **Esperar** o **Salir de todos modos** (APP-004).
  - Eventos `share.started`, `share.succeeded`, `share.failed` y `share.cancelled` (APP-031), con el ID del envío.
  - El lector de pantalla anuncia el inicio y el final de cada envío, no cada porcentaje.
- **Criterios de aceptación:**
  - [ ] Mientras sube un video de 200 MB se puede editar otra captura sin bloqueos, y la barra de estado muestra el porcentaje.
  - [ ] Cancelar a la mitad una subida a S3 no deja ningún objeto en el bucket.
  - [ ] Cortar la red 5 s durante una subida no la hace fallar: sigue al volver la red.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Share Link button, Status and notifications), §3.2 (Screencast: video optimization before upload)

### SHR-011 · Resultado: avisos, errores y reintento
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** completion notices through system notifications; "link copied" notice; share-failed error messages
- **Comportamiento:**
  - **Terminado:** notificación de la categoría Compartir (APP-016), por ejemplo "Enviado a Slack · #general — Enlace copiado", con acciones según el destino: **Copiar enlace**, **Abrir enlace** y **Mostrar código QR** (SMT-037) si hay enlace; **Mostrar en la carpeta** si hay archivo.
  - Los destinos cuyo efecto se ve al momento (Portapapeles, Aplicación, Correo, apps de oficina, Compartir del sistema) solo notifican si fallan (decisión de diseño).
  - **Fallido:** notificación con el destino, el elemento y el motivo en lenguaje claro, con **Reintentar** y **Detalles**. Motivos con mensaje propio:
    - sin conexión;
    - sesión caducada o revocada, con **Volver a conectar** (SHR-014);
    - sin permiso, sin espacio, o archivo mayor que el límite del servicio (dice el límite);
    - carpeta, servidor o app de destino no disponible o que no responde;
    - bloqueado por la configuración administrada o por las reglas de automatización (APP-032).
  - **Reintentar** vuelve a enviar lo que se preparó la primera vez, con las mismas opciones. Lo preparado se guarda en los datos locales de la app hasta que el envío termina o pasan 24 h (decisión de diseño).
  - **Detalles** muestra el error técnico (código y respuesta del servicio) para copiarlo, sin credenciales ni contenido.
  - El registro (APP-046) guarda el destino, el código de error y la hora; nunca el contenido, el enlace ni las credenciales.
- **Criterios de aceptación:**
  - [ ] Con la red desconectada, enviar a Google Drive falla con "Sin conexión" y Reintentar; al volver la red, Reintentar completa el envío.
  - [ ] Con la sesión de Dropbox revocada desde la web del servicio, el envío falla con Volver a conectar y, tras conectar, se completa sin elegir nada de nuevo.
  - [ ] Con las notificaciones de Compartir desactivadas, un envío fallido aparece igualmente como Fallido en el historial.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Status and notifications, Share Link button), §3.2 (Email)

### SHR-012 · Historial de envíos
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Share History (open the item or its location, Copy Link, Open Link, remove entries that stay removed; Open hidden if the local file moved; YouTube entries expire; thumbnails and sorting; shortcut on Mac)
- **Comportamiento:**
  - **Historial de envíos** se abre desde Compartir ▾ (SHR-002) y desde Preferencias > Compartir > **Envíos**, que es la misma vista que ya usa la Red local (SHR-044).
  - Una fila por envío y destino: miniatura, nombre, destino o instancia, cuenta, fecha y hora, estado (SHR-010, SHR-044) y enlace o ubicación remota. Se ordena por fecha (por defecto, lo último arriba), destino o nombre; se filtra por destino y estado; se busca por nombre.
  - Acciones de cada fila: **Abrir**, **Mostrar en la biblioteca**, **Abrir ubicación** (carpeta o página del servicio), **Copiar enlace**, **Abrir enlace**, **Mostrar código QR** (SMT-037), **Reintentar**, **Cancelar**, **Eliminar el enlace** (si el proveedor lo permite, SHR-039) y **Quitar del historial**.
  - Si el archivo local ya no existe, Abrir se desactiva y la fila dice "Archivo local eliminado" (LIB-015). Los enlaces con caducidad pasan a "Caducado" a su hora (SHR-041).
  - Lo quitado no vuelve tras reiniciar. **Borrar historial** vacía la lista, con confirmación; no borra nada remoto.
  - Se guarda en los datos locales de cada equipo, no en la biblioteca: una biblioteca sincronizada no comparte el historial. Como mucho 1000 filas o 1 año (decisión de diseño). Sin credenciales ni contenido.
  - Detalles de un elemento muestra su **Último envío** (LIB-012); un duplicado empieza sin historial (TRY-010).
  - Atajo: ninguno por defecto; en el perfil Compatibilidad, `Option+Shift+S` en macOS (APP-021).
- **Criterios de aceptación:**
  - [ ] Tras enviar una captura a Archivo y a Slack, el historial muestra dos filas, y Copiar enlace en la de Slack copia el mismo enlace que se copió al enviar.
  - [ ] Una fila quitada no vuelve al reiniciar la app.
  - [ ] Borrar el archivo local de un envío deja su fila con "Archivo local eliminado" y Abrir desactivado.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Share History), §10 (Share History)

### SHR-013 · Destinos en la barra de herramientas
- **Hito:** M6 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** add any share destination to the Editor toolbar through Customize Toolbar; custom toolbar destinations survive upgrades
- **Comportamiento:**
  - El panel Personalizar de la barra (EDT-004) tiene un grupo **Compartir** con los destinos e instancias activados; se añaden arrastrándolos, como cualquier elemento.
  - El botón muestra el ícono del destino y, con etiquetas visibles, su nombre visible. Un clic envía el documento activo con las opciones de la instancia, igual que su atajo (SHR-004).
  - Se desactiva, con el motivo en el tooltip, si el destino no acepta el documento activo.
  - Si la instancia se quita, se desactiva en Preferencias o la oculta la configuración administrada, el botón desaparece; si vuelve, reaparece en su sitio (decisión de diseño).
  - Los botones se conservan al actualizar la app y viajan con la barra en la exportación de ajustes (APP-011).
- **Criterios de aceptación:**
  - [ ] Un botón de "Slack — Marketing" en la barra envía el documento activo a ese espacio con un clic.
  - [ ] Desactivar el destino en Preferencias quita el botón; activarlo de nuevo lo devuelve a la misma posición.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Toolbar share buttons), §4 (Toolbar customization)

### SHR-014 · Cuentas conectadas y credenciales
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** sign-in/out per destination; delegated permissions; tokens stored locally; browser-based or system-native sign-in
- **Comportamiento:**
  - Preferencias > Compartir > **Cuentas conectadas** lista cada cuenta (servicio, nombre o correo, instancias que la usan) con **Cerrar sesión** y **Volver a conectar**.
  - **Iniciar sesión** se hace en el navegador del sistema, con OAuth 2.0 y PKCE; XcreenShot nunca muestra un formulario de contraseña de un servicio (decisión de diseño). La vuelta a la app usa el esquema `xcreenshot://` si el servicio lo admite; si solo admite una dirección local, XcreenShot escucha en un puerto aleatorio de `127.0.0.1` solo mientras dura el inicio de sesión (5 min como mucho) (decisión de diseño).
  - Se piden los **permisos mínimos** de cada servicio (los dice su ficha), explicados antes de abrir el navegador.
  - **Credenciales** (fichas de acceso, contraseñas de FTP, claves de S3): solo en el llavero del sistema (Administrador de credenciales en Windows, Llavero en macOS). Nunca en ajustes, en la exportación de ajustes (APP-011), en el historial, en los registros ni en la salida del CLI.
  - **Cerrar sesión** revoca la ficha en el servicio si lo permite y la borra del llavero. Desinstalar con "Borrar también ajustes y datos" borra también estas entradas (APP-042).
  - Con la sesión caducada, el envío falla con **Volver a conectar** (SHR-011) y, al conectar, sigue con lo pendiente.
  - Las cuentas son de cada usuario y equipo: no se sincronizan (XcreenShot no tiene cuenta propia, APP-054) ni viajan con el modo portable (APP-038).
  - Ni el CLI ni los agentes pueden iniciar sesión, leer credenciales ni cambiar cuentas; solo usan las ya conectadas (decisión de diseño; SHR-017).
  - Cada servicio exige registrar una app de desarrollador del proyecto (D-05); su identificador va en la app y no es secreto.
- **Criterios de aceptación:**
  - [ ] Tras conectar Slack, ningún archivo de las carpetas de ajustes y datos de XcreenShot, ni el registro detallado, contiene la ficha de acceso.
  - [ ] Cerrar sesión borra la entrada del llavero y el siguiente envío a ese destino pide conectar.
  - [ ] Con un servicio que vuelve por dirección local, el puerto de `127.0.0.1` se cierra al terminar el inicio de sesión o a los 5 minutos.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Per-destination options), §3.2 (OneDrive, Teams, Slack, Google Drive, Box)

### SHR-015 · Copiar todo desde la biblioteca y la bandeja
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Copy All of the selected Library item (Mac); Copy All of images and videos (Ctrl+Shift+C / Option+Cmd+C); copying a video copies the media, not a path
- **Comportamiento:**
  - **Copiar todo** (botón de la zona de compartir, menú contextual y `Ctrl`/`Cmd`+`Shift`+`C`) con elementos elegidos en la biblioteca o la bandeja copia, sin abrirlos:
    - una imagen: la imagen final aplanada, como EDT-090, y también como archivo PNG;
    - un video: el archivo MP4 con los cortes (VID-066);
    - un GIF: el archivo GIF y su primer fotograma como imagen;
    - una guía: su contenido con formato (SMT-034).
  - Varios elementos se copian como lista de archivos, para pegarlos en una carpeta, un correo o un chat.
  - Sigue SHR-006 (nombre) y SHR-008 (alta densidad). Los archivos temporales se escriben en los datos locales de la app y se borran cuando el portapapeles cambia o al salir de la app (decisión de diseño).
  - Un aviso dentro de la ventana confirma "Copiado" o "3 elementos copiados".
  - Desactivado para los elementos "No encontrado" (TRY-002).
- **Criterios de aceptación:**
  - [ ] Copiar todo una captura anotada desde la biblioteca y pegarla en un editor de documentos muestra las anotaciones.
  - [ ] Copiar todo un video desde la bandeja y pegarlo en una carpeta crea el MP4.
  - [ ] Con tres imágenes elegidas, Copiar todo y pegar en una carpeta crea tres PNG.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Copy All from Library), §6 (Copy All)

### SHR-016 · Arrastrar y soltar
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** drag tray and Library items into other apps; default file type for drags (Windows Advanced tab); Mac uses the default sharing format and Shift to skip Retina scaling; video drags keep edits
- **Comportamiento:**
  - Se arrastra hacia fuera desde la bandeja (TRY-015), la biblioteca y el asa **Arrastrar** de la barra de estado del editor (EDT-105).
  - Preferencia **Formato al arrastrar fuera de XcreenShot** (Preferencias > Compartir; clave `share.dragFormat`): **PNG** (por defecto), JPG, **El formato por defecto al compartir** (SHR-007) o **Archivo editable (`.xshot`)**.
  - Un PNG o JPG sin cambios se entrega tal cual. Un `.xshot`, o un documento con cambios sin guardar, se aplana al formato elegido en un archivo temporal de los datos locales de la app (`drag`), que se borra a la hora.
  - Videos: MP4 con los cortes (VID-066). GIF: tal cual. Guías: PDF (SMT-033).
  - Nombre: SHR-006. Alta densidad: SHR-008 (con `Shift` al empezar a arrastrar, sin reducir).
  - Siempre se entrega una copia: el original no se mueve ni cambia, aunque la otra app pida mover.
  - **Soltar dentro de XcreenShot** lo definen EDT-105 (editor), TRY-014 (combinar en el lienzo) y LIB-024 (importar a la biblioteca).
  - En M1 se entrega siempre PNG, sin la preferencia ni la reducción (TRY-015).
- **Criterios de aceptación:**
  - [ ] Con Formato al arrastrar = JPG, arrastrar una captura anotada a una carpeta crea un JPG con las anotaciones.
  - [ ] Con Archivo editable, arrastrar a una carpeta crea un `.xshot` que se abre en XcreenShot con los objetos editables.
  - [ ] Arrastrar con `Shift` sobre el Explorador no mueve ni borra el archivo de la biblioteca.
- **Fuentes:** research/04-biblioteca-compartir.md §2 (Drag out to other apps), §5.2 (Advanced), §5.3 (Advanced/Editor), §6 (Drag out of tray/Library, Drag in)

### SHR-017 · Comandos de compartir (CLI y MCP)
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (mejora; APP-027 asigna el área `share` a este archivo); salidas de imagen de la API COM (impresora, archivo, portapapeles, correo, FTP)
- **Comportamiento:**
  - Comandos del área `share`:
    - `share.destinations`: destinos e instancias con ID, nombre visible, tipo (local o en línea), medios, si admiten varios, si están activados y si tienen cuenta conectada; los ocultos por la configuración administrada no aparecen;
    - `share.send`: uno o varios elementos (documento abierto, ID de biblioteca o ruta) a uno o varios destinos o instancias, con opciones que sustituyen a las de la instancia solo en ese envío (nombre, formato, canal, carpeta…); devuelve por destino el ID del envío, su estado y la ruta o el enlace; `--wait` espera a que termine;
    - `share.link`: crea un Enlace para compartir y devuelve la dirección;
    - `share.history`, `share.cancel`, `share.retry` y `share.history_remove`;
    - los de la Red local (`share.lan_*`, SHR-044).
  - Reglas de APP-032: los destinos en línea y `share.link` se rechazan con `disabled_by_policy` mientras `automation.allowOnlineShare` esté desactivado; los locales siempre están permitidos.
  - Lo que en la interfaz se elige en un diálogo (canal de Slack o Teams, carpeta con "preguntar cada vez") se pasa como parámetro; si falta, el comando falla con `invalid_params` y lista los valores posibles, sin abrir ventanas (decisión de diseño). Correo abre el mensaje para escribir, como desde la interfaz.
  - No hay comandos para conectar cuentas ni leer credenciales (SHR-014). Cada envío publica sus eventos (SHR-010).
  - Cada comando tiene su ejemplo en `AGENTS.md` (APP-033).
- **Criterios de aceptación:**
  - [ ] `xcreenshot share destinations --json` lista las instancias con su ID y no incluye las ocultas por la configuración administrada.
  - [ ] `xcreenshot share send <id> --to file --wait --json` devuelve la ruta del archivo escrito.
  - [ ] Con `automation.allowOnlineShare` desactivado, `xcreenshot share link <id>` falla con código 6 sin hacer ninguna conexión.
- **Fuentes:** research/04-biblioteca-compartir.md §8 (COM Server API) (ver 05-app.md, APP-027, APP-030, APP-032)

## 10. Compartir: destinos

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

### SHR-020 · Correo
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Destino:** `email` · local · Imagen, Video, GIF, Guía · Varios: sí
- **Paridad con Snagit:** Email destination (image attached or embedded, preset subject and body, compose options, classic and new Outlook) (Windows); Microsoft Outlook destination (Mac)
- **Comportamiento:**
  - Abre un mensaje nuevo en la app de correo predeterminada con lo compartido, listo para escribir. XcreenShot nunca envía el correo por su cuenta.
  - Opciones: **Para**, **Asunto** y **Cuerpo** predefinidos (vacíos por defecto; admiten los marcadores de CAP-053, por ejemplo `{title}`), y **La imagen va**: **Adjunta** (por defecto) o **Dentro del mensaje**.
  - Cómo se abre (decisión de diseño):
    - Windows: por la interfaz de correo del sistema (MAPI simple), que usan Outlook clásico, Thunderbird y otros; si la app predeterminada no la tiene (por ejemplo, el Outlook nuevo), XcreenShot crea un borrador `.eml` con los adjuntos y lo abre con ella;
    - macOS: con el servicio de redactar correo del sistema, que abre Mail, Outlook u otra app predeterminada.
  - **Dentro del mensaje** solo funciona con las apps que lo admiten; con las demás, la imagen va adjunta y se avisa una vez. Videos y GIF van siempre adjuntos; las guías, como PDF adjunto o, dentro del mensaje, con el contenido de SMT-034.
  - Si lo adjunto pasa de 20 MB, se avisa antes de abrir el mensaje y se ofrece **Enviar un Enlace para compartir** en su lugar (SHR-038) (decisión de diseño).
  - Sin app de correo predeterminada, falla con un mensaje que explica cómo elegirla en el sistema.
- **Criterios de aceptación:**
  - [ ] Con Outlook clásico predeterminado en Windows, enviar dos capturas abre un mensaje nuevo con dos PNG adjuntos y el asunto predefinido.
  - [ ] Con el Outlook nuevo predeterminado en Windows, el mensaje se abre igualmente con el adjunto.
  - [ ] Con Mail predeterminado en macOS, enviar una captura abre un mensaje nuevo con la imagen adjunta.
  - [ ] Un video de 50 MB muestra el aviso de tamaño antes de abrir el mensaje.
- **Mejora sobre Snagit:** el mismo destino en las dos plataformas, con cualquier app de correo.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Email, Microsoft Outlook), §3.1 (Share dropdown)

### SHR-021 · Aplicación
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Destino:** `app` · local · Imagen, Video, GIF, Guía · Varios: sí
- **Paridad con Snagit:** Program destination (any .exe; display name, output type, path, parameters) (Windows); Application destination (any Mac app, including apps that accept all file types) (Mac)
- **Comportamiento:**
  - Abre lo compartido en otra aplicación del equipo. Cada app es una instancia (SHR-004): "Aplicación — Paint", "Aplicación — Vista Previa".
  - Al añadirla: **Elegir aplicación** (un `.exe` en Windows, una `.app` en macOS) o una de la lista **Apps que abren este tipo** que da el sistema; **Nombre visible**; **Acepta** imágenes, videos o ambos (por defecto, lo que la app declara abrir); **Formato** (SHR-007).
  - **Parámetros** (avanzado, opcional): con `{file}` (un archivo) o `{files}` (todos); sin ellos, los archivos van al final de la orden.
  - El archivo entregado se escribe en los datos locales de la app (`share`) y se borra a las 24 h, para que la otra app tenga tiempo de abrirlo (decisión de diseño).
  - Varios elementos: una sola llamada si los parámetros usan `{files}` o la app abre varios a la vez; si no, una por archivo.
  - Si la app ya no está en su ruta, el destino aparece desactivado con "No se encuentra la app" y ofrece **Elegir de nuevo**.
- **Criterios de aceptación:**
  - [ ] (Windows) Con Paint como instancia, enviar una captura anotada la abre en Paint con las anotaciones.
  - [ ] (macOS) Con Vista Previa como instancia, enviar dos capturas las abre en Vista Previa.
  - [ ] Con parámetros `--new-window {file}`, la app recibe exactamente esos argumentos.
  - [ ] Una instancia que solo acepta imágenes aparece desactivada con un video elegido.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Program, Application)

### SHR-022 · Impresora
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Destino:** `printer` · local · Imagen, Guía · Varios: sí
- **Paridad con Snagit:** Printer share destination (Windows); print from the Library (Mac)
- **Comportamiento:**
  - Imprime con la vista previa y las opciones de EDT-114 (tamaño, ajuste, orientación) y después el diálogo de impresión del sistema.
  - Varios elementos: una sola vista previa, un elemento por página (LIB-017). Guías: con la paginación de SMT-033.
  - Opción **Imprimir sin preguntar** (por defecto: no): imprime directamente en la impresora elegida en las opciones (por defecto, la predeterminada), con los últimos ajustes usados. Sirve para presets que imprimen cada captura (decisión de diseño).
  - Videos y GIF no se admiten.
- **Criterios de aceptación:**
  - [ ] Enviar una captura a Impresora muestra la vista previa de EDT-114 y, al aceptar, el diálogo del sistema.
  - [ ] Con Imprimir sin preguntar, un preset que comparte a Impresora pone el trabajo en la cola de la impresora elegida sin abrir ninguna ventana de XcreenShot.
- **Mejora sobre Snagit:** también en macOS.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Printer), §1.1 (Print from Library)

### SHR-023 · FTP y SFTP
- **Hito:** M6 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Destino:** `ftp` · en línea · Imagen, Video, GIF, Guía · Varios: sí
- **Paridad con Snagit:** FTP destination (passive mode; custom port on Mac; automatic naming at API level)
- **Comportamiento:**
  - Sube a un servidor propio. Opciones:
    - **Protocolo**: SFTP (por defecto), FTP con TLS (FTPS explícito) o FTP sin cifrar;
    - **Servidor** y **Puerto** (22 o 21 por defecto, según el protocolo);
    - **Usuario** y **Autenticación**: contraseña o, en SFTP, clave privada con su frase; contraseña y frase en el llavero (SHR-014);
    - **Carpeta remota** (se crea si no existe) y **Modo pasivo** (FTP; por defecto: sí);
    - **Nombre**: el patrón de CAP-053 (por defecto, el de SHR-006); **Si ya existe**: Añadir un número (por defecto) o Sobrescribir;
    - **Dirección pública** (opcional): la dirección web de la carpeta remota, por ejemplo `https://files.example.com/capturas/`; si está, al terminar se copia el enlace al archivo (decisión de diseño).
  - **Probar conexión** sube y borra un archivo pequeño.
  - SFTP: la primera vez se muestra la huella de la clave del servidor para aceptarla; si cambia después, el envío se detiene y avisa. Nunca se acepta en silencio (decisión de diseño, como SHR-043).
  - Con FTP sin cifrar, sus opciones muestran siempre un aviso: la contraseña y los archivos viajan sin cifrar.
- **Criterios de aceptación:**
  - [ ] Contra un servidor SFTP de prueba, una captura se sube a la carpeta remota con el nombre del patrón; con Dirección pública, el enlace copiado descarga ese mismo archivo.
  - [ ] Si la clave del servidor SFTP cambia, el envío no se hace y el aviso muestra la huella nueva.
  - [ ] Con una contraseña guardada, ni los ajustes, ni la exportación de ajustes, ni el registro la contienen.
- **Mejora sobre Snagit:** SFTP y FTPS, comprobación de la clave del servidor y enlace público opcional.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (FTP), §8 (COM Server API), §10 (FTP)

### SHR-024 · Word
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas (con Microsoft Word de escritorio instalado)
- **Destino:** `word` · local · Imagen, Guía · Varios: sí
- **Paridad con Snagit:** Microsoft Word destination (insert an image; at the cursor on Mac, with new document, current document or always ask; Step Capture guides as editable text plus images)
- **Comportamiento:**
  - Inserta lo compartido en Word. Solo aparece en el menú si Word está instalado (SHR-002).
  - **Dónde**: **Documento actual, en el cursor** (por defecto; si no hay ninguno abierto, uno nuevo), **Documento nuevo** o **Preguntar cada vez**.
  - La imagen va en línea con el texto, a su tamaño en pantalla (EDT-077) y nunca más ancha que el área de texto, con su nombre como texto alternativo (decisión de diseño). Varios elementos, cada uno en su párrafo.
  - Guías: el documento de SMT-030, con texto editable; con Documento actual, su contenido se inserta en el cursor.
  - Comunicación con Word (decisión de diseño): la automatización de Office en Windows y los Apple Events en macOS, que la primera vez piden el permiso de Automatización del sistema (APP-024). Si Word no responde en 30 s, falla con "Word no respondió" (SHR-011).
- **Criterios de aceptación:**
  - [ ] Con un documento abierto y el cursor a mitad del texto, enviar una captura la inserta en esa posición.
  - [ ] Una captura de 3000 px de ancho se inserta al ancho del área de texto, sin salirse de la página.
  - [ ] Enviar una guía a un documento nuevo da el mismo contenido que Exportar como Word (SMT-030), con el texto editable.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Microsoft Word), §8 (Office add-ins)

### SHR-025 · PowerPoint
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas (con Microsoft PowerPoint de escritorio instalado)
- **Destino:** `powerpoint` · local · Imagen, Video, Guía · Varios: sí
- **Paridad con Snagit:** Microsoft PowerPoint destination (insert onto a slide, "current slide" option, video support, Step Captures as a slideshow with a title slide and editable text)
- **Comportamiento:**
  - Inserta en PowerPoint. Solo aparece si está instalado.
  - **Dónde**: **Diapositiva nueva después de la actual** (por defecto), **Diapositiva actual**, **Presentación nueva** o **Preguntar cada vez**.
  - La imagen se centra y se ajusta a la diapositiva sin deformarse (si es más pequeña, a su tamaño en pantalla), con su nombre como texto alternativo. Varios elementos, una diapositiva nueva por elemento.
  - Video: el MP4 con sus cortes, incrustado en la diapositiva.
  - Guías: la presentación de SMT-030 (diapositiva de título y una por paso), nueva o añadida después de la actual.
  - Comunicación con la app: como Word (SHR-024).
- **Criterios de aceptación:**
  - [ ] Enviar 3 capturas con "Diapositiva nueva después de la actual" añade 3 diapositivas, una imagen en cada una, sin deformar.
  - [ ] Un video enviado a la diapositiva actual se reproduce al presentar.
  - [ ] Una guía de 5 pasos enviada a una presentación nueva tiene 6 diapositivas con el texto editable.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Microsoft PowerPoint)

### SHR-026 · Excel
- **Hito:** M6 · **Prioridad:** Could
- **Plataformas:** Ambas (con Microsoft Excel de escritorio instalado)
- **Destino:** `excel` · local · Imagen · Varios: sí
- **Paridad con Snagit:** Microsoft Excel destination (insert into a workbook)
- **Comportamiento:**
  - Inserta la imagen en Excel, en la hoja activa, con su esquina en la celda elegida. Solo aparece si Excel está instalado.
  - **Dónde**: **Libro actual** (por defecto; si no hay ninguno abierto, uno nuevo), **Libro nuevo** o **Preguntar cada vez**.
  - A su tamaño en pantalla, con su nombre como texto alternativo. Varios elementos, uno debajo de otro con una fila libre entre ellos (decisión de diseño).
  - Comunicación con la app: como Word (SHR-024).
- **Criterios de aceptación:**
  - [ ] Con la celda C5 elegida, la imagen se inserta con su esquina superior izquierda en C5.
  - [ ] Sin Excel abierto, se abre un libro nuevo con la imagen.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Microsoft Excel)

### SHR-027 · OneNote
- **Hito:** M6 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Destino:** `onenote` · local con OneNote de escritorio en Windows; en línea en los demás casos · Imagen, Guía · Varios: sí
- **Paridad con Snagit:** Microsoft OneNote destination (insert onto a page; needs desktop OneNote) (Windows)
- **Comportamiento:**
  - Crea una página de OneNote con lo compartido, titulada con el nombre de SHR-006.
  - Cómo (decisión de diseño):
    - Windows con OneNote de escritorio: a través de la propia app, sin cuenta en XcreenShot;
    - macOS, o Windows sin OneNote de escritorio: por el servicio en línea de OneNote, con la cuenta de Microsoft (SHR-014; permiso para crear páginas), la misma de OneDrive y Teams.
  - **Sección**: **Preguntar cada vez** (por defecto) o una sección fija (bloc y sección). **Abrir la página al terminar** (por defecto: sí).
  - Varios elementos, en una sola página, uno debajo de otro. Guías: una página con el título, el texto de cada paso y su imagen.
- **Criterios de aceptación:**
  - [ ] (Windows, OneNote de escritorio) Enviar una captura crea una página en la sección elegida, con la imagen y el título.
  - [ ] (macOS) Con la cuenta conectada, enviar una captura crea la página en la sección elegida y la abre.
- **Mejora sobre Snagit:** también en macOS.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Microsoft OneNote)

### SHR-028 · Evernote
- **Hito:** M6 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Destino:** `evernote` · en línea · Imagen, Video, GIF · Varios: sí
- **Paridad con Snagit:** Evernote destination (create a note) (Windows)
- **Comportamiento:**
  - Crea una nota con lo compartido, con la cuenta de Evernote conectada (SHR-014).
  - Opciones: **Libreta** (por defecto, la predeterminada de la cuenta), **Título** (SHR-006) y **Copiar las etiquetas de la biblioteca** (por defecto: no).
  - Imágenes y GIF dentro de la nota; videos como adjunto. Varios elementos, en una sola nota. Al terminar, **Abrir la nota**.
  - **Riesgo:** depende de que el servicio dé acceso a su interfaz de programación a una app nueva (D-05). Si no lo da, esta ficha se retira (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Enviar dos capturas crea una nota en la libreta elegida con las dos imágenes.
  - [ ] Con Copiar las etiquetas, la nota lleva las etiquetas que la captura tiene en la biblioteca.
- **Mejora sobre Snagit:** también en macOS.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Evernote)

### SHR-029 · OneDrive y SharePoint
- **Hito:** M6 · **Prioridad:** Should (orden según D-05)
- **Plataformas:** Ambas
- **Destino:** `onedrive` · en línea · Imagen, Video, GIF, Guía · Varios: sí
- **Paridad con Snagit:** OneDrive (Business and Personal) destination with a default folder and video playback in the browser; SharePoint destination (upload and copy link) (Windows); delegated file read/write permission
- **Comportamiento:**
  - Sube a OneDrive (personal, o de trabajo o escuela) o a una biblioteca de documentos de un sitio de SharePoint, con la cuenta de Microsoft conectada (SHR-014), y copia el enlace (SHR-005).
  - **Carpeta**: por defecto, la carpeta de la app (`Aplicaciones/XcreenShot`), que solo pide permiso sobre esa carpeta (decisión de diseño: permiso mínimo). **Elegir otra carpeta…** o un sitio de SharePoint pide entonces el permiso de leer y escribir los archivos del usuario.
  - **Quién puede abrir el enlace**: **Cualquiera con el enlace** (por defecto en cuentas personales), **Personas de mi organización** (por defecto en cuentas de trabajo) o **Solo quien ya tiene acceso**. Si la organización no permite la elegida, se usa la más cercana permitida y el aviso lo dice.
  - **Si ya existe**: Añadir un número (por defecto) o Sobrescribir.
  - Los videos se ven en el reproductor web del servicio. Las subidas grandes van por partes y se reanudan (SHR-010).
- **Criterios de aceptación:**
  - [ ] Con una cuenta personal, enviar una captura la sube a `Aplicaciones/XcreenShot` y el enlace copiado la abre en un navegador sin sesión.
  - [ ] Con una cuenta de trabajo y "Personas de mi organización", el enlace pide iniciar sesión en esa organización.
  - [ ] Un video de 300 MB se sube por partes y, tras cortar la red un momento, termina sin empezar de cero.
- **Mejora sobre Snagit:** también en macOS; por defecto, permiso solo sobre su carpeta.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Microsoft SharePoint, OneDrive)

### SHR-030 · Microsoft Teams
- **Hito:** M6 · **Prioridad:** Should (orden según D-05)
- **Plataformas:** Ambas
- **Destino:** `teams` · en línea · Imagen, Video, GIF, Guía · Varios: sí
- **Paridad con Snagit:** Microsoft Teams destination (Teams/channel or Chats, optional message, link copied; drag-and-drop into Teams; tenant admin must consent)
- **Comportamiento:**
  - Publica lo compartido en un canal de un equipo o en un chat, con la cuenta de Microsoft conectada (SHR-014).
  - Diálogo de envío: **Equipos y canales** o **Chats**; lista con búsqueda para elegir equipo y canal, o el chat; **Mensaje** opcional; **Enviar**.
  - Opción **Destino fijo**: un canal o chat concreto, para enviar sin diálogo (atajo, preset, barra de herramientas).
  - Los archivos se guardan donde Teams guarda los de ese canal o chat, y el mensaje los lleva adjuntos. Al terminar se copia el enlace al archivo.
  - Si la organización exige que un administrador apruebe la app, el inicio de sesión lo dice y explica qué pedirle (D-05).
  - Arrastrar a Teams desde la bandeja sigue funcionando sin cuenta (SHR-016).
- **Criterios de aceptación:**
  - [ ] Enviar una captura a un canal con un mensaje la publica en ese canal con el mensaje y la imagen visible.
  - [ ] Con Destino fijo en un chat, el atajo de la instancia envía sin abrir el diálogo.
  - [ ] En una organización que no aprobó la app, el inicio de sesión muestra la explicación de la aprobación, no un error técnico.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Microsoft Teams), §6 (Drag out of tray/Library)

### SHR-031 · Slack
- **Hito:** M6 · **Prioridad:** Should (orden según D-05)
- **Plataformas:** Ambas
- **Destino:** `slack` · en línea · Imagen, Video, GIF, Guía · Varios: sí
- **Paridad con Snagit:** Slack destination (workspace, Channels or DMs, recipient, optional message, link copied); Slack app with inline preview and playback; workspace authorization synced through the vendor account
- **Comportamiento:**
  - Publica lo compartido en un canal, un mensaje directo o un grupo, con la cuenta de Slack conectada (SHR-014). Cada espacio de trabajo es una instancia (SHR-004).
  - Diálogo: **Canales** o **Mensajes directos**, lista con búsqueda, **Mensaje** opcional y **Enviar**. Opción **Destino fijo**, como en Teams (SHR-030).
  - Imágenes, GIF y videos se ven y se reproducen dentro de Slack. Al terminar se copia el enlace permanente del mensaje.
  - Permisos: subir archivos, publicar mensajes y leer la lista de canales y personas, lo justo para el diálogo (decisión de diseño).
  - La sesión se guarda en el llavero de cada equipo y no se sincroniza (SHR-014). Si el espacio de trabajo exige aprobar las apps, el inicio de sesión lo explica.
- **Criterios de aceptación:**
  - [ ] Enviar un GIF a un canal con un mensaje lo publica con el mensaje y se anima dentro de Slack.
  - [ ] Con dos instancias de dos espacios de trabajo, cada una publica en el suyo.
  - [ ] El enlace copiado abre el mensaje publicado.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Slack), §8 (Slack app)

### SHR-032 · Google Drive
- **Hito:** M6 · **Prioridad:** Should (orden según D-05)
- **Plataformas:** Ambas
- **Destino:** `gdrive` · en línea · Imagen, Video, GIF, Guía · Varios: sí
- **Paridad con Snagit:** Google Drive destination (limited file scope; uploads only to an app folder with no folder choice; file-name prompt; permission customization)
- **Comportamiento:**
  - Sube a Google Drive con la cuenta de Google conectada (SHR-014) y copia el enlace.
  - **Permiso limitado** (decisión de diseño, el que el servicio da sin revisiones especiales): XcreenShot solo ve y crea sus propios archivos. Todo va a la carpeta **XcreenShot** de Mi unidad, que se crea la primera vez, con una **Subcarpeta** opcional (ninguna por defecto, o un patrón como `{date}`).
  - **Preguntar nombre cada vez** (SHR-005).
  - **Quién puede abrir el enlace**: **Cualquiera con el enlace puede ver** (por defecto), **Personas de mi organización** (cuentas de trabajo) o **Restringido** (solo quien ya tiene acceso). Si la organización no lo permite, como en SHR-029.
  - Las subidas grandes se reanudan (SHR-010).
- **Criterios de aceptación:**
  - [ ] El primer envío crea la carpeta XcreenShot, y el enlace copiado abre la captura en un navegador sin sesión.
  - [ ] Con Subcarpeta = `{date}`, dos capturas del mismo día van a la misma subcarpeta.
  - [ ] Con Restringido, el enlace pide acceso en un navegador sin sesión.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Google Drive)

### SHR-033 · Dropbox
- **Hito:** M6 · **Prioridad:** Could (orden según D-05)
- **Plataformas:** Ambas
- **Destino:** `dropbox` · en línea · Imagen, Video, GIF, Guía · Varios: sí
- **Paridad con Snagit:** Dropbox destination (upload and copy link; ask for file name and location each time, rename or overwrite, cancel on Mac)
- **Comportamiento:**
  - Sube a Dropbox con la cuenta conectada (SHR-014) y copia el enlace.
  - **Acceso** (decisión de diseño, permiso mínimo): por defecto, solo la carpeta de la app (`Aplicaciones/XcreenShot`). La opción **Todo mi Dropbox** permite elegir cualquier carpeta y pide de nuevo el permiso.
  - **Carpeta** por defecto dentro de lo permitido. **Preguntar nombre y carpeta cada vez** (por defecto: no) abre un diálogo propio con las carpetas de Dropbox.
  - **Si ya existe**: Añadir un número (por defecto), Sobrescribir o Preguntar.
  - **Quién puede abrir el enlace**: cualquiera con el enlace (por defecto) o solo los miembros del equipo (cuentas de empresa).
- **Criterios de aceptación:**
  - [ ] Enviar una captura la sube a `Aplicaciones/XcreenShot` y el enlace copiado la muestra en un navegador sin sesión.
  - [ ] Con Si ya existe = Preguntar, enviar dos veces con el mismo nombre pregunta antes de la segunda subida.
  - [ ] Cancelar la subida de un video no deja un archivo a medias en Dropbox.
- **Mejora sobre Snagit:** todas las opciones en las dos plataformas.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Dropbox)

### SHR-034 · Box
- **Hito:** M6 · **Prioridad:** Could (orden según D-05)
- **Plataformas:** Ambas
- **Destino:** `box` · en línea · Imagen, Video, GIF, Guía · Varios: sí
- **Paridad con Snagit:** Box destination (upload and copy link; browser-based sign-in)
- **Comportamiento:**
  - Sube a Box con la cuenta conectada (SHR-014) y copia el enlace compartido.
  - **Carpeta**: por defecto, **XcreenShot** en la raíz (se crea la primera vez); **Elegir carpeta…** muestra las carpetas de la cuenta.
  - **Quién puede abrir el enlace**: **Cualquiera con el enlace** (por defecto en cuentas personales), **Personas de mi empresa** (por defecto en cuentas de empresa) o **Solo colaboradores**; si la cuenta no permite la elegida, como en SHR-029.
  - **Si ya existe**: Añadir un número (por defecto) o **Nueva versión** del archivo existente.
- **Criterios de aceptación:**
  - [ ] Enviar una captura la sube a la carpeta XcreenShot y copia un enlace compartido que la abre.
  - [ ] Con Si ya existe = Nueva versión, enviar dos veces con el mismo nombre deja un archivo con dos versiones.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Box)

### SHR-035 · YouTube
- **Hito:** M6 · **Prioridad:** Could (orden según D-05)
- **Plataformas:** Ambas
- **Destino:** `youtube` · en línea · Video · Varios: sí (una subida por video)
- **Paridad con Snagit:** YouTube destination (upload video, copy link, embed code; sign-in grants channel management); YouTube entries expire in Share History (Mac)
- **Comportamiento:**
  - Sube videos a un canal de YouTube con la cuenta de Google conectada (SHR-014). Permiso: solo subir videos (decisión de diseño: el mínimo).
  - Diálogo antes de subir: **Título** (SHR-006; hasta 100 caracteres, con contador), **Descripción**, **Visibilidad**: **Privado** (por defecto), **Oculto** (solo con el enlace) o **Público**, y **¿Está hecho para niños?**, que el servicio exige contestar (sin valor por defecto).
  - Al terminar copia el enlace del video; el aviso y el historial ofrecen también **Copiar código para insertar**. El aviso dice que el servicio puede tardar unos minutos en procesarlo.
  - Al conectar la cuenta se muestran los enlaces a las condiciones de YouTube y a la política de privacidad de Google, como exige el servicio.
  - **Riesgo** (D-05): el servicio deja privados los videos que suben las apps que no ha revisado. Hasta que revise la de XcreenShot, la visibilidad se limita a Privado y se explica (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con la app ya revisada, un video subido como Oculto da un enlace que lo abre en un navegador sin sesión.
  - [ ] Copiar código para insertar copia un `<iframe>` con el identificador del video.
  - [ ] El diálogo no deja subir sin contestar si el video está hecho para niños.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (YouTube), §3.1 (Share History)

### SHR-036 · Pages, Keynote y Numbers
- **Hito:** M6 · **Prioridad:** Could
- **Plataformas:** macOS (estas apps solo existen en macOS)
- **Destino:** `pages` (Imagen, Guía), `keynote` (Imagen, Video, Guía) y `numbers` (Imagen) · local · Varios: sí
- **Paridad con Snagit:** Pages / Keynote / Numbers destinations (insert into documents, presentations or sheets; new or current document for Pages; Step Captures to Pages and Keynote as editable content)
- **Comportamiento:**
  - Tres destinos, uno por app; cada uno aparece solo si su app está instalada.
  - **Pages**, como Word (SHR-024): **Documento actual, en el cursor** (por defecto), **Documento nuevo** o **Preguntar cada vez**. Guías, como en SMT-031.
  - **Keynote**, como PowerPoint (SHR-025): diapositiva nueva después de la actual (por defecto), diapositiva actual o presentación nueva; videos incrustados. Guías, como en SMT-031.
  - **Numbers**, como Excel (SHR-026): en la hoja activa del documento actual o en uno nuevo.
  - Se comunican con las apps por Apple Events, con el permiso de Automatización del sistema (APP-024).
- **Criterios de aceptación:**
  - [ ] Con un documento de Pages abierto, enviar una captura la inserta en el cursor.
  - [ ] Enviar una guía a Keynote crea una presentación con diapositiva de título y una por paso, con texto editable.
  - [ ] Sin Numbers instalado, Numbers no aparece en el menú Compartir.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Pages / Keynote / Numbers)

### SHR-037 · AirDrop y Compartir del sistema
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas (AirDrop, solo macOS: es una función de los sistemas de Apple)
- **Destino:** `system` (ambas) y `airdrop` (macOS) · local · Imagen, Video, GIF, Guía · Varios: sí
- **Paridad con Snagit:** Apple apps through the macOS share menu (AirDrop, Mail, Messages, Notes, Photos, Freeform, Journal, Reminders); third-party share extensions enabled in System Settings (Mac)
- **Comportamiento:**
  - **Compartir del sistema** (`system`) abre la ventana de compartir del sistema con los archivos preparados (SHR-007):
    - macOS: el menú de compartir del sistema (AirDrop, Mail, Mensajes, Notas, Fotos, Freeform, Recordatorios y las extensiones de otras apps activadas en Ajustes del Sistema);
    - Windows: la ventana Compartir de Windows (Uso compartido en proximidad, correo y las apps que la admiten).
  - **AirDrop** (`airdrop`, macOS) abre directamente el selector de AirDrop.
  - XcreenShot solo entrega los archivos al sistema; lo que pasa después lo controla el sistema o la otra app. El historial anota "Entregado al sistema" (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] (macOS) AirDrop abre el selector con la captura y un iPhone la recibe con sus anotaciones.
  - [ ] (Windows) Compartir del sistema abre la ventana Compartir de Windows con el PNG, y Uso compartido en proximidad lo envía a otro PC.
  - [ ] (macOS) Una extensión de compartir activada en Ajustes del Sistema aparece en el menú de compartir del sistema.
- **Mejora sobre Snagit:** Compartir del sistema también en Windows.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Misc Apple apps), §3.1 (Mac third-party share extensions)

## 11. Enlace para compartir

El Enlace para compartir sustituye al servicio de enlaces del fabricante (SPEC §3.2, D-02): sube el elemento a un almacenamiento que elige el usuario o su organización y copia la dirección.

### SHR-038 · Enlace para compartir
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Destino:** `link` · en línea · Imagen, Video, GIF, Guía · Varios: no
- **Paridad con Snagit:** Share Link button (one click uploads the selected file and copies a URL; disabled states with tooltips; progress bar and "link copied" notice; cancellable; Ctrl+Shift+L / Option+Cmd+L, customizable)
- **Comportamiento:**
  - Botón **Enlace para compartir** de la zona de compartir (SHR-001), entradas en el menú Compartir y en el menú contextual, y atajo de la app `Ctrl+Shift+L` (Windows) / `Option+Cmd+L` (macOS), que se puede cambiar (APP-022).
  - Un clic prepara el elemento (SHR-007: imagen en el formato por defecto, video MP4, GIF, guía en PDF), lo sube con el proveedor configurado (SHR-039) y copia la dirección al portapapeles. Aviso "Enlace copiado" con **Abrir enlace** y **Mostrar código QR** (SMT-037).
  - Mientras sube, el botón muestra el progreso y se puede cancelar (SHR-010).
  - Desactivado, con el motivo en el tooltip: nada elegido, varios elementos, ese elemento ya se está subiendo, o elemento "No encontrado".
  - Sin proveedor, el primer clic abre un asistente corto para elegirlo (SHR-039) y después sube.
  - Si el elemento no ha cambiado desde su último enlace con el mismo proveedor y ese enlace no ha caducado, se copia ese enlace sin subir de nuevo; `Shift`+clic crea uno nuevo (decisión de diseño).
  - Cada enlace queda en el historial (SHR-012), desde donde se puede eliminar (SHR-039). Por CLI y MCP: `share.link` (SHR-017).
- **Criterios de aceptación:**
  - [ ] Con un proveedor configurado, un clic deja en el portapapeles una dirección que, abierta en otro equipo, muestra la imagen con sus anotaciones.
  - [ ] Dos clics sobre la misma captura sin cambios dan el mismo enlace, y la segunda vez no se sube nada.
  - [ ] Con dos elementos elegidos, el botón está desactivado y el tooltip lo explica.
  - [ ] Cancelar durante la subida no deja el enlace en el portapapeles ni el archivo en el proveedor.
- **Mejora sobre Snagit:** el almacenamiento lo elige el usuario o su organización, sin depender de un servicio del fabricante.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Share Link button); SPEC §3.2 y D-02

### SHR-039 · Proveedores del Enlace para compartir
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** customizable Share Link target (admins can redirect it to OneDrive or Google Drive, or disable it; one-time explanation for users)
- **Comportamiento:**
  - Preferencias > Compartir > **Enlace para compartir**: **Proveedor** y sus opciones. Por defecto: **ninguno** (decisión de diseño: nada sale hasta que el usuario elige dónde).
  - Proveedores (D-02):
    - **Carpeta sincronizada** (SHR-040), el primero que se construye;
    - **Almacenamiento compatible con S3** (SHR-041), que sirve también para un servidor propio;
    - **Un destino en línea ya conectado que devuelve enlace**: OneDrive (SHR-029), Google Drive (SHR-032), Dropbox (SHR-033) o Box (SHR-034), con la carpeta y el tipo de enlace de esa instancia.
  - Todos cumplen la misma interfaz interna (subir, dar la dirección, cancelar, borrar), para añadir otros sin tocar el resto ("configurable por plugin", D-02). En esta versión no se cargan proveedores de terceros (decisión de diseño).
  - **Probar** sube un archivo pequeño, comprueba que la dirección responde y lo borra.
  - **Eliminar el enlace** (historial, SHR-012) borra el archivo del proveedor; la fila pasa a "Enlace eliminado".
  - **Configuración administrada:** `policy.shareLinkProvider` (APP-035) fija el proveedor y sus opciones sin secretos, o lo quita de toda la app con `{"provider": "none"}`. Por ejemplo:
    ```json
    "policy": { "shareLinkProvider": { "provider": "s3", "endpoint": "https://s3.example.com", "bucket": "capturas", "link": "signed", "expiry": "7d" } }
    ```
    Las credenciales las pone siempre el usuario (SHR-014). Cuando la organización fija o cambia el proveedor, el usuario ve una vez un aviso que lo explica.
- **Criterios de aceptación:**
  - [ ] En una instalación nueva, Enlace para compartir no sube nada hasta elegir un proveedor; el primer clic abre el asistente.
  - [ ] Con `policy.shareLinkProvider` de S3, Preferencias muestra el proveedor y sus opciones bloqueados y solo pide la clave de acceso.
  - [ ] Con `{"provider": "none"}`, el botón y las entradas de menú no aparecen y `xcreenshot share link` falla con `disabled_by_policy`.
  - [ ] Eliminar el enlace desde el historial hace que la dirección deje de abrir el archivo.
- **Fuentes:** research/04-biblioteca-compartir.md §3.1 (Customizable Share Link target), §9 (Admin deployment controls); SPEC D-02; 05-app.md, APP-035

### SHR-040 · Proveedor: carpeta sincronizada
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (proveedor propio, D-02); equivale a redirigir el enlace a OneDrive o Google Drive
- **Comportamiento:**
  - Copia el archivo a una carpeta que un cliente de sincronización sube a la nube y copia una dirección para abrirlo.
  - **Carpeta**: una de las carpetas sincronizadas que XcreenShot detecta (OneDrive, Google Drive, Dropbox, Box; como en LIB-029) o cualquier otra. Dentro se usa `XcreenShot/Enlaces`, y el nombre es el de SHR-006 con un sufijo aleatorio corto, para no pisar nunca otro archivo (decisión de diseño).
  - XcreenShot espera a que el sistema indique que el archivo ya está subido (estado "Esperando a la sincronización", como mucho 10 min); si no llega, falla con "La carpeta no terminó de sincronizar".
  - **Tipo de enlace:**
    - **Enlace del servicio** (OneDrive y Dropbox): se pide al servicio el enlace de ese archivo con la cuenta conectada (SHR-014) y la opción Quién puede abrir el enlace de SHR-029 o SHR-033;
    - **Dirección base**: la dirección web que corresponde a la carpeta (por ejemplo, una carpeta que publica un servidor web propio); el enlace es esa dirección más el nombre del archivo. No necesita cuenta.
  - **Google Drive:** con el permiso limitado de SHR-032, XcreenShot no puede pedir el enlace de un archivo que subió el cliente de escritorio. Para Google Drive se usa el proveedor "destino conectado" con Google Drive (SHR-039), que sube por su cuenta (decisión de diseño).
  - Eliminar el enlace borra el archivo de la carpeta, y el cliente lo borra de la nube.
- **Criterios de aceptación:**
  - [ ] Con la carpeta de OneDrive y Enlace del servicio, el enlace copiado abre la captura en un navegador sin sesión, una vez sincronizada.
  - [ ] Con Dirección base `https://capturas.example.com/` servida desde la carpeta, el enlace es esa dirección más el nombre del archivo y lo abre.
  - [ ] Con el cliente de sincronización cerrado, el envío espera y, al pasar el límite, falla con el mensaje de sincronización.
- **Mejora sobre Snagit:** el enlace sale del almacenamiento del propio usuario.
- **Fuentes:** SPEC D-02; research/04-biblioteca-compartir.md §1.2 (Cloud Library: detección de clientes de sincronización), §3.1 (Customizable Share Link target)

### SHR-041 · Proveedor: almacenamiento compatible con S3
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (proveedor propio, D-02: servidor propio)
- **Comportamiento:**
  - Sube a un almacenamiento compatible con S3: el de Amazon, otros compatibles o un servidor propio con un servicio compatible (por ejemplo, el Mac Mini de D-02).
  - Opciones: **Dirección del servicio** (vacía = Amazon, según la región), **Región**, **Bucket**, **Prefijo** (por defecto `xcreenshot/`), **Usar rutas** (para servidores propios), **ID de clave de acceso** y **Clave secreta** (en el llavero, SHR-014).
  - **Tipo de enlace:**
    - **Firmado con caducidad** (por defecto): una dirección firmada que caduca a la hora, al día o a los **7 días** (por defecto; el máximo que permite la firma). El bucket puede seguir siendo privado;
    - **Público**: una dirección base pública (el propio bucket o un CDN delante) más la ruta del objeto.
  - Ruta del objeto: `<prefijo><AAAA-MM>/<código aleatorio de 22 caracteres>/<nombre>`, que no se puede adivinar ni recorrer (decisión de diseño). El objeto lleva su tipo de contenido, para que el navegador lo muestre en vez de descargarlo.
  - Archivos grandes por partes; cancelar aborta la subida sin dejar partes (SHR-010).
  - Eliminar el enlace borra el objeto. Los enlaces firmados pasan a "Caducado" en el historial a su hora (SHR-012).
- **Criterios de aceptación:**
  - [ ] Contra un servidor compatible con S3 de prueba en el equipo, una captura se sube y el enlace firmado la devuelve idéntica (mismo hash).
  - [ ] Un enlace firmado con caducidad de 1 hora lleva esa caducidad y el servidor lo rechaza pasada la hora.
  - [ ] Con enlace Público y una dirección base, el enlace es la dirección base más la ruta del objeto.
  - [ ] La clave secreta no aparece en los ajustes, en la exportación de ajustes ni en los registros.
- **Mejora sobre Snagit:** enlaces en un almacenamiento propio o de la organización, con caducidad.
- **Fuentes:** SPEC D-02; research/04-biblioteca-compartir.md §3.1 (Customizable Share Link target)

## 12. Compartir con otros usuarios de XcreenShot en la red local

Función propia, aprobada por Carlos el 2026-11-14 (IDEA-005, con sus propuestas). Se envía de un equipo a otro **dentro de la misma red** (oficina, casa o VPN), sin servidor ni servicio de terceros. Por internet haría falta un servidor que guarde los envíos: queda fuera de este alcance (IDEA-005).

### SHR-042 · Red local: activar y ver quién está
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas (Windows y macOS se ven y se envían entre sí)
- **Paridad con Snagit:** — (función propia; IDEA-005)
- **Comportamiento:**
  - Preferencia **Compartir en la red local** (Preferencias > Compartir; por defecto: **desactivada**). Mientras está desactivada, XcreenShot no anuncia nada, no busca a nadie y no abre ningún puerto (APP-054).
  - Al activarla:
    - el equipo genera su par de claves, que se guarda en el llavero del sistema (nunca en archivos ni en logs);
    - pide el **nombre visible** (por defecto, el nombre del usuario del sistema) y lo muestra junto al nombre del equipo;
    - el sistema puede preguntar por el firewall la primera vez; XcreenShot explica antes por qué lo necesita.
  - **Descubrimiento:** los equipos con la preferencia activada se anuncian y se encuentran solos en la red local (DNS-SD/mDNS), sin configurar nada. Cada uno aparece con su nombre visible, su equipo, si ya es contacto (SHR-043) y si está **en línea**.
  - La lista de **Personas en la red** está en el menú Compartir (destino **Red local**), en la biblioteca y en Preferencias > Compartir. Se actualiza sola: quien se va desaparece o pasa a "sin conexión" (sus contactos siguen en la lista).
  - El anuncio solo dice el nombre visible, el equipo y una huella de su clave pública: nunca qué capturas hay ni ningún otro dato.
- **Criterios de aceptación:**
  - [ ] Con la preferencia desactivada, un monitor de red no ve ningún paquete mDNS de XcreenShot ni ningún puerto abierto por la app.
  - [ ] Dos equipos en la misma red, con la preferencia activada, se ven en la lista en menos de 5 s.
  - [ ] Al cerrar XcreenShot en uno, el otro lo muestra como sin conexión en menos de 30 s.
  - [ ] Un Windows y un Mac en la misma red se ven entre sí.
- **Mejora sobre Snagit:** función propia.
- **Fuentes:** IDEA-005 (spec/IDEAS.md), decisión de Carlos del 2026-11-14

### SHR-043 · Red local: emparejar y contactos
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (función propia; IDEA-005)
- **Comportamiento:**
  - **Emparejar la primera vez**, como un dispositivo:
    - quien envía elige a alguien de la lista que todavía no es contacto, y el otro recibe un aviso "Carlos (PC-OFICINA) quiere añadirte como contacto";
    - los dos ven el mismo **código de verificación** de 6 dígitos, derivado de las claves de ambos, y aceptan solo si coincide;
    - aceptado por los dos, cada uno guarda la clave pública del otro: a partir de ahí cada envío comprueba que el otro es quien dice ser.
  - Sin emparejar no se envía nada: un desconocido de la red no puede mandar capturas.
  - **Contactos** (Preferencias > Compartir > Red local):
    - lista con nombre visible, equipo, última vez visto y la opción **Aceptar siempre sus envíos** (desactivada por defecto);
    - se pueden renombrar y quitar; quitar un contacto borra su clave y hay que volver a emparejar;
    - **Bloquear** a alguien hace que sus peticiones de contacto y sus envíos se ignoren sin aviso.
  - Si un contacto reinstala o cambia de equipo, su clave cambia: aparece como desconocido y hay que emparejar de nuevo, con un aviso de que la clave ya no es la misma (decisión de diseño: nunca se acepta una clave distinta en silencio).
- **Criterios de aceptación:**
  - [ ] Emparejar muestra el mismo código en los dos equipos; si uno rechaza, ninguno queda como contacto del otro.
  - [ ] Un equipo no emparejado no puede enviar: su intento no llega al otro.
  - [ ] Tras quitar un contacto, enviarle vuelve a pedir el emparejamiento.
  - [ ] Si el otro reinstala XcreenShot, el envío no sale hasta emparejar de nuevo, y el aviso lo explica.
- **Mejora sobre Snagit:** función propia.
- **Fuentes:** IDEA-005 (spec/IDEAS.md), decisión de Carlos del 2026-11-14

### SHR-044 · Red local: enviar, pendientes y recibir
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (función propia; IDEA-005)
- **Comportamiento:**
  - **Enviar:** desde el menú Compartir (destino **Red local**) del editor, de la bandeja o de la biblioteca, y desde el menú contextual de un elemento. Se eligen uno o varios contactos y uno o varios elementos: imagen (`.xshot` editable, o PNG si se elige), video, GIF o guía.
  - El envío va **directo** del equipo de quien envía al del otro, **cifrado de extremo a extremo** con las claves del emparejamiento; nada pasa por un servidor.
  - **Si el otro no está en línea:**
    - el envío queda **pendiente** en el equipo de quien envía, en el historial de envíos (SHR-012), y sale solo cuando el otro aparece en la red, mientras XcreenShot esté abierto (también en segundo plano);
    - un pendiente se puede cancelar;
    - caduca a los **7 días**, con un aviso (decisión de diseño).
  - **Recibir:**
    - llega un aviso "Carlos te envía 2 capturas (3,4 MB)" con **Aceptar** y **Rechazar**; de un contacto con **Aceptar siempre** se recibe sin preguntar, con un aviso al terminar;
    - lo aceptado llega a la **biblioteca**, como la carpeta de recepción (CAP-043), con la etiqueta "De: <nombre visible>", y aparece en la bandeja;
    - lo rechazado no deja nada.
  - **Estados**, en el historial de envíos (SHR-012): Pendiente, Enviando (con progreso), Entregado, Rechazado, Caducado, Cancelado. Quien envía ve "Entregado" o "Rechazado" cuando el otro decide.
  - **Límites:** sin límite de tamaño en la red local; un envío grande muestra el progreso y se puede cancelar en los dos lados (SHR-010).
  - **Automatización:** comandos para listar a las personas de la red, emparejar, enviar, ver los pendientes y aceptar o rechazar lo recibido (SHR-017). Aceptar un emparejamiento o un envío por automatización exige que la preferencia **Permitir por automatización** esté activada (desactivada por defecto), para que un agente no acepte capturas en nombre del usuario sin que lo sepa (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Enviar una captura `.xshot` a un contacto en línea, que la acepta, la deja en su biblioteca idéntica (mismo contenido, comprobado por hash) y editable.
  - [ ] Con el otro equipo apagado, el envío queda Pendiente y sale solo, sin intervenir, cuando el otro vuelve a la red: quien envía ve "Entregado".
  - [ ] Rechazar un envío no deja ningún archivo en el equipo que lo rechaza.
  - [ ] Con **Aceptar siempre**, el envío llega sin preguntar y avisa al terminar.
  - [ ] Capturar el tráfico de red durante un envío no permite ver el contenido de la captura.
  - [ ] Un video de 500 MB llega completo, con progreso en los dos lados, y se puede cancelar a la mitad sin dejar archivos a medias.
- **Mejora sobre Snagit:** función propia.
- **Fuentes:** IDEA-005 (spec/IDEAS.md), decisión de Carlos del 2026-11-14
