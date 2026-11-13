# Paridad — lista de verificación

Estados: `—` pendiente · `En curso` · `Win` / `Mac` verificado en esa plataforma · `Hecho` verificado en ambas.

Si un ID reparte su comportamiento entre hitos (el spec lo dice con "En M1: … En M2: …"), `Win`, `Mac` y `Hecho` se refieren a la parte de su hito; lo demás se sigue en la prueba manual del ID.

## 01-captura.md

| ID | Función | Hito | Prioridad | Estado |
|---|---|---|---|---|
| CAP-001 | Panel de captura | M1 | Must | En curso |
| CAP-002 | Pestaña Imagen y lista Selección | M1 | Must | En curso |
| CAP-003 | Restablecer ajustes de captura | M1 | Should | En curso |
| CAP-004 | Lanzador de borde | M3 | Should | En curso (Windows: Preferencias > Captura > **Interfaz de captura** (panel, lanzador o ambos) y **Mostrar el lanzador de borde** en el menú Más del panel; pestaña en el borde superior, izquierdo o derecho que al pasar el puntero muestra **Capturar** (siempre Universal al editor con el cursor, sin retardo, efectos ni destinos) y **Presets** (la lista; un clic lo ejecuta); se arrastra por su asa y se pega al borde más cercano del monitor bajo el puntero, y lo recuerda; si su monitor no está, vuelve al borde superior del principal; globo con el nombre de un preset nuevo; se oculta mientras una app llena su monitor; nunca sale en las capturas. Soltar archivos sobre él los importa (CAP-040, CAP-041). Falta: macOS) |
| CAP-005 | Menú de captura del ícono de bandeja o barra de menús | M1 | Must | En curso |
| CAP-006 | Atajo global Capturar | M1 | Must | En curso |
| CAP-007 | Repetir última captura | M1 | Should | En curso |
| CAP-008 | Comandos de captura (CLI y MCP) | M1 | Should | En curso |
| CAP-009 | Flujo de captura universal | M1 | Must | En curso |
| CAP-010 | Opciones de la pestaña Universal | M1 | Must | En curso |
| CAP-011 | Ajustar la selección y barra de captura | M1 | Must | En curso |
| CAP-012 | Rehacer la selección y cambiar de tipo | M1 | Should | En curso |
| CAP-013 | Pantalla congelada y cruceta | M1 | Must | En curso |
| CAP-014 | Cruceta simplificada | M1 | Could | En curso |
| CAP-015 | Lupa | M1 | Must | En curso |
| CAP-016 | Detección automática de ventanas y subáreas | M1 | Must | En curso |
| CAP-017 | Teclado durante la selección | M1 | Must | En curso |
| CAP-018 | Bloqueo de proporción | M1 | Should | En curso |
| CAP-019 | Guías de tamaño estándar | M1 | Could | En curso |
| CAP-020 | Ayuda en pantalla durante la captura | M1 | Could | En curso |
| CAP-021 | Varios monitores y DPI mixto | M1 | Must | En curso |
| CAP-022 | Pantallas HDR | M3 | Should | — |
| CAP-023 | Ocultar XcreenShot al capturar | M1 | Must | En curso |
| CAP-024 | Permisos que faltan al capturar | M1 | Must | En curso |
| CAP-025 | Región | M1 | Must | En curso |
| CAP-026 | Región fija y posición | M3 | Should | En curso (Windows: Imagen > Selección = Región > Región fija con Ancho y Alto (un rectángulo de ese tamaño sigue al puntero; clic o `Enter` captura, flechas lo mueven) y Posición fija con Monitor, X e Y (captura al momento, sin selección), **Elegir en pantalla**, límite al tamaño del monitor con aviso; `capture.region` con `rect.monitor` o `size`, y `capture.pick_area`. Si el monitor no está o la región no cabe, no se captura y avisa; faltan los botones Elegir de nuevo y Usar el monitor principal en el aviso) |
| CAP-027 | Ventana | M1 | Must | En curso |
| CAP-028 | Pantalla completa | M1 | Must | En curso |
| CAP-029 | Ventanas como objetos | M3 | Could | Hecho en Windows (Selección = Pantalla completa > **Cada ventana como un objeto** y `capture.fullscreen --windows`: cada ventana visible (no minimizada ni de otro escritorio) entra entera como objeto de imagen con el nombre de su app, en su sitio y orden; la base es el escritorio sin las ventanas: la pantalla tal como se veía con el fondo de pantalla (relleno, ajuste, estirar, centrar, mosaico o extender, y el color de fondo) donde estaban las ventanas, así que quedan la barra de tareas y los íconos que se veían; si el fondo no se puede leer, la pantalla tal como se veía; portapapeles y archivos reciben la imagen plana. Falta macOS) |
| CAP-030 | Selector de ventanas | M3 | Should | Hecho en Windows (Imagen > Selección = Selector de ventanas y su atajo `Ctrl+Alt+Shift+M`; ventana sobre el monitor del puntero, fuera de las capturas, con todas las ventanas del escritorio actual (también tapadas y minimizadas, no las de otros escritorios ni las ventanas auxiliares), app y título, miniaturas que llegan mientras se abre; escribir filtra, flechas, `Enter`, clic, `Esc`; captura a tamaño real; una minimizada se restaura un momento y vuelve a minimizarse; el cursor solo si estaba sobre esa ventana; probado con una ventana minimizada (se captura entera y vuelve a minimizarse). Falta macOS) |
| CAP-031 | Menú | M3 | Should | En curso (Windows: Imagen > Selección = Menú con Incluir submenús abiertos e Incluir la barra de menús; desde el panel, cuenta atrás (el Retardo, o 5 s) para abrir el menú; el comando `capture.menu` (atajo asignable) captura el menú ya abierto. Los menús del sistema se resaltan; clic o `Enter` los captura con su forma (transparente entre niveles y en las esquinas redondeadas); arrastrar hace una región. Los menús que dibuja la propia app (Edge, Office, apps nuevas) no se detectan: avisa y sigue como región. Falta el atajo por defecto, ver QUESTIONS) |
| CAP-032 | Mano alzada | M3 | Could | En curso (Windows: Selección = Mano alzada con Forma Libre (al soltar se cierra), Polígono (clics; `Enter` o doble clic cierra, `Retroceso` quita la última esquina) o Elipse (`Shift`: círculo); puntero de tijeras; la imagen es el rectángulo que rodea la forma, transparente fuera; `capture.region --freehand`. Falta: comprobar el pegado con forma en otras apps y macOS) |
| CAP-033 | Múltiples áreas | M3 | Should | En curso (Windows: Imagen > Selección = Múltiples áreas, y `Ctrl` + arrastrar en Universal y Región (no con el perfil de compatibilidad, que usa `Ctrl` para la proporción); cada arrastre o clic en una ventana resaltada añade un área numerada con su **×**; `Retroceso` quita la última; **Terminar** arriba, `Enter`, o soltar `Ctrl` con dos o más; Composición Posición original, Apiladas o En fila con separación; en el editor, cada área es un objeto de imagen sobre un lienzo transparente; al portapapeles y a los archivos va aplanada. Con el perfil de compatibilidad, el clic derecho también termina) |
| CAP-034 | Desplazamiento manual | M3 | Must | En curso (Windows: botón **Desplazamiento** (`S`) de la barra de captura y Selección = Desplazamiento con Tipo = Manual; panel junto al área, fuera de la captura, con la vista previa y el tamaño en vivo, **Listo** (`Enter`) y **Cancelar** (`Esc`) que funcionan sin quitarle el foco a la app; Preferencias > Captura > Pedir Iniciar antes de desplazar; área mínima de 50 × 50 px. y **Terminar desplazamiento** en el menú del ícono mientras dura) |
| CAP-035 | Desplazamiento automático | M3 | Must | En curso (Windows: flechas ↓, → y ↘ y teclas `D`, `R` y `X` en la barra de captura, y Selección = Desplazamiento con Tipo = Automático, Dirección, Empezar desde, Espera entre pasos y Volver a la posición inicial; XcreenShot gira la rueda sobre el área hasta que el contenido deja de moverse; **Detener aquí** (`Enter`), **Cancelar** (`Esc`) y el clic o la rueda del usuario dentro del área lo detienen; si nada se movió, avisa y deja el área visible. Las flechas aparecen sobre la selección, no al pasar por cualquier área desplazable: no se comprueba antes si se deja desplazar desde fuera) |
| CAP-036 | Unión y procesamiento del desplazamiento | M3 | Must | En curso (Windows: unión por la parte común de tramos seguidos, cabeceras y pies fijos una sola vez, barras laterales a lo largo del resultado, botones flotantes en su posición de la última vista; aviso de uniones dudosas; tope de 60 000 px o 250 megapíxeles con aviso; metadatos con `mode` = `scrolling`. La unión se hace mientras se captura, así que al pulsar Listo el resultado sale al momento: no hace falta barra de progreso de procesamiento. La casilla Detectar elementos fijos está en las propiedades de Desplazamiento (activada por defecto; desactivada, cada tramo entra entero)) |
| CAP-037 | Página web por URL | M3 | Should | En curso (Windows: Selección = Página web y Capturar abre una caja con la dirección (rellena desde el portapapeles o con la última), `Enter` captura y `Esc` cancela, con "Cargando…" y Cancelar; `capture.webpage`; la página se carga en una vista privada de WebView2 fuera de pantalla y se toma entera por el protocolo DevTools; propiedades Ancho de la vista (320–3840, con 375/768/1280/1440/1920), Escala (la del monitor, 1× o 2×), Esperar tras cargar y Cargar el contenido diferido; solo http y https (añade `https://`); 30 s como mucho; sin conexión no crea nada y lo dice; una página de error se captura y se avisa; corte a 60 000 px con aviso; dirección final y título en los metadatos; con Intervalo o Programada, la dirección dada al empezar o la última capturada. Soltar un acceso directo `.url` sobre el panel o el lanzador captura su página. Falta: soltar el texto de una dirección arrastrada desde el navegador, y macOS) |
| CAP-038 | Cámara web como imagen | M3 | Could | — |
| CAP-039 | Portapapeles como fuente | M3 | Should | Hecho en Windows (Imagen > Selección = Portapapeles y `capture.clipboard` (atajo asignable): una imagen (con su transparencia), archivos de imagen copiados en el Explorador (una captura cada uno) o texto (un objeto Texto editable con todo el texto, ajustado a 1000 px, márgenes de 20 px, lienzo blanco; cortado a 60 000 px con aviso); vacío u otra cosa avisa y no crea nada; origen "clipboard" en los metadatos. Con Intervalo, cada vuelta lee el portapapeles y descarta lo que no cambió. Metadatos con la app que copió el contenido cuando Windows la conoce (mientras esa app sigue abierta). Falta macOS, cuyo portapapeles no lo indica) |
| CAP-040 | Archivo como fuente | M3 | Could | Hecho en Windows (Selección = Archivo con el selector del sistema (varios archivos) o un **Archivo fijo**; una captura por archivo en orden, con efectos y destinos; PNG, JPG, BMP, TIFF e ICO con los decodificadores de la app y GIF, WebP y HEIC (si Windows lo tiene) con los del sistema; un PDF abre Importar PDF y un `.xshot` el editor; los dañados se omiten y se avisa; el original no cambia; ruta y fecha del original en los metadatos; con Intervalo o Programada, el mismo archivo cada vez. Soltar archivos sobre el panel o el lanzador hace lo mismo. La fecha de toma de las fotos (EXIF) va en los metadatos. Los SVG se dibujan con el motor web del sistema (privado y fuera de pantalla, como Página web) al tamaño que dicen (width/height o viewBox), al doble si son pequeños, con su transparencia y sus textos. Falta macOS) |
| CAP-041 | Importar PDF multipágina | M3 | Should | En curso (Windows, con el motor PDF del sistema: Selección = PDF o `capture.pdf`; ventana Importar PDF con miniaturas marcadas, rango `1-3, 7`, Resolución 72–300 ppp, Una captura por página o Todas en una imagen (20 px sobre gris claro, repartida si pasa de 60 000 px), fondo blanco o transparente; contraseña con reintento; progreso y Cancelar; 200 páginas como mucho; con Abrir en = Editor se abre la primera; archivo y página en los metadatos. Soltar un PDF sobre el panel o el lanzador abre Importar PDF. Falta: el texto de la página en los metadatos, Abrir con desde el sistema y macOS) |
| CAP-042 | Capturas del sistema | M3 | Should | En curso (Windows: Preferencias > Captura > **Abrir en XcreenShot las capturas del sistema** (desactivada por defecto), **Al recibirla** (editor o solo biblioteca) y **Conservar el archivo original**; vigila el portapapeles solo cuando la imagen viene de la Herramienta Recortes y la carpeta Capturas de pantalla del sistema; una misma captura en los dos sitios entra una vez; solo las hechas con la opción activada; en cola mientras XcreenShot captura; origen "system" y ruta y fecha del archivo en los metadatos. Falta: las grabaciones de video del sistema (llegan con la biblioteca de video, M4), probar `Win+Shift+S` de verdad y macOS) |
| CAP-043 | AirDrop y carpeta de recepción | M3 | Could | En curso (Windows: Preferencias > Captura > **Carpeta de recepción** (Elegir…/Ninguna), **Abrir en el editor** y **Dejar los archivos donde están**; cada imagen o PDF nuevo se importa una vez cuando deja de crecer 2 s (PDF: todas las páginas a 150 ppp; con contraseña, aviso) y pasa a `Importados`; lo que ya estaba al elegir la carpeta no se importa. Falta: los videos (M4), AirDrop y macOS) |
| CAP-044 | iPhone o iPad | M3 | Could | — |
| CAP-045 | Abrir en | M1 | Must | En curso |
| CAP-046 | Copiar al portapapeles | M1 | Must | En curso |
| CAP-047 | Capturar cursor | M1 | Must | En curso |
| CAP-048 | Retardo | M1 | Must | En curso |
| CAP-049 | Intervalo | M3 | Should | En curso (Windows: Imagen > Tiempo = Intervalo con Cada (3 s a 24 h; menos de 3 se sube a 3 con aviso), Detener (manualmente, tras N capturas, tras un tiempo, a una hora), Descartar si no hay cambios y Sensibilidad (exacta o ignorar < 0,5 %); Iniciar/Detener en el panel con cuántas lleva y la hora de la siguiente; reloj en el ícono y **Detener captura: <nombre>** en su menú; región y ventana se eligen una vez; Pantalla completa, Menú y Portapapeles sin selección; no abre el editor; serie e índice en los metadatos; vueltas omitidas si hay otra captura o la sesión está bloqueada; se detiene con aviso si la ventana se cierra o el monitor desaparece; varias series a la vez; un preset con intervalo es un interruptor; salir avisa. También con Página web (la dirección dada o la última) y Archivo (el fijo o uno elegido al empezar). Falta agrupar la serie en la biblioteca (`LIB-`)) |
| CAP-050 | Programada | M3 | Should | En curso (Windows: Imagen > Tiempo = Programada con Fecha (vacía = hoy), Hora y Repetir (una vez, cada día, días laborables, cada semana); **Programar** / **Cancelar programación** en el panel con la próxima hora; se hace en el segundo 0; una hora pasada se rechaza; se conserva al reiniciar; si la app vuelve en menos de 5 min la hace tarde y anota el retraso en los metadatos, más tarde la omite con aviso; sesión bloqueada o ventana inexistente: se omite con aviso; reloj en el ícono y **Detener captura: <nombre> (próxima: …)** en su menú; un preset programado es un interruptor. Falta probar con el equipo dormido de verdad y en macOS) |
| CAP-051 | Efectos al capturar | M3 | Should | Hecho en Windows (Imagen > Efectos: Añadir efecto (borde, sombra, bordes decorativos, esquinas redondeadas, perspectiva, página doblada, reflejo, desvanecer, escalar, girar, filtros, ajuste de color, recortar bordes, marca de agua, detalles de captura) o un Efecto guardado, subir, quitar y Ninguno; llegan al documento como efectos editables y al portapapeles y los archivos ya aplicados; `effects` en los comandos; el engranaje de cada efecto abre sus opciones bajo la fila (las mismas que en la pestaña Efectos del editor); arrastrar para reordenar; Detalles de captura con **Pedir texto al capturar**: al terminar la captura aparece un cuadro, Enter pone el texto y Esc lo deja vacío sin cancelar (las series y las programadas no preguntan); `capture.details_text`. Falta verlo en macOS) |
| CAP-052 | Compartir al capturar | M3 | Should | En curso (Windows: Compartir en las pestañas Universal e Imagen con el único destino que existe, **Archivo**; `share` en los comandos de captura. Con Abrir en = Editor no se envía al momento: el editor muestra "Pendiente de enviar a: Archivo" con **Enviar** (`Ctrl+Enter`) y **No enviar**; Enviar escribe la imagen como esté entonces, y cerrar sin enviar no envía nada (`capture.send_pending`, `capture.pending_shares`). Los demás destinos llegan con SHR- en M6) |
| CAP-053 | Nombre, formato y carpeta del destino Archivo | M3 | Should | En curso (Windows: carpeta (por defecto Imágenes\XcreenShot), prefijo, patrón con los marcadores del spec, **Insertar** y vista previa, contador por carpeta, patrón y formato con **Siguiente número**, `/` para subcarpetas, limpieza del nombre igual en las dos plataformas, PNG, JPG con calidad o `.xshot`, Añadir un número o Sobrescribir; si la carpeta no se puede usar, la captura queda en la biblioteca y avisa; evento `file.written`. También **Preguntar nombre y carpeta en cada captura**, **Si ya existe → Preguntar** (el diálogo de guardar del sistema; cancelar deja la captura solo en la biblioteca) e **Incluir los metadatos de captura en el archivo** (en PNG: título, app, fecha, dirección). Con "Incluir los metadatos", el JPG lleva un bloque EXIF (título, app, dirección, fecha de captura con su zona horaria y "XcreenShot") que leen el Explorador y las apps de fotos. Falta el formato por defecto al compartir (`SHR-`)) |
| CAP-054 | Ruta de la captura | M1 | Must | En curso |
| CAP-055 | Confirmación de captura | M1 | Should | En curso |
| CAP-056 | Metadatos de captura | M1 | Must | En curso |
| CAP-057 | Anotación rápida de imagen | M3 | Should | En curso (Windows, función en prueba `experimental.quickAnnotate`: atajo `Ctrl+Alt+Shift+Q`, entrada del menú del ícono y Abrir en = Anotación rápida solo con ella activada; selección como en Universal; barra con Flecha, Forma (rectángulo o elipse), Texto, Resaltar, Desenfocar y Paso (`1`–`6`), seis colores, tres grosores, Deshacer/Rehacer (`Ctrl+Z`, `Ctrl+Shift+Z`), Copiar (`Enter`, `Ctrl+C`), Abrir en el editor (`Ctrl+E`) y Cancelar; las marcas son objetos del editor; `Esc` con marcas pide confirmación; preferencias Herramienta inicial y Guardar en la biblioteca al copiar. Una marca se elige con un clic, se mueve arrastrándola y `Supr` la borra (sin elegida, quita la última). Los colores son los del tema activo (EDT-017), empezando por el primero que no es casi negro ni casi blanco. Falta: probar en macOS) |
| CAP-058 | Lista de presets | M3 | Must | En curso (Windows: sección Presets del panel con ícono del tipo, nombre, resumen, atajo, aviso de ajustes que no existen y Capturar al pasar el puntero; doble clic, `Enter`, `F2`, `Supr`, flechas; Filtrar desde 9; submenú Presets del menú del ícono en el mismo orden; `preset.run` y `preset.list`. La lista está también en el Lanzador de borde (CAP-004) y un preset con su intervalo o su programada activos dice "en marcha". Se eligen varios con `Ctrl`/`Mayús` + clic para exportarlos o eliminarlos juntos. Faltan los destinos (`SHR-`)) |
| CAP-059 | Crear un preset | M3 | Must | En curso (Windows: ＋ > Desde los ajustes actuales (queda el nombre propuesto para editar en la fila), Nuevo preset de imagen o universal con los valores por defecto; nombres de 1 a 64 caracteres, únicos con " (2)"; `preset.create`. Viajan en Exportar ajustes (APP-011, grupo Presets de captura, con sus atajos). Los ajustes de los modos nuevos (desplazamiento, archivo, PDF, página web, mano alzada, intervalo, programada) se guardan en el preset como los demás) |
| CAP-060 | Editar, renombrar, duplicar y eliminar | M3 | Must | En curso (Windows: Editar preset con Guardar y Cancelar, Actualizar con los ajustes actuales (con confirmación, solo del mismo tipo), Cambiar nombre en la fila conservando el atajo, Duplicar "(copia)" sin atajo, Eliminar con confirmación desactivable y Deshacer durante 10 s; `preset.update`, `rename`, `duplicate`, `delete`, `restore`. Con varios elegidos (`Ctrl`/`Mayús` + clic), Exportar y Eliminar actúan sobre todos) |
| CAP-061 | Reordenar | M3 | Should | En curso (Windows: arrastrar por el asa con la línea de destino, `Alt`+`↑`/`↓`, sin arrastre mientras se filtra; el orden es el mismo en el menú del ícono y se guarda al momento; `preset.move`) |
| CAP-062 | Atajo por preset | M3 | Must | En curso (Windows: atajo global por preset desde su menú ⋯ o desde Preferencias > Atajos (grupo Presets, con su nombre), con Reasignar para uno en uso; se conserva al cambiar el nombre y al cambiar de perfil, la copia no lo hereda, se libera al eliminar y vuelve con Deshacer. Un preset con Intervalo o Programada es un interruptor: su atajo inicia y detiene la serie, o programa y cancela) |
| CAP-063 | Exportar e importar `.xpreset` | M3 | Should | En curso (Windows: Exportar de un preset o de todos, Importar presets… con el resumen, una casilla por preset y Conservar ambos, Reemplazar u Omitir; atajos convertidos de la otra plataforma y sin atajo si está en uso; un archivo dañado o más nuevo no importa nada; `preset.export`, `preset.import`. Soltar el archivo sobre el panel lo importa. Doble clic en un `.xpreset` abre el panel con el resumen (APP-001). Un preset cuya carpeta del destino Archivo o cuyo archivo fijo no existen en este equipo muestra el aviso "… no existe en este equipo. Revísalo." en su fila. Falta `policy.presets` (configuración administrada, APP-035)) |
| CAP-064 | Presets incorporados | M3 | Should | En curso (Windows: Región al portapapeles y Extraer texto, sin atajo y con el nombre en el idioma de la app hasta que se renombra; Restaurar presets incorporados pregunta por los cambiados; uno nuevo de una versión futura se añade al final. desde M3.3 también **Desplazamiento** (automático, vertical, desde el principio; Abrir en = Editor). El Grabador de pasos llega con su función) |
| CAP-065 | Acciones de captura y atajos por defecto | M1 | Must | En curso |
| CAP-066 | Perfil de compatibilidad | M1 | Should | En curso |
| CAP-067 | Multicaptura (función propia, adelantada) | M1 | Should | En curso |

## 02-editor.md

| ID | Función | Hito | Prioridad | Estado |
|---|---|---|---|---|
| EDT-001 | Ventana principal | M1 | Must | En curso |
| EDT-002 | Menús del editor | M1 | Must | Win |
| EDT-003 | Barra de herramientas y menú Más | M1 | Must | En curso (Windows: barra por defecto, Más agrupado y hueco temporal; falta la búsqueda en Más; en una ventana estrecha la barra se compacta por pasos sin montar nada, y lo que no cabe pasa a Más y a ⋯) |
| EDT-004 | Personalizar la barra de herramientas | M2 | Should | En curso (Windows hecho; faltan los destinos de compartir, con `SHR-`, y que viaje en la exportación de ajustes, APP-011) |
| EDT-005 | Panel lateral | M1 | Must | En curso |
| EDT-006 | Barra de edición rápida | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-007 | Zoom y vista | M1 | Must | Win |
| EDT-008 | Cuadrícula de píxeles | M2 | Could | Hecho en Windows (macOS pendiente) |
| EDT-009 | Pantalla completa | M2 | Could | Hecho en Windows (macOS pendiente) |
| EDT-010 | Área de trabajo | M2 | Could | Hecho en Windows (macOS pendiente) |
| EDT-011 | Barra de estado | M1 | Should | Win |
| EDT-012 | Cambios sin guardar y copia de recuperación | M1 | Must | Win |
| EDT-013 | Accesibilidad del lienzo | M1 | Should | Win |
| EDT-014 | Propiedades y estilo actual | M1 | Must | En curso |
| EDT-015 | Estilos guardados | M2 | Must | Hecho en Windows (macOS pendiente) |
| EDT-016 | Favoritos | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-017 | Temas | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-018 | Exportar e importar temas `.xtheme` | M2 | Should | En curso (faltan el doble clic en el archivo y `policy.themes`) |
| EDT-019 | Selector de color | M1 | Must | Win |
| EDT-020 | Degradados | M2 | Should | En curso (falta el borde de la cámara, con `VID-`) |
| EDT-021 | Sombra de los objetos | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-022 | Selección, tiradores y modificadores | M1 | Must | Win |
| EDT-023 | Seleccionar todos los objetos de una herramienta | M2 | Should | En curso (Windows: botón bajo el título de Propiedades de cada herramienta de anotación, desactivado con su motivo si no hay, y clic derecho > Seleccionar todos los de este tipo; trae el grupo entero y deja fuera los bloqueados) |
| EDT-024 | Flecha | M1 | Must | Win |
| EDT-025 | Línea | M2 | Should | En curso |
| EDT-026 | Texto | M1 | Must | Win |
| EDT-027 | Corrector ortográfico | M2 | Should | En curso (Windows: subrayado del corrector del sistema al escribir textos y globos, sugerencias y Añadir al diccionario en el clic derecho, preferencia `editor.spellcheck`; falta elegir el idioma de corrección y el diccionario propio con "XcreenShot", ver QUESTIONS) |
| EDT-028 | Globo | M2 | Must | En curso |
| EDT-029 | Forma: rectángulo y elipse | M1 | Must | Win |
| EDT-030 | Forma: rectángulo redondeado, polígono y otras formas | M2 | Should | En curso |
| EDT-031 | Sello | M2 | Should | En curso |
| EDT-032 | Sellos propios y Guardar como sello | M2 | Should | — |
| EDT-033 | Emoji | M2 | Could | — |
| EDT-034 | Paquetes de sellos `.xstamps` | M2 | Should | — |
| EDT-035 | Paso | M2 | Must | En curso |
| EDT-036 | Resaltador | M2 | Should | En curso |
| EDT-037 | Lápiz | M2 | Should | En curso |
| EDT-038 | Lupa | M2 | Should | En curso |
| EDT-039 | Foco | M2 | Should | En curso |
| EDT-040 | Desenfocar | M1 | Must | Win |
| EDT-041 | Simplificar | M2 | Should | En curso |
| EDT-042 | Mover | M1 | Must | En curso |
| EDT-043 | Selección | M2 | Must | En curso (falta mantenerla al cambiar el tamaño de la imagen, con EDT-069) |
| EDT-044 | Acciones sobre la selección | M2 | Must | En curso (faltan efectos, guardar, imprimir, sello y texto: llegan con sus IDs) |
| EDT-045 | Relleno del hueco | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-046 | Varita mágica | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-047 | Relleno | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-048 | Borrador | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-049 | Recortar | M1 | Must | Win |
| EDT-050 | Recorte no destructivo | M1 | Must | Win |
| EDT-051 | Recortar al lienzo y recortar bordes | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-052 | Quitar franja y añadir espacio | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-053 | Panel Efectos | M2 | Must | Hecho en Windows (macOS pendiente) |
| EDT-054 | Efectos guardados | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-055 | Borde | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-056 | Bordes decorativos | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-057 | Sombra de la imagen | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-058 | Perspectiva | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-059 | Página doblada | M2 | Could | Hecho en Windows (macOS pendiente) |
| EDT-060 | Reflejo | M2 | Could | Hecho en Windows (macOS pendiente) |
| EDT-061 | Desvanecer bordes | M2 | Could | Hecho en Windows (macOS pendiente) |
| EDT-062 | Filtros | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-063 | Ajuste de color | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-064 | Reemplazo de color | M2 | Could | Hecho en Windows (macOS pendiente) |
| EDT-065 | Efecto Foco y lupa | M2 | Could | Hecho en Windows (macOS pendiente) |
| EDT-066 | Marca de agua | M2 | Could | Hecho en Windows (macOS pendiente) |
| EDT-067 | Detalles de captura | M3 | Should | Hecho en Windows (macOS pendiente) |
| EDT-068 | Efectos de transformación | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-069 | Cambiar tamaño de la imagen | M2 | Must | Hecho en Windows (macOS pendiente) |
| EDT-070 | Tamaño del lienzo | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-071 | Ampliar el lienzo automáticamente | M1 | Should | Win |
| EDT-072 | Girar | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-073 | Voltear | M2 | Could | Hecho en Windows (macOS pendiente) |
| EDT-074 | Color del lienzo | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-075 | Esquinas redondeadas | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-076 | Volver al original | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-077 | Resolución y densidad de píxeles | M2 | Could | En curso (Windows hecho; falta comprobar los 192 PPP de una pantalla Retina en Mac) |
| EDT-078 | Imágenes flotantes (lo básico, adelantado a M1 por CAP-067) | M2 | Must | En curso |
| EDT-079 | Selección múltiple | M2 | Must | En curso |
| EDT-080 | Transformar varios objetos | M2 | Should | En curso |
| EDT-081 | Orden de apilado | M2 | Should | En curso |
| EDT-082 | Agrupar y desagrupar | M2 | Should | En curso |
| EDT-083 | Alinear y distribuir | M2 | Should | En curso |
| EDT-084 | Ajuste magnético y guías | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-085 | Bloquear objetos | M2 | Should | En curso |
| EDT-086 | Posición y tamaño numéricos | M2 | Must | En curso |
| EDT-087 | Mover con el teclado | M2 | Should | En curso |
| EDT-088 | Duplicar objetos | M2 | Should | En curso |
| EDT-089 | Cortar, copiar y pegar | M1 | Must | Win |
| EDT-090 | Copiar todo | M1 | Must | Win |
| EDT-091 | Aplanar y desaplanar | M2 | Should | En curso |
| EDT-092 | Deshacer y rehacer | M1 | Must | Win |
| EDT-093 | Cursor como objeto | M2 | Should | En curso (Windows: el puntero capturado es un Cursor que se mueve, redimensiona con proporción, borra o aplana; Tamaño 50–400 %, Forma (la capturada, flecha, mano, texto, cruz) y Resalte (ninguno, amarillo, acento); en el lienzo y en la imagen final igual) |
| EDT-094 | Medición en vivo | M5 | Could | — |
| EDT-095 | Nuevo desde plantilla | M2 | Should | En curso (Windows: Archivo > Nuevo desde plantilla, Aplicar plantilla, Combinar desde la Biblioteca, filtro "Para N imágenes", máximo 20 con aviso; falta la selección múltiple en la Bandeja reciente) |
| EDT-096 | Plantillas incorporadas | M2 | Should | Hecho en Windows (11 plantillas en 4 familias, con el tema activo) |
| EDT-097 | Ajustes de la plantilla | M2 | Should | Hecho en Windows |
| EDT-098 | Zonas de imagen | M2 | Should | En curso (Windows: llenar arrastrando un archivo, pegando, eligiendo o con Capturar; encuadrar, intercambiar, quitar; zonas vacías fuera de la imagen final. Falta arrastrar desde la Bandeja reciente y desde una imagen flotante con el ratón: hoy, por comando `fromObject`) |
| EDT-099 | Secciones | M2 | Should | En curso (Windows: añadir, quitar, subir y bajar desde la pestaña Plantilla y el divisor en el lienzo; faltan el + y la papelera sobre el propio lienzo y arrastrar el asa para reordenar) |
| EDT-100 | Aplicar un tema a objetos existentes | M2 | Could | En curso (núcleo y CLI hechos; falta la entrada Aplicar tema… del menú ⋯ de la pestaña Estilos) |
| EDT-101 | Plantillas propias `.xtemplate` | M2 | Should | Hecho en Windows (falta comprobar que una plantilla exportada en macOS se importa igual en Windows) |
| EDT-102 | Paginación al imprimir y en PDF | M2 | Should | En curso (PDF e impresión hechos; faltan las plantillas) |
| EDT-103 | Nueva imagen | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-104 | Nueva desde el portapapeles | M2 | Should | Hecho en Windows (macOS pendiente) |
| EDT-105 | Abrir, arrastrar y soltar archivos | M1 | Must | En curso |
| EDT-106 | Formato `.xshot` | M1 | Must | Win |
| EDT-107 | Guardar, Guardar como y Guardar todo | M1 | Must | Win |
| EDT-108 | Cerrar y Cerrar todo | M1 | Must | En curso |
| EDT-109 | Renombrar, duplicar, eliminar y mostrar | M2 | Should | En curso |
| EDT-110 | Exportar a PNG y JPG | M1 | Must | En curso |
| EDT-111 | Más formatos de exportación | M2 | Should | En curso (Windows: PNG, JPG, GIF, BMP, TIFF, PDF, ICO y PSD; faltan WebP y HEIC, y varias miniaturas de la bandeja o la biblioteca en un archivo) |
| EDT-112 | Opciones de cada formato | M2 | Should | En curso (faltan JPG progresivo, tramado en GIF y TIFF con compresión JPEG) |
| EDT-113 | Exportación por lotes | M2 | Should | En curso (Windows hecho, también desde el menú de la biblioteca, LIB-018; falta abrirla desde la bandeja y la pila de efectos propia) |
| EDT-114 | Imprimir y configurar página | M2 | Should | En curso (Windows: vista previa propia e impresión desde un PDF exacto en el visor del sistema; falta el diálogo de impresión directo, ver QUESTIONS) |
| EDT-115 | Atajos de herramientas | M1 | Must | Win |
| EDT-116 | Atajos de edición, vista y archivo | M1 | Must | Win |
| EDT-117 | Teclas del lienzo y modificadores | M1 | Must | Win |
| EDT-118 | Preferencias del editor | M1 | Should | Win |
| EDT-119 | Comandos del editor (CLI y MCP) | M1 | Must | En curso |
| EDT-120 | Reabrir los documentos al iniciar (función propia) | M1 | Should | En curso |

## 03-video.md

| ID | Función | Hito | Prioridad | Estado |
|---|---|---|---|---|
| VID-001 | Pestaña Video del panel de captura | M4 | Must | En curso (Windows: tercera pestaña del panel con Selección (Región o Pantalla completa), Capturar cursor y Grabar el audio del sistema (los mismos ajustes que la barra y Preferencias > Video) y Temporizador (sin límite o 1 a 60 min); **Capturar** con ella abre la selección o deja la barra en *Lista*. Faltan Ventana y Solo cámara, Propiedades, Compartir, Abrir en, cámara, micrófono, efectos del cursor, Dibujo en vivo y los presets de video) |
| VID-002 | Iniciar una captura de video | M4 | Must | En curso (Windows: menú del ícono > Captura de video y su atajo `Ctrl+Alt+Mayús+V` (`record.video`): elegir el área en pantalla y queda *Lista* con la barra; durante una grabación, la detiene; `record.start`. Faltan la pestaña Video, el botón `V` de la captura universal, presets de video, Repetir última captura y que la pantalla no se congele al elegir) |
| VID-003 | Área de grabación | M4 | Must | En curso (Windows: el área se elige como una región de imagen (tiradores, teclado, guías), mínimo 64 × 64, tamaño par, dentro de un monitor; la barra muestra el tamaño final del video. Faltan Ancho y Alto editables y mover el área en *Lista*, y cruzar monitores) |
| VID-004 | Pantalla completa de un monitor | M4 | Must | En curso (Windows: Selección = Pantalla completa graba el monitor bajo el puntero, sin selección; `record.start --source screen` con `monitor` primary, pointer o su número. Falta elegir el monitor en Propiedades) |
| VID-005 | Grabar una ventana | M4 | Should | En curso (Windows: Selección = Ventana en la pestaña Video (se elige en pantalla) y `window` en `record.start`, en modo **Área de la ventana**: el área sigue a la ventana cuando se mueve o cambia de tamaño (con bandas negras si cambia la proporción), conserva el último fotograma mientras está minimizada y, si se cierra, la grabación se detiene y se guarda con un aviso (comprobado moviendo, redimensionando y cerrando una ventana de prueba). Faltan **Solo la ventana** (necesita la captura por ventana de Windows), seguirla a otro monitor y que la barra la acompañe) |
| VID-006 | Resumen Listo para grabar | M4 | Should | — |
| VID-007 | Empezar a grabar sin esperar | M4 | Could | — |
| VID-008 | Cuenta atrás | M4 | Should | En curso (Windows: preferencia **Cuenta atrás antes de grabar** (Ninguna, 3 s por defecto, 5 s o 10 s) al pulsar **Grabar** en la barra, con la cuenta atrás de las capturas con retardo; `Esc` vuelve a *Lista*; `record.status` dice `countdown`; desde automatización solo con `countdown`. Faltan los números en el centro del área, saltarla pulsando **Grabar** otra vez y preparar cámara y micrófono durante ella) |
| VID-009 | Presets de video | M4 | Should | — |
| VID-010 | Convertir una grabación en imagen | M4 | Could | — |
| VID-011 | Barra de grabación | M4 | Must | En curso (Windows: ventana flotante bajo el área (encima si no cabe), fuera del video, que se arrastra por su asa: tiempo `hh:mm:ss`, tamaño del video, **Grabar** en *Lista*, **Pausar**/**Reanudar** y **Detener** con sus atajos en el tooltip, **×** que pregunta antes de cancelar. Faltan cursor, audio, cámara, micrófono, Dibujo en vivo, temporizador y el menú Más) |
| VID-012 | Grabar, pausar y reanudar | M4 | Must | En curso (Windows: el motor y `record.pause` / `record.resume`; la pausa queda fuera del video. Falta la barra de grabación, VID-011) |
| VID-013 | Detener | M4 | Must | En curso (Windows: `record.stop` espera al archivo y devuelve ruta, duración, tamaño, fps y codificador; aviso "Video guardado". Falta el botón de la barra, Abrir en y la bandeja) |
| VID-014 | Reiniciar y cancelar | M4 | Should | En curso (Windows: `record.cancel` termina sin archivo. Falta Reiniciar y la barra) |
| VID-015 | Temporizador de parada automática | M4 | Should | En curso (Windows: `limit` en `record.start` y el Temporizador de la pestaña Video (1 a 60 min) detienen la grabación sola y guardan el archivo. Falta el control en la barra) |
| VID-016 | Posición de la barra y controles minimizados | M4 | Must | — |
| VID-017 | XcreenShot fuera del video | M4 | Must | En curso (Windows: la barra, su borde, la cuenta atrás y el aviso "Trabajando" nunca salen; con **Ocultar XcreenShot al capturar** (CAP-023) el panel, el editor y las demás ventanas de XcreenShot tampoco, aunque siguen en pantalla, desde el primer fotograma (comprobado grabando el área del panel: con la opción sale lo de detrás, sin ella el panel); una captura de imagen durante la grabación no los vuelve a mostrar. Faltan las ventanas que se abren durante la grabación y los marcos de videollamada) |
| VID-018 | Escritorio limpio | M4 | Could | — |
| VID-019 | Aviso de micrófono silenciado | M4 | Should | — |
| VID-020 | Indicadores de grabación | M4 | Must | En curso (Windows: borde alrededor del área, fuera del video (azul en *Lista*, rojo grabando, discontinuo en pausa); reloj en el ícono y **Detener grabación** en su menú; aviso del sistema si la empezó una automatización. Falta lo demás del ID) |
| VID-021 | Mantener el equipo despierto | M4 | Should | En curso (Windows: mientras se graba, Windows no se suspende ni apaga la pantalla por inactividad (lo que pide el usuario sigue funcionando); al terminar vuelve a lo de siempre. Faltan el estado *Lista*, la reproducción y pausar al suspender a mano) |
| VID-022 | Recuperar grabaciones interrumpidas | M4 | Must | — |
| VID-023 | Grabaciones largas y espacio en disco | M4 | Should | En curso (Windows: cada 2 s se mira el espacio libre del disco del video; con menos de 1 GB la grabación se detiene y se guarda sola, y el resultado y el aviso dicen el motivo. Faltan los minutos que caben en el resumen, la sugerencia de los 60 min y la prueba de 4 h) |
| VID-024 | Comandos de video (CLI y MCP) | M4 | Should | En curso (Windows: `record.start` (pantalla de un monitor o región), `record.pause`, `record.resume`, `record.stop`, `record.cancel` y `record.status`, con `busy` y `not_found` ("not_recording"); eventos `record.*` y `file.written`; desde automatización, aviso del sistema; "Detener grabación" y el reloj en el ícono. Faltan ventana y cámara como origen, `record.restart`, preset, cámara, micrófono, audio, cursor y `video.*`) |
| VID-025 | Calidad y fotogramas por segundo | M4 | Must | En curso (Windows: Preferencia `video.quality` de Muy baja (5 fps) a Muy alta (60 fps), Alta por defecto, con la tasa de bits proporcional; frecuencia variable: solo se escribe un fotograma cuando la pantalla cambia (2 s de pantalla quieta a 1280 × 720: unos 90 KB). Faltan las medidas de los criterios y la UI de Preferencias > Video) |
| VID-026 | Formato del archivo | M4 | Must | En curso (Windows: MP4 con H.264 (perfil High) de los codificadores del sistema (Media Foundation); BT.709 desde 720p y BT.601 por debajo, comprobado con colores conocidos (±2 niveles). Faltan el audio (AAC), el título y la fecha en el MP4 y la conversión HDR) |
| VID-027 | Códec H.265 | M4 | Could | — |
| VID-028 | Tamaño del video y límites | M4 | Must | En curso (Windows: preferencia `video.maxSize` (Original, Reducir pantallas de alta densidad por defecto, Reducir a 1080p), máximo 3840 × 2160 y tamaño siempre par; el recorte y el escalado los hace la GPU. Falta avisarlo en el resumen, VID-006) |
| VID-029 | Codificación por hardware o por software | M4 | Should | En curso (Windows: el codificador por hardware con los fotogramas en la GPU y, si no hay o falla al empezar, el de software; preferencia `video.softwareEncoding`. Faltan seguir en software si el de hardware falla a mitad, el método de captura alternativo y el aviso de grabación en negro) |
| VID-030 | Sincronía y rendimiento | M4 | Must | — |
| VID-031 | Activar la cámara y elegir dispositivo | M4 | Must | — |
| VID-032 | Imagen en imagen: posición y tamaño | M4 | Must | — |
| VID-033 | Forma y proporción de la cámara | M4 | Should | — |
| VID-034 | Cámara a pantalla completa | M4 | Should | — |
| VID-035 | Cambiar entre imagen en imagen y pantalla completa | M4 | Should | — |
| VID-036 | Espejo y borde de la cámara | M4 | Could | — |
| VID-037 | Fondo de la cámara | M4 | Should | — |
| VID-038 | Cámara ocupada, tapada o sin permiso | M4 | Should | — |
| VID-039 | Micrófono | M4 | Must | — |
| VID-040 | Audio del sistema | M4 | Must | En curso (Windows: preferencia **Grabar el audio del sistema** (desactivada), interruptor **Sonido** en la barra en *Lista* y `systemAudio` en `record.start`; WASAPI en bucle invertido de la salida por defecto, estéreo de 16 bits a 44,1 o 48 kHz, AAC de 128 kbps en el mismo MP4, con silencio donde no suena nada para no perder la sincronía; si se cambia de salida durante la grabación, sigue con la nueva; sin salida de sonido activa, el video se guarda sin pista y lo avisa. Comprobado con un tono sintético (pista AAC estéreo de 2 s); esta noche el equipo no tenía ninguna salida activa. Faltan el medidor de nivel, cambiarlo durante la grabación y no grabar los sonidos de XcreenShot) |
| VID-041 | Reducción de ruido | M4 | Should | — |
| VID-042 | Pistas de audio | M4 | Could | — |
| VID-043 | Conectar y desconectar dispositivos | M4 | Must | — |
| VID-044 | Capturar cursor | M4 | Must | En curso (Windows: el puntero se dibuja en cada fotograma con su forma real (la duplicación del escritorio no lo trae) y moverlo cuenta como cambio; preferencia `video.cursor` (activada), parámetro `cursor` de `record.start`, interruptor **Puntero** en la barra y `record.cursor`, también durante la grabación. Faltan los efectos (VID-045, VID-046) y el interruptor en la pestaña Video) |
| VID-045 | Resaltar cursor | M4 | Should | — |
| VID-046 | Animación de clic | M4 | Should | — |
| VID-047 | Mostrar teclas pulsadas | M4 | Should | — |
| VID-048 | Modo Dibujo en vivo | M4 | Should | — |
| VID-049 | Herramientas de Dibujo en vivo | M4 | Should | — |
| VID-050 | Color, grosor y desvanecimiento | M4 | Should | — |
| VID-051 | Disponibilidad de Dibujo en vivo | M4 | Could | — |
| VID-052 | Abrir en | M4 | Must | — |
| VID-053 | Compartir al terminar | M4 | Should | — |
| VID-054 | Guardado en la biblioteca y metadatos | M4 | Must | En curso (Windows: el MP4 va a la carpeta del mes de la biblioteca ("Video AAAA-MM-DD HH.MM.SS"), que lo lista como video con su tamaño y la miniatura de su primer fotograma; abrirlo usa el reproductor del sistema hasta que llegue el del editor (VID-056); doble clic en un `.mp4` también. Faltan los metadatos de captura del video y el filtro Tipo = Video en la interfaz) |
| VID-055 | Importar videos | M4 | Should | — |
| VID-056 | Reproductor de video | M4 | Must | — |
| VID-057 | Velocidad de reproducción y clic para reproducir | M4 | Could | — |
| VID-058 | Selección en la línea de tiempo | M4 | Must | — |
| VID-059 | Cortar | M4 | Must | — |
| VID-060 | Guardar la selección como nuevo video | M4 | Should | — |
| VID-061 | Guardar fotograma como imagen | M4 | Must | — |
| VID-062 | Extraer texto de un fotograma | M4 | Should | — |
| VID-063 | Copia sin sonido | M4 | Could | — |
| VID-064 | Información del video | M4 | Could | — |
| VID-065 | Sin anotaciones sobre el video | M4 | Must | — |
| VID-066 | Guardar, exportar, copiar y arrastrar videos | M4 | Must | — |
| VID-067 | Combinar videos | M4 | Should | — |
| VID-068 | Crear GIF animado | M4 | Must | — |
| VID-069 | Optimizar para | M4 | Must | — |
| VID-070 | Ajustes personalizados del GIF | M4 | Should | — |
| VID-071 | Crear diapositivas narradas | M4 | Should | — |
| VID-072 | Barra y navegación de las diapositivas | M4 | Should | — |
| VID-073 | Anotar las diapositivas en vivo | M4 | Could | — |
| VID-074 | Tamaño y fondo del video de diapositivas | M4 | Should | — |
| VID-075 | Atajos globales de video | M4 | Must | En curso (Windows: Captura de video `Ctrl+Alt+Mayús+V`, Grabar/Pausar/Reanudar `Ctrl+Mayús+F9` y Detener `Ctrl+Mayús+F10`; con Compatibilidad, `Mayús+F9` y `Mayús+F10`. Faltan Dibujo en vivo, Cambiar vista de cámara, Barra de grabación y Diapositivas, que llegan con su función) |
| VID-076 | Teclas contextuales de video | M4 | Should | — |
| VID-077 | Permisos que necesita el video | M4 | Must | — |

## 04-biblioteca-compartir.md

| ID | Función | Hito | Prioridad | Estado |
|---|---|---|---|---|
| LIB-001 | Vista Biblioteca y cambio con el editor | M1 | Must | En curso |
| LIB-002 | Cuadrícula, selección y menú contextual | M1 | Must | En curso |
| LIB-003 | Tamaño de las miniaturas | M2 | Should | En curso (Windows: deslizador de 50 a 300 % con el porcentaje, `Ctrl` + rueda y `Ctrl` +/-, la selección sigue a la vista, sin nombres por debajo de 80 %, guardado) |
| LIB-004 | Ordenar y vista de lista | M2 | Must | En curso (Windows: seis criterios, orden natural, dirección, vista Lista con cabeceras que ordenan, se recuerda; columnas que se redimensionan arrastrando su borde y se ocultan desde el menú de la cabecera, guardadas) |
| LIB-005 | Panel de filtros: tipo y fecha | M2 | Must | En curso (Windows: panel plegable con Favoritos, Tipo, Fecha con accesos rápidos y Año ▸ Mes ▸ Día, Aplicaciones, Sitios web y Etiquetas, conteos sobre las demás secciones, fichas y Quitar filtros; faltan los tipos que aún no existen y probar en red) |
| LIB-006 | Filtros por aplicación y sitio web | M2 | Should | En curso (Windows: sin el ícono de cada aplicación) |
| LIB-007 | Campo de búsqueda | M2 | Must | En curso (Windows: falta el texto de las guías de pasos, que llegan en M5) |
| LIB-008 | Buscar el texto de las imágenes | M5 | Should | — |
| LIB-009 | Etiquetas: añadir y quitar | M2 | Must | En curso (Windows: botón Etiquetas, menú contextual y Detalles, LIB-012; falta Imagen > Editar etiquetas en el editor) |
| LIB-010 | Gestionar etiquetas | M2 | Should | En curso (Windows: renombrar, fusionar, eliminar y seleccionar; una etiqueta que llega en un `.xmeta` se une a la grafía que ya tiene la biblioteca) |
| LIB-011 | Favoritos | M2 | Should | En curso (Windows: estrella, menú contextual y Detalles, LIB-012; falta la bandeja, TRY-016) |
| LIB-012 | Detalles del elemento | M2 | Should | En curso (Windows: panel Detalles que sigue a la selección, botón, Alt+Enter / Cmd+I y menú contextual; nombre, etiquetas y favorito editables; copiar cada dato; varios elementos; `library.get --details` y `library.open_url`. Faltan duración y fps de videos y pasos de guías (M3 y M4), y Último envío, que llega con el historial de SHR-012. El tooltip de la miniatura es el del sistema) |
| LIB-013 | Vista rápida | M2 | Should | En curso (Windows: Espacio en la biblioteca y en la bandeja, ← →, Esc o Espacio cierran, Intro abre; recorre solo la selección si hay varios; pie con nombre, tamaño, fecha y posición; menú contextual. Faltan video, GIF y guías, que llegan con M3 y M4) |
| LIB-014 | Mostrar en la biblioteca | M2 | Should | En curso (Windows: Archivo > Mostrar en la biblioteca y `library.reveal`: cambia a la Biblioteca, selecciona y desplaza hasta el elemento, quita los filtros con aviso; y desde el menú de la bandeja, TRY-016; falta Deshacer en el aviso) |
| LIB-015 | Eliminar | M1 | Must | En curso |
| LIB-016 | Crear desde la biblioteca | M2 | Should | En curso (Windows: Crear ▾ en la barra y Crear ▸ en el menú contextual con Nuevo desde plantilla; Diapositivas narradas y Combinar videos aparecen desactivados hasta el video, M4) |
| LIB-017 | Imprimir desde la biblioteca | M2 | Could | En curso (Windows: Imprimir… del menú contextual y Ctrl+P en la biblioteca, una imagen por página ajustada y girada como mejor cabe, vista previa de todas, sin abrirlas ni cambiar sus archivos; `document.print` y `print_pages` con `paths`) |
| LIB-018 | Exportación por lotes desde la biblioteca | M2 | Should | En curso (Windows: Exportar… del menú contextual: uno con Guardar como, varios con la exportación por lotes de EDT-113 ya cargada; aviso si el destino está en la biblioteca. Videos, GIF y guías llegan con M3 y M4; el progreso va en el diálogo, no en la barra de estado de LIB-026) |
| LIB-019 | Guías de pasos en la biblioteca | M5 | Should | — |
| LIB-020 | Autoguardado de capturas | M1 | Must | En curso |
| LIB-021 | Guardar como y copia editable | M1 | Must | En curso |
| LIB-022 | Índice propio de la biblioteca | M1 | Must | En curso |
| LIB-023 | Reconstruir y comprobar la biblioteca | M2 | Must | En curso (Windows: Reconstruir el índice y Comprobar desde el menú ⋯ de la Biblioteca, `library.rebuild` y `library.check` con limpieza de `.xmeta` huérfanos; faltan el progreso en segundo plano mientras se usa la biblioteca, mover los dañados aparte, el aviso de poco espacio y la entrada en Preferencias) |
| LIB-024 | Archivos añadidos a la carpeta e importación | M2 | Should | En curso (Windows: los archivos que llegan a la carpeta aparecen solos; fecha EXIF de JPG y PNG; Importar archivos… e Importar una carpeta… en el menú ⋯ de la biblioteca y al soltar sobre la vista Biblioteca, con la etiqueta de la carpeta; `library.import`. Solo `.xshot`, PNG y JPG: GIF, WebP, BMP, TIFF, HEIC y videos llegan con sus funciones) |
| LIB-025 | Bibliotecas grandes | M2 | Must | En curso (Windows: la cuadrícula y la lista solo dibujan lo visible; consultas medidas con 50.000 elementos; faltan los marcadores con el ícono del tipo, el límite de 2 GB de la caché de miniaturas y el aviso por encima de 100.000) |
| LIB-026 | Barra de estado de la biblioteca | M2 | Could | En curso (Windows: totales, visibles, seleccionados con su tamaño, ubicación local, sincronizada con su servicio, de red o no disponible; reconstruir, importar y exportar con progreso y Cancelar; plegable y recordada; aviso al lector al terminar. Faltan reconocer texto (OCR, M4) y los avisos de autoguardado fallido; mover la biblioteca muestra su progreso en Preferencias) |
| LIB-027 | Ubicación por defecto | M1 | Must | En curso |
| LIB-028 | Cambiar o mover la biblioteca | M2 | Must | En curso (Windows: Preferencias > Biblioteca > Cambiar…, mover con progreso y Cancelar que devuelve todo, empezar vacía, usar la que hay, informe de lo que no se movió, los documentos abiertos siguen a sus archivos, la automatización no puede; espacio libre comprobado antes (en el mismo disco no hace falta), tiempo restante, Reintentar; Fusionarla con la mía si la carpeta ya tiene una biblioteca (LIB-031); falta el bloqueo por configuración administrada) |
| LIB-029 | Biblioteca en una carpeta sincronizada | M2 | Must | En curso (Windows: Cambiar… ofrece las carpetas de OneDrive (personal y trabajo), Dropbox, Google Drive, Box e iCloud y dice si ya tienen una biblioteca; los archivos solo en la nube no se leen al indexar ni al hacer miniaturas, llevan la insignia "En la nube" y la búsqueda por etiqueta los encuentra; las etiquetas de otro equipo llegan por el `.xmeta`. Las copias en conflicto junto a su original llevan la insignia "Conflicto" y Resolver (quedarse con una o conservar las dos, `library.resolve`). Abrir un elemento solo en la nube avisa mientras se descarga; mover allí una biblioteca de más de 5 GB avisa del espacio en el servicio) |
| LIB-030 | Biblioteca en una ubicación de red | M2 | Should | En curso (Windows: ruta UNC o unidad de red asignada, con el índice en este equipo y todos los filtros, conteos y búsquedas; sin conexión, "Biblioteca no disponible" en la barra de estado y la búsqueda sigue respondiendo con el índice; las capturas nuevas esperan en los datos de la app y pasan solas a la red al volver, con aviso, y sus documentos abiertos las siguen; borrar allí es definitivo y siempre se pregunta; falta probar un volumen SMB en macOS) |
| LIB-031 | Fusionar bibliotecas | M2 | Should | En curso (Windows: Biblioteca > ⋯ > Fusionar otra biblioteca… y, en Preferencias > Biblioteca > Cambiar…, "Fusionarla con la mía" cuando la carpeta ya tiene una; copia con sus `.xmeta`, sin tocar la otra carpeta; los duplicados por contenido no se copian y le pasan sus etiquetas y favorito al de aquí; nombre ocupado con " (2)"; etiquetas unidas sin distinguir mayúsculas; progreso en la barra de estado, Cancelar que quita lo copiado e informe final; `library.merge` en el CLI y MCP) |
| LIB-032 | Llevar la biblioteca a otro equipo | M2 | Should | En curso (Windows: la carpeta copiada a otro sitio y con otro perfil, con su índice o sin él, muestra todo con sus IDs, etiquetas y favoritos, sin renombrar ni añadir archivos; se elige en la bienvenida (Usar una biblioteca existente…) o en Preferencias > Biblioteca > Cambiar…; falta probar la carpeta de Windows en un Mac) |
| LIB-033 | Exportar toda la biblioteca a archivos normales | M2 | Should | En curso (Windows: ⋯ > Exportar toda la biblioteca…, toda o lo filtrado, PNG o JPG, subcarpetas, `metadatos.csv`, progreso, Cancelar y resumen; `library.export`. Videos y guías llegan con M3 y M4) |
| LIB-034 | Comandos de la biblioteca (CLI y MCP) | M2 | Must | En curso (`library.list`, `library.search`, `library.get`, `library.tag`, `library.tags`, `library.favorite`, `library.reveal`, `library.delete` con `--yes`, `library.rebuild`, `library.check`, `library.location`; más `library.export` (LIB-018, LIB-033), `library.import` (LIB-024), `library.duplicate` (TRY-010), `library.resolve` (LIB-029) y `library.open_url` (LIB-012)) |
| TRY-001 | Mostrar y ocultar la bandeja | M1 | Must | En curso |
| TRY-002 | Contenido y orden | M1 | Must | En curso |
| TRY-003 | Capacidad | M2 | Should | En curso (Windows: `tray.capacity` de 1 a 200, 50 por defecto; sale el más antiguo libre; fijados, con cambios sin guardar y el que se ve nunca salen solos; reducirla actúa al momento; el aviso "N de M" va en la propia bandeja) |
| TRY-004 | Fijar y desfijar | M2 | Should | En curso (Windows: chincheta al pasar el ratón, menú contextual, `Ctrl+Shift+P`, a la izquierda con una línea, se conservan entre sesiones y con la limpieza al salir; varios a la vez llegan con la selección múltiple, TRY-012) |
| TRY-005 | Reordenar | M2 | Should | En curso (Windows: arrastrar dentro de la bandeja con una línea de destino, fija o desfija según dónde cae, `Ctrl+Shift+←/→` con anuncio, orden guardado; varios a la vez con TRY-012) |
| TRY-006 | Renombrar | M2 | Should | En curso |
| TRY-007 | Tamaño de las miniaturas | M2 | Could | En curso (Windows: menú de una zona vacía de la bandeja y Preferencias > Biblioteca, `tray.thumbnailSize` 48, 72 o 112 px) |
| TRY-008 | Cerrar elementos | M1 | Must | En curso (M2: Cerrar los no fijados y la limpieza al salir conservan los fijados) |
| TRY-009 | Eliminar desde la bandeja | M1 | Must | En curso |
| TRY-010 | Duplicar | M2 | Should | En curso (Windows: menú contextual de la bandeja y de la biblioteca, Archivo > Duplicar y `Ctrl+Shift+D`, varios a la vez, `library.duplicate`; falta "sin historial de envíos", que llega con SHR-012) |
| TRY-011 | Indicador de cambios sin guardar | M1 | Must | En curso |
| TRY-012 | Selección múltiple y acciones | M2 | Should | En curso (Windows: Ctrl+clic, Mayús+clic y Ctrl+A en la bandeja; Fijar, Cerrar, Eliminar, Duplicar, Etiquetas, Favorito, Exportar y Crear: Combinar en plantilla. Compartir, Enlace, Diapositivas narradas y Combinar videos llegan con sus funciones) |
| TRY-013 | Navegación con teclado | M1 | Should | En curso |
| TRY-014 | Combinar arrastrando al lienzo | M2 | Must | En curso (Windows: arrastrar desde la bandeja al lienzo de otro documento crea imágenes flotantes en cascada donde se sueltan, con una marca de dónde caerán; sobre una zona de plantilla, la llena; falta la prueba manual de Carlos, porque el arrastre nativo no se puede automatizar, y lo de video llega con VID-) |
| TRY-015 | Arrastrar a otras aplicaciones | M1 | Must | En curso |
| TRY-016 | Detalles, etiquetas y otras acciones | M2 | Should | En curso (Windows: Detalles, Etiquetas…, Favorito, Vista rápida, Mostrar en la biblioteca, Mostrar en el Explorador, Copiar ruta, Copiar todo y Exportar… desde la bandeja; Compartir y Enlace llegan con SHR-) |
| TRY-017 | Comandos de la bandeja (CLI y MCP) | M2 | Should | En curso (Windows: `tray.list`, `tray.open`, `tray.activate`, `tray.pin`, `tray.unpin`, `tray.move`, `tray.close`) |
| SHR-018 | Archivo | M1 | Must | En curso |
| SHR-019 | Portapapeles | M1 | Must | En curso |

## 05-app.md

| ID | Función | Hito | Prioridad | Estado |
|---|---|---|---|---|
| APP-001 | Proceso en segundo plano e instancia única | M0 | Must | En curso (Windows: una instancia; abrirla otra vez muestra la ventana principal; abrir archivos con XcreenShot (doble clic, "Abrir con", línea de comandos) los entrega a la instancia en marcha: `.xshot`, PNG y JPG al editor, `.xtheme` y `.xtemplate` a su importación en el editor y `.xpreset` al panel de captura; el instalador registra `.xshot`, `.xpreset`, `.xtheme` y `.xtemplate` (nunca PNG ni JPG); `app.open_files`. Faltan `.xstamps` y `.xsettings` (su importación aún no tiene ventana), dos usuarios a la vez y macOS) |
| APP-002 | Ícono de XcreenShot y menú rápido | M0 | Must | En curso |
| APP-003 | Iniciar al iniciar sesión | M0 | Must | Win |
| APP-004 | Cerrar ventanas y salir | M0 | Must | En curso |
| APP-005 | Primer arranque | M1 | Should | En curso (M2: el paso Inicio y biblioteca ofrece Usar una biblioteca existente… e Importar ajustes…, APP-055) |
| APP-006 | Ventana de Preferencias | M0 | Must | En curso |
| APP-007 | Sección General | M0 | Must | En curso |
| APP-008 | Secciones Captura, Video, Editor, Biblioteca y Compartir | M1 | Must | — |
| APP-009 | Sección Avanzado | M1 | Should | En curso |
| APP-010 | Restablecer ajustes y avisos | M1 | Should | En curso |
| APP-011 | Exportar e importar ajustes | M2 | Should | En curso (Windows: Preferencias > Avanzado > Archivo de ajustes, `.xsettings` con preferencias, atajos y perfil, temas, favoritos y efectos guardados, y la barra; resumen previo por grupo; conversión de atajos Windows ↔ macOS; mismos nombres con Reemplazar, Conservar ambos u Omitir; `settings.export` y `settings.import`. Los presets de captura viajan con sus atajos desde M3.1; los destinos de compartir se añaden con M6) |
| APP-012 | Tema de la interfaz | M0 | Must | En curso (Windows: Sistema, Claro u Oscuro; aspecto Cristal desde el 2026-11-12 con Intensidad del cristal, `general.glassIntensity`, y Color de acento, `general.accent`, en Preferencias > General) |
| APP-013 | Idioma de la interfaz | M0 | Must | En curso |
| APP-014 | Tooltips y pistas | M1 | Should | En curso |
| APP-015 | Ayuda, Acerca de y comentarios | M1 | Should | En curso |
| APP-016 | Notificaciones | M1 | Must | En curso |
| APP-017 | Ámbitos y reglas de los atajos | M0 | Must | En curso |
| APP-018 | Sección Atajos | M0 | Must | En curso |
| APP-019 | Detección de conflictos | M0 | Must | En curso |
| APP-020 | Tomar atajos ocupados | M0 | Must | En curso |
| APP-021 | Perfiles de atajos | M0 | Must | En curso |
| APP-022 | Atajos de presets, destinos y otros comandos | M1 | Should | En curso |
| APP-023 | Hoja de atajos | M2 | Could | En curso (Windows: Ayuda > Atajos de teclado y `Ctrl+/` en la ventana principal, grupos, búsqueda, inactivos marcados, teclas contextuales, Imprimir y Guardar como PDF de 1–2 páginas A4; `shortcuts.export` pdf, html y json. Falta abrirla con `Ctrl+/` desde las demás ventanas y la advertencia de VoiceOver en macOS) |
| APP-024 | Permisos de macOS y Reparar permisos | M0 | Must | En curso |
| APP-025 | Aviso periódico de acceso a la pantalla (macOS) | M1 | Should | — |
| APP-026 | Privilegios y privacidad en Windows | M1 | Should | En curso |
| APP-027 | Registro de comandos | M0 | Must | Win |
| APP-028 | CLI `xcreenshot` | M0 | Must | En curso |
| APP-029 | Servidor MCP | M2 | Should | En curso (Windows: herramientas con sus esquemas e indicaciones, `include_image` con 1568 px, recursos de la guía, capturas recientes, bandeja y eventos, y eventos por suscripción; recurso de presets desde M3.1; falta el de destinos, que llega con M6) |
| APP-030 | Catálogo mínimo de comandos | M1 | Must | En curso |
| APP-031 | Eventos | M1 | Should | En curso |
| APP-032 | Seguridad de la automatización | M0 | Must | En curso |
| APP-033 | `AGENTS.md` | M2 | Should | En curso (Windows: se genera del registro de comandos más un texto escrito a mano; `xcreenshot agents-md`, recurso MCP `xcreenshot://agents` y en la carpeta de instalación; una prueba falla si el del repositorio no está al día; faltan las recetas de grabar y extraer texto, que llegan con `VID-` y `SMT-`) |
| APP-034 | Funciones en prueba | M1 | Should | En curso |
| APP-035 | Configuración administrada | M6 | Should | — |
| APP-036 | Actualizaciones | M6 | Must | — |
| APP-037 | Instalador de Windows | M6 | Must | — |
| APP-038 | Modo portable (Windows) | M6 | Could | — |
| APP-039 | Paquete de macOS | M6 | Must | — |
| APP-040 | Requisitos del sistema | M0 | Must | En curso |
| APP-041 | Windows en ARM | M6 | Should | — |
| APP-042 | Desinstalación | M6 | Must | — |
| APP-043 | Navegación con teclado | M1 | Must | En curso |
| APP-044 | Lectores de pantalla | M1 | Must | En curso |
| APP-045 | Contraste, escalado y movimiento | M1 | Should | En curso |
| APP-046 | Registros e información de diagnóstico | M0 | Must | En curso |
| APP-047 | Recuperación tras un cierre inesperado | M1 | Must | En curso |
| APP-048 | Del atajo a la cruceta | M1 | Must | En curso |
| APP-049 | De la captura al editor | M1 | Must | En curso |
| APP-050 | Editor con imágenes muy grandes | M2 | Must | En curso (Windows: límite de 32.000 px y 500 MP con mensaje claro; 10.000 × 30.000 px con 200 objetos se desplaza a más de 70 fps, guarda en 3,5 s y exporta en 3,5 s (release); aviso "Exportando…" al exportar; memoria medida: unos 1,6 GB sobre el reposo con ese documento, tras arreglar que se descodificaba la imagen una vez por petición; falta ofrecer dividir las capturas con desplazamiento demasiado largas, CAP- de M3) |
| APP-051 | Consumo en reposo y arranque | M1 | Should | En curso |
| APP-052 | Escala de la biblioteca | M2 | Must | En curso (Windows: búsqueda y filtros sobre 50.000 elementos en 33 a 136 ms en release; con 20.000 en depuración, primeras miniaturas en 0,7 s y recorrido completo sin fotogramas de más de 100 ms; falta medir 50.000 con el instalador y la reconstrucción en segundo plano) |
| APP-053 | Cruceta con varios monitores 4K | M1 | Must | En curso |
| APP-054 | Local y privado | M0 | Must | En curso |
| APP-055 | Portabilidad de datos y migración entre equipos | M2 | Should | En curso (Windows: Ayuda > Mover XcreenShot a otro equipo… explica los dos pasos con Mostrar la carpeta (o que ya está sincronizada) y Exportar ajustes…, y qué hacer en el equipo nuevo; la bienvenida ofrece Usar una biblioteca existente… e Importar ajustes…; una carpeta con biblioteca ofrece Usarla o Fusionarla (LIB-031); un `.xshot` se abre con el lector ZIP de Windows y trae `document.json` y `base.png`; falta el criterio Windows → Mac) |

## 06-inteligentes.md

| ID | Función | Hito | Prioridad | Estado |
|---|---|---|---|---|
| SMT-001 | Procesamiento local y sin conexión | M3 | Must | En curso (Windows: Extraer texto usa el reconocimiento del sistema, sin red ni descargas; el texto y la zona viven en memoria hasta cerrar su ventana; el registro solo guarda palabras, tiempo e idioma; falta el progreso con Cancelar de más de 500 ms y comprobarlo con un monitor de red durante 30 min) |
| SMT-002 | Activar y desactivar las funciones inteligentes | M5 | Should | — |
| SMT-003 | Motor de reconocimiento de texto | M3 | Must | En curso (Windows: el reconocimiento del sistema, bloques, líneas y palabras con su rectángulo, color del texto y del fondo y altura, orden de lectura con columnas y párrafos, texto claro sobre oscuro, imágenes altas por tramos; medido 1920×1080 con 600 palabras en 249 ms y 400 líneas en 20.000 px sin perder ni repetir; falta la familia de letra y la confianza, que el motor de Windows no da) |
| SMT-004 | Idiomas del reconocimiento | M3 | Must | En curso (Windows: Preferencias > Captura > Idioma del reconocimiento con Automático (el de la interfaz y los del sistema) o hasta 3; los que faltan salen "no está en este equipo" con el enlace a Configuración; Reconocer en otro idioma en la ventana; `language` en los comandos. Portugués, francés y alemán dependen de que Windows los tenga: ver QUESTIONS) |
| SMT-005 | Extraer texto como modo de captura | M3 | Must | En curso (Windows: Selección = Extraer texto en la pestaña Imagen (Abrir en y Capturar cursor desactivados con su motivo), acción con atajo `Ctrl+Alt+Mayús+T`, entrada en el menú del ícono, preset incorporado y `smart.extract_text` (pantalla, `rect` o archivo); copia el texto con y sin formato; ventana de resultados o notificación "Texto copiado (N palabras)" con Ver; sin texto, el aviso y el portapapeles intacto; Guardar también la imagen en la biblioteca. Si tarda más de 300 ms, un indicador "Leyendo el texto…" aparece junto a la zona) |
| SMT-006 | Ventana de resultados de Extraer texto | M3 | Must | En curso (Windows: ventana Texto extraído con el texto editable con formato (colores y tamaños) o Sin formato, Copiar todo (`Ctrl+Mayús+C`), Copiar y cerrar (`Ctrl+Enter`), Más > Guardar como texto, Reconocer en otro idioma y Unir las líneas de cada párrafo; corrector ortográfico; `Esc` cierra; varias a la vez; sin texto, consejos y Repetir. Recuerda su tamaño y posición. Falta la miniatura de las palabras dudosas, que el motor de Windows no marca) |
| SMT-007 | Extraer texto en el editor | M5 | Must | — |
| SMT-008 | Extraer texto de un fotograma de video | M5 | Should | — |
| SMT-009 | Editar texto de la imagen | M5 | Should | — |
| SMT-010 | Inicio y análisis de la Redacción automática | M5 | Must | — |
| SMT-011 | Categorías de datos sensibles | M5 | Must | — |
| SMT-012 | Palabras y patrones personalizados | M5 | Should | — |
| SMT-013 | Estilo de las redacciones | M5 | Must | — |
| SMT-014 | Añadir y quitar redacciones con un clic | M5 | Should | — |
| SMT-015 | Redactar todas las coincidencias | M5 | Should | — |
| SMT-016 | Redactar varias capturas a la vez | M5 | Should | — |
| SMT-017 | Rendimiento y privacidad de la redacción | M5 | Must | — |
| SMT-018 | Separar elementos | M5 | Should | — |
| SMT-019 | Simplificar automático | M5 | Should | — |
| SMT-020 | Quitar fondo | M5 | Should | — |
| SMT-021 | Aplicar solo al texto | M5 | Should | — |
| SMT-022 | Iniciar el Grabador de pasos | M5 | Must | — |
| SMT-023 | Barra del grabador | M5 | Must | — |
| SMT-024 | Qué crea un paso | M5 | Must | — |
| SMT-025 | Texto automático de cada paso | M5 | Must | — |
| SMT-026 | Imagen de cada paso | M5 | Must | — |
| SMT-027 | Límites, rendimiento, ventanas elevadas y recuperación | M5 | Must | — |
| SMT-028 | Estructura y formato de la guía | M5 | Must | — |
| SMT-029 | Editar la guía | M5 | Must | — |
| SMT-030 | Guía a Word y PowerPoint | M5 | Must | — |
| SMT-031 | Guía a Pages y Keynote (macOS) | M5 | Should | — |
| SMT-032 | Guía a Markdown y HTML | M5 | Should | — |
| SMT-033 | Guía a PDF e impresión | M5 | Must | — |
| SMT-034 | Imágenes de la guía por separado y portapapeles | M5 | Should | — |
| SMT-035 | Guías en la biblioteca | M5 | Should | — |
| SMT-036 | Crear un código QR | M5 | Could | — |
| SMT-037 | Código QR de un enlace compartido | M5 | Could | — |
| SMT-038 | Leer códigos QR | M5 | Could | — |
| SMT-039 | Activar la IA en la nube: clave y modelos (función propia, adelantada) | M1 | Should | En curso |
| SMT-040 | Preguntar a la IA sobre una imagen (función propia) | M1 | Should | En curso |
| SMT-041 | Sugerir nombre con IA (función propia) | M1 | Should | En curso |
| SMT-042 | Privacidad y costos de la IA en la nube | M1 | Must | En curso |
| SMT-043 | IA desde un cliente MCP (función propia) | M1 | Should | En curso |

