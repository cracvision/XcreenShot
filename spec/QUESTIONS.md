# Preguntas y decisiones abiertas

Formato: ID afectado · pregunta · propuesta · estado. Carlos decide; el equipo de construcción anota aquí sus dudas.

## Decisiones de SPEC.md §9 (pendientes de Carlos)

| ID | Pregunta | Propuesta | Estado |
|---|---|---|---|
| D-01 | Licencia del repo | Privado; MIT o Apache-2.0 si se abre | Abierta |
| D-02 | Proveedor del Enlace para compartir | Carpeta sincronizada de OneDrive o Google Drive primero | Abierta |
| D-03 | Impresora virtual | No; importar PDF (CAP-041) | Abierta |
| D-04 | Codificación de video | Codificadores nativos, sin FFmpeg | Abierta |
| D-05 | Destinos con OAuth prioritarios | Los que usen Carlos y sus colegas | Abierta |
| D-06 | Firma de código | Authenticode + Apple Developer Program | Abierta |
| D-07 | Idiomas | Inglés y español | Abierta |
| D-08 | Atajos por defecto | Perfil "Coexistencia" mientras Snagit siga instalado | Abierta |

## Dudas que dejaron los redactores del spec

| ID | Pregunta | Propuesta | Estado |
|---|---|---|---|
| CAP-065 | Los atajos de coexistencia usan `Ctrl+Alt+Shift` (Windows) y `⌃⌥⇧` (Mac). `Ctrl+Alt` equivale a `AltGr` en algunos teclados, y `⌃⌥` choca con VoiceOver | Probar en los teclados de Carlos antes de M1 | Abierta |
| CAP-037 | La captura de página web por URL carga la página sin las cookies del navegador, así que las páginas con inicio de sesión salen como las ve un visitante | Aceptar en v1 | Abierta |
| CAP-041 | En Windows, importar PDF exige imprimir primero a PDF | Aceptar (D-03) | Abierta |
| APP-021 | Nombre del perfil de atajos: "Compatibilidad con Snagit" usa la marca en la UI | Llamarlo "Compatibilidad" | Abierta |
| APP-040 | Versión mínima de macOS | macOS 14+, compilación universal | Abierta |
| SPEC §7 | Faltan en la tabla de formatos `.xsettings` (exportar preferencias), `managed.json` (configuración administrada) y la estructura del documento `guide` | Añadirlos cuando se cierre el spec de M1 | Abierta |
| VID-036 | ¿La cámara se muestra en espejo por defecto? | Sí; Carlos puede preferir que no | Abierta |
| VID-025 | Calidad de video por defecto | Alta (30 fps) | Abierta |
| VID-075 | Posibles choques entre atajos de video y del editor | Revisar con EDT- al cerrar los atajos | Abierta |
| VID-062 / SMT- | Extraer texto de un fotograma depende del OCR, que llega en M3/M5 | Mover VID-062 a M5 | Abierta |
| SMT-010 | Redacción automática: URLs y fechas desactivadas por defecto; documentos de identidad de EE. UU., España y Brasil | Aceptar | Abierta |
| 05-app.md | Referencia otras funciones por nombre, no por ID | Pasada de enlaces antes de M1 | Abierta |
| 02 / 04 | Faltan tablas de cierre en `02-editor.md` y el marco de compartir y destinos en `04-biblioteca-compartir.md` | Completar antes de M2 | Abierta |

## Dudas del equipo de construcción (M0)

| ID | Pregunta | Propuesta (implementada como provisional) | Estado |
|---|---|---|---|
| SPEC §4 (M0) | El criterio de salida pide "un atajo global que ejecute un comando de prueba", pero el spec no fija cuál | `app.ping` con `Ctrl+Alt+Shift+K` (Windows) y `⌃⌥⇧K` (macOS) en ambos perfiles: familia Coexistencia y una letra que CAP-065 no usa. Se puede quitar o cambiar en Preferencias > Atajos | Abierta |
| APP-020 | Sin ganchos de teclado de bajo nivel (que el propio spec prohíbe), Windows no deja registrar una combinación que ya registró otra app. No sabemos si `PrtScn` se puede tomar con la Herramienta Recortes activa | La opción y su estado "Tomado de otra app" existen; si el sistema no lo permite, el atajo queda "En conflicto" con el botón para abrir el ajuste del sistema. Probarlo en el equipo de Carlos (`tests/manual/APP-020.md`) | Abierta |
| APP-021 | El aviso "la otra app de captura está en marcha" necesita su nombre de proceso, y CLAUDE.md no permite esa marca en el código ni investigarlo en su instalación | No implementar el aviso hasta que Carlos dé el nombre del proceso (se puede ver en el Administrador de tareas). El perfil se llama "Compatibilidad" en la interfaz, según la propuesta ya anotada sobre APP-021 | Abierta |
| APP-002 | El clic izquierdo (Windows) abre el panel de captura o la captura universal, que llegan en M1 | Hasta M1, todas las opciones abren la ventana principal | Provisional |
| APP-003 | En macOS, el arranque al iniciar sesión usa un LaunchAgent; las versiones recientes lo muestran en "Permitir en segundo plano" y no en "Abrir al iniciar sesión" | Pasar a `SMAppService` en M6, junto con la firma | Abierta |
| APP-017 | El spec pide mostrar los atajos con el carácter de la distribución de teclado activa. WebView2 (Windows) ofrece esa información; WKWebView (macOS) no | En macOS se muestran con la etiqueta de la distribución estadounidense; los atajos funcionan igual, porque se guardan por tecla física | Abierta |
| APP-017 | Re-registro automático de los atajos tras suspender o cambiar de sesión | No hecho en M0: los registros del sistema suelen sobrevivir a la suspensión. Se comprueba con `tests/manual/APP-017.md` y se implementa si falla | Abierta |
| APP-024 | macOS no distingue "denegado" de "nunca pedido" para Grabación de pantalla y Accesibilidad | XcreenShot recuerda qué permisos pidió y muestra "Denegado" solo después de pedirlo | Provisional |
| APP-040 | Identificador de paquete (necesario para firma y permisos de macOS) | `com.cracvision.xcreenshot` | Abierta |
| APP-042 | El instalador de Windows pone la app en `%LOCALAPPDATA%\XcreenShot` y los registros de la app viven en `%LOCALAPPDATA%\XcreenShot\data`, dentro de la misma carpeta. Al desinstalar puede quedar la carpeta a medias, o borrarse los registros sin preguntar | Separar los datos de la carpeta de instalación y decidir qué borra el desinstalador (preguntar si conservar ajustes y registros) al implementar APP-037/APP-042 en M6 | Abierta |

## Dudas del equipo de construcción (M1)

| ID | Pregunta | Propuesta (implementada como provisional) | Estado |
|---|---|---|---|
| CAP-054 / LIB-020 | La ruta de la captura empieza guardando en la biblioteca, que llega en la parte 3 de M1 | Desde la entrega 4a (decisión A del plan, Carlos, 2026-10-24) cada captura se guarda como `.xshot` en `Documentos\XcreenShot\AAAA-MM\` antes de abrirse. El índice, la vista Biblioteca y la bandeja completa siguen en la entrega 5. Las capturas PNG anteriores se quedan donde están | Decidido |
| CAP-045 | "Abrir en = Editor" necesita el editor (parte 2 de M1) | Desde la entrega 4a abre la captura en el editor | Decidido |
| CAP-008 | Valores por defecto de las capturas lanzadas por CLI o agentes | Los del panel (copiar al portapapeles = sí), pero sin abrir ventanas salvo `--open editor` (APP-030 pide anotar y exportar "sin abrir ninguna ventana"). Avisan con la notificación de automatización | Provisional |
| CAP-023 | Excluir las ventanas de XcreenShot de las capturas de forma permanente también las ocultaría al compartir pantalla en Teams o Zoom | La exclusión se activa solo durante cada captura | Decidido |
| CAP-047 | El cursor es un objeto del documento, que llega con el editor | `--cursor` se acepta y avisa en `warnings`; se implementa con el editor | Abierta |
| Versiones | El spec no fija la numeración de versiones; todas las compilaciones decían 0.1.0 y no se distinguían | Una versión menor por hito (0.1 = M0, 0.2 = M1…) y el commit de la compilación junto a la versión, por ejemplo `0.2.0 (670038c)`, en la ventana principal, el diagnóstico y `xcreenshot --version` | Provisional |
| APP-017 | XcreenShot acepta como atajo global combinaciones que las apps usan a diario (`Ctrl+P`, `Ctrl+C`, `Ctrl+S`…), y entonces ninguna otra app las recibe mientras XcreenShot está abierto | Al asignar una de esas combinaciones como global, avisar ("Ctrl+P deja de imprimir en las demás apps") y pedir confirmación, sin prohibirlo | Abierta |
| CAP-009 | Carlos encuentra más claro "Capturar" que "Imagen" en la barra de captura | Decidido por Carlos (2026-10-18): el botón se llama **Capturar** (`Enter`; `I` sigue funcionando). Revisar cuando lleguen Video y Desplazamiento (M3–M4), porque entonces la barra tendrá tres tipos | Decidido |
| APP-013 | Carlos pide cambiar de idioma con un clic | Selector rápido **EN \| ES** en la ventana principal y en Preferencias; cambia `general.language`. La opción completa, con "Sistema", sigue en Preferencias > General | Decidido |
| CAP-015 | `C` copia el color sin ninguna confirmación visible | Aviso breve "Color #RRGGBB copiado" sobre la selección (no sale en la captura) | Decidido |
| CAP-065 | Las acciones de un solo modo (Captura de ventana, Ventana activa, Región, Pantalla completa) toman de la pestaña Imagen lo que no es el modo. ¿También cuando las lanza el CLI o un agente? | Desde atajos y menús, sí. Desde el CLI y MCP usan valores fijos (los de CAP-008), para que un script dé siempre el mismo resultado aunque cambies el panel. `capture.universal`, `capture.image` y `capture.capture` sí parten del panel en todos los casos | Provisional |
| CAP-001 | "Se oculta mientras dura la captura (CAP-023)": ¿también con "Ocultar XcreenShot al capturar" desactivado? | Solo con esa preferencia activada; si la desactivas, el panel se queda y puede salir en la captura. Vuelve a su estado al terminar; si se abre el editor, el editor queda delante | Provisional |
| CAP-001 / CAP-065 | El atajo "Mostrar u ocultar el panel" y el ícono "Abrir el panel" | El atajo oculta el panel si es la ventana de delante y, si no, lo trae al frente. El ícono y el menú siempre lo abren | Provisional |
| CAP-001 | Menú Más: varias entradas dependen de funciones de M3 (presets, Lanzador de borde), de macOS (iPhone o iPad) o de la Ayuda (APP-015) | Solo aparecen las que ya funcionan: Preferencias de captura y Restablecer ajustes de captura | Provisional |
| CAP-045 | Opciones de "Abrir en" antes de la biblioteca y el editor | Editor (por ahora, la ventana principal con la última captura) y Nada (solo guardar) | Provisional |
| CAP-048 | `Esc` cancela la cuenta atrás aunque XcreenShot no tenga el foco | Mientras corre el contador, `Esc` queda reservado para XcreenShot: no llega a las demás apps (un menú abierto no se cierra con `Esc`, se cierra con un clic). Al terminar se libera | Provisional |
| CAP-048 | Propiedad "Mostrar la cuenta atrás" y cancelar desde el menú del ícono | Pendiente: llega con el resto de propiedades de Tiempo. Por ahora el contador siempre se ve y se cancela con su botón o con `Esc` | Abierta |

## Dudas del equipo de construcción (M1, entrega 4: editor)

| ID | Pregunta | Propuesta (implementada como provisional) | Estado |
|---|---|---|---|
| EDT-106 / EDT-110 | Cómo conseguir que la imagen exportada sea idéntica en Windows y macOS y que el CLI anote y exporte sin ventanas | Un único motor de dibujo en Rust para la imagen final; Konva solo dibuja en pantalla con la misma geometría (ADR 0004). Librerías nuevas: `tiny-skia`, `zip`, `png`, `cosmic-text` (4b), `konva`, `react-konva` y `@tauri-apps/plugin-dialog`. Rust mínimo 1.88 | Decidido (Carlos, 2026-10-24) |
| LIB-020 / SHR-006 | Nombre de los archivos de la biblioteca: SHR-006 aún no está escrito | `<App> AAAA-MM-DD HH.MM.SS.xshot` (el ejemplo de LIB-020), con ` (2)`… si existe; sin app conocida, `XcreenShot` | Provisional |
| EDT-108 / APP-004 | `Ctrl+W` es "Cerrar ventana" en toda la app, y EDT-116 lo usa para "Cerrar documento" | En la ventana del editor cierra el documento; en las demás ventanas, la ventana. Hasta la entrega 4c los atajos del editor son fijos; entonces pasan a Preferencias > Atajos | Provisional |
| EDT-003 | Sin los menús (entrega 4c), no hay forma de deshacer con el ratón | Botones Deshacer y Rehacer en la barra de herramientas. Con los menús se quedan (ver abajo) | Decidido |
| EDT-005 | El panel lateral recuerda visibilidad, anchura y pestaña; el spec pide que viaje en la exportación de ajustes (APP-011) | Por ahora se recuerda en la propia ventana. Pasa al estado del editor cuando llegue APP-011 | Provisional |
| EDT-107 / LIB-021 | Al guardar un PNG o JPG de fuera de la biblioteca con anotaciones, LIB-021 pide guardar también una copia `.xshot` editable en la biblioteca | Llega con la entrega 5 (biblioteca). En la 4a se avisa antes de integrar las anotaciones en la imagen, con "No volver a preguntar" | Abierta |
| EDT-110 | El spec pide guardar el perfil de color en PNG y JPG | PNG lleva la marca sRGB y la resolución. JPG lleva la resolución, sin perfil incrustado (los visores lo tratan como sRGB). Incrustar un perfil sRGB compacto cuando lleguen las opciones de formato (EDT-112, M2) | Abierta |
| APP-047 | "Más tarde" debe dejar los elementos en Ayuda > Recuperar trabajo | Los elementos se conservan y se ofrecen en el siguiente arranque; la entrada de Ayuda llega con los menús (entrega 4c) | Provisional |
| EDT-012 / EDT-118 | `Esc` cierra el editor y el resto de Preferencias > Editor | Llegaron en la entrega 4c (Carlos las probó, 2026-10-28) | Decidido |
| EDT-119 | `document.update_object` y `document.delete_object` son M2 en la tabla, pero el editor los necesita para mover y borrar | Se adelantan a la 4a, con los parámetros de la tabla (`documentId`, `objectId` u `objectIds`, `set`) | Decidido |
| EDT-002 | La ventana principal de Windows aún tiene el menú de M0 (Archivo: Preferencias, Cerrar, Salir; Ayuda) | Sustituido por los menús del editor en la entrega 4c (Carlos los probó, 2026-10-28) | Decidido |
| EDT-026 | El tamaño del texto va en puntos, pero la imagen está en píxeles físicos | Píxeles = puntos × ppp del documento / 72 (una captura a 150 % tiene 144 ppp, así que 14 pt se ve como 14 pt en esa pantalla). Interlineado 1,2. Una fuente que no está instalada se dibuja con otra y se conserva su nombre | Provisional |
| EDT-040 | El spec no fija qué significa cada valor de Intensidad (1–100) | Desenfoque: radio de 1 + intensidad × 0,25 px (60 → 16 px). Pixelado: bloques de 2 + intensidad × 0,3 px (60 → 20 px). Se calcula sobre lo que hay debajo del objeto. Sólido no usa la intensidad | Provisional |
| EDT-019 | El cuentagotas de pantalla usa la función del motor web; existe en Windows (WebView2), pero no en el de macOS | En Windows, botón Cuentagotas en el selector. En macOS el botón no aparece hasta tener uno nativo (pendiente para cuando empiecen las pruebas en Mac). `I` sobre el lienzo funciona en los dos | Abierta |
| EDT-089 | Cómo saber, al pegar, si el portapapeles aún tiene los objetos copiados en XcreenShot o algo copiado después en otra app | Al copiar se guarda una huella de la imagen que se puso en el portapapeles. Si al pegar la imagen coincide, se pegan los objetos editables; si no, el texto (como objeto de texto) o la imagen | Provisional |
| EDT-089 | Pegar una imagen de otra app con un documento abierto la añade como objeto de imagen, que llega en M2 | Sin documentos abiertos, la imagen se pega como documento nuevo de la biblioteca. Con un documento abierto, un aviso explica que llega en M2 | Provisional |
| EDT-013 | Tamaño de los objetos que inserta `Ctrl+Enter` | 200 × 120 px (flecha de 200 px) en el centro de la vista; el texto se abre para escribir | Provisional |
| EDT-019 | El selector de color se cortaba dentro del panel lateral | Se abre flotando sobre la ventana, junto al botón, con desplazamiento si no cabe | Decidido |
| EDT-019 | Cuándo se cierra el selector de color | Un cuadrito (tema o paleta), el área de color, los campos y el cuentagotas preparan el color sin cerrar, para poder ajustar más cosas. Se aplica con **Aceptar**, `Enter` o un clic en cualquier parte fuera; `Esc` cancela | Decidido (Carlos, 2026-10-27) |
| EDT-116 / CAP-065 | En Windows, Exportar (`Ctrl+Alt+Shift+S`, EDT-116) usa la misma combinación que Capturar imagen (CAP-065), que es global y gana siempre: en el editor esa combinación hace una captura | Exportar pasa a `Ctrl+Alt+Shift+E` en Windows, en los dos perfiles (libre, y AltGr no la usa para ningún símbolo). En macOS no choca y queda `Cmd+Option+Shift+S`. Una prueba automática comprueba que ningún atajo por defecto se repite | Provisional |
| EDT-115 / EDT-116 / APP-017 | Los atajos del editor, ¿en qué ámbito? `Ctrl+W` es Cerrar ventana en la app y Cerrar documento en el editor | Ámbito nuevo **Editor** en Preferencias > Atajos: funciona en la ventana del editor y allí gana a un atajo de la app con la misma combinación (por eso `Ctrl+W` cierra el documento). No puede repetir uno global ni otro del editor. Rehacer y Eliminar tienen dos filas, una por combinación. `Ctrl+1…9`, las flechas, `Tab`, `Enter`, `Esc`, `I`, `[` y `]` siguen fijos (EDT-115, EDT-117) | Provisional |
| EDT-002 | Qué entradas tienen los menús en M1 | Solo las que ya funcionan (el spec lo pide): Archivo, Edición, Ver y Ayuda. Imagen y Objeto aparecen en M2 con sus primeras entradas. En Windows, la barra de menús nativa bajo el título; el atajo se muestra como texto a la derecha, sin registrar la tecla, porque el editor la atiende él mismo según Preferencias > Atajos | Provisional |
| EDT-002 | Menús de macOS | La barra de menús del sistema muestra las mismas entradas, aún sin los atajos a la derecha. Se completa cuando empiecen las pruebas en Mac | Abierta |
| EDT-115 | El spec activa las herramientas con una tecla sola "con el foco en el lienzo" | Funcionan con el foco en cualquier parte del editor salvo en campos de texto, para no tener que hacer clic en el lienzo después de usar la barra o el panel | Provisional |
| EDT-118 | Preferencias > Editor lista ajustes de funciones que llegan en M2 (color del área de trabajo, cuadrícula, barra de edición rápida…) | Aparecen los siete cuyas funciones existen; los demás, con su función | Provisional |
| EDT-002 / EDT-116 | Los menús escriben las teclas con su nombre en inglés (`Delete`, no `Supr`) | Pendiente de decidir si se traducen los nombres de teclas especiales | Abierta |
| EDT-003 | ¿Se quedan los botones de Deshacer y Rehacer cuando llegan los menús? | Se quedan en la barra | Decidido (Carlos, 2026-10-28) |
| EDT-071 / EDT-049 | Carlos quiere que, al sacar un objeto del borde, el lienzo crezca para mostrarlo entero. El spec lo dejaba para M2 y solo al pegar o soltar | Adelantado a M1 para dibujar, mover, redimensionar, girar y pegar, con la preferencia `editor.autoExpandCanvas` (por defecto: sí). El lienzo se encoge solo cuando el objeto vuelve, sin bajar de la imagen o del último recorte. Lo que deja fuera un recorte se queda fuera aunque se mueva. Desenfocar no amplía el lienzo, y se ignora lo que sobresale menos de medio píxel. Spec de EDT-071 actualizado | Decidido (Carlos, 2026-10-27) |
