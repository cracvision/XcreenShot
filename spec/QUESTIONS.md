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
| CAP-054 / LIB-020 | La ruta de la captura empieza guardando en la biblioteca, que llega en la parte 3 de M1 | Hasta entonces cada captura se guarda como PNG en la ubicación por defecto de la biblioteca (`Documentos\XcreenShot`; en tu equipo, Documentos está en OneDrive). Al llegar la biblioteca se guardará como `.xshot` con índice | Provisional |
| CAP-045 | "Abrir en = Editor" necesita el editor (parte 2 de M1) | Hasta entonces muestra la ventana principal con la última captura | Provisional |
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
