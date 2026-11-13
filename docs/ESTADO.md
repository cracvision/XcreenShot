# Estado del proyecto

Este archivo es el traspaso entre sesiones de Claude Code. Cada sesión nueva lo lee al empezar (lo pide `CLAUDE.md`). Se actualiza al cerrar una entrega o un bloque de trabajo, y siempre antes de pasar a una sesión nueva. Es un resumen: el detalle está en las fuentes de abajo.

**Última actualización:** 2026-11-13 (madrugada), con el aspecto **Cristal**, con M1 y **M2 terminados en Windows** (salvo EDT-032, EDT-033 y EDT-034, que esperan a Carlos), **M3 terminado en Windows** salvo HDR (CAP-022), la cámara (CAP-038) y unos flecos, y **M4 empezado**: se puede grabar video (entrega M4.1 de [`docs/plan-M4.md`](plan-M4.md)). **Instalador nuevo: `00a0b45`** (2026-11-13, 4:19). Siguiente: la caminata de Carlos con ese instalador (ver "Siguiente paso") y que apruebe o cambie el plan de M4.

**Decisión de Carlos (2026-11-12, tarde):** los instaladores intermedios son para **caminatas informales**; las pruebas formales (los guiones de `tests/manual/`) se hacen **cuando esté todo implementado**.

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
| Aspecto nuevo: Cristal (APP-012, EDT-001, EDT-005, EDT-011, TRY-002) | Pedido por Carlos el 2026-11-12 (maqueta aprobada ese día): sustituye a Snap. Neutros fríos, un solo acento sólido, esquinas de 8 a 14 px, sin degradados. En el editor, barra, panel lateral, barra de estado y bandeja flotan como cristal sobre el lienzo y la captura pasa por debajo. Preferencias > General: Intensidad del cristal y Color de acento | Hecho; falta la caminata de Carlos ([`tests/manual/APP-012-cristal.md`](../tests/manual/APP-012-cristal.md)) |
| 5 · Biblioteca | Vista Biblioteca, índice, bandeja reciente completa y persistente, copia editable al guardar en otro formato, arrastrar a otras apps | Hecha (2026-11-11); entra en la caminata de Carlos |
| 6 · Lo que falta de captura y app | Cursor, aviso de capturas en negro, sonido y destello, notificaciones con clic, restablecer ajustes, Avanzado, Funciones en prueba, Ayuda y Acerca de, bienvenida, ayuda en pantalla, guías de tamaño, cruceta simplificada, teclas del perfil de compatibilidad, eventos (`events watch`), tooltips y pistas | Hecha en Windows (2026-11-11). APP-025 es solo de macOS |
| 7 · Calidad | Teclado, lector de pantalla, contraste y movimiento, y rendimiento en todo M1 | Hecha (2026-11-11). Medidas y lo que no cumple en [`tests/manual/APP-rendimiento-M1.md`](../tests/manual/APP-rendimiento-M1.md) |

Plan aprobado por Carlos el 2026-10-24: la entrega 4 en tres partes (A: adelantar el `.xshot` de cada captura y una tira simple; B: un único motor de dibujo en Rust, ADR 0004; C: rama `m1-editor`).

macOS: el código compila en la CI, pero ese trabajo no bloquea los PR (`continue-on-error`). Las pruebas en Mac empiezan cuando Carlos lo diga. Hasta entonces, ningún ID pasa de `Win` a `Hecho` en `PARITY.md`.

## Cambios de la última sesión

- **M4 empezado: grabar video (2026-11-13, madrugada, después del instalador `4627c71`).** Hecho en Windows, probado en la instancia de desarrollo aislada y subido; entra en el instalador `00a0b45`. Plan propuesto en [`docs/plan-M4.md`](plan-M4.md) (provisional hasta que Carlos lo apruebe); esta es su entrega M4.1. Guía para Carlos: [`tests/manual/VID-M4-1.md`](../tests/manual/VID-M4-1.md).
  - **Motor de grabación** (crate `xs-video`): duplicación del escritorio de Windows (solo llegan imágenes cuando la pantalla cambia: frecuencia variable), recorte, escalado y conversión en la GPU, y MP4 H.264 con los codificadores del sistema (por hardware, o por software si no hay). Sin FFmpeg. La primera imagen se toma con GDI, porque con la pantalla quieta (o el monitor en reposo, como esta noche) la duplicación no entrega nada y el video salía negro. Colores comprobados (±2 niveles) con los dos codificadores.
  - **Captura de video** (menú del ícono y `Ctrl+Alt+Mayús+V`): se elige el área como en una imagen y aparece la **barra de grabación** con el tamaño del video y **Grabar**; cuenta atrás de 3 s; tiempo, **Pausar**/**Reanudar** (`Ctrl+Mayús+F9`), **Detener** (`Ctrl+Mayús+F10`) y **×** (pregunta antes de cancelar). Un borde marca el área (azul, rojo al grabar, discontinuo en pausa). Barra y borde no salen en el video. Mientras graba, el ícono muestra su reloj y **Detener grabación**.
  - **Puntero en el video** con su forma real (interruptor **Puntero** en la barra) y **audio del sistema** (interruptor **Sonido**, desactivado por defecto; AAC en el mismo MP4, con silencio donde no suena nada para mantener la sincronía).
  - **Preferencias > Video:** Calidad (Muy baja 5 fps a Muy alta 60 fps; Alta por defecto), Tamaño máximo del video (Reducir pantallas de alta densidad por defecto), Capturar cursor, Grabar el audio del sistema, Cuenta atrás y Usar siempre codificación por software.
  - El video va a la **biblioteca** (carpeta del mes, "Video AAAA-MM-DD HH.MM.SS.mp4"), que ahora lista los MP4 con la miniatura de su primer fotograma; abrirlos usa el reproductor de Windows hasta que el editor los reproduzca (VID-056).
  - Mientras graba, Windows no se suspende ni apaga la pantalla; con menos de 1 GB libre se detiene y guarda sola.
  - **Compilación empaquetada probada** (instancia aislada): grabar una región por comandos (la barra aparece con "600 × 400", Puntero, Pausar y Detener; MP4 de 600 × 400 con el codificador por hardware) y la biblioteca lo lista como video; Captura de video con selección en pantalla → *Lista* → cuenta atrás → grabando → detener con Captura de video; `app quit --force` la cierra.
  - **Pestaña Video del panel de captura** (VID-001): Selección (Región, Ventana o Pantalla completa del monitor donde está el ratón), Capturar cursor, Grabar el audio del sistema y Temporizador (1 a 60 min). **Capturar** con esa pestaña graba.
  - **Grabar una ventana** (VID-005, modo Área de la ventana): el video la sigue cuando se mueve o cambia de tamaño; si se minimiza conserva el último fotograma; si se cierra, la grabación se detiene y se guarda sola. "Solo la ventana" (aunque otras la tapen) queda para más adelante.
  - **Barra de grabación, más:** **Reiniciar** (↺) vuelve a *Lista* con la misma área; **Imagen** (o `I`) toma una captura del área en lugar de grabar; Cancelar, Reiniciar e Imagen solo preguntan si hay más de 10 s grabados; `Esc` en la barra cancela en *Lista*. En *Lista* la barra resume los fps y el códec y cuánto cabe en el disco (en ámbar si caben menos de 10 min). Si se arrastra la barra, la siguiente grabación la pone donde se dejó; con pantalla completa y otro monitor, va al otro (sin probar: esta noche solo había uno).
  - **Arreglo importante:** el complemento que recuerda el tamaño y el sitio de las ventanas también "recordaba" la barra y el borde de la grabación, y desde la segunda grabación los ponía con un tamaño viejo (la barra quedaba como una tira). Ya no los toca (ni a otras ventanas que coloca la app). Estaba en el instalador `eb477bf`; el `2edc209` lo corrige.
  - **Presets de video** (VID-009): desde la pestaña Video, + > Desde los ajustes actuales crea un preset "Video: <selección>" (ícono "V") que guarda la selección y el temporizador; ejecutarlo (lista, su atajo, menú del ícono) deja la barra lista para grabar.
  - **XcreenShot fuera del video** (VID-017): con "Ocultar XcreenShot al capturar", el panel y el editor no salen en el video aunque sigan en pantalla (comprobado grabando el área del panel).
  - **Comandos** para el CLI y el MCP: `record.start`, `record.stop`, `record.pause`, `record.resume`, `record.cancel`, `record.status`, `record.video`, `record.toggle`, `record.cursor` y `video.info` (tamaño, duración, fps medio y máximo, códec, tasa de bits y sonido de un MP4).
  - De paso, en las pruebas: el puntero del ratón se movió un momento (volvió a su sitio) y aparecieron unos segundos ventanas de prueba de colores (se cerraron solas). Este equipo no tenía ninguna salida de sonido activa esta noche, así que el audio del sistema se comprobó con un tono sintético.
  - **No hechos:** micrófono y cámara (encenderían los dispositivos de Carlos sin su permiso), la pestaña Video del panel, la pantalla sin congelar al elegir el área, el reproductor y la edición de video, GIF y diapositivas: son las siguientes entregas del plan de M4.
- **M3, flecos (2026-11-13, madrugada, después del instalador `68c42b4`, mientras Carlos dormía).** Hechos en Windows, probados en la instancia de desarrollo aislada y subidos; entran en el instalador `4627c71`.
  - **Carpeta de recepción** (CAP-043): Preferencias > Captura > Carpeta de recepción; cada imagen o PDF nuevo que llega se importa una vez y pasa a la subcarpeta "Importados" (salvo con "Dejar los archivos donde están"). `tests/manual/CAP-043.md`.
  - **Anotación rápida** (CAP-057): las marcas se eligen, se mueven y se borran, y usa los colores del tema activo.
  - **Extraer texto** (SMT-005, SMT-006): indicador "Trabajando" junto a la zona; la ventana de resultados recuerda su tamaño y su sitio.
  - **Desplazamiento** (CAP-036): "Detectar partes fijas" se puede apagar.
  - **Página web** (CAP-037) con Intervalo o Programada: usa la última dirección.
  - **Presets** (CAP-058 a CAP-063): marca de "en marcha" en los de intervalo o programada; elegir varios con `Ctrl`/`Mayús` + clic para exportarlos o borrarlos juntos; soltar un `.xpreset` sobre el panel; aviso en la fila cuando su carpeta o su archivo no existen en este equipo.
  - **Destino Archivo** (CAP-052, CAP-053): Pedir nombre y carpeta, Preguntar si ya existe, y los detalles de la captura dentro del archivo, en PNG y ahora también en JPG (EXIF: título, app, dirección y fecha de captura, que el Explorador muestra). Con Abrir en = Editor la captura **no se envía al momento**: el editor muestra "Pendiente de enviar a: Archivo" con **Enviar** (`Ctrl+Enter`) y **No enviar**, y envía la imagen tal como esté entonces.
  - **Múltiples áreas** (CAP-033): con el perfil de compatibilidad, el clic derecho termina.
  - **Archivo como fuente** (CAP-040): la fecha en que se tomó una foto va en sus detalles, y ahora también **SVG**, dibujados por el motor web del sistema, al doble si son pequeños y con su transparencia.
  - **Efectos al capturar** (CAP-051): el engranaje ⚙ de cada efecto abre sus opciones en el panel, se reordenan arrastrando, y Detalles de captura tiene **Pedir texto al capturar** (al terminar la captura aparece un cuadro; `Esc` deja el texto vacío).
  - **Abrir archivos con XcreenShot** (APP-001): doble clic en un `.xshot` (al editor), `.xpreset` (al panel, con el resumen de la importación), `.xtheme` o `.xtemplate` (a su importación en el editor). Van a la instancia que ya está en marcha. El instalador registra solo esos tipos propios, nunca PNG ni JPG. Comando `app.open_files`.
  - **Portapapeles como fuente** (CAP-039): los detalles dicen qué app copió la imagen, si sigue abierta.
  - **Ventanas como objetos** (CAP-029): la base del documento es el escritorio sin las ventanas (el fondo de pantalla donde estaban, con la barra de tareas), así que al quitar una ventana se ve el escritorio.
  - **Selector de ventanas** (CAP-030): probado con una ventana minimizada; se captura entera y vuelve a minimizarse.
  - **Compilación empaquetada probada** (instancia aislada): un SVG entra a 480 × 240 con transparencia; una pantalla completa con ventanas como objetos (7 objetos), Detalles de captura con Pedir texto (el cuadro aparece, lo escrito queda en el efecto) y destino Archivo en JPG con EXIF; abrir la app con un `.xpreset` cuando ya está en marcha muestra el resumen de la importación en el panel; `app quit --force` la cierra. El instalador registra los tipos con identificadores `XcreenShot.Capture`, `.Presets`, `.Theme` y `.Template`.
  - Arreglo: las capturas de apps de la Tienda (Bloc de notas, Calculadora…) se llamaban "CProgram FilesWindowsApps…"; ahora llevan el nombre de la app.
  - De paso, en las pruebas: el portapapeles de Carlos quedó con una imagen de prueba (un rectángulo de color) puesta desde PowerShell.
  - Decisiones provisionales nuevas en `spec/QUESTIONS.md` (secciones "M3, entrega 8" a "19").
  - **No hechos:** HDR (CAP-022, necesita un monitor en modo HDR), cámara web (CAP-038, encendería la cámara de Carlos sin su permiso), iPhone/iPad (CAP-044, macOS), soltar una dirección arrastrada desde el navegador (CAP-037: el sistema de arrastre de Tauri solo entrega archivos), el texto de las páginas de un PDF (CAP-041: el motor PDF de Windows no lo da) y agrupar una serie en la biblioteca (CAP-049: falta su ID `LIB-`).
- **M3, más modos y tiempos (2026-11-12, noche, mientras Carlos dormía).** Hechos en Windows, probados en la instancia de desarrollo aislada y subidos; todavía no están en ningún instalador.
  - **Selector de ventanas** (CAP-030, `Ctrl+Alt+Mayús+M`): todas las ventanas del escritorio, también tapadas y minimizadas, con miniatura, filtro y teclado. `tests/manual/CAP-030.md`.
  - **Destino Archivo** (CAP-052, CAP-053): Compartir > Guardar en un archivo, con carpeta, patrón de nombre (`{prefix}_{date}_{time}`, contador…), formato y qué hacer si existe. `tests/manual/CAP-052-053.md`.
  - **Efectos al capturar** (CAP-051): borde, sombra, escala, etc. desde el panel; quedan editables en el documento. `tests/manual/CAP-051.md`.
  - **Portapapeles como fuente** (CAP-039): imagen, archivos de imagen o texto (un objeto Texto editable). `tests/manual/CAP-039.md`.
  - **Anotación rápida** (CAP-057), primera **función en prueba** (APP-034): se activa en Preferencias > Funciones en prueba; `Ctrl+Alt+Mayús+Q`, arrastrar un área, marcarla (flecha, forma, texto, resaltar, desenfocar, paso) y `Enter` para copiar sin abrir el editor. `tests/manual/CAP-057.md`.
  - **Intervalo** (CAP-049): Tiempo = Intervalo, cada 3 s a 24 h, con parada manual, por número, por tiempo o a una hora y descarte de capturas iguales; reloj en el ícono y **Detener captura** en su menú. `tests/manual/CAP-049.md`.
  - **Programada** (CAP-050): Tiempo = Programada, con fecha, hora y repetición; sobrevive a reinicios y hace tarde (hasta 5 min) la que se perdió. `tests/manual/CAP-050.md`.
  - **Capturas del sistema** (CAP-042): Preferencias > Captura > Abrir en XcreenShot las capturas del sistema; lo de Recortes (`Win+Mayús+S`) llega una sola vez, del portapapeles o de la carpeta Capturas de pantalla. `tests/manual/CAP-042.md`.
  - **Archivo como fuente e Importar PDF** (CAP-040, CAP-041): Selección = Archivo (varios a la vez, o un Archivo fijo) y Selección = PDF con la ventana Importar PDF (miniaturas, rango, resolución, una por página o todas en una, contraseña). Los PDF los dibuja Windows. `tests/manual/CAP-040-041.md`.
  - **Lanzador de borde** (CAP-004): pestaña fina en un borde con Capturar y Presets; se activa en el menú ⋯ del panel o en Preferencias > Captura > Interfaz de captura. `tests/manual/CAP-004.md`.
  - **Página web por URL** (CAP-037): Selección = Página web; carga la página en privado y la captura entera. `tests/manual/CAP-037.md`.
  - **Mano alzada** (CAP-032): libre, polígono o elipse, con transparencia fuera. `tests/manual/CAP-032.md`.
  - **Ventanas como objetos** (CAP-029): pantalla completa con cada ventana como objeto de imagen. `tests/manual/CAP-029.md`.
  - **Soltar archivos** sobre el panel o el lanzador: imágenes, PDF, `.xshot` y accesos directos `.url`.
  - **No hechos esta noche:** HDR (CAP-022), porque comprobarlo necesita un monitor en modo HDR, y la cámara web (CAP-038), porque probarla encendería la cámara de Carlos sin su permiso.
  - De paso, en las pruebas: el portapapeles de Carlos quedó con una dirección de prueba (`http://127.0.0.1:8765/larga.html`), y el puntero del ratón se movió un momento (se devolvió a su sitio).
  - Decisiones provisionales nuevas en `spec/QUESTIONS.md` (secciones "M3, entrega 5", "6" y "7").
  - De paso: durante las pruebas, una anotación rápida dejó una imagen de prueba en el portapapeles de Carlos (no se pudo restaurar lo que había). Y las capturas de prueba de pantalla completa llevan en el nombre el de la app que estaba al frente.
- **M3.3 y parte de M3.4 (2026-11-12, noche, después del instalador Cristal).** Hechos en Windows y probados en la instancia de desarrollo; todavía no están en ningún instalador.
  - **Captura con desplazamiento** (CAP-034 a CAP-036): manual (botón **Desplazamiento** `S` de la barra de captura) con panel de vista previa al lado del área, **Listo** `Enter` y **Cancelar** `Esc` sin quitar el foco a la app; automática (flechas ↓ → ↘ y teclas `D` `R` `X`), que gira la rueda sola hasta el final y deja el contenido donde estaba; unión con cabeceras y pies fijos una sola vez; tope de 60 000 px; Selección = Desplazamiento con sus propiedades; preset incorporado **Desplazamiento**; **Terminar desplazamiento** en el menú del ícono; `capture.scrolling`, `capture.scroll_action`, `capture.scroll_state`. Probado con una página web de 60 secciones (cabecera y pie fijos) y con el Bloc de notas. Prueba: [`tests/manual/CAP-034-036.md`](../tests/manual/CAP-034-036.md).
  - **Región fija y posición** (CAP-026), **Múltiples áreas** (CAP-033) y **Captura de menú** (CAP-031). Pruebas: `tests/manual/CAP-026.md`, `CAP-033.md`, `CAP-031.md`.
  - **Pregunta nueva para Carlos:** el atajo de Captura de menú que pide el spec (`Ctrl+Alt+Shift+E`) es el de Exportar en el editor. Por ahora queda sin atajo (ver QUESTIONS).
  - De paso: durante las pruebas, Edge abrió con la cuenta de Windows de Carlos en un perfil de prueba aislado y mostró un aviso de sincronización; cerré esa ventana y sus procesos enseguida. Puede quedar en su historial de Edge una visita a "Pagina larga de prueba" (un archivo local). Y abrí `largo.txt` (un texto de prueba) como pestaña en su Bloc de notas.
- **Aspecto nuevo: Cristal (2026-11-12, noche).** Carlos pidió dejar el aspecto violeta con degradados y botones de píldora por algo limpio, con cristal de intensidad ajustable; aprobó la maqueta y pidió llevarla a la app con instalador.
  - Paleta nueva en `app/src/styles.css` (claro y oscuro), botón principal sólido, esquinas de 8 a 14 px, menús y avisos de cristal, movimiento más sobrio.
  - El editor ahora tiene un área de trabajo (`editor/Workspace.tsx`): el lienzo la ocupa entera y la barra, el panel lateral, la barra de estado y la bandeja flotan encima. El lienzo recibe cuánto tapan (`editor/insets.ts`), encaja y centra la imagen en la parte visible y dibuja sus propias barras de desplazamiento finas (`editor/CanvasBars.tsx`). Comprobado que el puntero, el zoom y el dibujo caen en el píxel correcto.
  - Preferencias > General: **Intensidad del cristal** (`general.glassIntensity`, 0–100, por defecto 60) y **Color de acento** (`general.accent`: azul, violeta, turquesa, grafito).
  - Arreglado de paso: en el instalador `2f8ca52` la barra del editor recortaba sus menús (Más, Compartir, ⋯).
  - Propuesta pendiente de Carlos: la ventana translúcida (Mica). Tauri 2 ya la trae; ver `spec/QUESTIONS.md`, sección "Aspecto nuevo: Cristal".
  - Reglas de diseño actualizadas en `CLAUDE.md` (sección "Diseño: Cristal").
- **Caminata de Carlos (2026-11-12, tarde), con el instalador `7f50273`.** Lo único que encontró: con tantos íconos, al estrechar la ventana se montaban unos sobre otros.
  - Arreglado (`2f8ca52`): la barra del editor y la de la Biblioteca miden su ancho y se compactan por pasos, sin montar ni cortar nada, y todo vuelve al agrandar la ventana. Detalle del orden en `spec/QUESTIONS.md` (sección "Arreglos tras la caminata del 2026-11-12") y prueba en [`tests/manual/EDT-001-barra-estrecha.md`](../tests/manual/EDT-001-barra-estrecha.md).
  - Para probar la app de desarrollo con la app instalada de Carlos abierta, la instancia aislada necesita también su propia carpeta de WebView2 (`WEBVIEW2_USER_DATA_FOLDER`); sin ella, WebView2 no arranca (`0x8007139F`).
- **Sesión del 2026-11-12 (mañana y tarde): M3 empezado.** Plan en [`docs/plan-M3.md`](plan-M3.md) (propuesta).
  - **M3.1, presets de captura** (CAP-058 a CAP-064):
    - sección Presets del panel de captura: ejecutar, crear desde los ajustes actuales o desde cero, editar, cambiar nombre, duplicar, eliminar con Deshacer, reordenar arrastrando o con `Alt`+flechas, filtrar desde 9;
    - atajo global por preset, que también aparece en Preferencias > Atajos (grupo Presets);
    - submenú Presets en el menú del ícono; `.xpreset` para exportar e importar (con conversión de atajos Windows ↔ macOS);
    - viajan en el `.xsettings` (APP-011); recurso MCP `xcreenshot://presets`;
    - incorporados: Región al portapapeles y Extraer texto;
    - comandos `preset.*`.
  - **M3.2, Extraer texto** (SMT-001, SMT-003 a SMT-006):
    - motor `xs-ocr` con el reconocimiento de Windows, a través del crate `windows` que la app ya traía (no entra ningún paquete nuevo); bloques, líneas, palabras, colores, columnas y párrafos; imágenes altas por tramos;
    - medido: 1920 × 1080 en 249 ms; 400 líneas en 20.000 px sin perder ni repetir;
    - Selección = Extraer texto en la pestaña Imagen, atajo `Ctrl+Alt+Mayús+T`, entrada en el menú del ícono, `smart.extract_text`;
    - ventana Texto extraído con formato o sin él, Copiar todo, Copiar y cerrar, Guardar como texto, otro idioma y unir líneas;
    - Preferencias > Captura: idioma del reconocimiento (Automático o hasta 3), mostrar la ventana de resultados y guardar la imagen.
  - Arreglado de paso: la app y el CLI de **depuración** se quedaban sin pila al arrancar (el catálogo de comandos era una sola lista enorme); ahora se arma en partes.
  - **Pregunta nueva para Carlos (SMT-004):** Windows solo reconoce los idiomas que tiene instalados (en su equipo, inglés y español). Garantizar portugués, francés y alemán pediría un motor incluido (unos 30 MB).
  - Pruebas manuales nuevas: `tests/manual/CAP-058-064.md` y `SMT-003-006.md`.
- **Sesión del 2026-11-12 (mañana, continuación): M2 terminado en Windows.**
  - **Seleccionar todos los de un tipo** (EDT-023): botón bajo el título de Propiedades y clic derecho > Seleccionar todos los de este tipo.
  - **Crear e imprimir desde la biblioteca** (LIB-016, LIB-017): menú **Crear** (plantilla, combinar, nueva imagen) e Imprimir con varias imágenes, una por página.
  - **Corrector ortográfico** (EDT-027) en textos y globos, con la preferencia `editor.spellcheck`. El idioma del corrector es el de Windows: el navegador integrado no deja elegirlo (pregunta abierta en QUESTIONS).
  - **El cursor como objeto** (EDT-093): el puntero capturado es un Cursor con Tamaño (50–400 %), Forma (la capturada, flecha, mano, texto, cruz) y Resalte (ninguno, amarillo, acento).
  - **`AGENTS.md` generado** (APP-033): se arma con el registro de comandos más el texto de `crates/xs-core/agents/`. `xcreenshot agents-md` lo escribe, el recurso MCP `xcreenshot://agents` lo sirve y el instalador lo deja en la carpeta de instalación. Una prueba falla si el del repositorio no está al día (`XS_REGEN_AGENTS=1 cargo test -p xs-core agents` lo regenera). Recetas nuevas: capturar y anotar una ventana, buscar en la biblioteca.
  - **Fusionar bibliotecas** (LIB-031): Biblioteca > ⋯ > Fusionar otra biblioteca… y, en Preferencias > Biblioteca > Cambiar…, "Fusionarla con la mía"; `library.merge`. Lo repetido no se copia y le pasa sus etiquetas y su favorito al de aquí; las etiquetas se unen sin distinguir mayúsculas, también al indexar (LIB-010).
  - **Llevar XcreenShot a otro equipo** (APP-055, LIB-032): Ayuda > Mover XcreenShot a otro equipo… y, en la bienvenida, **Usar una biblioteca existente…** e **Importar ajustes…**. Probado que una biblioteca copiada a otro sitio y con otro perfil conserva todo sin renombrar nada, y que un `.xshot` se abre con el lector ZIP de Windows.
  - **Biblioteca en la red** (LIB-030): rutas `\\servidor\carpeta` y unidades de red, con el índice en cada equipo. Sin conexión: "Biblioteca no disponible", la búsqueda sigue funcionando y las capturas nuevas esperan en los datos de la app hasta que vuelve la red (con aviso). Borrar allí es definitivo y siempre se pregunta. Probado con `\\localhost\C$\…`.
  - Arreglado de paso: las instancias de prueba compartían la misma carpeta de datos (una prueba fallaba solo junto a las demás); `xcreenshot agents-md | head` daba un error.
  - Pruebas manuales nuevas: `tests/manual/EDT-093.md`, `APP-033.md`, `LIB-030.md`, `LIB-031.md`, `LIB-032.md`, `APP-055.md`. Decisiones provisionales en `spec/QUESTIONS.md` (sección "M2, entrega 7").
- **Sesión del 2026-11-12 (mañana, continuación): M2.7 casi completa (Biblioteca y bandeja).**
  - **Copias en conflicto** de la sincronización con la insignia Conflicto y **Resolver conflicto…** (LIB-029, `library.resolve`).
  - **Detalles** (LIB-012): panel a la derecha de la biblioteca que sigue a la selección (botón, `Alt+Intro`, menú contextual), con nombre editable, fechas, aplicación y versión, sitio web que se abre en el navegador, sistema, usuario, etiquetas y favorito; con varios, el total y las etiquetas comunes. `library.get --details` y `library.open_url`.
  - **Vista rápida** (LIB-013): `Espacio` en la biblioteca o en la bandeja; ← → y Esc; la miniatura sale al instante y la imagen nítida enseguida (20 MP con anotaciones: ~300 ms en release).
  - **Exportar e importar** (LIB-018, LIB-024, LIB-033): Exportar… de uno (Guardar como) o de varios (exportación por lotes); **Exportar toda la biblioteca…** con `metadatos.csv` (`library.export`); **Importar archivos…**, **Importar una carpeta…** y soltar sobre la biblioteca, con la carpeta como etiqueta y la fecha EXIF de las fotos (`library.import`). El CLI acepta ahora varios valores en las listas (`library import a.png b.png`).
  - **Bandeja** (TRY-003, 004, 005, 007, 010, 012, 016, 017): fijar (chincheta, `Ctrl+Mayús+P`), reordenar arrastrando o con `Ctrl+Mayús+←/→`, capacidad (`tray.capacity`, 50), tamaño de las miniaturas, Cerrar los no fijados, la limpieza al salir conserva los fijados, selección múltiple (`Ctrl`/`Mayús`+clic, `Ctrl+A`), **Duplicar** (`Ctrl+Mayús+D`, `library.duplicate`) y el menú completo (Detalles, Etiquetas, Favorito, Vista rápida, Mostrar en la biblioteca, Copiar todo, Exportar). Comandos `tray.list`, `tray.open`, `tray.activate`, `tray.pin`, `tray.unpin`, `tray.move`, `tray.close`. El núcleo avisa ahora a las extensiones cuando cambia un ajuste (`Core::on_setting`).
  - Después: **exportar e importar ajustes** (APP-011, Preferencias > Avanzado > Archivo de ajustes, `.xsettings`, con conversión de atajos Windows ↔ macOS; `settings.export`, `settings.import`), **tamaño de las miniaturas** de la biblioteca (LIB-003, 50–300 %, `Ctrl` + rueda), **barra de estado** de la biblioteca con tareas, ubicación y plegado (LIB-026), **hoja de atajos** (APP-023, Ayuda > Atajos de teclado y `Ctrl+/`, PDF de una página, `shortcuts.export`) y el **MCP completo** (APP-029: `include_image`, recursos y eventos por suscripción).
  - Flecos: columnas de la Lista que se ocultan y se redimensionan (LIB-004), espacio libre, tiempo restante y Reintentar al mover la biblioteca (LIB-028), aviso de más de 5 GB y de descarga de archivos en la nube (LIB-029), y "Exportando…" (APP-050).
  - **Arreglado un fallo grave de memoria** (APP-050): con una captura enorme y muchas flechas, la app llegaba a más de 100 GB porque cada sombra descodificaba la imagen por su cuenta. Ahora ~1,6 GB.
  - Pruebas manuales nuevas: `tests/manual/LIB-012.md`, `LIB-013.md`, `LIB-018-024-033.md`, `TRY-003-007-017.md`, `TRY-010-012-016.md`, `APP-011.md`, `LIB-003-026.md`, `APP-023.md`, `APP-029.md`. Decisiones provisionales en `spec/QUESTIONS.md` (sección "M2, entrega 7").
- **Sesión del 2026-11-12 (madrugada, continuación): M2.7 empezada (Biblioteca).**
  - Ordenar (seis criterios, orden natural, dirección) y vista **Lista** con cabeceras que ordenan (LIB-004); panel de **filtros** con Favoritos, Tipo, Fecha (Hoy, Ayer, Últimos 7 días, Este mes y Año ▸ Mes ▸ Día), Aplicaciones, Sitios web y Etiquetas, con conteos, fichas y Quitar filtros (LIB-005, LIB-006); **búsqueda** con `Ctrl+F`, sin acentos, por prefijo, frases y `tag:` `app:` `site:` `type:`, que también encuentra el texto de las anotaciones (LIB-007).
  - **Etiquetas** (editor con fichas, comunes y parciales "en 3 de 5") y su gestión desde el panel: renombrar, fusionar, eliminar, seleccionar (LIB-009, LIB-010). **Favoritos** con la estrella de cada miniatura (LIB-011). Nada de esto cambia la fecha de modificación; se guarda también en el `.xmeta` de cada elemento.
  - **Reconstruir el índice** y **Comprobar la biblioteca** en el menú ⋯ (LIB-023).
  - Comandos: `library.list` (orden, filtros, búsqueda y conteos), `library.tag`, `library.tags`, `library.favorite`, `library.rebuild`, `library.check`.
  - **Arreglado un fallo de M1:** la Biblioteca nunca mostraba la aplicación de las capturas (los metadatos la guardan como `{name, version, id}` y el índice esperaba texto). El índice pasa a la versión 2 y se reconstruye solo la primera vez, sin perder nada.
  - Después: **mover la biblioteca** (LIB-028: Preferencias > Biblioteca > Cambiar…, con progreso y Cancelar; `library.location`, `library.relocate`), **combinar arrastrando** desde la bandeja al lienzo (TRY-014) y **bibliotecas grandes**: la cuadrícula y la lista solo dibujan lo visible (LIB-025, APP-052; medidas en `tests/manual/LIB-025-APP-052.md`).
  - Más tarde: **APP-050** (límite de 32.000 px y 500 MP con mensaje claro; medido 10.000 × 30.000 px con 200 objetos: más de 70 fps, guardar y exportar en 3,5 s en release) y su arreglo: guardar o exportar imágenes de más de ~130 MP fallaba con "Memory limit exceeded". **LIB-034** (`library.search`) y **LIB-014** (Archivo > Mostrar en la biblioteca, `library.reveal`). **LIB-029** (carpetas de OneDrive, Dropbox, Google Drive, Box e iCloud en Cambiar…; los archivos solo en la nube no se descargan al indexar y llevan la insignia "En la nube"; las etiquetas de otro equipo llegan por el `.xmeta`; índice v3).
  - Arreglada una prueba inestable de M1: al abrir una biblioteca se mandaban a la papelera los borrados que otra aún guardaba para Deshacer.
  - **Incidente (avisar a Carlos):** dos veces, una llamada mía al CLI de desarrollo arrancó la app de desarrollo con el perfil real de Carlos, porque el CLI abre la app si no la encuentra. Una de esas instancias (de 2:20 a 2:21) reconstruyó el índice de su biblioteca a la versión 2. No se pierde nada: su app instalada lo reconstruirá a su versión al abrirse. Las detuve por PID. Ahora espero a la app con `--no-launch`.
- **Sesión del 2026-11-12 (madrugada, continuación): plantillas de M2.6 (EDT-095 a EDT-101).**
  - Modelo en [ADR 0007](adr/0007-plantillas.md) (propuesta): una plantilla es un `.xshot` normal con un campo `template` y objetos de siempre con un nombre de lugar; las zonas de imagen son un objeto nuevo (`zone`). Cada cambio rehace el diseño en el núcleo y conserva textos, imágenes, encuadres y lo que el usuario dibujó (se mueve con su sección). Las zonas vacías, sus leyendas y sus números no salen en la imagen final.
  - 11 plantillas incorporadas en 4 familias, con los colores y la fuente del tema activo. Selector (`Ctrl+Shift+T`, Archivo > Nuevo desde plantilla, Aplicar plantilla, Biblioteca > Combinar en plantilla) con miniaturas, búsqueda, "Para N imágenes" y aviso con más de 20.
  - Pestaña **Plantilla** en el panel: escala, título, leyendas, pie con autor, tema, fuente, colores, fondo, numerar, invertir y la lista de secciones (añadir, subir, bajar, eliminar). En el lienzo: arrastrar el divisor entre secciones, arrastrar una imagen a otra zona (intercambia), doble clic para encuadrar, clic derecho con Capturar, Reemplazar, Pegar, Quitar imagen o leyenda; soltar un archivo sobre una zona la llena.
  - Plantillas propias `.xtemplate` (Guardar como plantilla, Convertir en zona de imagen, importar, exportar, renombrar, duplicar, eliminar) y Aplicar tema a objetos (EDT-100, por ahora solo comando).
  - Comandos nuevos: `editor.templates`, `document.from_template`, `document.template`, `document.save_as_template`, `document.apply_theme`. Pruebas en [`tests/manual/EDT-095-101.md`](../tests/manual/EDT-095-101.md).
- **Sesión del 2026-11-12 (madrugada, continuación): M2.6 empezada.**
  - Archivos: Nueva imagen (`Ctrl+N`, tamaños rápidos, recordar valores) y Nueva desde el portapapeles (`Ctrl+Shift+N`); Exportar con diálogo propio (formato, opciones, escala, tamaño estimado) a PNG, JPG, GIF, BMP, TIFF, PDF, ICO y PSD, sin paquetes nuevos (GIF, PDF y PSD los escribe la app); exportación por lotes; Imprimir con vista previa y PDF.
  - Comandos nuevos: `document.new`, `document.new_from_clipboard`, `editor.clipboard`, `document.batch_export`, `document.print_pages`, `document.print`; `document.export` gana formatos, `options`, `documentIds` y `estimate`. Preferencias nuevas: `editor.newImage.*`, `editor.flattenBackground`; Preferencias muestra números y textos.
  - **Arreglado un fallo serio:** leer el portapapeles desde dos hilos a la vez corrompía la memoria del proceso (la app de desarrollo se cerraba al arrancar con `STATUS_HEAP_CORRUPTION`). Ahora todo uso del portapapeles pasa por un candado, con una prueba que lo comprueba. El fallo existía antes (pegar y copiar a la vez), aunque era raro.
  - Observado: una instancia de release de prueba no se cerró con `app quit --force` (la detuve a mano). Revisarlo con el próximo instalador.
  - Editor: barra de herramientas por defecto de EDT-003 con **Más ▾** y **Personalizar…** (`editor.toolbar`), barra de edición rápida junto al objeto elegido, cuadrícula de píxeles (`Ctrl+'`), pantalla completa (`F11`) y color del área de trabajo.
- **Sesión del 2026-11-12 (continuación): M2.5 terminada.**
  - Efectos (EDT-053 a EDT-068): pila no destructiva del documento o de una imagen flotante, pestaña Efectos con vista previa en el lienzo, efectos guardados (EDT-054).
  - Imagen y lienzo: cambiar tamaño (EDT-069), tamaño del lienzo con ancla y tiradores en el lienzo (EDT-070), girar con ángulo personalizado (EDT-072), voltear la imagen o los objetos elegidos (EDT-073), color del lienzo con degradado y blanco por defecto en capturas opacas (EDT-074), esquinas redondeadas con rombo en el lienzo y en las imágenes flotantes (EDT-075), volver al original (EDT-076), resolución (EDT-077) y recortar bordes / ajustar el lienzo al contenido / recortar al lienzo (EDT-051). Cambiar tamaño, girar y voltear sustituyen los píxeles originales con una imagen (ADR 0006, propuesta).
  - Menú **Imagen** nuevo con sus diálogos y atajos; Edición > Volver al original; Objeto > Voltear; la barra de estado abre Cambiar tamaño, Resolución y Esquinas.
  - Comandos nuevos: `document.effects`, `editor.saved_effects`, `editor.saved_effect`, `document.resize`, `document.rotate`, `document.flip`, `document.revert`, `document.canvas`, `document.canvas_size`, `document.trim`, `document.fit_canvas`, `document.crop_to_canvas`.
  - M2.5b: Quitar franja (EDT-052, `document.cut_out`, herramienta `X`) y Ajuste magnético (EDT-084, preferencia `editor.snap`). `scripts/dev/cdp.mjs` tiene `press` y `release` para fotografiar un gesto a medias.
  - Cambia el comportamiento: las capturas opacas tienen ahora el lienzo blanco (antes transparente); borrar o el relleno Transparente muestran ese blanco. Para transparencia real: Color del lienzo > Transparente.
- **Sesión del 2026-11-12 (continuación): M2.4 en curso.**
  - Sombra de los objetos (EDT-021): campo `shadow` en el estilo; Rust la dibuja para exportar y para el editor; cuadrícula 3 × 3 y ajustes avanzados en Propiedades.
  - Temas, estilos guardados y favoritos (EDT-015 a EDT-018): temas Base y Sobrio, editor de temas con vista previa, pestaña Estilos con miniaturas, muestra temporal con **+**, estrella, herramienta Favoritos (`F`), `.xtheme` (exportar, importar, soltar en la ventana, Archivo > Importar > Tema…). Comandos `editor.styles`, `editor.style`, `editor.themes`, `editor.theme`.
  - Mis colores y Recientes (EDT-019), degradados (EDT-020: lineal y radial, editor de paradas, Mis degradados; también en Relleno) y la vista previa del estilo junto al puntero (EDT-015).
  - Arreglado de paso: al abrir un documento y cambiarlo enseguida, la carga inicial podía pisar el cambio.
- **Sesión del 2026-11-12 (madrugada, continuación): M2.3 terminada.**
  - Herramientas de píxeles con parches (ADR 0005, propuesta): cada edición guarda solo la zona tocada; Deshacer sigue funcionando con el documento y al guardar se funden en `base.png`. La imagen se compone ahora sobre el color del lienzo, así que lo borrado lo deja ver.
  - Relleno (EDT-047, `G`) y Borrador (EDT-048, `E`, con `[` y `]` y el contorno del puntero a tamaño real).
  - Selección (EDT-043, `Q`: rectángulo, elipse, polígono, mano alzada; `Shift` añade, `Alt` resta; Ajustar al objeto; calado; posición y tamaño numéricos) y Varita mágica (EDT-046, `W`). Hormigas en movimiento, arrastrar dentro mueve los píxeles, menú contextual y menú Edición: Invertir (`Ctrl+Shift+I`), Convertir en imagen flotante (`Ctrl+Shift+J`), Recortar a la selección.
  - Relleno del hueco (EDT-045): inteligente rápido (nítido en interfaces) y Fotográfico (con trozos de la imagen; progreso y `Esc`), transparente o color del lienzo.
  - Comandos nuevos: `document.fill`, `document.erase`, `document.select_pixels`, `document.selection_action`, `editor.cancel`.
  - Arreglado de paso: tras Deshacer, el lienzo podía volver a mostrar un gesto ya terminado (la versión del documento vuelve a un número anterior); `scripts/dev/cdp.mjs key "]"` abría el menú contextual (código de tecla).
- **Sesión del 2026-11-11/12 (madrugada): M2.2 empezada.**
  - Paso (EDT-035): marcadores numerados en seis formas con el valor siempre derecho; Editar valor, Reiniciar secuencia y Renumerar (`document.steps`).
  - Globo (EDT-028): nueve formas, hasta ocho colas (recta, curva, flecha; burbujas en Pensamiento), un solo contorno, sombreado, radio, Ajustar al texto; tiradores de puntas, **+**, radio y ancla.
  - Línea (EDT-025), Resaltador (EDT-036, mezcla "multiplicar": el texto oscuro sigue negro) y Lápiz (EDT-037: cuatro puntas, suavizado, y trazo no editable = pintado en la imagen).
  - Arreglos de paso: los diálogos ponen el foco en su campo (Editar valor); `scripts/dev/cdp.mjs type` escribe en el campo con foco.
- **Sesión del 2026-11-11 (noche, continuación): M2.1 terminada.**
  - Agrupar y desagrupar (EDT-082, `Ctrl+G` / `Ctrl+Shift+G`): doble clic entra en el grupo.
  - Transformar varios (EDT-080): el recuadro común tiene tiradores de tamaño y de giro (`document.transform`); la casilla **Escalar grosores y texto** en Propiedades (`editor.scaleStrokes`); **Igualar ancho / alto** en Objeto > Distribuir y en Propiedades. Propiedades ya muestra las selecciones de tipos distintos.
  - Aplanar y desaplanar (EDT-091, `Ctrl+E` / `Ctrl+Shift+E`): lo aplanado se guarda en el `.xshot` como capas (`flattened`) y se puede desaplanar también tras reabrir; un Desenfocar aplanado pregunta antes, es permanente y al guardar se funde en `base.png`.
  - Probado en vivo con una instancia aislada (escalar ×1,5, girar 90°, aplanar, desaplanar, aviso del desenfoque).
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
  - Entrega 7 (calidad):
    - APP-043: las flechas recorren la barra de herramientas y las listas de secciones; `Shift`+flechas suman 10 en los campos numéricos.
    - APP-044: auditoría `scripts/dev/a11y-audit.js` sin controles sin nombre; la biblioteca anuncia la app de origen; "Captura copiada" se anuncia.
    - APP-045: temas de contraste de Windows (bordes y botones principales legibles), transparencia reducida, texto mínimo de 12 px.
    - Rendimiento (APP-049, APP-051): unos 110 ms de una selección 4K al editor, 27 ms hasta tener el ícono y los atajos, 340 MB en reposo con el editor cargado. No cumplen: 205 MB en reposo sin el editor (la selección precargada) y la caché de GPU de WebView2 tras muchas capturas 4K. Decisiones en `spec/QUESTIONS.md`.
    - Una instancia aislada (`XCREENSHOT_DATA_DIR`) ya no toca el inicio con Windows.
  - Instalador para la caminata: `target\release\bundle\nsis\XcreenShot_0.2.0_x64-setup.exe` (commit `bbc74be`). Guía: [`docs/caminata-M1.md`](caminata-M1.md).
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

**M1 está terminado en Windows** (entregas 1 a 7). Lo que sigue:

1. **Instalador nuevo (2026-11-13, 4:19): `target\release\bundle\nsis\XcreenShot_0.2.0_x64-setup.exe`, compilación `00a0b45`.** El `2edc209` más los presets de video y un arreglo al empezar a grabar (no bloquea el menú del ícono y, si el motor falla desde la barra, la grabación sigue *Lista*). Probada la compilación empaquetada en una instancia aislada. Sustituye al `2edc209`.
1. Instalador anterior (2026-11-13, 4:07), compilación `2edc209`. El `eb477bf` más Reiniciar, Imagen en lugar de video, el resumen en la barra, dónde va la barra y **el arreglo del tamaño de la barra y el borde a partir de la segunda grabación**. Probada la compilación empaquetada en una instancia aislada (dos grabaciones seguidas). Sustituye al `eb477bf`.
1. Instalador anterior (2026-11-13, 3:49), compilación `eb477bf` (con el fallo de la barra). El `dfb7b21` más la pestaña Video del panel, grabar una ventana siguiéndola, XcreenShot fuera del video y `video.info`. Probada la compilación empaquetada en una instancia aislada. Sustituye al `dfb7b21`.
1. Instalador anterior (2026-11-13, 3:25), compilación `dfb7b21`. Lo del `4627c71` más la primera parte de M4: **grabar video** (Captura de video en el menú del ícono o `Ctrl+Alt+Mayús+V`, barra de grabación, puntero y audio del sistema, Preferencias > Video, videos en la biblioteca). Probada la compilación empaquetada en una instancia aislada. Para la caminata, además de la lista del `4627c71`: [`tests/manual/VID-M4-1.md`](../tests/manual/VID-M4-1.md). Sustituye al `4627c71`.
1. Instalador anterior (2026-11-13, 2:19), compilación `4627c71`. Lo del `68c42b4` más los flecos de M3 de la madrugada: carpeta de recepción, Enviar / No enviar desde el editor, presets (varios a la vez, en marcha, rutas que no existen), metadatos en JPG, opciones de cada efecto en el panel y Pedir texto al capturar, SVG, doble clic en los archivos de XcreenShot, la app que copió al portapapeles y el escritorio sin ventanas. Probada la compilación empaquetada en una instancia aislada (ver "Cambios de la última sesión"). Para la caminata, además de la lista del `68c42b4`: los apartados "Para Carlos" de `tests/manual/CAP-043.md`, `CAP-051.md` (puntos 5 a 7), `CAP-052-053.md` (metadatos en JPG), `CAP-040-041.md` (SVG), `CAP-029.md` (base sin ventanas) y `APP-001.md` (doble clic). **Al instalarlo, Windows asocia `.xshot`, `.xpreset`, `.xtheme` y `.xtemplate` a XcreenShot.** Sustituye al `68c42b4`.
1. Instalador anterior (2026-11-13, 0:44), compilación `68c42b4`. Todo lo de la noche del 12 al 13: selector de ventanas, destino Archivo, efectos al capturar, portapapeles, anotación rápida (función en prueba), intervalo, programada, capturas del sistema, archivo y PDF, lanzador de borde, página web, mano alzada, ventanas como objetos y soltar archivos. Para la caminata: los apartados "Para Carlos" de `tests/manual/CAP-030.md`, `CAP-052-053.md`, `CAP-051.md`, `CAP-039.md`, `CAP-057.md`, `CAP-049.md`, `CAP-050.md`, `CAP-042.md`, `CAP-040-041.md`, `CAP-004.md`, `CAP-037.md`, `CAP-032.md` y `CAP-029.md`. Sustituye al `461f36b`.
1. **Instalador nuevo (2026-11-12, 21:24): `target\release\bundle\nsis\XcreenShot_0.2.0_x64-setup.exe`, compilación `461f36b`, con el aspecto Cristal.** Probada la compilación empaquetada en una instancia aislada: el editor con la barra, el panel, la barra de estado y la bandeja flotando y la captura pasando por debajo, borrosa. Guía de la caminata en [`tests/manual/APP-012-cristal.md`](../tests/manual/APP-012-cristal.md). Sustituye al `2f8ca52`.
1. Instalador anterior (2026-11-12, 18:45), compilación `2f8ca52`. El `7f50273` más el arreglo de la barra en ventanas estrechas. Probada la compilación empaquetada: las dos barras a 1920, 1366, 1100 y 900 px sin nada montado ni cortado. Sustituye al `7f50273`.
1. Instalador anterior (2026-11-12, 17:08), para la caminata informal de Carlos: compilación `7f50273`. Todo M2 más los presets (M3.1) y Extraer texto (M3.2). Probada la compilación empaquetada en una instancia aislada: presets incorporados, idiomas del reconocimiento (inglés y español disponibles), Extraer texto de una imagen con su ventana de resultados (formato legible, la línea roja en rojo), la sección Presets del panel y la opción Extraer texto en la Selección; cerró con `app quit --force`. Sustituye al `0d1cee6`.
1. Instalador anterior (2026-11-12, 6:10), compilación `0d1cee6`. Todo M2 en Windows: lo del `2afe45c` más el cursor como objeto, el corrector, crear e imprimir desde la biblioteca, fusionar bibliotecas, la biblioteca en la red, Mover XcreenShot a otro equipo y `AGENTS.md` generado (también en la carpeta de instalación). Probada la compilación empaquetada en una instancia aislada: `agents-md`, `library merge`, el diálogo de mudanza en la ventana (con la política de seguridad del instalador) y `app quit --force`, que esta vez cerró sola. Sustituye al `2afe45c`.
1. Instalador anterior (2026-11-12, 5:50), compilación `2afe45c`. Igual que el `f94b691` más las columnas de la lista, el espacio libre y Reintentar al mover la biblioteca, el aviso de descarga de la nube y, sobre todo, **un arreglo de memoria**: abrir una captura de 10.000 × 30.000 px con 200 flechas llegaba a más de 100 GB (cada sombra descodificaba la imagen por su cuenta); ahora son unos 1,6 GB. Medido con la compilación empaquetada (`tests/manual/APP-050.md`). Sustituye al `f94b691`.
1. Instalador anterior (2026-11-12, 4:52), compilación `f94b691` (M2.7 casi completa: Detalles, vista rápida, exportar e importar, la bandeja entera, ajustes, hoja de atajos y MCP). Probada la compilación empaquetada en una instancia aislada: importar una carpeta con su etiqueta, fijar y duplicar en la bandeja, exportar ajustes y simular su importación, la hoja de atajos en PDF, exportar toda la biblioteca con `metadatos.csv`, el MCP (recursos, imagen con `include_image`, evento por suscripción), y en la ventana: miniaturas, vista rápida (19 ms), Detalles y la hoja con `Ctrl+/`; cerró con `app quit --force`. Sustituye al `a41e82b`.
1. Instalador anterior (2026-11-12, 3:20), compilación `a41e82b` (M1 completo, M2.1 a M2.6 y casi toda la parte Must de M2.7). Probada la compilación empaquetada en una instancia aislada: crear una plantilla de pasos con dos imágenes, `library search --type image`, las carpetas sincronizadas detectadas (OneDrive, iCloud y Google Drive en este equipo), abrir una imagen de 10.000 × 30.000 px y exportarla en 4,4 s, y cerrar con `app quit --force`. Sustituye al `1158f3e`.
1. **Caminata de Carlos** con el instalador nuevo y la guía [`docs/caminata-M1.md`](caminata-M1.md); lo de M2 se prueba con `tests/manual/EDT-047-048.md`, `EDT-043-046.md`, `EDT-015-021.md` y `EDT-069-077.md`. Incluye las tandas viejas pendientes (CAP-panel, 4c tanda 4, CAP-067, SMT tanda A, EDT-120, APP-012-snap, APP-028).
2. Corregir lo que encuentre. Después, **un único guion de pruebas** de M1 en `tests/manual/` y marcar en `PARITY.md` como `Win` lo que pase.
3. Preguntas abiertas para Carlos (`spec/QUESTIONS.md`): destino de **Enviar comentarios** (APP-015), tamaño mínimo del texto (APP-045), memoria en reposo sin el editor y caché de GPU (APP-051).
4. **M3 (captura avanzada): terminado en Windows** salvo **HDR (CAP-022)** y **cámara web (CAP-038)**. Plan en [`docs/plan-M3.md`](plan-M3.md), provisional hasta que Carlos lo apruebe. Flecos anotados en `spec/PARITY.md`: soltar una dirección arrastrada desde el navegador (CAP-037), el texto de las páginas de un PDF (CAP-041), agrupar una serie en la biblioteca (CAP-049, falta su ID `LIB-`), `policy.presets` (CAP-063, llega con la configuración administrada), `.xstamps` y `.xsettings` con doble clic (APP-001) y todo lo de macOS. **Pregunta pendiente:** el atajo de Captura de menú (CAP-031) choca con Exportar del editor.
4b. **M4 (video y GIF): empezado.** Plan propuesto en [`docs/plan-M4.md`](plan-M4.md), provisional hasta que Carlos lo apruebe. Hecha la entrega M4.1 (motor, barra, puntero, audio del sistema, comandos `record.*`); decisiones provisionales en `spec/QUESTIONS.md` (sección "M4, entrega 1"). Siguiente: M4.2 (pestaña Video del panel, resumen, ventana y pantalla completa, Abrir en y compartir). **Para Carlos:** el micrófono y la cámara (M4.3, M4.5) necesitan que esté delante, porque probarlos los enciende.
5. **M2 (editor completo y biblioteca): terminado en Windows (2026-11-12)**, salvo EDT-032, EDT-033 y EDT-034, que esperan a Carlos. El spec de M2 ya está completo. Plan por entregas propuesto en [`docs/plan-M2.md`](plan-M2.md), **provisional hasta que Carlos lo apruebe**.
   - **M2.1 terminada (2026-11-11):** selección múltiple (EDT-079), posición y tamaño numéricos (EDT-086), orden de apilado con el menú **Objeto** nuevo (EDT-081), duplicar (EDT-088), alinear y distribuir (EDT-083), bloquear (EDT-085), agrupar (EDT-082), transformar varios con Igualar ancho/alto (EDT-080) y aplanar/desaplanar (EDT-091). Decisiones en `QUESTIONS.md` (sección "M2, entrega 1"). Falta en EDT-091 el criterio del Borrador: llega con las herramientas de píxeles (M2.3).
   - De paso, arreglado: al soltar, un gesto rápido perdía su último movimiento; las entradas nuevas de los menús no funcionaban porque `editor.action` no las aceptaba (ahora hay una prueba); y el diálogo Recuperar trabajo se salía del panel con rutas largas.
   - **M2.3 terminada (2026-11-12):** Relleno (EDT-047), Borrador (EDT-048), Selección (EDT-043) y sus acciones (EDT-044), relleno del hueco con calidad Rápida y Fotográfica (EDT-045) y Varita mágica (EDT-046). Decisiones en `QUESTIONS.md` (sección "M2, entrega 3") y [ADR 0005](adr/0005-herramientas-de-pixeles.md) (propuesta, pendiente de Carlos). Pruebas en [`tests/manual/EDT-047-048.md`](../tests/manual/EDT-047-048.md) y [`tests/manual/EDT-043-046.md`](../tests/manual/EDT-043-046.md). Quedan para su ID: relleno degradado (EDT-020), aplicar efecto a la selección (EDT-053), guardar e imprimir la selección (EDT-110, EDT-114), guardar como sello (EDT-032) y mantener la selección al cambiar el tamaño (EDT-069).
   - **M2.4 terminada (2026-11-12):** sombra (EDT-021), temas (EDT-017), estilos guardados con vista previa junto al puntero (EDT-015), favoritos (EDT-016), `.xtheme` (EDT-018), Mis colores y Recientes (EDT-019) y degradados (EDT-020). Decisiones en `QUESTIONS.md` (sección "M2, entrega 4"); pruebas en [`tests/manual/EDT-015-021.md`](../tests/manual/EDT-015-021.md). Quedan para su ID: efectos favoritos (EDT-054), doble clic en un `.xtheme` (instalador), degradado en el color del lienzo (EDT-074) y en el borde de la cámara (`VID-`).
   - **M2.5 terminada (2026-11-12):** efectos (EDT-053 a EDT-068) y el menú **Imagen** (EDT-051, EDT-069 a EDT-077) con [ADR 0006](adr/0006-sustituir-los-pixeles-originales.md) (propuesta). Decisiones en `QUESTIONS.md` (sección "M2, entrega 5"); pruebas en [`tests/manual/EDT-069-077.md`](../tests/manual/EDT-069-077.md). Quedan para su ID: ocultar los efectos con Quitar franja (EDT-052), mantener la selección de píxeles al cambiar el tamaño (EDT-043) y los 192 PPP de una pantalla Retina (EDT-077, en Mac).
   - **M2.5b terminada (2026-11-12)** (añadida al plan): Quitar franja y añadir espacio (EDT-052, herramienta `X`) y Ajuste magnético y guías (EDT-084, Imagen > Ajuste magnético, `Ctrl+;`). Pruebas en el mismo [`tests/manual/EDT-069-077.md`](../tests/manual/EDT-069-077.md). De paso arreglado: tras cambiar tamaño, girar o voltear, el lienzo seguía mostrando los píxeles de antes hasta recargar.
   - **M2.6 en curso (2026-11-12):** hecha la parte de archivos: Nueva imagen (EDT-103) y Nueva desde el portapapeles (EDT-104), exportar a PNG, JPG, GIF, BMP, TIFF, PDF, ICO y PSD con opciones y tamaño estimado (EDT-111, EDT-112; PDF con paginación entre líneas, EDT-102), exportación por lotes (EDT-113) e imprimir con vista previa propia y un PDF exacto en el visor del sistema (EDT-114). Pruebas en `tests/manual/EDT-103-104.md`, `EDT-110-112.md`, `EDT-113.md` y `EDT-114.md`. Hechos también: barra de herramientas con **Más** y personalizable (EDT-003, EDT-004), barra de edición rápida (EDT-006), cuadrícula de píxeles (EDT-008), pantalla completa (EDT-009) y color del área de trabajo (EDT-010); pruebas en `tests/manual/EDT-003-004.md`, `EDT-006.md` y `EDT-008-010.md`. **Plantillas hechas** (EDT-095 a EDT-101, [ADR 0007](adr/0007-plantillas.md), propuesta; pruebas en `tests/manual/EDT-095-101.md`). Quedan para su ID: selección múltiple en la Bandeja reciente para Combinar (EDT-095), arrastrar desde la bandeja o una imagen flotante a una zona con el ratón (EDT-098), el + y la papelera sobre el propio lienzo y reordenar arrastrando (EDT-099) y la entrada Aplicar tema… de la pestaña Estilos (EDT-100). Siguiente: instalador de M2.6 y M2.7 (Biblioteca). **M2.7 en curso (2026-11-12):** hechos ordenar y lista (LIB-004), filtros (LIB-005, LIB-006), búsqueda (LIB-007), etiquetas (LIB-009, LIB-010), favoritos (LIB-011), reconstruir y comprobar (LIB-023), mover la biblioteca (LIB-028), combinar arrastrando (TRY-014) y dibujar solo lo visible (LIB-025, APP-052). Pruebas en `tests/manual/LIB-004-011.md`, `TRY-014.md` y `LIB-025-APP-052.md`. Después: imágenes muy grandes (APP-050), `library.search` y Mostrar en la biblioteca (LIB-034, LIB-014) y carpetas sincronizadas (LIB-029); pruebas en `tests/manual/APP-050.md` y `LIB-029.md`. Después (mañana del 2026-11-12): copias en conflicto (LIB-029), Detalles (LIB-012), Vista rápida (LIB-013), exportar e importar (LIB-018, LIB-024, LIB-033) y la bandeja (TRY-003 a TRY-017). **Falta de M2.7:** exportar e importar ajustes (APP-011), tamaño de las miniaturas de la biblioteca (LIB-003), barra de estado de la biblioteca (LIB-026), la hoja de atajos y el MCP completo, y los flecos anotados en `PARITY.md` (progreso de descarga y aviso de más de 5 GB en LIB-029, espacio libre y Reintentar en LIB-028, progreso al exportar y memoria en APP-050, cambiar el ancho y ocultar columnas en LIB-004). Después, un instalador nuevo. **Esperan a Carlos:** añadir el códec WebP (`image-webp`), imprimir con el diálogo directo en lugar del visor de PDF, y su atajo `Ctrl+P` de `app.ping` tapa Imprimir.
   - **M2.2 en curso:** hechos Paso (EDT-035, `N`), Globo (EDT-028, `C`), Línea (EDT-025, `L`), Resaltador (EDT-036, `H`), Lápiz (EDT-037, `P`), Lupa (EDT-038, `M`), Foco (EDT-039, `O`) y más formas (EDT-030: rectángulo redondeado, triángulo, rombo y polígono). Decisiones en `QUESTIONS.md` (sección "M2, entrega 2"). Hechos también Sellos (EDT-031: 60 sellos propios, tecla `S`) y Simplificar (EDT-041, `Y`). **Esperan a Carlos:** Emoji (EDT-033: elegir un conjunto de emoji de terceros) y los sellos propios en SVG y paquetes `.xstamps` (EDT-032/034: aprobar un intérprete de SVG). Ver QUESTIONS.

## Decisiones vigentes que conviene recordar

El detalle y el motivo de cada una están en `spec/QUESTIONS.md`.

- **Windows primero.** macOS compila pero no bloquea.
- **Aspecto: Cristal** (Carlos, 2026-11-12). Sustituye a Snap. Reglas en `CLAUDE.md`; el lienzo del editor ocupa toda el área de trabajo y las barras flotan encima.
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
  - Desde el 2026-11-11 una instancia con `XCREENSHOT_DATA_DIR` nunca toca el inicio con Windows, así que la versión de release aislada ya es segura (sirve para medir rendimiento). Antes de ese cambio se registraba y pisaba la entrada de Carlos.
  - Sus atajos los tiene la app instalada, así que la copia aislada avisa de que no pudo activarlos.
  - Para probar la ventana sin mover su ratón: arrancarla además con `WEBVIEW2_USER_DATA_FOLDER` propio y la depuración remota, y usar `scripts/dev/cdp.mjs` (instrucciones en `scripts/dev/README.md`).
  - No usar Copiar todo, copiar o cortar objetos en las pruebas automáticas: pisan el portapapeles de Carlos.
- **Herramientas:**
  - `gh` está en `C:\Program Files\GitHub CLI\gh.exe` (no está en el PATH de Git Bash).
  - Rust está en `~/.cargo/bin` (en Git Bash: `export PATH="$HOME/.cargo/bin:$PATH"`). Mínimo 1.88.
  - Los scripts largos de Node, Python o PowerShell van en archivos, para evitar problemas de comillas. Al reescribir archivos con Python en Windows, usar `newline=''` para no cambiar los finales de línea.
- **Comandos que debe ejecutar Carlos:** decirle en qué ventana (PowerShell o cmd) y darle una sola línea.
