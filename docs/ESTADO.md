# Estado del proyecto

Este archivo es el traspaso entre sesiones de Claude Code. Cada sesión nueva lo lee al empezar (lo pide `CLAUDE.md`). Se actualiza al cerrar una entrega o un bloque de trabajo, y siempre antes de pasar a una sesión nueva. Es un resumen: el detalle está en las fuentes de abajo.

**Última actualización:** 2026-11-11 (noche), con la entrega 5 terminada y la 6 avanzada.

## Fuentes

| Qué | Dónde |
|---|---|
| Definición del producto | [`spec/SPEC.md`](../spec/SPEC.md) y los archivos `spec/0x-*.md` |
| Avance por ID | [`spec/PARITY.md`](../spec/PARITY.md) |
| Decisiones y dudas abiertas | [`spec/QUESTIONS.md`](../spec/QUESTIONS.md) |
| Pruebas manuales y sus resultados | [`tests/manual/`](../tests/manual/) |
| Decisiones de arquitectura | [`docs/adr/`](adr/) (la 0004 es la del editor) |
| Formato `.xshot` | [`docs/formats/`](formats/) |
| Comandos para agentes (CLI y MCP) | [`AGENTS.md`](../AGENTS.md) |
| Historial de cambios | `git log` y los PR en [GitHub](https://github.com/cracvision/XcreenShot/pulls) |
| Ayudas para probar en Windows | [`scripts/dev/`](../scripts/dev/README.md) |
| Conversaciones anteriores | Se guardan en este equipo y se pueden buscar cuando falta un detalle que no está en el repositorio |

## Dónde estamos

- **Hito actual:** M1 · Captura y edición básicas. Versión 0.2.0.
- **M0** está terminado y verificado en Windows ([PR #1](https://github.com/cracvision/XcreenShot/pull/1), fusionado).
- **M1, entregas 1 a 3:** rama `m1-capture` ([PR #2](https://github.com/cracvision/XcreenShot/pull/2), abierto, CI en verde).
- **M1, entrega 4 (editor):** rama `m1-editor`, que sale de `m1-capture`. Su PR apunta a `m1-capture` hasta que se fusione el PR #2; después, a `main`.

| Entrega de M1 | Contenido | Estado |
|---|---|---|
| 1 · Motor de captura | Pantalla completa, región y ventana; portapapeles; metadatos; comandos `capture.*` en el CLI | Hecha y probada por Carlos |
| 2 · Selección en pantalla | Pantallas congeladas, cruceta, lupa, detección de ventanas, barra de captura con **Capturar** | Hecha y probada por Carlos |
| 3 · Panel de captura | Pestañas Universal e Imagen, retardo con cuenta atrás, menú del ícono, Repetir última captura y Repetir última área, restablecer ajustes | Hecha; faltan 3 pruebas de Carlos (abajo) |
| 4a · De la captura al archivo | Ventana Editor con lienzo, zoom y barra de estado; Mover y Flecha; deshacer y rehacer; guardar `.xshot`, exportar PNG y JPG, Copiar todo, abrir y soltar archivos, cerrar; aviso de cambios sin guardar y recuperación; cada captura se guarda como `.xshot` en la biblioteca; tira de documentos abiertos; comandos `document.*` y `editor.*` | Hecha y probada por Carlos (tres tandas, 2026-10-25) |
| 4b · Las demás herramientas | Texto, Forma, Desenfocar, Recortar sin perder lo recortado; tiradores con `Shift`/`Alt`; selector de color completo; cortar, copiar y pegar objetos; teclas de herramienta | Hecha y probada por Carlos (cinco tandas, 2026-10-28), con el lienzo que crece (EDT-071) |
| 4c · Menús, teclado y accesibilidad | Barra de menús del editor y menú contextual, tabla completa de atajos (asignables), accesibilidad del lienzo, Preferencias > Editor | Hecha y probada por Carlos (tres tandas, 2026-10-28); falta una tanda corta del arreglo del marco de foco |
| IA en la nube (función propia, SMT-039 a SMT-043) | Aprobada por Carlos el 2026-11-01 (IDEA-002): cambia el principio "todo local" para esta función; las dos vías (la app con la clave de cada uno, y Claude Desktop o Claude Code por MCP); cada colega paga su clave. Preguntar a la IA sobre una imagen, sugerir nombre, servidor MCP | Hecha. Tanda B PASS (2026-11-01). Falta la tanda A (necesita su clave) |
| Multicaptura (función propia, CAP-067) | Pedida por Carlos el 2026-11-01: cada clic captura la ventana bajo el puntero y llega a la app; arrastrar captura un rectángulo; `Esc` termina; panel con las capturas de la sesión para insertarlas en un documento. Adelanta lo básico de las imágenes flotantes (EDT-078) | Hecha; faltan las pruebas de Carlos (dos tandas) |
| Reabrir los documentos al iniciar (función propia, EDT-120) | Pedida por Carlos el 2026-11-01: al volver a abrir XcreenShot, el editor tiene los documentos que había en la tira, en su orden y con el que estaba a la vista. Preferencia en Preferencias > Editor | Hecha; falta la tanda de Carlos |
| Aspecto nuevo: Snap con los colores del logo (APP-012, EDT-001, EDT-002) | Pedido por Carlos el 2026-11-05: interfaz moderna y divertida, con animaciones en menús y botones. Sistema Snap (de Claude Design) llevado a todo el editor, con la paleta, la letra (Outfit) y el logo de Carlos. El logo aparece en el editor vacío con el REC parpadeando, hay ícono nuevo y la barra de menús de Windows y los clics derechos están dibujados por la app | Hecho en la rama `snap-ui` y unido a `m1-editor`; falta la tanda de Carlos |
| 5 · Biblioteca | Vista Biblioteca, índice, bandeja reciente completa y persistente, copia editable al guardar en otro formato, arrastrar a otras apps | Hecha (2026-11-11); entra en la caminata de Carlos |
| 6 · Lo que falta de captura y app | Cursor, aviso de capturas en negro, sonido y destello, notificaciones con clic, restablecer ajustes, Avanzado, Funciones en prueba, Ayuda y Acerca de, bienvenida, ayuda en pantalla, guías de tamaño, cruceta simplificada, teclas del perfil de compatibilidad, eventos (`events watch`), tooltips y pistas | Hecha en Windows (2026-11-11). APP-025 es solo de macOS |
| 7 · Calidad | Teclado, lector de pantalla, contraste y movimiento, y rendimiento en todo M1 | En curso |

Plan aprobado por Carlos el 2026-10-24: la entrega 4 en tres partes (A: adelantar el `.xshot` de cada captura y una tira simple; B: un único motor de dibujo en Rust, ADR 0004; C: rama `m1-editor`).

macOS: el código compila en la CI, pero ese trabajo no bloquea los PR (`continue-on-error`). Las pruebas en Mac empiezan cuando Carlos lo diga. Hasta entonces, ningún ID pasa de `Win` a `Hecho` en `PARITY.md`.

## Cambios de la última sesión

- **Sesión del 2026-11-11 (noche), con la PC libre ("no te detengas hasta completar todas las fases"):**
  - Entrega 5 terminada: arrastrar de la bandeja o la biblioteca a otras apps (TRY-015, crate `drag`). Probado con el ratón real contra una ventana de prueba propia: un PNG va tal cual, un `.xshot` o un documento con cambios llega aplanado, tres seleccionados dan tres archivos, y soltar sobre XcreenShot no abre nada.
  - Entrega 6 (cada punto con su commit; decisiones provisionales en `spec/QUESTIONS.md`, sección "entrega 6"):
    - CAP-047 Capturar cursor: el puntero real llega como imagen flotante que se mueve o borra; al portapapeles va aplanado. La casilla del panel ya está activa.
    - CAP-024/APP-026: una captura toda negra (contenido protegido) avisa.
    - CAP-055 y APP-016: sonido al capturar (hecho por la app), destello que nunca sale en la captura, categorías de notificaciones en General, clic en el aviso abre la captura, y Permisos dice si Windows bloquea las notificaciones.
    - APP-010, APP-009, APP-034: Restablecer esta sección / todos los ajustes (`settings.reset`), Volver a mostrar avisos y pistas (`app.warnings`), carpeta de datos y comando de terminal en Avanzado, sección Funciones en prueba.
    - APP-015: ventana Ayuda con páginas propias en inglés y español (`F1`), Acerca de XcreenShot con licencias (`scripts/licenses.mjs`) y Enviar comentarios.
    - APP-005: bienvenida del primer arranque (solo en equipos donde XcreenShot no arrancó nunca).
    - CAP-020, CAP-019, CAP-014, CAP-066: panel de teclas (`H`/`F1`), guías de tamaño (`G`), cruceta simplificada y las teclas del perfil de compatibilidad.
    - APP-031: eventos y `xcreenshot events watch` (una línea JSON por evento).
    - APP-014: nivel de detalle de los tooltips y pistas de una sola vez.
  - Pregunta abierta para Carlos: a dónde lleva **Enviar comentarios** si sus colegas no ven el repositorio (QUESTIONS, APP-015).

- **Plan acordado con Carlos (2026-11-10):** Claude implementa seguido las entregas 5, 6 y 7, que cierran M1 en Windows. Después Carlos hace una **caminata por la app con un instalador**, Claude corrige lo que encuentre y solo entonces escribe **un único guion de pruebas** consolidado. Las tandas pendientes de antes (CAP-panel, 4c tanda 4, CAP-067, SMT tanda A, EDT-120, APP-012-snap, APP-028) entran en esa caminata. Luego M2 y siguientes con el mismo ciclo; las partes del spec sin escribir se proponen a Carlos antes de construirlas.
- **Limpieza aprobada por Carlos (2026-11-10):** PR #4 cerrado; carpeta `C:\Projects\XcreenShot-snap` y rama `snap-ui` borradas; las 16 capturas de prueba del 2026-10-18 movidas a `OneDrive\Documents\XcreenShot\pruebas`.
- **Entrega 5, biblioteca y bandeja reciente (2026-11-10 y 11, en pausa a pedido de Carlos):**
  - Crate `xs-library`:
    - índice SQLite (`rusqlite`) que sigue a la carpeta (escaneo más vigilante `notify-debouncer-mini`), con IDs en archivos ocultos `.xmeta` que sobreviven a renombrar desde el Explorador;
    - miniaturas en caché;
    - borrar a la Papelera de reciclaje con **Deshacer** (`trash`);
    - renombrar y mostrar en la carpeta.
  - Con la biblioteca en OneDrive, el índice vive en los datos locales.
  - Comandos nuevos:
    - `library.list`, `library.get`, `library.delete` (por ID o por ruta, también archivos de fuera), `library.restore`, `library.rename` y `library.show_in_folder`;
    - `document.locate`;
    - `document.close --document-ids` y `editor.show --view library`.
  - Editor:
    - autoguardado desactivable (las capturas esperan fuera de la biblioteca hasta guardarlas, LIB-020);
    - copia editable `.xshot` al guardar un PNG o JPG anotado (LIB-021);
    - vaciar la bandeja al salir (TRY-008);
    - renombrar desde la biblioteca un elemento abierto lo renombra también en el editor.
  - Interfaz:
    - conmutador **Editor | Biblioteca** arriba, con `Ctrl+Shift+1`/`2`;
    - vista Biblioteca: cuadrícula, selección con clic, `Ctrl`, `Shift`, rectángulo, `Ctrl+A` y flechas; menú contextual, `Supr`, `F2`, `Enter` y barra de estado;
    - bandeja reciente:
      - lo más nuevo primero, botón **Recientes** con punto si llegan capturas estando oculta, `Ctrl+Alt+R`, `Ctrl+RePág`/`AvPág`, flechas, `F6`;
      - menú con Cerrar, Cerrar los demás, Cerrar todos (con **Deshacer**), Renombrar, Mostrar en el Explorador, Copiar ruta y Mover a la papelera;
      - "No encontrado" con **Buscar…** y **Quitar**;
    - diálogos de eliminar y renombrar;
    - Preferencias > Biblioteca con la ubicación y **Abrir carpeta**.
  - Arreglo de paso: el diálogo común devolvía el foco a su primer botón cada vez que se redibujaba, así que al escribir en un campo el foco se escapaba tras la primera letra.
  - Probado en una instancia aislada con imágenes de prueba en una carpeta temporal: selección, teclado, menú, borrar y Deshacer contra la Papelera real, abrir, bandeja, "No encontrado", renombrar, cerrar los demás con Deshacer, español y Preferencias.
  - Decisiones provisionales: sección "entrega 5" de `spec/QUESTIONS.md`.

- **Instalador con un cliente MCP conectado (APP-028, 2026-11-09):** Claude Desktop mantiene `xcreenshot.exe mcp` en marcha y el instalador se paraba con "Error opening file for writing". Ahora aparta el archivo en uso y coloca el nuevo (`app/src-tauri/installer-hooks.nsh`), sin cortar la conexión. Prueba de Carlos en [`tests/manual/APP-028.md`](../tests/manual/APP-028.md), en su próxima actualización.

- **Aspecto nuevo (2026-11-05):** Carlos quería una interfaz menos aburrida, con animaciones en menús y botones. Claude Design había hecho el sistema "Snap" (PR #4), pero sobre `main`, sin el editor.
  - Se llevó a `m1-editor` en la rama `snap-ui`, preparada en una carpeta aparte (`C:\Projects\XcreenShot-snap`).
  - Los nombres de color viejos pasaron a los de Snap. `MenuButton` y el menú Más del panel pasaron al `Menu` animado.
  - Editor:
    - pestañas y herramientas con un resaltado que se desliza;
    - íconos con micro-animaciones y botón principal con brillo;
    - tira con miniaturas que se levantan y esquinas amarillas en la captura actual;
    - avisos y cuenta atrás animados, y una rejilla de puntos detrás del lienzo.
  - Logo de Carlos:
    - paleta del logo, con tokens con nombre de función (`--brand`, `--star`, `--magenta`, `--rec`);
    - el logo en el editor vacío, en versión clara u oscura, con el REC parpadeando;
    - ícono nuevo, el emblema simplificado (`scripts/make-icon.mjs`);
    - títulos en Outfit, la fuente gratuita más parecida a la del logo, empaquetada.
  - Los clics derechos del editor y, en Windows, la barra de menús ya no son nativos: los dibuja la app (`MenuBar.tsx`), con `Alt`, `F10`, letras de acceso, flechas y submenús. macOS conserva su barra del sistema.
  - Arreglo de paso: las teclas del editor se quedaban con flechas, `Enter` y `Escape` aunque se pulsaran dentro de un menú.
  - El PR #4 queda superado: su contenido llegó por `snap-ui`.

- **Reabrir los documentos al iniciar, EDT-120** (2026-11-01; Carlos: "sí me interesa", IDEA-003):
  - Spec: sección 12 de `02-editor.md`, la fila en la tabla de EDT-118 y dos decisiones provisionales en `QUESTIONS.md`.
  - `xs-editor` guarda la lista de la tira en el estado local (`editor.openDocuments`) cada vez que cambia, así que sobrevive a Salir, a un apagado y a un cierre inesperado. Al arrancar, la reabre en segundo plano, sin ventanas, y los comandos `document.*` y `editor.*` esperan a que termine (10 s como mucho).
  - Ajuste `editor.reopenDocuments` (por defecto sí).
  - Se omiten sin avisar los archivos que ya no existen y los que tienen copia de recuperación, que llegan por la recuperación.
  - Probado en una instancia aislada: tres documentos, la app arrancada otra vez por el CLI, y la tira igual que antes.
- **Corrección del CLI (APP-028):** en Windows, la app que arranca el CLI heredaba sus tuberías de salida. Un script, un agente o un cliente MCP que leía la salida del CLI esperaba hasta que la app se cerrara. Ahora el CLI no deja heredar sus manejadores estándar.
- **Tanda B de la IA (SMT-043):** PASS. La configuración de Claude Desktop de Carlos ya tenía cinco servidores, así que Claude Code añadió la entrada `xcreenshot`, con su permiso y con una copia de seguridad. Propuesta IDEA-004 (botón **Añadir a Claude Desktop**), pendiente de su visto bueno.

- **IA en la nube, SMT-039 a SMT-043** (2026-11-01; Carlos aprobó IDEA-002: cambiar "todo local" para esta función, las dos vías, cada colega con su clave):
  - Spec: sección 8 de `06-inteligentes.md`; SPEC §2.3 y §3.2, APP-054 (quinta conexión permitida) y la privacidad de `CLAUDE.md` actualizados.
  - Crate nuevo `xs-ai`:
    - la clave en el llavero (`CredWriteW`/`CredReadW` en Windows; Llavero de macOS con `security-framework`);
    - peticiones HTTPS a la API de Claude con `reqwest` (TLS y proxy del sistema), en streaming, con `fallbacks: "default"` en Opus y Sonnet;
    - comandos `ai.key_set` (solo la interfaz), `ai.key_clear`, `ai.key_status`, `ai.key_test`, `ai.open_console`, `ai.ask`, `ai.suggest_name` y `ai.cancel`.
  - Ajustes: `ai.enabled` (por defecto no; solo desde Preferencias), `ai.model` (Opus 5.5) y `ai.namingModel` (Haiku 5.5). Código de error nuevo `feature_disabled`.
  - Interfaz: sección **IA** en Preferencias, con el aviso al activar, la clave, **Probar la clave**, los modelos y la vía MCP. Menú **IA** del editor con **Preguntar a la IA…** (respuesta por partes, peticiones rápidas, seguimiento, **Detener**, **Insertar en la imagen**, tokens) y **Sugerir nombre…** (**Renombrar**, **Otra sugerencia**).
  - Editor: `document.rename` y `document.image`.
  - Vía B: `xcreenshot mcp` (APP-029 adelantado: herramientas; faltan recursos y eventos). Las imágenes van como imagen MCP. Ninguna herramienta `ai_*` por MCP. `app.mcp_config` y los botones **Copiar** en Preferencias > IA.
  - El instalador incluye ahora el CLI: `npm run package` en `app/` (`scripts/sidecar.mjs` y `tauri.bundle.json`).
  - El registro ahora guarda también las líneas de `xs_editor` y `xs_ai`. De la IA solo quedan el modelo, los tokens, el tiempo y el resultado.
  - Probado en la instancia aislada contra un servidor local que imita la API, con una clave inventada. Ver [`tests/manual/SMT-039-043.md`](../tests/manual/SMT-039-043.md).
  - Tropiezo: la espera de esa prueba lanzó un momento la app de depuración vieja sin carpeta aislada, porque el CLI arranca la app si no responde. Se cerró en seguida; no escribió en la configuración de Carlos. Para esperar a una instancia aislada, usar siempre `--no-launch`.

Sesión "Lee docs/ESTADO.md y continúa" (2026-10-24 y 25). El detalle está en `git log`.

- **Plan de la entrega 4** propuesto y aprobado (tres partes, decisiones A, B y C). ADR 0004.
- **Entrega 4a:**
  - Modelo de documento y formato `.xshot` con esquema publicado; guardado seguro; versiones.
  - Motor de dibujo en Rust (`xs-image`): flecha con 10 extremos y 4 tipos de línea, opacidad, PNG con resolución y sRGB, JPG.
  - Crate nuevo `xs-editor`: documentos abiertos, 200 pasos de deshacer, guardado automático de la biblioteca, preguntar antes de perder cambios (cerrar y salir), copias de recuperación cada 30 s.
  - Las capturas se guardan como `.xshot` en `Documentos\XcreenShot\AAAA-MM\` y se abren en el editor.
  - Ventana Editor (Konva): lienzo con zoom y desplazamiento, Mover y Flecha, panel Propiedades, barra de estado, tira de documentos, diálogos.
  - 20 comandos nuevos (`document.*`, `editor.*`); `app.quit` pregunta por cambios sin guardar.
- **Herramientas de prueba:** `scripts/dev/cdp.mjs` prueba la ventana de una instancia aislada sin mover el ratón real.
- **Fallos que encontró Carlos en sus pruebas, corregidos:**
  - el editor instalado no mostraba la captura (la política de seguridad bloqueaba las imágenes del protocolo `xsdoc`; ahora hay una prueba);
  - la captura temblaba con la ventana acoplada a media pantalla (barra de desplazamiento que aparecía y desaparecía por fracciones de píxel);
  - al alternar documentos el zoom se copiaba o se volvía a aplicar.
- **Lección:** antes de dar un instalador, probar con `npx tauri build --debug --no-bundle`, que empaqueta la interfaz como la app instalada (el servidor de desarrollo no aplica la política de seguridad).
- **Entrega 4b** (2026-10-25):
  - Texto con `cosmic-text` y fuentes del sistema: Rust lo dibuja también en pantalla; mientras se escribe, un campo encima con la misma medida.
  - Forma (rectángulo y elipse) y Desenfocar (desenfoque, pixelado y sólido), calculados en Rust sobre lo que hay debajo.
  - Tiradores para redimensionar y girar, con `Shift` y `Alt`.
  - Recortar sin perder lo recortado (se recupera volviendo a recortar) y aviso de objetos fuera del lienzo. La preferencia `editor.deleteCroppedContent` recorta de verdad al guardar.
  - Selector de color completo, con cuentagotas de pantalla (Windows) y `I` sobre el lienzo.
  - Cortar, copiar y pegar objetos: editables dentro de la app, PNG con transparencia para las demás; texto e imágenes de otras apps.
  - Teclas de herramienta, `Tab`, `Ctrl+Enter`, `[` y `]`. 2 comandos nuevos (`document.copy_objects`, `document.paste`).
  - Tras las pruebas de Carlos (2026-10-27): el selector de color se cierra con cualquier clic fuera (los cuadritos no lo cierran); el clic derecho ya no muestra el menú del motor web (salvo en campos de texto).
  - **Lienzo que crece (EDT-071, decisión de Carlos del 2026-10-27):** crece al sacar un objeto del borde, se encoge solo cuando vuelve (nunca por debajo de la imagen o del último recorte), y respeta lo que deja fuera un recorte. Desenfocar no lo amplía. Campos nuevos en el `.xshot`: `frame` y `outside`. Preferencia `editor.autoExpandCanvas`.
  - Decisiones provisionales nuevas en `spec/QUESTIONS.md` (tamaño del texto en puntos, intensidad del desenfoque, cuentagotas en macOS, huella del portapapeles, pegar imágenes con un documento abierto, tamaño de `Ctrl+Enter`).

- **Multicaptura, CAP-067** (2026-11-01, pedida y decidida por Carlos; ver `spec/IDEAS.md` IDEA-001):
  - `xs-capture/src/multi.rs`: los gestos (clic, arrastre, doble clic) sin llamadas al sistema, con pruebas; la sesión, sus capturas y los comandos `capture.multi_start`, `multi_stop`, `multi_state`, `multi_remove`, `multi_clear`.
  - `xs-capture/src/multi_win.rs` (Windows): gancho de ratón y teclado de bajo nivel en un hilo propio. La pulsación se retiene, se captura la ventana y se devuelve el clic con `SendInput` (marcado, para no volver a capturarlo). Cuatro ventanas finas hacen el borde resaltado (`WDA_EXCLUDEFROMCAPTURE`). Las ventanas de apps elevadas no se retienen (Windows no deja devolverles el clic).
  - Cada captura se guarda en la biblioteca sin abrirse (`xs_editor::save_capture`). Mientras dura, las demás capturas responden `busy`.
  - Panel `multicapture` (React, `windows/MultiCapture.tsx`): miniaturas por `xsdoc://…/multi/<id>`, elegir, X, Limpiar, Insertar, Terminar, Capturar más y Cerrar. Nunca sale en las capturas (`content_protected`). Entrada **Multicaptura** en el menú del ícono y atajo asignable, sin atajo por defecto.
  - **Imágenes flotantes (EDT-078, lo básico, adelantado):** objeto `image` (`image`, `rect`, `rotation`, `style.opacity`) con su imagen en `images/` del `.xshot`; dibujado en Rust (`render.rs`) y en Konva. Se mueve, redimensiona (proporción desde la esquina; `Shift` la libera) y gira como una forma, en píxeles enteros. Comando `document.insert_images`. Pegar una imagen de otra app con un documento abierto ya la inserta como imagen flotante.
  - Probado en la instancia aislada con una ventana de prueba propia y clics reales: ver [`tests/manual/CAP-067.md`](../tests/manual/CAP-067.md).
  - `scripts/dev/cdp.mjs` acepta `XS_CDP_WINDOW=<etiqueta>` para manejar otras ventanas (el panel, Preferencias).
- **Entrega 4c** (2026-10-28):
  - Barra de menús nativa del editor en Windows (Archivo, Edición, Ver, Ayuda), con el atajo de cada entrada y las que no aplican en gris; menú del clic derecho en el lienzo y en la barra (EDT-002).
  - Atajos del editor asignables: ámbito nuevo **Editor** en Preferencias > Atajos, que en el editor gana a los de la app (EDT-115, EDT-116). Comando nuevo `editor.action` (lo usan menús y atajos) y `editor.menu_state` (solo interfaz).
  - Exportar pasa a `Ctrl+Alt+Shift+E` en Windows: la del spec choca con Capturar imagen. Una prueba vigila que ningún atajo por defecto se repita.
  - Preferencias > Editor con siete ajustes (EDT-118), `Esc` cierra el editor (EDT-012), Solo íconos (EDT-003).
  - Accesibilidad (EDT-013): `F6` entre zonas, anuncios para el lector de pantalla, `Shift`+flechas en los deslizadores.
  - Botones de Deshacer y Rehacer: se quedan (Carlos, 2026-10-28).
  - Tras las pruebas de Carlos (2026-10-28, las tres tandas PASS): un clic de ratón en la barra de herramientas ya no deja el foco en el botón. Al pulsar después una tecla, su marco de foco hacía parecer que la herramienta anterior seguía elegida. Con `Enter` o `Espacio` el foco se queda, para quien usa el teclado.
  - `PARITY.md`: EDT-002, EDT-012, EDT-013, EDT-115, EDT-116, EDT-117 y EDT-118 pasan a `Win`. EDT-003 sigue en curso (el menú Más llega con las herramientas de M2).

## Pendiente inmediato

1. **Pruebas de Carlos con el instalador `c97ce05`** (entrega 3, [`tests/manual/CAP-panel.md`](../tests/manual/CAP-panel.md)):
   - grabar otra vez `Ctrl+Shift+PrtScn` en el campo Atajo del panel;
   - cambiar de pestaña con las flechas sin que asome el turquesa en la pestaña vecina;
   - **Repetir última área** con un atajo asignado en Preferencias > Atajos.
   - Otras casillas sin marcar: Preferencias de captura desde el menú ⋯, Universal con Retardo de 5 s, y Captura universal desde el menú del ícono.
   - El instalador de la 4a (`9ddb1ba`) sustituye al de la entrega 3: las pruebas que faltan se pueden hacer con él.
2. Hecho (2026-11-10): las 16 capturas de prueba del 2026-10-18 están en `OneDrive\Documents\XcreenShot\pruebas`.
3. **Entrega 4c:** Carlos marcó PASS las tres tandas de [`tests/manual/EDT-editor-4c.md`](../tests/manual/EDT-editor-4c.md) (2026-10-28, instalador `66a0f62`). Falta la tanda 4, corta, con el arreglo del marco de foco (instalador posterior a `66a0f62`). Carlos no ha objetado `Ctrl+Alt+Shift+E` para Exportar (la probó en la tanda 2); sigue provisional en `spec/QUESTIONS.md`.
4. **Entrega 4b:** terminada. Carlos marcó PASS las cinco tandas de [`tests/manual/EDT-editor-4b.md`](../tests/manual/EDT-editor-4b.md) (2026-10-28, instalador `5a68118`).
5. **Multicaptura (CAP-067):** hecha; Carlos tiene dos tandas en [`tests/manual/CAP-067.md`](../tests/manual/CAP-067.md). Las decisiones provisionales (arrastres que no llegan a la app, `Esc` deja el panel, dónde se insertan, sin portapapeles ni aviso) están en `spec/QUESTIONS.md`.
6. **IA en la nube (SMT-039 a SMT-043):** hecha. Carlos tiene dos tandas en [`tests/manual/SMT-039-043.md`](../tests/manual/SMT-039-043.md): la A necesita su propia clave de la API (platform.claude.com) y la B, Claude Desktop o Claude Code. Las decisiones provisionales están en `spec/QUESTIONS.md` (filas SMT-039 a SMT-043, APP-028 y APP-029). Tanda B: PASS (2026-11-01). Su configuración de Claude Desktop ya tiene la entrada `xcreenshot` (copia de seguridad en `%APPDATA%\Claude\claude_desktop_config.backup-2026-11-01.json`). Propuesta pendiente de su visto bueno: IDEA-004, el botón **Añadir a Claude Desktop**, porque el bloque copiado no sirve si ya hay otros servidores.
7. **Reabrir los documentos al iniciar (EDT-120):** hecha; Carlos tiene una tanda en [`tests/manual/EDT-120.md`](../tests/manual/EDT-120.md). Necesita el instalador nuevo.
8. **Aspecto nuevo (Snap con el logo):** hecho; Carlos tiene una tanda en [`tests/manual/APP-012-snap.md`](../tests/manual/APP-012-snap.md). El PR #4 está cerrado y la carpeta `XcreenShot-snap` borrada (2026-11-10).
9. **Propuesta, pendiente de aprobar:** cuando Carlos confirme la entrega 3, fusionar el PR #2 (antes, cambiarle el título, que dice "capture engine"). Después, el [PR #3](https://github.com/cracvision/XcreenShot/pull/3) (entregas 4a, 4b y 4c) pasa a apuntar a `main` y se puede fusionar cuando Carlos lo apruebe (la 4c ya está probada).

## Siguiente paso

**Carlos pidió una pausa el 2026-11-11:** trabaja en otro proyecto y las ventanas de la instancia de prueba lo interrumpían. Al volver, **no arrancar instancias de prueba sin avisarle antes**: sus ventanas salen en su escritorio aunque estén aisladas.

1. **Terminar la entrega 5 con TRY-015 (arrastrar a otras apps):**
   - Plan: crate `drag` 2.1.1 (de CrabNebula, el que usa el plugin de Tauri) en `app/src-tauri`. Añade `windows` 0.52 en Windows: anotarlo en `QUESTIONS.md`.
   - Comando de Tauri `xs_drag_out { docs, items }`. Prepara los archivos con los comandos de siempre:
     - un PNG o JPG sin cambios se entrega tal cual;
     - un `.xshot` o un documento con cambios se exporta como PNG aplanado a `local_dir/drag/` (se limpia al empezar cada arrastre).
   - Después llama a `drag::start_drag` con `run_on_main_thread`, con una miniatura de unos 160 px como imagen del arrastre.
   - En la interfaz: empezar al mover más de 4 px con el botón pulsado sobre un elemento de la bandeja o de la biblioteca (la selección entera si el elemento está seleccionado).
2. Marcar en `spec/PARITY.md` LIB-001, LIB-002, LIB-015, LIB-020, LIB-021, LIB-022, LIB-027, TRY-001, TRY-002, TRY-008, TRY-009, TRY-011 y TRY-013 (y TRY-006 y EDT-109, adelantados).
3. **Entrega 6:** repasar en `PARITY.md` los IDs de captura y app de M1 que siguen pendientes y hacerlos.
4. **Entrega 7:** calidad (teclado, lector de pantalla, rendimiento).
5. Instalador (probar antes con `npx tauri build --debug --no-bundle`), caminata de Carlos, arreglos y un único guion de pruebas.

## Decisiones vigentes que conviene recordar

El detalle y el motivo de cada una están en `spec/QUESTIONS.md`.

- **Windows primero.** macOS compila pero no bloquea.
- **Versión:** un número menor por hito (0.1 = M0, 0.2 = M1) más el commit de la compilación.
- **Atajo de prueba:** Carlos cambió el de `app.ping` a `Ctrl+P` a propósito. No tocarlo.
- **Barra de captura:** el botón se llama **Capturar** (`Enter`; `I` también funciona). El selector rápido **EN | ES** está en la ventana principal y en Preferencias.
- **Dónde se guardan las capturas:** como `.xshot` en `Documentos\XcreenShot\AAAA-MM\`, con el nombre `<App> AAAA-MM-DD HH.MM.SS` (provisional hasta SHR-006). En el equipo de Carlos esa carpeta está en OneDrive. Las PNG de antes se quedan donde estaban.
- **Editor (ADR 0004):** el documento vive en Rust; la interfaz es una vista que manda comandos. Cada gesto es un solo comando al soltar. La imagen final la dibuja solo Rust; la geometría está duplicada en `geometry.ts` y `boxes.ts` y la vigilan `tests/fixtures/arrow-geometry.json` y `shape-geometry.json`. El texto y el desenfoque los dibuja Rust también en pantalla (protocolo `xsdoc`).
- **Atajos del editor:** ámbito Editor en Preferencias > Atajos; en el editor ganan a los de la app (`Ctrl+W` cierra el documento, no la ventana). Las teclas del lienzo (`Ctrl+1…9`, flechas, `Tab`, `Enter`, `Esc`, `I`, `[`, `]`) son fijas. Los menús de la ventana muestran el atajo como texto: no lo registran.
- **Valores por defecto según quién lanza la captura:**
  - Desde atajos y menús, las acciones de un solo modo (ventana, ventana activa, región, pantalla completa) toman de la pestaña Imagen lo que no es el modo.
  - Desde el CLI y MCP usan valores fijos y no abren ventanas salvo `--open editor`.
- **Ocultar al capturar:** el panel se aparta durante la captura solo si "Ocultar XcreenShot al capturar" está activado. Las ventanas de XcreenShot se excluyen de la imagen solo mientras dura la captura.
- **`Esc` en la cuenta atrás:** queda reservado para XcreenShot mientras corre el contador.

## Cómo trabajamos con Carlos

- **Idioma:** siempre en español, también las instrucciones de prueba. El código, los comentarios y los commits van en inglés.
- **Perfil:** Carlos es consultor de tecnología, no programador. Las pruebas manuales van guiadas paso a paso, numeradas y en tandas cortas. Él responde PASS o fallo, con capturas.
- **Instalador para sus pruebas:** `npm run tauri build` en `app/`. El archivo queda en `target\release\bundle\nsis\XcreenShot_0.2.0_x64-setup.exe` y lo ejecuta él.
- **Antes de `npx tauri dev`:** cerrar la app instalada con `target/debug/xcreenshot.exe app quit --no-launch`, porque la instancia única ocupa el canal.
- **Probar sin tocar su app:** usar una instancia aislada de **depuración** con `XCREENSHOT_IPC_NAME`, `XCREENSHOT_DATA_DIR` y `XCREENSHOT_LIBRARY_DIR`.
  - Nunca la versión de release aislada: al primer arranque se registra para iniciar con Windows y pisa su entrada.
  - Sus atajos los tiene la app instalada, así que la copia aislada avisa de que no pudo activarlos.
  - Para probar la ventana sin mover su ratón: arrancarla además con `WEBVIEW2_USER_DATA_FOLDER` propio y la depuración remota, y usar `scripts/dev/cdp.mjs` (instrucciones en `scripts/dev/README.md`).
  - No usar Copiar todo, copiar o cortar objetos en las pruebas automáticas: pisan el portapapeles de Carlos.
- **Herramientas:**
  - `gh` está en `C:\Program Files\GitHub CLI\gh.exe` (no está en el PATH de Git Bash).
  - Rust está en `~/.cargo/bin` (en Git Bash: `export PATH="$HOME/.cargo/bin:$PATH"`). Mínimo 1.88.
  - Los scripts largos de Node, Python o PowerShell van en archivos, para evitar problemas de comillas. Al reescribir archivos con Python en Windows, usar `newline=''` para no cambiar los finales de línea.
- **Comandos que debe ejecutar Carlos:** decirle en qué ventana (PowerShell o cmd) y darle una sola línea.
