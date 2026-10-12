# 03 · Video, cámara, audio y GIF

**Archivo:** `03-video.md` · **Prefijo de IDs:** `VID-` · **Versión:** 0.1 (borrador) · **Fecha:** 2026-10-12

Este archivo especifica la grabación de video de XcreenShot: cómo se inicia y cómo se elige qué grabar (área, monitor, ventana o solo la cámara), la barra de grabación y el ciclo de vida de una grabación, la calidad y el formato del archivo, la cámara, el audio, los efectos de cursor y de teclado, el Dibujo en vivo, a dónde va el video al terminar, la edición básica de videos en el editor, la creación de GIF animados y las Diapositivas narradas. También fija los atajos de video y los permisos que necesita cada función. La mecánica de selección en pantalla (cruceta, lupa, detección, teclado, proporciones y guías) es de `CAP-` y aquí solo se añaden las reglas propias del video. El sistema de presets es de `CAP-`, la gestión de atajos y permisos de `APP-`, la biblioteca, la bandeja y los destinos de `LIB-`, `TRY-` y `SHR-`, y el reconocimiento de texto de `SMT-`. Salvo que una función diga otra cosa, todo vale igual en Windows y en macOS (SPEC §2.1), incluidas las funciones que en Snagit son de una sola plataforma.

**Convenciones de este archivo**
- **Teclas.** `Ctrl`/`Cmd` significa `Ctrl` en Windows y `Command` (⌘) en macOS; `Alt`/`Option` igual. En macOS, `⌃` es Control, `⌥` es Option, `⇧` es Shift y `⌘` es Command; por ejemplo, `⌃⌥⇧V` es Control+Option+Shift+V.
- **(decisión de diseño)** marca un comportamiento que la documentación pública de Snagit no describe, o que XcreenShot define a propósito de otra forma. Lo fija este spec.
- **"Por defecto"** es el valor en una instalación nueva, sin configuración administrada.
- **Estados de una grabación.** *Selección* (se elige qué grabar) → *Lista* (barra de grabación visible, aún sin grabar) → *Cuenta atrás* → *Grabando* ⇄ *En pausa* → *Finalizando* (se cierra el archivo) → *Terminada*. Desde Lista, Grabando o En pausa se puede *Reiniciar* (vuelve a Lista) o *Cancelar* (termina sin archivo).
- **Área de grabación.** El rectángulo de pantalla (o la ventana) que se graba. **Video resultante:** el archivo final, que puede ser más pequeño que el área (VID-028).
- **Referencias a otros archivos.** Por ID cuando existe (por ejemplo, CAP-018) o por `PREFIJO- (nombre de la función)` cuando el archivo aún no está escrito.

---

## Índice

1. **Iniciar una grabación** — VID-001 Pestaña Video del panel de captura · VID-002 Iniciar una captura de video · VID-003 Área de grabación · VID-004 Pantalla completa de un monitor · VID-005 Grabar una ventana · VID-006 Resumen Listo para grabar · VID-007 Empezar a grabar sin esperar · VID-008 Cuenta atrás · VID-009 Presets de video · VID-010 Convertir una grabación en imagen
2. **Barra de grabación y ciclo de vida** — VID-011 Barra de grabación · VID-012 Grabar, pausar y reanudar · VID-013 Detener · VID-014 Reiniciar y cancelar · VID-015 Temporizador de parada automática · VID-016 Posición de la barra y controles minimizados · VID-017 XcreenShot fuera del video · VID-018 Escritorio limpio · VID-019 Aviso de micrófono silenciado · VID-020 Indicadores de grabación · VID-021 Mantener el equipo despierto · VID-022 Recuperar grabaciones interrumpidas · VID-023 Grabaciones largas y espacio en disco · VID-024 Comandos de video (CLI y MCP)
3. **Calidad y codificación** — VID-025 Calidad y fotogramas por segundo · VID-026 Formato del archivo · VID-027 Códec H.265 · VID-028 Tamaño del video y límites · VID-029 Codificación por hardware o por software · VID-030 Sincronía y rendimiento
4. **Cámara** — VID-031 Activar la cámara y elegir dispositivo · VID-032 Imagen en imagen: posición y tamaño · VID-033 Forma y proporción de la cámara · VID-034 Cámara a pantalla completa · VID-035 Cambiar entre imagen en imagen y pantalla completa · VID-036 Espejo y borde de la cámara · VID-037 Fondo de la cámara · VID-038 Cámara ocupada, tapada o sin permiso
5. **Audio** — VID-039 Micrófono · VID-040 Audio del sistema · VID-041 Reducción de ruido · VID-042 Pistas de audio · VID-043 Conectar y desconectar dispositivos
6. **Cursor, clics y teclas** — VID-044 Capturar cursor · VID-045 Resaltar cursor · VID-046 Animación de clic · VID-047 Mostrar teclas pulsadas
7. **Dibujo en vivo** — VID-048 Modo Dibujo en vivo · VID-049 Herramientas de Dibujo en vivo · VID-050 Color, grosor y desvanecimiento · VID-051 Disponibilidad de Dibujo en vivo
8. **Después de detener** — VID-052 Abrir en · VID-053 Compartir al terminar · VID-054 Guardado en la biblioteca y metadatos · VID-055 Importar videos
9. **Edición de video en el editor** — VID-056 Reproductor de video · VID-057 Velocidad de reproducción y clic para reproducir · VID-058 Selección en la línea de tiempo · VID-059 Cortar · VID-060 Guardar la selección como nuevo video · VID-061 Guardar fotograma como imagen · VID-062 Extraer texto de un fotograma · VID-063 Copia sin sonido · VID-064 Información del video · VID-065 Sin anotaciones sobre el video · VID-066 Guardar, exportar, copiar y arrastrar videos · VID-067 Combinar videos
10. **GIF animado** — VID-068 Crear GIF animado · VID-069 Optimizar para · VID-070 Ajustes personalizados del GIF
11. **Diapositivas narradas** — VID-071 Crear diapositivas narradas · VID-072 Barra y navegación de las diapositivas · VID-073 Anotar las diapositivas en vivo · VID-074 Tamaño y fondo del video de diapositivas
12. **Atajos y permisos** — VID-075 Atajos globales de video · VID-076 Teclas contextuales de video · VID-077 Permisos que necesita el video
13. [Relacionado](#relacionado) · [Excluido](#excluido) · [Cobertura](#cobertura)

---

## 1. Iniciar una grabación

### VID-001 · Pestaña Video del panel de captura
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Video tab del Capture Window (Open In, Selection, Share, Camera, Screen Draw, Cursor, Microphone, System Audio, Presets; engranaje de Selection)
- **Comportamiento:**
  - Es la tercera pestaña del Panel de captura (CAP-001). Controles, en este orden:
    - **Selección**: Región (por defecto), Ventana (VID-005), Pantalla completa (VID-004) y Solo cámara (VID-034). El engranaje **Propiedades** muestra las opciones del modo elegido: región fija y posición (CAP-026), modo de ventana (VID-005) o monitor (VID-004).
    - **Compartir** (VID-053): solo destinos que aceptan video (`SHR-`).
    - **Abrir en** (VID-052).
    - **Cámara**: interruptor y menú de opciones (VID-031 a VID-037).
    - **Micrófono**: interruptor y menú de opciones (VID-039, VID-041).
    - **Audio del sistema**: interruptor (VID-040).
    - **Cursor y teclado**: interruptor del cursor y menú **Efectos** con Resaltar cursor, Animación de clic y Teclas pulsadas (VID-044 a VID-047).
    - **Dibujo en vivo**: interruptor de disponibilidad y opciones (VID-048 a VID-051).
    - **Temporizador**: Sin límite o una duración (VID-015).
  - El botón **Capturar**, el campo **Atajo** y la lista de presets funcionan como en CAP-001 y CAP-058.
  - Valores por defecto, a los que vuelve **Restablecer ajustes de captura** (CAP-003) (decisión de diseño):
    - Selección = Región, sin región fija; Compartir = ninguno; Abrir en = Editor;
    - Cámara activada si el equipo tiene alguna (VID-031); Micrófono activado con el dispositivo predeterminado del sistema; Audio del sistema desactivado;
    - Cursor activado, sin efectos; Teclas pulsadas desactivado; Dibujo en vivo disponible; Temporizador = Sin límite.
  - Hay una sola fuente de ajustes de video: la grabación que empieza desde Captura universal (CAP-009) usa estos mismos ajustes, y los cambios que se hacen en la barra de grabación (VID-011) se guardan aquí al terminar. Excepción: si la grabación viene de un preset, el preset no cambia (VID-009).
  - Si la configuración administrada desactiva el video (APP-035), la pestaña no aparece.
- **Criterios de aceptación:**
  - [ ] Con Selección = Pantalla completa y Micrófono desactivado, **Capturar** muestra la barra de grabación del monitor bajo el puntero y el resumen indica "Micrófono: desactivado".
  - [ ] Si se apaga la cámara en la barra durante una grabación, al terminar la pestaña Video muestra la cámara desactivada.
  - [ ] Tras **Restablecer ajustes de captura**, todos los controles de la pestaña muestran los valores por defecto de esta ficha.
  - [ ] Con `policy.disableVideoCapture`, la pestaña Video no aparece.
- **Fuentes:** research/03-video.md §A (Video tab, All-in-One → Video Capture button), §B (Selection gear); research/05-plataformas-versiones.md §1A (2026.1.0)

### VID-002 · Iniciar una captura de video
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** botón Capture y atajo global con la pestaña Video; botón Video Capture del All-in-One; atajo Video Capture de Mac; Capture Widget; Repeat Last Capture
- **Comportamiento:**
  - Formas de iniciar una grabación:
    - **Capturar** (botón o atajo, CAP-006) con la pestaña Video activa;
    - el atajo **Captura de video** (VID-075), sea cual sea la pestaña activa;
    - el botón **Video** (`V`) de la barra de la captura universal (CAP-009), también cuando se lanzó desde el Lanzador de borde (CAP-004);
    - el menú del ícono > **Captura de video** (CAP-005);
    - un preset de video (VID-009) o el comando `record.start` (VID-024).
  - Con Selección = Región aparece la cruceta con las mismas herramientas que en una imagen (CAP-013, CAP-015, CAP-016, CAP-017). Diferencia: en la captura de video directa la pantalla **no se congela** (decisión de diseño), para que el usuario vea en vivo lo que va a grabar.
  - Desde la captura universal, el área ya elegida pasa a la grabación sin volver a seleccionarla.
  - Al confirmar el área, la grabación pasa al estado *Lista*: aparecen la barra de grabación (VID-011) y el resumen Listo para grabar (VID-006). La grabación empieza al pulsar **Grabar**, o sola si está activado VID-007.
  - Solo hay una grabación a la vez:
    - durante una grabación, el atajo **Captura de video** la detiene (VID-013);
    - cualquier otra forma de iniciar un video durante una grabación muestra la barra de la grabación en curso y no crea otra (decisión de diseño);
    - las capturas de imagen sí se permiten durante una grabación (VID-017).
  - **Repetir última captura** (CAP-007) repite una captura de video con sus mismos ajustes; el usuario vuelve a elegir el área salvo si es una región fija con posición.
- **Criterios de aceptación:**
  - [ ] Con la pestaña Imagen activa, el atajo **Captura de video** muestra la cruceta y, tras arrastrar un área, la barra de grabación.
  - [ ] Captura universal → arrastrar 800×600 → `V` muestra la barra de grabación con 800×600 sin pedir otra selección.
  - [ ] Durante una grabación, pulsar **Captura de video** la detiene y el video aparece en la biblioteca.
  - [ ] Mientras se elige el área de un video, un video que se reproduce en pantalla sigue moviéndose.
- **Fuentes:** research/03-video.md §A (Video tab, All-in-One → Video Capture button, Capture Widget, Global capture hotkey, Mac dedicated Video Capture hotkey, Repeat Last Capture)

### VID-003 · Área de grabación
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** selección de región para video; campos Width/Height de la barra de grabación; bloqueos de proporción; guías de video (`G`); mover el área y la barra antes de grabar
- **Comportamiento:**
  - El área se elige y se ajusta como una región de imagen: tiradores, mover y campos X, Y, Ancho y Alto (CAP-011); teclado (CAP-017); bloqueo de proporción 1:1, 16:9 y 4:3 en ambas plataformas (CAP-018); guías de tamaño con `G`, que incluyen 1920×1080, 1280×720, 640×480 y 320×240 (CAP-019); y región fija con o sin posición (CAP-026).
  - Mientras la grabación está en estado *Lista*, el área se puede mover y cambiar de tamaño, y la barra de grabación muestra los campos **Ancho** y **Alto**. Al pulsar **Grabar**, el área queda fija hasta detener o reiniciar.
  - **Tamaños pares** (decisión de diseño): el video necesita un ancho y un alto pares. Si el área es impar, se reduce 1 px por la derecha o por abajo. El rótulo de tamaño y los campos muestran siempre el valor final.
  - **Tamaño mínimo:** 64×64 px. Con un área menor, **Grabar** (y el botón Video de la captura universal) están desactivados y su tooltip explica el motivo.
  - El área puede cruzar monitores; con escalados distintos se aplica la regla de CAP-021. El tamaño máximo lo fija VID-028.
  - Si el video resultante será más pequeño que el área (VID-028), el rótulo lo muestra, por ejemplo "2560×1440 → 1920×1080".
  - El área se marca en pantalla con un borde (VID-020) que no sale en el video.
- **Criterios de aceptación:**
  - [ ] Escribir Ancho = 1280 y Alto = 720 en la barra produce un video de 1280×720 px.
  - [ ] Un área de 801×601 px produce un video de 800×600 px, y el rótulo ya mostraba 800×600 antes de grabar.
  - [ ] Con las guías visibles, al acercar el arrastre a 1920×1080 el área se ajusta a ese tamaño exacto.
  - [ ] Caso límite: un área de 40×40 px no permite grabar y el tooltip de **Grabar** explica el mínimo.
- **Mejora sobre Snagit:** el bloqueo 4:3 y las teclas de proporción son iguales en las dos plataformas; también se pueden escribir X e Y.
- **Fuentes:** research/03-video.md §B (Region selection, Aspect-ratio lock while dragging, Video selection guides, Width/Height fields on the recording toolbar, Move the recording area and toolbar before recording); research/05-plataformas-versiones.md §3 (Video: 4:3 aspect-ratio lock)

### VID-004 · Pantalla completa de un monitor
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** botón Fullscreen; Fullscreen en el menú Selection de Mac; un solo monitor en Mac
- **Comportamiento:**
  - Durante la selección, el botón **Pantalla completa** de cada monitor o la tecla `F` eligen ese monitor entero (CAP-021).
  - En la pestaña Video, Selección = Pantalla completa no muestra cruceta: pasa directamente al estado *Lista*. Su propiedad **Monitor** tiene los mismos valores que CAP-028: Bajo el puntero (por defecto), Principal o un monitor concreto.
  - **Todos los monitores** (`A`) no está disponible para video (decisión de diseño): el tamaño combinado suele superar lo que admite la codificación (VID-028). En la selección de video ese botón no aparece, `A` no hace nada y la ayuda en pantalla lo explica.
  - La barra de grabación se coloca según VID-016 (en otro monitor o minimizada).
  - Si la resolución o el escalado del monitor cambian durante la grabación, el video conserva su tamaño y el contenido se ajusta dentro, con bandas negras si cambia la proporción (decisión de diseño).
  - Si se desconecta el monitor que se graba, la grabación se detiene y se guarda (VID-013), con un aviso.
- **Criterios de aceptación:**
  - [ ] Con dos monitores, `F` sobre el secundario graba solo ese monitor, a su resolución (o al tamaño reducido de VID-028).
  - [ ] Cambiar el escalado del monitor grabado de 100 % a 150 % durante la grabación no la detiene y el video mantiene su tamaño.
  - [ ] Caso límite: desconectar el monitor grabado deja en la biblioteca un video reproducible hasta ese momento.
- **Mejora sobre Snagit:** la misma elección de monitor (bajo el puntero, principal o concreto) en las dos plataformas.
- **Fuentes:** research/03-video.md §B (Fullscreen); research/05-plataformas-versiones.md §3 (Captura: Multi-monitor full screen)

### VID-005 · Grabar una ventana
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** grabación de una ventana que la sigue al moverse (Mac, Selection > Window); en Windows la cruceta se ajusta a la ventana pero graba una región fija
- **Comportamiento:**
  - Con Selección = **Región**, un clic en una ventana resaltada usa su rectángulo como área fija, sin seguirla.
  - Con Selección = **Ventana**, un clic elige la ventana y la grabación la sigue. Propiedad **Modo de ventana**:
    - **Solo la ventana** (por defecto): el video muestra solo el contenido de esa ventana, aunque otras ventanas la tapen, y nada más.
    - **Área de la ventana**: graba lo que se ve en pantalla dentro del rectángulo de la ventana, incluidos sus menús y diálogos y lo que tenga encima; el rectángulo se mueve con la ventana.
  - En ambos modos la grabación sigue a la ventana cuando se mueve, cambia de tamaño o pasa a otro monitor. El video conserva el tamaño inicial; si la ventana crece o se encoge, su imagen se escala para caber, con bandas negras si cambia la proporción (decisión de diseño).
  - Si la ventana se minimiza, pasa a otro escritorio virtual o se oculta, el video conserva el último fotograma hasta que vuelve (decisión de diseño).
  - Si la ventana se cierra, la grabación se detiene y se guarda (VID-013), con un aviso.
  - Las zonas fuera de las esquinas redondeadas de la ventana se rellenan de negro: el video no tiene transparencia.
  - La barra de grabación acompaña a la ventana, a su lado, salvo que el usuario la haya movido (VID-016).
- **Criterios de aceptación:**
  - [ ] Con Solo la ventana, otra ventana arrastrada por encima durante la grabación no aparece en el video.
  - [ ] Si la ventana grabada se mueve a otro monitor durante la grabación, el video la sigue mostrando sin cortes.
  - [ ] Con Área de la ventana, un menú desplegable abierto en la app grabada aparece en el video.
  - [ ] Caso límite: cerrar la ventana grabada detiene la grabación y deja el video en la biblioteca.
- **Notas de plataforma:** con Solo la ventana, los menús y diálogos que la app abre como ventanas aparte no salen en el video en ninguna de las dos plataformas; para incluirlos se usa Área de la ventana. La ayuda lo explica.
- **Mejora sobre Snagit:** existe en Windows, con dos modos a elegir.
- **Fuentes:** research/03-video.md §B (Window selection); research/05-plataformas-versiones.md §1B (2025.0.0, 2026.0.0), §3 (Captura: Window capture), §5

### VID-006 · Resumen Listo para grabar
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Ready to Record window (resumen de dispositivos y ajustes; destino al abrir)
- **Comportamiento:**
  - En el estado *Lista*, junto a la barra de grabación aparece un panel que resume lo que se va a grabar:
    - tamaño del área y del video resultante, calidad, fotogramas por segundo y códec (VID-025 a VID-028);
    - vista previa en vivo de la cámara en su posición y forma (VID-031 a VID-037);
    - micrófono y audio del sistema, cada uno con su medidor de nivel;
    - efectos de cursor, teclas pulsadas, Dibujo en vivo y temporizador;
    - Abrir en y Compartir (VID-052, VID-053);
    - espacio estimado por minuto y tiempo de grabación que cabe en el disco (VID-023).
  - Muestra también los **avisos** que conviene ver antes de empezar, cada uno con su acción: permiso que falta (VID-077), cámara ocupada o sin imagen (VID-038), micrófono silenciado en el sistema (VID-019), dispositivo guardado que no está conectado (VID-043), poco espacio en disco (VID-023), tamaño reducido por límite de codificación (VID-028), compatibilidad de H.265 (VID-027).
  - Botones: **Grabar** (`Enter`), **Capturar imagen** (`I`, VID-010) y **Cancelar** (`Esc`).
  - Preferencia **Mostrar el resumen Listo para grabar** (por defecto: sí), también desde la casilla "No volver a mostrar" del panel. Sin el resumen, los avisos aparecen como una franja en la barra de grabación.
  - El panel desaparece al empezar la grabación y nunca sale en el video.
- **Criterios de aceptación:**
  - [ ] Tras elegir el área, el resumen muestra el micrófono elegido, su medidor se mueve al hablar y la vista previa de la cámara aparece en la esquina configurada.
  - [ ] Con el permiso de micrófono denegado, el aviso aparece en el resumen antes de grabar, con el enlace para resolverlo.
  - [ ] Con la preferencia desactivada no aparece el panel, pero un aviso de poco espacio en disco se muestra como franja en la barra.
  - [ ] El resumen no aparece en ningún fotograma del video.
- **Fuentes:** research/03-video.md §A (Ready to Record window); research/05-plataformas-versiones.md §1A (2024.1.2, 2026.1.0), §1B (2024.2.3)

### VID-007 · Empezar a grabar sin esperar
- **Hito:** M4 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** preferencia "Automatically start video recording"
- **Comportamiento:**
  - Preferencia **Empezar a grabar sin esperar** (por defecto: no). Si está activada, la grabación empieza (tras la cuenta atrás, VID-008) en cuanto se confirma el área, se pulsa **Video** en la captura universal o se elige un monitor o ventana, sin pulsar **Grabar** ni mostrar el resumen.
  - Si hay un aviso que impide grabar bien (falta el permiso de Grabación de pantalla, o el área es menor que el mínimo), la grabación no empieza sola: se muestra el estado *Lista* con el aviso.
  - Los demás avisos (por ejemplo, cámara ocupada) se muestran como franja en la barra durante la grabación.
- **Criterios de aceptación:**
  - [ ] Con la preferencia activada y cuenta atrás de 3 s, soltar el arrastre del área inicia la grabación 3 s después sin más clics.
  - [ ] Sin permiso de Grabación de pantalla (macOS), la grabación no empieza sola y se ve el aviso.
- **Fuentes:** research/03-video.md §A (Automatically start video recording, All-in-One → Video Capture button)

### VID-008 · Cuenta atrás
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** "Show video countdown" (3 segundos)
- **Comportamiento:**
  - Preferencia **Cuenta atrás antes de grabar**: Ninguna, 3 s (por defecto), 5 s o 10 s (decisión de diseño; Snagit solo documenta 3 s).
  - Al pulsar **Grabar** aparecen números grandes en el centro del área (o del monitor, si el área es pequeña). La grabación empieza justo al llegar a cero.
  - La cámara y el micrófono se preparan durante la cuenta atrás, para que el video no empiece con fotogramas negros ni con silencio de arranque.
  - Durante la cuenta atrás:
    - `Esc` la cancela y vuelve al estado *Lista*;
    - pulsar otra vez **Grabar** (botón o atajo) la salta y empieza a grabar al momento;
    - no se puede pausar.
  - Reanudar tras una pausa no tiene cuenta atrás. Reiniciar (VID-014) sí la tiene.
  - La cuenta atrás nunca sale en el video (VID-017).
- **Criterios de aceptación:**
  - [ ] Con 5 s, el primer fotograma del video corresponde a 5 s (±0,2 s) después de pulsar **Grabar**, y ningún fotograma muestra los números.
  - [ ] `Esc` durante la cuenta atrás devuelve al estado *Lista* sin crear ningún archivo.
  - [ ] Pulsar **Grabar** durante la cuenta atrás empieza la grabación en ≤ 200 ms.
- **Mejora sobre Snagit:** duración elegible y forma de saltarla.
- **Fuentes:** research/03-video.md §A (Show video countdown), §S (countdown length)

### VID-009 · Presets de video
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** capture presets con ajustes de video (cámara, micrófono, destino, etc.) y atajo propio
- **Comportamiento:**
  - El sistema de presets (lista, crear, editar, reordenar, atajo por preset, exportar e importar `.xpreset`) es de CAP-058 a CAP-064. Un preset creado desde la pestaña Video guarda todos sus ajustes:
    - Selección con sus propiedades (región fija y posición, modo de ventana, monitor);
    - cámara (activada, dispositivo, colocación, tamaño, forma, proporción, espejo, borde y fondo);
    - micrófono (activado, dispositivo, reducción de ruido, nivel automático) y audio del sistema;
    - cursor, efectos y teclas pulsadas; Dibujo en vivo; temporizador;
    - Abrir en y Compartir.
  - Las preferencias generales de video (calidad, códec, tamaño máximo, cuenta atrás, pistas de audio) no forman parte del preset (decisión de diseño).
  - Al ejecutar un preset se usan **sus** dispositivos, aunque la pestaña Video tenga otros. Si un dispositivo guardado no está conectado, se usa el predeterminado (micrófono) o se desactiva (cámara), y el resumen lo avisa.
  - Los cambios hechos en la barra durante una grabación lanzada desde un preset no modifican el preset (VID-001).
  - Un preset con región fija con posición y con **Empezar a grabar sin esperar** permite grabar siempre la misma zona con una sola pulsación.
- **Criterios de aceptación:**
  - [ ] Un preset guardado con el micrófono "USB A" graba con "USB A" aunque la pestaña Video tenga seleccionado otro micrófono.
  - [ ] Si "USB A" está desconectado, el preset graba con el micrófono predeterminado y el resumen lo indica.
  - [ ] Apagar la cámara en la barra durante una grabación de preset no cambia el preset: la siguiente ejecución vuelve a tener la cámara activada.
- **Fuentes:** research/03-video.md §A (Capture presets for video)

### VID-010 · Convertir una grabación en imagen
- **Hito:** M4 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** cambiar a imagen una captura de video iniciada por error (Mac)
- **Comportamiento:**
  - Comportamiento acordado con CAP-012:
    - en el estado *Lista*, **Capturar imagen** (botón del resumen, menú Más ⋯ de la barra o `I`) toma una imagen del área y cierra la grabación sin crear video;
    - durante la grabación o en pausa, **Capturar imagen** (menú Más ⋯) detiene y descarta lo grabado y toma una imagen del contenido actual del área.
  - La imagen sigue la ruta normal de una captura de imagen (CAP-054) con el Abrir en de la pestaña Video. En el modo Solo la ventana (VID-005) la imagen es la de la ventana, como en CAP-027.
  - Si ya hay más de 10 s grabados, pide confirmación antes de descartar (como VID-014).
- **Criterios de aceptación:**
  - [ ] Desde el estado *Lista*, `I` crea una imagen del área y no deja ningún video en la biblioteca.
  - [ ] A los 5 s de grabación, **Capturar imagen** crea la imagen sin preguntar y no deja ningún video ni archivo temporal.
- **Mejora sobre Snagit:** disponible también en Windows y también durante la grabación.
- **Fuentes:** research/03-video.md §A (Switch an accidental video capture to an image capture)

---

## 2. Barra de grabación y ciclo de vida

### VID-011 · Barra de grabación
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Video Recording toolbar (interruptores de cursor, audio del sistema, cámara y micrófono separados de sus menús; botón Screen Draw; Timer; Record, Pause/Resume, Stop; medidores de nivel; tiempo transcurrido)
- **Comportamiento:**
  - Aparece en el estado *Lista* y se mantiene durante toda la grabación. Controles, de izquierda a derecha:
    - **Cursor**: interruptor y menú con los efectos (VID-044 a VID-047);
    - **Audio del sistema**: interruptor con medidor de nivel (VID-040);
    - **Cámara**: interruptor y menú de opciones (dispositivo, colocación, forma, espejo, borde, fondo), más el botón **Cambiar vista** (VID-035);
    - **Micrófono**: interruptor con medidor de nivel y menú (dispositivo, reducción de ruido, nivel automático);
    - **Dibujo en vivo** (VID-048);
    - **Temporizador** (VID-015);
    - **Tiempo transcurrido** (`hh:mm:ss`; en el estado *Lista* muestra `00:00:00`);
    - **Grabar** / **Pausar** / **Reanudar** (VID-012), **Detener** (VID-013);
    - menú **Más** (⋯): **Reiniciar** y **Cancelar** (VID-014), **Capturar imagen** (VID-010), **Ocultar los íconos del escritorio** (VID-018) y **Ajustes de video…** (Preferencias > Video, APP-006).
  - Cada interruptor está separado de su menú: un clic en el ícono activa o desactiva; un clic en la flecha abre las opciones.
  - **Cambios en vivo:** todos los interruptores y opciones se pueden cambiar en *Lista*, *Grabando* o *En pausa*. Durante la grabación, el cambio afecta al video desde ese momento (por ejemplo, apagar la cámara la quita del video a partir de ese fotograma). Excepciones: el dispositivo de cámara y el de micrófono solo se cambian en *Lista* o *En pausa*, para no producir cortes (decisión de diseño); en *Grabando* esas opciones aparecen desactivadas con un tooltip que lo explica.
  - Los medidores de nivel funcionan también en *Lista*, para probar el sonido antes de grabar.
  - **Teclado:** el atajo **Barra de grabación** (VID-075) lleva el foco a la barra sin interrumpir la grabación. Con el foco en la barra, `Tab` y las flechas recorren los controles y `Enter`/`Espacio` los activan; `Esc` devuelve el foco a la app de antes. Todos los controles tienen nombre accesible y tooltip con su atajo (APP-043, APP-044).
  - La barra usa el tema de la app (APP-012) y nunca sale en el video (VID-017).
- **Criterios de aceptación:**
  - [ ] Desactivar el audio del sistema a los 10 s de grabación produce un video con audio del sistema solo en esos 10 s primeros.
  - [ ] El tiempo transcurrido avanza durante la grabación, se detiene en pausa y nunca muestra un valor vacío.
  - [ ] En *Grabando*, el menú de dispositivos del micrófono está desactivado; en *En pausa* se puede cambiar y la grabación sigue con el nuevo micrófono al reanudar.
  - [ ] Solo con el teclado (atajo Barra de grabación, `Tab`, `Enter`) se puede pausar y detener una grabación.
- **Fuentes:** research/03-video.md §C (Video Recording toolbar, Elapsed recording time display), §F (Audio level meters); research/05-plataformas-versiones.md §1A (2024.0.0)

### VID-012 · Grabar, pausar y reanudar
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Record / Pause / Resume (Shift+F9 en Windows; Control+Shift+Space en Mac)
- **Comportamiento:**
  - **Grabar** empieza la grabación (tras la cuenta atrás, VID-008). El mismo botón pasa a ser **Pausar** y, en pausa, **Reanudar**.
  - El atajo **Grabar / Pausar / Reanudar** (VID-075) hace lo mismo desde cualquier app. También están en el menú del ícono (VID-020) y en los comandos (VID-024).
  - En pausa:
    - no se graba imagen ni sonido; el tiempo transcurrido y el temporizador se detienen;
    - al reanudar, el video sigue sin hueco ni fotograma negro: el instante de la pausa y el de la reanudación quedan seguidos;
    - el usuario puede mover ventanas, cambiar dispositivos o preparar la siguiente escena, y nada de eso queda en el video;
    - el área sigue fija y el borde cambia de aspecto (VID-020).
  - No hay límite de pausas.
  - Pulsar el atajo en *Lista* inicia la grabación. Pulsarlo durante la cuenta atrás la salta (VID-008).
- **Criterios de aceptación:**
  - [ ] Grabar 10 s, pausar 20 s y reanudar 10 s produce un video de 20 s (±0,2 s) sin fotogramas de la pausa.
  - [ ] En el video, el sonido del micrófono sigue sincronizado con la imagen después de 5 pausas.
  - [ ] El atajo pausa la grabación aunque el foco esté en otra app y la barra esté minimizada.
- **Fuentes:** research/03-video.md §C (Record / Pause / Resume / Stop), §P

### VID-013 · Detener
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Stop (Shift+F10 en Windows; Control+Shift+V en Mac)
- **Comportamiento:**
  - **Detener** (botón, atajo **Detener grabación** o **Captura de video** durante una grabación, menú del ícono o `record.stop`) termina la grabación y guarda el video.
  - La barra pasa a *Finalizando* con una indicación de progreso. El video queda guardado en la biblioteca (VID-054) y sigue lo indicado en Abrir en y Compartir (VID-052, VID-053).
  - Tiempo de finalización: ≤ 2 s para una grabación de hasta 30 min en el equipo de referencia de `05-app.md` §11 (decisión de diseño).
  - Detener en pausa guarda lo grabado hasta la pausa.
  - Detener con menos de 1 s grabado no crea video y avisa "Grabación demasiado corta" (decisión de diseño).
  - La grabación también se detiene y se guarda sola, siempre con un aviso que dice el motivo:
    - fin del temporizador (VID-015);
    - espacio en disco agotándose (VID-023);
    - se cierra la ventana grabada (VID-005) o se desconecta el monitor grabado (VID-004);
    - el usuario cierra sesión, apaga o reinicia el equipo, o sale de XcreenShot y elige **Detener y guardar** (APP-004);
    - se retira un permiso necesario durante la grabación (VID-077).
  - Si guardar falla (por ejemplo, la carpeta de la biblioteca no está disponible), el video queda en la carpeta de recuperación y se ofrece en Recuperar trabajo (VID-022).
- **Criterios de aceptación:**
  - [ ] Pulsar el atajo Detener con el foco en otra app guarda el video y lo abre según Abrir en.
  - [ ] Una grabación de 30 min queda lista en la biblioteca ≤ 2 s después de detenerla.
  - [ ] Apagar el equipo durante una grabación no bloquea el apagado y, al volver, el video está en la biblioteca o en Recuperar trabajo.
  - [ ] Caso límite: detener a los 0,5 s no crea ningún video y muestra el aviso.
- **Fuentes:** research/03-video.md §C (Record / Pause / Resume / Stop), §P

### VID-014 · Reiniciar y cancelar
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** reiniciar y cancelar una grabación (comportamiento citado en notas de versión; Esc cancela)
- **Comportamiento:**
  - **Reiniciar** (menú Más ⋯ o atajo, VID-075): descarta lo grabado y vuelve al estado *Lista* con la misma área y los mismos ajustes, incluidos la cámara encendida, su posición y tamaño, y los dispositivos elegidos. Al pulsar **Grabar** hay de nuevo cuenta atrás.
  - **Cancelar** (menú Más ⋯ o atajo): descarta lo grabado y termina sin crear ningún video. La barra, el borde y la cámara se cierran.
  - Si ya hay más de 10 s grabados, ambos piden confirmación ("Se perderán 00:02:15 de grabación") (decisión de diseño).
  - `Esc` (decisión de diseño):
    - en *Selección*, *Lista* y *Cuenta atrás*: cancela sin preguntar;
    - en *Grabando* o *En pausa*: solo actúa si la barra tiene el foco, y equivale a **Cancelar** con su confirmación. Pulsar `Esc` en la app que se está grabando nunca cancela la grabación.
  - Lo descartado no deja archivos temporales ni entradas en la biblioteca ni en Recuperar trabajo.
- **Criterios de aceptación:**
  - [ ] Tras **Reiniciar**, la cámara sigue encendida en la misma esquina y con el mismo tamaño, y el micrófono es el mismo.
  - [ ] **Cancelar** a los 5 s no pide confirmación y no deja ningún archivo en la biblioteca ni en la carpeta temporal.
  - [ ] Pulsar `Esc` dentro de un editor de texto que se está grabando no cancela ni pausa la grabación.
  - [ ] Caso límite: **Cancelar** a los 2 min pide confirmación; si se responde No, la grabación sigue sin perder nada.
- **Fuentes:** research/03-video.md §C (Restart / cancel a recording), §S (Restart/Delete names)

### VID-015 · Temporizador de parada automática
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Timer (Mac 2026.4.0): horas, minutos y segundos; la grabación se detiene al cumplirse
- **Comportamiento:**
  - Botón **Temporizador** en la barra y control equivalente en la pestaña Video: **Sin límite** (por defecto) o una duración en horas, minutos y segundos, de 5 s a 12 h (decisión de diseño). Hay accesos rápidos: 30 s, 1 min, 5 min, 15 min, 30 min, 1 h.
  - Cuenta el **tiempo grabado**, no el reloj: las pausas no consumen tiempo (decisión de diseño).
  - Con temporizador, la barra muestra el tiempo restante junto al transcurrido, y los últimos 10 s se destacan.
  - Al llegar a cero, la grabación se detiene y se guarda como con **Detener** (VID-013).
  - Se puede cambiar durante la grabación. Si el nuevo valor es menor o igual que lo ya grabado, el campo lo rechaza y lo explica.
  - Con un temporizador mayor que la duración recomendada (VID-023), el resumen lo advierte.
  - El comando `record.start` acepta un límite de tiempo (VID-024).
- **Criterios de aceptación:**
  - [ ] Con 30 s, la grabación se detiene sola y el video dura 30 s (±0,2 s).
  - [ ] Con 1 min, grabar 20 s, pausar 1 min y reanudar produce un video de 1 min: la pausa no cuenta.
  - [ ] Durante la grabación, cambiar el temporizador a un valor menor que lo grabado se rechaza con un mensaje.
- **Mejora sobre Snagit:** también en Windows; las pausas no consumen el tiempo.
- **Fuentes:** research/03-video.md §C (Timer (auto-stop)), §S; research/05-plataformas-versiones.md §1B (2026.4.0), §3 (Video: Recording auto-stop timer)

### VID-016 · Posición de la barra y controles minimizados
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** barra arrastrable y colocada debajo o encima del área; preferencia "Minimize controls when recording fullscreen with one monitor" (controles pequeños de Screen Draw, pausa y cámara)
- **Comportamiento:**
  - **Posición por defecto:** fuera del área, centrada debajo; si no cabe, encima; si tampoco, en otro monitor. La barra se puede arrastrar a cualquier sitio antes y durante la grabación, y recuerda su posición durante la sesión.
  - **Pantalla completa con varios monitores:** si el área ocupa todo un monitor y hay otro, la barra va al otro monitor (decisión de diseño).
  - **Pantalla completa con un solo monitor:** preferencia **Minimizar los controles al grabar a pantalla completa con un solo monitor** (por defecto: sí). La barra se sustituye por una pastilla pequeña y arrastrable con el tiempo transcurrido, **Pausar/Reanudar**, **Detener**, **Dibujo en vivo** y **Cámara**. Los botones tienen tooltip. Un clic en el tiempo la expande a la barra completa y otro la vuelve a contraer.
  - Si la preferencia está desactivada, la barra completa queda sobre el área (siempre fuera del video, VID-017).
  - Si el área cambia de sitio (ventana que se mueve, VID-005), la barra la acompaña salvo que el usuario la haya arrastrado.
  - La barra nunca queda fuera de la parte visible de la pantalla, tampoco en escritorios remotos (APP-026). Si su monitor se desconecta, pasa al monitor principal.
- **Criterios de aceptación:**
  - [ ] Con dos monitores y pantalla completa en el primero, la barra aparece en el segundo.
  - [ ] Con un monitor y pantalla completa, aparece la pastilla minimizada; se puede arrastrar y su botón Pausar pausa la grabación.
  - [ ] Con la preferencia desactivada, la barra completa queda visible sobre el área y no aparece en el video.
- **Fuentes:** research/03-video.md §B (Move the recording area and toolbar before recording), §C (Mini or minimized toolbar for fullscreen on one monitor)

### VID-017 · XcreenShot fuera del video
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** ocultar Snagit durante la captura y quitar sus controles de las capturas terminadas (también menús y tooltips de la barra); borde verde de Zoom quitado (Mac); capturas del sistema hechas durante la grabación
- **Comportamiento:**
  - Nunca aparecen en el video, aunque estén dentro del área: la barra de grabación y sus menús, tooltips y avisos; el resumen Listo para grabar; la cuenta atrás; el borde del área; los controles minimizados; la barra de herramientas de Dibujo en vivo (los trazos dibujados sí aparecen, VID-048); las notificaciones de XcreenShot (APP-016).
  - Con **Ocultar XcreenShot al capturar** activada (CAP-023), tampoco aparecen el panel de captura ni el editor. Si el usuario quiere grabar el propio XcreenShot, desactiva esa preferencia: entonces el editor y el panel sí salen, pero los controles de grabación siguen sin salir.
  - Durante una grabación se pueden hacer capturas de imagen (CAP-009, CAP-025): la cruceta y la barra de captura no aparecen en el video, y la imagen no incluye los controles de grabación.
  - **Marcos de otras apps** (decisión de diseño): opción **Quitar los marcos de compartir pantalla de las videollamadas** (por defecto: sí). Los marcos de color que algunas apps de videollamada dibujan alrededor de la pantalla compartida no aparecen en el video.
  - Las capturas que el usuario haga con la herramienta del sistema durante la grabación las gestiona CAP-042.
- **Criterios de aceptación:**
  - [ ] Con la barra arrastrada al centro del área, ningún fotograma del video la muestra, tampoco con su menú Más abierto.
  - [ ] Una captura de región hecha durante la grabación no deja la cruceta en el video.
  - [ ] Con la preferencia CAP-023 desactivada y el editor dentro del área, el editor aparece en el video y la barra de grabación no.
- **Notas de plataforma:**
  - En versiones de Windows 10 en las que el sistema no permite excluir ventanas propias de la captura (APP-040), XcreenShot coloca sus controles fuera del área y lo avisa en el resumen.
  - Quitar los marcos de videollamada: en macOS se pueden excluir del video las ventanas de otras apps. En Windows, al grabar una región o un monitor, el sistema no permite excluir ventanas de otras apps, así que el marco solo se quita al grabar una ventana en modo Solo la ventana (VID-005); en los demás casos la opción aparece desactivada con el motivo.
- **Fuentes:** research/03-video.md §C (Hide Snagit during capture / remove Snagit controls from completed captures, Zoom meeting border removal, System screenshots taken during a recording); research/05-plataformas-versiones.md §1A (2024.1.0), §1B (2025.1.0, 2026.3.0)

### VID-018 · Escritorio limpio
- **Hito:** M4 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Hide desktop icons during video capture (Mac); conmutador en el menú del ícono durante la grabación
- **Comportamiento:**
  - Preferencia **Ocultar los íconos del escritorio al grabar** (por defecto: no). Al empezar a grabar (o al entrar en *Lista*, para que el usuario vea el resultado) se ocultan los íconos del escritorio; las ventanas abiertas se ven igual.
  - Durante la grabación se puede activar o desactivar desde el menú Más ⋯ de la barra y desde el menú del ícono (VID-020).
  - Al terminar o cancelar, los íconos vuelven exactamente como estaban, en sus posiciones.
  - Si XcreenShot se cierra de forma inesperada con los íconos ocultos, los restaura al volver a arrancar (decisión de diseño).
  - Si el usuario ya tenía los íconos ocultos por su cuenta, XcreenShot no los muestra al terminar.
- **Criterios de aceptación:**
  - [ ] Con la preferencia activada, una grabación de pantalla completa no muestra los íconos del escritorio y, al detener, vuelven en las mismas posiciones.
  - [ ] Si se mata el proceso durante la grabación, al volver a abrir XcreenShot los íconos reaparecen.
  - [ ] Con los íconos ya ocultos antes de grabar, siguen ocultos después.
- **Mejora sobre Snagit:** también en Windows.
- **Fuentes:** research/03-video.md §C (Hide desktop icons during video capture); research/05-plataformas-versiones.md §1B (2024.3.0), §3 (Video)

### VID-019 · Aviso de micrófono silenciado
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Muted Mic Alert y preferencia "Show alert when microphone is muted"; Esc lo cierra
- **Comportamiento:**
  - Preferencia **Avisar si el micrófono está silenciado** (por defecto: sí).
  - Tres situaciones, durante *Lista*, *Grabando* o *En pausa*:
    - **Micrófono apagado en la barra y el usuario habla:** aparece en la barra la franja "Parece que hablas, pero el micrófono no graba", con **Activar micrófono** y **Cerrar**. Para detectarlo, XcreenShot escucha el dispositivo sin grabarlo; el sonido se analiza solo en memoria y nunca se guarda (APP-054).
    - **Micrófono activado, pero silenciado en el sistema o con el volumen de entrada a cero:** XcreenShot detecta ese estado (no hace falta hablar) y muestra "El micrófono está silenciado en el sistema", con **Abrir ajustes de sonido**. En *Lista* aparece en el resumen (VID-006).
    - **Micrófono activado y sin ninguna señal durante 10 s seguidos** (por ejemplo, un botón de silencio del propio auricular, que el sistema no comunica): "No llega sonido del micrófono" (decisión de diseño).
  - Detecta voz, no ruido: un ventilador, el teclado o la música no disparan el primer aviso, también con la reducción de ruido activada (VID-041).
  - Tras cerrar un aviso, ese mismo aviso no vuelve a aparecer en 2 minutos en esa grabación (decisión de diseño). `Esc` lo cierra si la barra tiene el foco.
  - Sin el permiso de micrófono, el primer aviso no funciona y la preferencia lo indica.
  - Con la barra minimizada (VID-016), el aviso aparece junto a la pastilla. Nunca sale en el video.
- **Criterios de aceptación:**
  - [ ] Con el micrófono apagado en la barra, hablar 2 s durante la grabación muestra el aviso; **Activar micrófono** lo enciende y el resto se graba con voz.
  - [ ] Un ruido continuo (ventilador) sin voz no muestra el aviso durante 1 minuto.
  - [ ] Tras cerrar el aviso, hablar otra vez durante el minuto siguiente no lo vuelve a mostrar.
  - [ ] Con el micrófono activado y silenciado en los ajustes de sonido del sistema, el resumen muestra "El micrófono está silenciado en el sistema" sin necesidad de hablar.
- **Fuentes:** research/03-video.md §C (Muted Mic Alert); research/05-plataformas-versiones.md §1A (2024.1.0), §1B (2024.2.0)

### VID-020 · Indicadores de grabación
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** ícono de la bandeja que cambia al grabar (Windows); ícono rojo de la barra de menús con opción en blanco y negro (Mac); conmutadores en el menú del ícono
- **Comportamiento:**
  - El ícono de XcreenShot cambia al estado de grabación o de pausa como define APP-002 (forma, no solo color; también en modo monocromo).
  - Entradas de **Estado** del menú rápido (APP-002) durante una grabación:
    - "Grabando · 00:03:12" (o "En pausa · 00:03:12"), que muestra la barra al elegirla;
    - **Pausar** / **Reanudar**, **Detener**, **Reiniciar**, **Cancelar**;
    - **Ocultar los íconos del escritorio** (casilla, VID-018);
    - **Dibujo en vivo** (VID-048).
  - **Borde del área** (decisión de diseño): una línea alrededor del área, fuera de ella, que nunca sale en el video:
    - *Lista*: línea discontinua del color de acento;
    - *Grabando*: línea continua roja;
    - *En pausa*: línea discontinua ámbar con el símbolo de pausa en una esquina.
  - Preferencia **Mostrar el borde del área al grabar** (por defecto: sí). En pantalla completa el borde se dibuja como una franja fina en los bordes del monitor.
  - Los indicadores de grabación no se pueden ocultar por comando (APP-032).
- **Criterios de aceptación:**
  - [ ] Al pasar de *Grabando* a *En pausa*, el ícono y el borde cambian de forma en ≤ 500 ms.
  - [ ] Durante una grabación, el menú del ícono muestra el tiempo, y **Detener** guarda el video.
  - [ ] En una captura de pantalla en escala de grises se distinguen los estados Lista, Grabando y En pausa del borde.
- **Fuentes:** research/03-video.md §C (Recording status indicator, Hide desktop icons during video capture)

### VID-021 · Mantener el equipo despierto
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** prevención de suspensión durante la grabación y la reproducción (Mac)
- **Comportamiento:**
  - Mientras hay una grabación en *Lista*, *Grabando* o *En pausa*, y mientras se reproduce un video en el editor, XcreenShot impide que el equipo se suspenda por inactividad y que la pantalla se apague o arranque el salvapantallas.
  - Al terminar, todo vuelve a funcionar como antes.
  - No impide la suspensión que pide el usuario (cerrar la tapa, menú de apagado). Si el equipo se suspende durante una grabación, la grabación pasa a pausa; al volver, la barra aparece en pausa con un aviso "La grabación se pausó porque el equipo se suspendió" (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con la suspensión del sistema fijada en 1 min, una grabación de 5 min sin tocar el ratón termina bien y la pantalla no se apaga.
  - [ ] Tras detener la grabación, el equipo vuelve a suspenderse según su ajuste.
  - [ ] Suspender el equipo a mano durante una grabación y reanudarlo deja la grabación en pausa y el video se puede guardar.
- **Mejora sobre Snagit:** también en Windows.
- **Fuentes:** research/03-video.md §C (Sleep prevention)

### VID-022 · Recuperar grabaciones interrumpidas
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** recuperación de grabaciones no guardadas tras un fallo (carpeta temporal y "Recover" al reabrir)
- **Comportamiento:**
  - Mientras se graba, el video se escribe en el disco de forma progresiva en una carpeta de recuperación dentro de los datos de la app, no solo en memoria.
  - Si la grabación se interrumpe (fallo de XcreenShot, corte de luz, apagado forzado, error al guardar), la siguiente vez que arranca la app aparece en **Recuperar trabajo** (APP-047) con su fecha, duración y una miniatura.
  - **Recuperar** convierte lo grabado en un MP4 reproducible, lo guarda en la biblioteca (VID-054) con sus metadatos y lo abre según Abrir en. **Descartar** borra los temporales.
  - **Pérdida máxima** (decisión de diseño): los últimos 5 s antes de la interrupción. Imagen y sonido recuperados quedan sincronizados.
  - Vale también para las Diapositivas narradas (VID-071).
  - Al recuperar se respetan las pausas: lo que no se grabó no aparece.
- **Criterios de aceptación:**
  - [ ] Matar el proceso a los 2 min de grabación y volver a abrir la app ofrece recuperar un video de entre 1:55 y 2:00 que se reproduce con sonido sincronizado.
  - [ ] **Descartar** elimina los archivos temporales de esa grabación.
  - [ ] Una grabación terminada normalmente no deja nada en la carpeta de recuperación.
- **Fuentes:** research/03-video.md §D (Crash or interruption recovery); research/05-plataformas-versiones.md §1B (2026.1.0)

### VID-023 · Grabaciones largas y espacio en disco
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** sin límite de duración, con recomendación de grabar 1 hora o menos y combinar clips cortos
- **Comportamiento:**
  - No hay un límite duro de duración. XcreenShot debe grabar sin fallos al menos **4 horas** seguidas a 1080p y 30 fps en el equipo de referencia (decisión de diseño).
  - **Duración recomendada:** 60 min. Al llegar a 60 min aparece una vez en la barra, fuera del video, la sugerencia de detener y seguir en otro clip, con el recordatorio de que los clips se pueden combinar (VID-067).
  - **Espacio en disco:**
    - el resumen (VID-006) muestra los minutos que caben con los ajustes actuales;
    - si caben menos de 10 min, el resumen y la barra lo avisan;
    - si al grabar quedan menos de 1 GB libres en el disco de la biblioteca o en el de la carpeta de recuperación, la grabación se detiene y se guarda sola (VID-013), con el motivo.
  - El uso de memoria no crece con la duración: es el mismo al minuto 1 y al minuto 240 (±10 %).
- **Criterios de aceptación:**
  - [ ] Una grabación de 4 h a 1080p30 termina, se guarda y se reproduce completa, con el sonido sincronizado al final (desfase ≤ 50 ms).
  - [ ] Con 3 GB libres, una grabación se detiene sola al quedar 1 GB, el video se guarda y el aviso dice el motivo.
  - [ ] La sugerencia de los 60 min aparece una sola vez y no sale en el video.
- **Mejora sobre Snagit:** duración probada y protección ante disco lleno.
- **Fuentes:** research/03-video.md §D (Maximum length)

### VID-024 · Comandos de video (CLI y MCP)
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** grabación de video de la API COM (reemplazada por CLI y MCP)
- **Comportamiento:**
  - Comandos (APP-030): `record.start`, `record.pause`, `record.resume`, `record.stop`, `record.cancel`, `record.restart`, `record.status`, más `video.gif`, `video.trim`, `video.combine`, `video.frame`, `video.silent_copy` y `video.info` para la edición (secciones 9 y 10).
  - Parámetros de `record.start`:
    - **origen**: `region` (monitor, x, y, ancho, alto, como CAP-026), `window` (activa, por título o por proceso; modo de VID-005), `screen` (monitor) o `camera`;
    - **preset** (sus ajustes son la base; los demás parámetros los sobrescriben);
    - **camera** (sí/no, dispositivo), **microphone** (sí/no, dispositivo), **system_audio**, **cursor**, **effects**;
    - **limit** (duración, como VID-015);
    - **open_in**, **share** y **file** (ruta de copia adicional).
  - `record.start` vuelve en cuanto empieza la grabación (sin resumen ni cuenta atrás, salvo que se pida `--countdown`). `record.stop` espera a que el archivo esté guardado y devuelve: estado, identificador en la biblioteca, ruta, duración, tamaño en píxeles, fps y tamaño del archivo.
  - `record.status` devuelve el estado (`ready`, `countdown`, `recording`, `paused`, `finalizing` o `idle`; ver los estados en las convenciones de este archivo), el tiempo grabado, el origen y los dispositivos.
  - Si ya hay una grabación, `record.start` responde `busy`. Los demás comandos sin grabación activa responden `not_recording`.
  - Eventos `record.*` y `file.written` según APP-031. Los indicadores de grabación son siempre visibles (APP-032).
- **Criterios de aceptación:**
  - [ ] `record.start --screen primary --limit 10s` graba 10 s y deja un MP4 en la biblioteca; `record.status` durante la grabación devuelve `recording` y el tiempo.
  - [ ] `record.start` con otra grabación en curso devuelve `busy` y no la interrumpe.
  - [ ] Una grabación iniciada por MCP muestra la barra, el borde y la notificación de automatización.
- **Mejora sobre Snagit:** comandos multiplataforma que también pueden usar los agentes.
- **Fuentes:** research/03-video.md §C (Record / Pause / Resume / Stop), §P; research/05-plataformas-versiones.md §3 (Default hotkeys)

---

## 3. Calidad y codificación

### VID-025 · Calidad y fotogramas por segundo
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Video Quality (Windows: Very Low a Very High) y Frame Rate (Mac: Low a Very High); 60 fps; frecuencia de fotogramas variable
- **Comportamiento:**
  - Preferencia **Calidad de video** (Preferencias > Video), con los mismos niveles en las dos plataformas:

    | Nivel | Fotogramas por segundo máximos | Uso típico |
    |---|---|---|
    | Muy baja | 5 | Documentar pasos, archivos mínimos |
    | Baja | 10 | Demostraciones de interfaz sencillas |
    | Media | 15 | Uso general |
    | **Alta** (por defecto) | 30 | Tutoriales con desplazamientos y animaciones |
    | Muy alta | 60 | Movimiento rápido, juegos, video dentro de la pantalla |

  - El nivel por defecto es Alta (decisión de diseño; Snagit usa Media en Windows).
  - Cada nivel fija también la tasa de bits, proporcional al tamaño del video y a los fps, para que el texto de la pantalla se lea sin artefactos en Alta y Muy alta.
  - **Frecuencia variable:** solo se guarda un fotograma nuevo cuando la pantalla cambia, hasta el máximo del nivel. Una pantalla quieta ocupa muy poco. Al reproducir, el video se ve con su ritmo real en cualquier reproductor.
  - Si el equipo no puede mantener los fps (por ejemplo, 60 fps en 4K sin codificación por hardware), graba los que pueda sin perder la sincronía y el resumen lo advierte antes de empezar.
  - La cámara y los efectos de cursor se graban a los mismos fps que la pantalla.
- **Criterios de aceptación:**
  - [ ] En Muy alta, grabar un video que se reproduce a 60 fps produce un archivo con picos de 60 fps (±2) según la información del video (VID-064).
  - [ ] En Media, ningún tramo del video supera 15 fps.
  - [ ] 1 min de pantalla quieta con el reloj del sistema visible ocupa menos de 2 MB en Alta a 1080p.
  - [ ] Un texto de 11 pt grabado en Alta se lee en un fotograma pausado sin bloques visibles.
- **Mejora sobre Snagit:** los mismos niveles con los mismos nombres en Windows y macOS.
- **Fuentes:** research/03-video.md §D (Video Quality (Win), Frame Rate (Mac), Variable frame rate, Recording engines); research/05-plataformas-versiones.md §1A (2024.2.0), §1B (2024.1.0), §3 (Video: 60 fps)

### VID-026 · Formato del archivo
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** salida `.mp4` H.264 por defecto; audio de 128 kbps estéreo
- **Comportamiento:**
  - Las grabaciones se guardan como **`.mp4`** con video **H.264** (por defecto) o H.265 (VID-027) y audio **AAC**, 48 kHz, 128 kbps por pista (decisión de diseño sobre el detalle).
  - Audio: el micrófono en mono (duplicado en los dos canales al combinar) y el audio del sistema en estéreo, conservando izquierda y derecha. Cómo se combinan las pistas: VID-042.
  - Un video sin ninguna fuente de audio no lleva pista de audio.
  - El archivo se puede reproducir sin XcreenShot en los reproductores del sistema de Windows 10/11 y macOS, en los navegadores principales y en servicios de video habituales, y se puede editar en editores de video comunes.
  - Los metadatos de captura (VID-054) se guardan en el índice de la biblioteca (SPEC §7); en el propio MP4 solo se escriben la fecha de creación y el título.
  - La codificación usa los codificadores del sistema (D-04): no se incluye ningún codificador de terceros en la app.
  - **Pantallas HDR** (decisión de diseño): el video se graba siempre en rango estándar (SDR, 8 bits). Si el área está en un monitor en modo HDR, se aplica la misma conversión que a las imágenes, según la preferencia Corrección de color HDR (CAP-022), para que los blancos se vean blancos y los colores no salgan lavados.
- **Criterios de aceptación:**
  - [ ] Un video grabado con XcreenShot se abre y se reproduce con sonido en el reproductor integrado de Windows 10, de Windows 11 y de macOS, y en Chrome, Edge, Firefox y Safari.
  - [ ] Con micrófono y audio del sistema activados y pistas combinadas, el archivo tiene una única pista AAC estéreo de 48 kHz.
  - [ ] Un video grabado sin micrófono ni audio del sistema no tiene pista de audio.
  - [ ] En un monitor HDR, una página blanca grabada da blanco (≥ 250 en los tres canales) en un fotograma del video.
- **Mejora sobre Snagit:** corrección de color en videos grabados en pantallas HDR (Snagit solo la aplica a imágenes).
- **Fuentes:** research/03-video.md §D (Encoding: H.264 (default) or H.265), §F (Audio format, System Audio), §O (MP4 output), §B (HDR)

### VID-027 · Códec H.265
- **Hito:** M4 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** grabación H.265 (Mac 2025.2.0): archivos más pequeños, más CPU, compatibilidad limitada
- **Comportamiento:**
  - Preferencia **Códec**: **H.264 (más compatible)** (por defecto) o **H.265 (archivos más pequeños)**.
  - Junto a la opción H.265 se explica que los archivos ocupan menos (alrededor de la mitad, a igual calidad), que codificar usa más recursos y que algunos equipos, navegadores y servicios no los reproducen (por ejemplo, Windows 10 sin la extensión de video correspondiente).
  - H.265 solo se ofrece si el equipo puede codificarlo; si no, la opción aparece desactivada con el motivo.
  - Con H.265, el resumen (VID-006) muestra un recordatorio de compatibilidad.
  - Las operaciones del editor que generan un video nuevo (cortar, guardar selección, combinar, copia sin sonido) mantienen el códec del video de origen. Combinar videos con códecs distintos produce H.264 (decisión de diseño).
  - Exportar un video H.265 ofrece **Convertir a H.264** para compartirlo con quien no puede reproducirlo (VID-066).
- **Criterios de aceptación:**
  - [ ] Con H.265, 1 min de grabación a 1080p ocupa al menos un 30 % menos que el mismo contenido en H.264 con la misma calidad.
  - [ ] En un equipo sin codificador H.265, la opción aparece desactivada con el motivo.
  - [ ] Un video H.265 convertido a H.264 se reproduce en Windows 10 sin extensiones.
- **Mejora sobre Snagit:** también en Windows, con conversión a H.264 al exportar.
- **Fuentes:** research/03-video.md §D (Encoding: H.264 (default) or H.265), §Q (Windows requirements for video); research/05-plataformas-versiones.md §1B (2025.2.0), §3 (Video: H.265 recording)

### VID-028 · Tamaño del video y límites
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** "Scale captures down to 1920×1080" (Windows), "Downsample Retina video recordings" (Mac); límites de 3840×2160 y 64×64 en modo hardware
- **Comportamiento:**
  - Preferencia **Tamaño máximo del video** (decisión de diseño, une las dos opciones de Snagit):
    - **Original**: el video tiene los píxeles físicos del área.
    - **Reducir pantallas de alta densidad** (por defecto): en monitores con escalado mayor de 100 % (Retina, 150 %, 200 %), el video tiene el tamaño lógico del área; en monitores al 100 % no cambia nada.
    - **Reducir a 1080p**: si el área supera 1920×1080 (en cualquier orientación), el video se reduce hasta caber, manteniendo la proporción.
  - Junto al ajuste se explica que reducir da archivos más pequeños y que Original mantiene más nítido el texto.
  - El ajuste se aplica igual a región, pantalla completa y ventana (VID-005).
  - **Límites del video:** de 64×64 px (VID-003) a 3840×2160 px, o el máximo que admita la codificación del equipo si es mayor (por ejemplo, 4096×2304 o 7680×4320). Si el área supera el máximo, el video se reduce para caber y el rótulo y el resumen lo indican.
  - El tamaño final siempre es par (VID-003).
- **Criterios de aceptación:**
  - [ ] En un Mac Retina, grabar un área de 1000×600 puntos con Reducir pantallas de alta densidad produce un video de 1000×600 px; con Original, de 2000×1200 px.
  - [ ] Con Reducir a 1080p, grabar un monitor de 2560×1440 produce un video de 1920×1080.
  - [ ] En Windows a 100 %, Reducir pantallas de alta densidad no cambia el tamaño.
  - [ ] Caso límite: un área de 5120×2880 en un equipo cuyo máximo es 3840×2160 produce un video de 3840×2160 y el resumen avisaba antes de grabar.
- **Mejora sobre Snagit:** las dos formas de reducir en ambas plataformas, y el límite se avisa antes de grabar en vez de fallar.
- **Fuentes:** research/03-video.md §B (Scale captures down to 1920×1080 / Downsample Retina video recordings, Dimension limits in hardware mode); research/05-plataformas-versiones.md §3 (Video: Retina downsampling, Scale recording to 1920×1080)

### VID-029 · Codificación por hardware o por software
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** "Disable hardware accelerated recording" (Windows) y cambio automático a software ante errores del codificador por hardware; ajustes de registro para el método de captura de pantalla y de audio
- **Comportamiento:**
  - Por defecto se usa la codificación por hardware del equipo si existe.
  - **Cambio automático:** si el codificador por hardware falla al empezar o durante la grabación, XcreenShot sigue con codificación por software sin perder lo grabado y avisa al terminar. Si falla al empezar, la grabación empieza igual con software.
  - Preferencia avanzada **Usar siempre codificación por software** (Preferencias > Video, ajustes avanzados; por defecto: no). Solo hace falta si un controlador de video da problemas. Junto al ajuste se explica que usa más CPU y puede bajar los fps.
  - Preferencia avanzada **Método de captura de pantalla alternativo** (por defecto: no), en la sección Avanzado (APP-009): sirve para equipos en los que la grabación sale en negro o congelada. Al activarlo, XcreenShot usa otro método del sistema para obtener la imagen de la pantalla (decisión de diseño).
  - Si una grabación sale en negro (todos los fotogramas iguales y negros), al terminar se avisa y se sugieren estos dos ajustes.
  - Si el equipo no tiene codificación por hardware, se usa software sin avisar en cada grabación; el resumen lo indica.
- **Criterios de aceptación:**
  - [ ] Con **Usar siempre codificación por software**, una grabación de 1080p30 de 1 min se completa y se reproduce bien.
  - [ ] Simulando un fallo del codificador por hardware a mitad de grabación (prueba de diagnóstico), el video final está completo y aparece el aviso.
  - [ ] Una grabación con todos los fotogramas negros muestra al terminar el aviso con los dos ajustes sugeridos.
- **Mejora sobre Snagit:** opción visible en ambas plataformas, sin tocar el registro.
- **Fuentes:** research/03-video.md §D (Disable hardware accelerated recording, Screen-capture method fallback (registry)); research/05-plataformas-versiones.md §4 (Windows: CPU)

### VID-030 · Sincronía y rendimiento
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** mejor sincronía entre pantalla, micrófono y audio del sistema; motores de grabación más estables y archivos más pequeños
- **Comportamiento:**
  - Requisitos medidos en el equipo de referencia definido en `05-app.md` §11:
    - **Sincronía:** desfase entre imagen, micrófono, audio del sistema y cámara ≤ 40 ms en cualquier punto del video, también tras pausas y en grabaciones largas (VID-023).
    - **Arranque:** desde que termina la cuenta atrás (o se pulsa Grabar sin cuenta atrás) hasta el primer fotograma: ≤ 300 ms.
    - **Carga:** grabar 1080p30 con cámara y micrófono usa de media ≤ 15 % de CPU con codificación por hardware.
    - **Fluidez:** a 1080p30 no se pierde más del 1 % de los fotogramas; la app grabada no se nota más lenta.
    - **Tamaño:** 1 min de una demostración de interfaz típica a 1080p30 en Alta ocupa ≤ 20 MB en H.264.
  - Cada requisito tiene una prueba automática o un procedimiento manual repetible, como en los requisitos no funcionales de `05-app.md` (APP-048 a APP-055).
- **Criterios de aceptación:**
  - [ ] En un video de prueba con una señal de luz y sonido simultáneos cada 10 s durante 30 min, el desfase medido nunca supera 40 ms.
  - [ ] En 20 grabaciones, el primer fotograma llega ≤ 300 ms después de pulsar Grabar (sin cuenta atrás).
  - [ ] La grabación de referencia de 1 min a 1080p30 ocupa ≤ 20 MB.
- **Fuentes:** research/03-video.md §D (Recording engines); research/05-plataformas-versiones.md §1A (2024.0.0, 2024.1.4, 2024.2.0)

---

## 4. Cámara

### VID-031 · Activar la cámara y elegir dispositivo
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** interruptor Camera y selección de dispositivo; cámara activada por defecto; preferencia "Enable webcam"; cámaras virtuales y Continuity Camera
- **Comportamiento:**
  - Interruptor **Cámara** en la pestaña Video y en la barra. Se puede activar o desactivar antes y durante la grabación (VID-011).
  - Menú de dispositivo: todas las cámaras que el sistema ofrece (integradas, USB, iPhone como cámara en macOS, cámaras virtuales de otras apps), con el nombre del sistema. Opción **Predeterminada del sistema**.
  - Preferencia **Cámara activada por defecto en videos nuevos** (por defecto: sí). La configuración administrada la puede fijar (APP-035).
  - Preferencia **Usar la cámara en videos** (por defecto: sí). Si se desactiva, el interruptor desaparece y XcreenShot nunca abre la cámara, lo que evita conflictos con otras apps.
  - XcreenShot solo abre la cámara cuando se ve (vista previa en *Lista*, *Grabando*, *En pausa*) y la libera al terminar, para que el indicador de cámara del sistema se apague.
  - XcreenShot recuerda la cámara elegida. Si no está conectada, usa la predeterminada y lo avisa (VID-043).
  - Las cámaras virtuales funcionan si el sistema las ofrece a las apps; XcreenShot no las trata de forma especial.
- **Criterios de aceptación:**
  - [ ] Con dos cámaras, elegir la segunda muestra su imagen en la vista previa y en el video.
  - [ ] Al detener la grabación, el indicador de cámara del sistema se apaga en ≤ 2 s.
  - [ ] Con **Usar la cámara en videos** desactivado, el indicador de cámara del sistema no se enciende en ningún momento de una grabación.
  - [ ] Una cámara virtual de otra app aparece en la lista y su imagen se graba.
- **Fuentes:** research/03-video.md §E (Camera toggle and device selection, Virtual camera devices, "Enable webcam" / "Enable Camera" preference); research/05-plataformas-versiones.md §1B (2025.2.0)

### VID-032 · Imagen en imagen: posición y tamaño
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Picture-in-Picture (esquinas, arrastrar entre esquinas, redimensionar desde una esquina, ajuste a las esquinas, posición recordada)
- **Comportamiento:**
  - Con la cámara activada y **Colocación = Imagen en imagen** (por defecto), la imagen de la cámara se superpone a la grabación de pantalla.
  - **Posición:** Arriba izquierda, Arriba derecha, Abajo izquierda o **Abajo derecha** (por defecto), con un margen de 24 px respecto al borde del área (decisión de diseño). Se elige en el menú o arrastrando la cámara: al soltarla, se ajusta a la esquina más cercana.
  - **Tamaño:** se cambia arrastrando el tirador de la esquina interior. Va del 10 % al 50 % del lado menor del área; por defecto, 25 % (decisión de diseño). El menú tiene Pequeño, Mediano y Grande.
  - Se puede mover y cambiar de tamaño en *Lista*, *Grabando* y *En pausa*; el video refleja el cambio desde ese momento.
  - Posición, tamaño, forma y proporción se recuerdan entre grabaciones y al reiniciar (VID-014).
  - La cámara queda grabada dentro del video: después no se puede mover ni quitar (VID-065).
- **Criterios de aceptación:**
  - [ ] Arrastrar la cámara cerca de la esquina superior izquierda la deja ajustada a esa esquina y el video la muestra ahí.
  - [ ] Cambiar el tamaño durante la grabación se ve en el video desde ese instante.
  - [ ] Tras una grabación con la cámara en Arriba derecha y tamaño Grande, la siguiente grabación empieza igual.
- **Fuentes:** research/03-video.md §E (Picture-in-Picture (PiP), Webcam is burned in)

### VID-033 · Forma y proporción de la cámara
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** formas Circle, Rectangle, Rounded Rectangle, Square, Rounded Square; proporción 4:3 o 16:9 (Windows)
- **Comportamiento:**
  - **Forma:** Círculo, **Rectángulo redondeado** (por defecto; decisión de diseño), Rectángulo, Cuadrado y Cuadrado redondeado.
  - **Proporción** (solo para Rectángulo y Rectángulo redondeado): **16:9** (por defecto) o 4:3.
  - La imagen de la cámara se recorta por el centro para llenar la forma, sin deformarse.
  - La forma y la proporción se pueden cambiar en *Lista*, *Grabando* y *En pausa*.
- **Criterios de aceptación:**
  - [ ] Con Círculo, el video muestra la cámara en un círculo y fuera de él se ve la pantalla grabada.
  - [ ] Con Rectángulo 4:3, la vista de la cámara en el video tiene proporción 4:3 (±1 px) y la cara no aparece deformada.
- **Mejora sobre Snagit:** la elección 4:3 / 16:9 existe en las dos plataformas.
- **Fuentes:** research/03-video.md §E (PiP shapes, PiP aspect ratio 4:3 or 16:9)

### VID-034 · Cámara a pantalla completa
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** colocación Fullscreen / Fullscreen Camera (solo la cámara, sin pantalla)
- **Comportamiento:**
  - **Colocación = Pantalla completa**: durante la grabación, el video muestra solo la cámara, a todo el tamaño del video, y no la pantalla.
  - **Selección = Solo cámara** (VID-001): graba únicamente la cámara, sin elegir área (decisión de diseño). El tamaño del video es el de la cámara (por ejemplo, 1920×1080), limitado por VID-028. La vista previa aparece en una ventana de XcreenShot con la barra debajo; esa ventana no sale en el video.
  - Si la proporción de la cámara no coincide con la del video, la imagen se recorta por el centro para llenarlo (decisión de diseño).
  - Las opciones de espejo, borde (solo en imagen en imagen) y fondo (VID-036, VID-037) también se aplican.
  - Los efectos de cursor y el Dibujo en vivo no se aplican mientras la cámara ocupa el video.
- **Criterios de aceptación:**
  - [ ] Con Selección = Solo cámara, el video resultante solo contiene la imagen de la cámara y el sonido elegido.
  - [ ] Con Colocación = Pantalla completa sobre un área de 1280×720, el video es de 1280×720 y solo muestra la cámara.
- **Fuentes:** research/03-video.md §E (Fullscreen camera (webcam only))

### VID-035 · Cambiar entre imagen en imagen y pantalla completa
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** cambio con un clic entre cámara a pantalla completa y PiP durante la grabación (botón; Control+Shift+F en Mac)
- **Comportamiento:**
  - Botón **Cambiar vista** en la barra (y en los controles minimizados, VID-016) y atajo **Cambiar vista de cámara** (VID-075): alterna entre imagen en imagen y cámara a pantalla completa en *Lista*, *Grabando* y *En pausa*.
  - El cambio es inmediato, sin corte ni fotograma negro. Al volver a imagen en imagen, la cámara recupera su posición, tamaño y forma.
  - Permite, por ejemplo, empezar presentándose a pantalla completa y pasar luego a la demostración.
  - Si la cámara está apagada, el botón aparece desactivado.
- **Criterios de aceptación:**
  - [ ] Grabar 5 s en imagen en imagen, pulsar el atajo, grabar 5 s y volver a pulsarlo produce un video con los tres tramos y sin fotogramas negros entre ellos.
  - [ ] Tras volver a imagen en imagen, la cámara está en la misma esquina y con el mismo tamaño que antes.
- **Fuentes:** research/03-video.md §E (Swap PiP and Fullscreen during recording), §P; research/05-plataformas-versiones.md §1A (2024.1.0), §1B (2024.0.0)

### VID-036 · Espejo y borde de la cámara
- **Hito:** M4 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Mirror camera (Mac); Camera border sólido o degradado (Mac)
- **Comportamiento:**
  - **Espejo** (por defecto: activado; decisión de diseño): invierte la imagen de izquierda a derecha en la vista previa **y** en el video, para que el presentador se vea como en un espejo. Desactivado, el texto que se muestre a la cámara se lee bien en el video.
  - **Borde** (solo en imagen en imagen): Ninguno (por defecto), **Color sólido** o **Degradado** (dos colores y dirección). Grosor de 2 a 16 px; por defecto 4 px. Los colores se eligen con el selector de color de la app (`EDT-`).
  - El borde sigue la forma de la cámara (círculo, rectángulo redondeado…).
  - Los cambios se ven al momento en la vista previa y se pueden hacer durante la grabación.
- **Criterios de aceptación:**
  - [ ] Con Espejo desactivado, un texto mostrado a la cámara se lee correctamente en el video.
  - [ ] Con un borde degradado de 8 px en forma de círculo, el video muestra un anillo de 8 px con el degradado alrededor de la cámara.
- **Mejora sobre Snagit:** también en Windows.
- **Fuentes:** research/03-video.md §E (Mirror camera, Camera border); research/05-plataformas-versiones.md §1B (2025.4.0), §3 (Video: Camera mirror, border)

### VID-037 · Fondo de la cámara
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Blur camera background (Mac); fondos virtuales con imágenes incluidas o propias (Mac, Apple Silicon)
- **Comportamiento:**
  - Pestaña **Fondo** en las opciones de la cámara (pestaña Video y barra):
    - **Ninguno** (por defecto);
    - **Desenfocar**, con intensidad Baja, Media (por defecto) o Alta;
    - **Imagen**: una de las imágenes incluidas en XcreenShot (al menos 8: colores lisos, degradados, oficina, estantería; todas propias o con licencia libre) o una imagen propia con el botón **+** (PNG, JPG o HEIC). Las imágenes propias se copian a los datos de la app y aparecen en la lista para la próxima vez; cada una se puede quitar.
  - La persona se separa del fondo en el propio equipo, sin conexión (APP-054), en tiempo real y a los fps del video.
  - La vista previa muestra el resultado antes de grabar. El fondo se puede cambiar durante la grabación.
  - Si el equipo no puede separar a la persona del fondo con la fluidez necesaria, la opción aparece desactivada con el motivo; si lo hace con dificultad, el resumen lo advierte.
  - Funciona en imagen en imagen y a pantalla completa (VID-034).
- **Criterios de aceptación:**
  - [ ] Con Desenfocar, el video muestra a la persona nítida y el fondo desenfocado, sin que se desenfoquen la cara ni las manos al moverse.
  - [ ] Una imagen propia añadida con **+** aparece en la lista en la siguiente grabación.
  - [ ] Con una imagen de fondo, durante una grabación de 1 min la cámara mantiene los fps del video (±10 %) en el equipo de referencia.
  - [ ] Con la red desconectada, el fondo funciona igual.
- **Notas de plataforma:** en equipos con poca capacidad de cálculo para este proceso (por ejemplo, algunos Mac con Intel o PC antiguos), Imagen puede no estar disponible y Desenfocar sí; la opción lo indica.
- **Mejora sobre Snagit:** también en Windows, y desenfoque con intensidad elegible.
- **Fuentes:** research/03-video.md §E (Blur camera background, Virtual backgrounds); research/05-plataformas-versiones.md §1B (2024.0.0, 2025.0.0), §2 (Webcam blur / virtual background), §3 (Video)

### VID-038 · Cámara ocupada, tapada o sin permiso
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** mensaje de cámara ocupada (Windows 2026.4.0); aviso al cambiar opciones de cámara sin permiso (Mac 2026.3.0); punto negro o ícono de cámara tachada en el video cuando la cámara está bloqueada
- **Comportamiento:**
  - XcreenShot comprueba la cámara al entrar en *Lista* y, si hay un problema, lo explica en el resumen (VID-006) y en la vista previa, con una acción:
    - **Sin permiso** (macOS, o privacidad de Windows): "XcreenShot no tiene permiso para usar la cámara", con **Reparar permisos** (APP-024, APP-026).
    - **Ocupada por otra app**: "La cámara está en uso por otra aplicación", con el nombre de la app si el sistema lo indica, y **Reintentar**.
    - **Sin imagen** (tapa de privacidad cerrada, imagen negra): "La cámara no envía imagen".
    - **Desconectada**: ver VID-043.
  - Si el problema sigue, el usuario puede **Grabar sin cámara**. En ningún caso se graba un recuadro negro o un ícono en lugar de la cámara: si no hay imagen, la cámara no aparece en el video (decisión de diseño).
  - Si la cámara deja de funcionar durante la grabación, desaparece del video, la grabación sigue y aparece un aviso en la barra con **Reintentar**.
  - Cambiar las opciones de la cámara sin permiso muestra el mismo aviso de permiso.
- **Criterios de aceptación:**
  - [ ] Con la cámara abierta en una app de videollamada (en un sistema que no permite compartirla), el resumen muestra "La cámara está en uso" y **Grabar sin cámara** produce un video sin recuadro de cámara.
  - [ ] Con el permiso de cámara denegado (macOS), activar la cámara muestra el aviso con **Reparar permisos**.
  - [ ] Caso límite: desenchufar la cámara a mitad de grabación quita la cámara del video desde ese momento y la grabación continúa.
- **Fuentes:** research/03-video.md §E (Camera-in-use and permission messaging, Webcam indicator in video, "Enable webcam" / "Enable Camera" preference), §Q (macOS permissions)

---

## 5. Audio

### VID-039 · Micrófono
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** interruptor Microphone y menú de dispositivo; micrófonos integrados, USB y analógicos; interfaces de audio; Automatically adjust input (Mac)
- **Comportamiento:**
  - Interruptor **Micrófono** y menú con:
    - lista de dispositivos de entrada que ofrece el sistema (integrados, USB, analógicos, interfaces de audio, auriculares), más **Predeterminado del sistema** (por defecto);
    - **Reducir ruido de fondo** (VID-041);
    - **Ajustar el nivel automáticamente** (por defecto: activado): mantiene un volumen de voz constante, sube las voces bajas y evita la saturación. Si se desactiva, se usa el nivel del sistema tal cual.
    - **Ajustes de sonido del sistema…**, que abre la página del sistema.
  - Medidor de nivel en vivo en la barra y en el resumen (VID-006, VID-011), también en *Lista*. Una marca indica saturación.
  - Se admiten dispositivos de 44,1 y 48 kHz, 16 y 24 bits, mono o estéreo; el resultado siempre se guarda como en VID-026. En interfaces de varios canales se graba el primer par de entradas (decisión de diseño).
  - Los auriculares inalámbricos funcionan, pero el menú indica junto a ellos que pueden reducir la calidad de la voz (decisión de diseño, aviso informativo).
  - El dispositivo elegido se recuerda; los presets guardan el suyo (VID-009).
  - Sin permiso de micrófono, el aviso aparece antes de grabar (APP-024, VID-077).
- **Criterios de aceptación:**
  - [ ] Con un micrófono USB elegido, el medidor responde al hablar en el estado *Lista* y la voz se oye en el video.
  - [ ] Con Ajustar el nivel automáticamente, una frase dicha en voz baja y otra en voz alta quedan en el video con un volumen parecido (diferencia ≤ 6 dB).
  - [ ] Con una interfaz de audio de 24 bits a 44,1 kHz, el video tiene la voz sin cortes ni cambio de tono.
- **Mejora sobre Snagit:** el nivel automático existe también en Windows.
- **Fuentes:** research/03-video.md §F (Microphone, Automatically adjust input (mic gain leveling), Audio level meters, Mic + call audio); research/05-plataformas-versiones.md §3 (Video: Automatic mic input gain)

### VID-040 · Audio del sistema
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** System Audio (sonidos del sistema, apps y web; captura nativa en macOS 13+; mejora de canales izquierdo y derecho); grabar una llamada con micrófono y audio del sistema
- **Comportamiento:**
  - Interruptor **Audio del sistema** (por defecto: desactivado) con medidor de nivel. Graba lo que suena en el equipo: avisos, apps, navegador, la otra parte de una videollamada.
  - Se graba en estéreo, con izquierda y derecha correctas, sea cual sea el dispositivo de salida (altavoces, auriculares, Bluetooth).
  - No se graban los sonidos de XcreenShot (cuenta atrás, avisos) (decisión de diseño).
  - Si el usuario cambia de dispositivo de salida durante la grabación (por ejemplo, conecta auriculares), el audio del sistema sigue grabándose sin cortes.
  - Para grabar una llamada (micrófono + otra parte), la ayuda recomienda usar auriculares para evitar eco. XcreenShot no cancela el eco.
  - No requiere instalar ningún componente adicional.
- **Criterios de aceptación:**
  - [ ] Con el audio del sistema activado, un video de navegador con sonido queda en el video con el sonido sincronizado (VID-030).
  - [ ] Un tono solo en el canal izquierdo queda solo en el canal izquierdo del video (con pistas combinadas en estéreo).
  - [ ] Conectar unos auriculares Bluetooth durante la grabación no detiene ni corta el audio del sistema grabado.
- **Notas de plataforma:** en macOS, el audio del sistema usa el mismo permiso que la grabación de pantalla (APP-024); no hace falta un permiso aparte.
- **Fuentes:** research/03-video.md §F (System Audio, Mic + call audio)

### VID-041 · Reducción de ruido
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Remove background noise (filtro en vivo de ventiladores, aire acondicionado, conversaciones, teclado, ladridos)
- **Comportamiento:**
  - Opción **Reducir ruido de fondo** en el menú del micrófono (por defecto: desactivada; decisión de diseño).
  - Filtra en vivo, mientras se graba, los ruidos que no son voz: ventiladores, aire acondicionado, zumbidos, tecleo, conversaciones lejanas, animales. La voz principal se mantiene clara.
  - Se procesa en el equipo, sin conexión (APP-054), y se aplica solo al micrófono, no al audio del sistema.
  - Se puede activar o desactivar durante la grabación; el medidor muestra el nivel ya filtrado.
  - El resultado filtrado es el que se graba: no se guarda una copia sin filtrar (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con un ventilador encendido junto al micrófono, el ruido del ventilador en los silencios del video baja al menos 20 dB respecto a la misma grabación sin la opción.
  - [ ] Con la opción activada, una frase de prueba sigue siendo inteligible y no se corta el principio de las palabras.
  - [ ] Con la red desconectada, la opción funciona igual.
- **Fuentes:** research/03-video.md §F (Remove background noise); research/05-plataformas-versiones.md §1A (2025.0.0), §1B (2024.3.0), §2

### VID-042 · Pistas de audio
- **Hito:** M4 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Combine audio tracks (Mac): micrófono y audio del sistema en una pista, o pistas separadas
- **Comportamiento:**
  - Preferencia **Combinar las pistas de audio** (por defecto: sí):
    - **Sí:** micrófono y audio del sistema se mezclan en una sola pista estéreo (VID-026). Es lo más compatible.
    - **No:** el MP4 lleva dos pistas: 1 = micrófono, 2 = audio del sistema, cada una con su nombre. Sirve para ajustar cada una por separado en un editor de video.
  - Con pistas separadas, el reproductor de XcreenShot reproduce las dos mezcladas, y Copia sin sonido (VID-063) quita las dos.
  - El resumen indica que algunos reproductores solo reproducen la primera pista.
- **Criterios de aceptación:**
  - [ ] Con la opción desactivada y las dos fuentes activas, el MP4 tiene dos pistas de audio con los nombres indicados.
  - [ ] Con la opción activada, el MP4 tiene una sola pista en la que se oyen las dos fuentes.
- **Mejora sobre Snagit:** también en Windows.
- **Fuentes:** research/03-video.md §F (Combine audio tracks)

### VID-043 · Conectar y desconectar dispositivos
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** dispositivos detectados sin reiniciar; desconectar el micrófono ya no hace perder el video
- **Comportamiento:**
  - Las listas de cámaras y micrófonos se actualizan solas al conectar o desconectar un dispositivo, sin reiniciar la app.
  - **Antes de grabar:** si el dispositivo guardado no está, el resumen avisa y se usa el predeterminado (micrófono) o se desactiva (cámara). Al conectarlo, vuelve a usarse sin más pasos.
  - **Durante la grabación**, si se desconecta (decisión de diseño):
    - **micrófono**: la grabación sigue; ese tramo queda sin voz, la barra avisa con **Usar otro micrófono** (pausa, cambia y reanuda) y, si el mismo dispositivo vuelve, se reanuda solo;
    - **cámara**: como en VID-038; si vuelve, reaparece en el video;
    - **salida de audio**: no afecta (VID-040).
  - Nunca se pierde lo grabado por un cambio de dispositivo.
- **Criterios de aceptación:**
  - [ ] Desenchufar el micrófono USB a mitad de una grabación de 1 min deja un video completo de 1 min, con voz hasta el corte y la barra avisando.
  - [ ] Volver a enchufar el mismo micrófono sin pausar hace que la voz vuelva a grabarse en ≤ 3 s.
  - [ ] Conectar una cámara nueva con el panel de captura abierto la añade a la lista sin reiniciar.
- **Fuentes:** research/03-video.md §F (Device hot-plug handling)

---

## 6. Cursor, clics y teclas

### VID-044 · Capturar cursor
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** interruptor Capture Cursor; elegir un efecto activa el cursor; al volver a activarlo se recuperan los efectos elegidos
- **Comportamiento:**
  - Interruptor **Cursor** (por defecto: activado) en la pestaña Video y en la barra. Activado, el puntero se ve en el video con su forma real en cada momento (flecha, mano, texto, cambio de tamaño).
  - Desactivado, el puntero no aparece y tampoco los efectos (VID-045, VID-046); las teclas pulsadas (VID-047) no dependen del cursor.
  - Elegir un efecto con el cursor desactivado lo activa.
  - Al desactivar y volver a activar el cursor, vuelven los efectos que estaban elegidos.
  - Se puede cambiar durante la grabación.
- **Criterios de aceptación:**
  - [ ] Con el cursor desactivado, ningún fotograma muestra el puntero ni sus efectos.
  - [ ] Elegir Resaltar cursor con el cursor desactivado lo activa.
  - [ ] Con Resaltar cursor y Animación de clic elegidos, apagar y encender el cursor deja los dos efectos activos.
- **Fuentes:** research/03-video.md §G (Capture Cursor)

### VID-045 · Resaltar cursor
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Highlight Cursor (sus opciones no están documentadas)
- **Comportamiento:**
  - Efecto **Resaltar cursor** (por defecto: desactivado): un círculo de color, semitransparente, centrado en la punta del puntero, que lo sigue en el video. Opciones (decisión de diseño):
    - **Color**: amarillo por defecto; cualquier color del selector de la app. Elegir un color activa el efecto.
    - **Tamaño**: diámetro de 20 a 120 px; por defecto 48 px.
    - **Opacidad**: de 10 % a 90 %; por defecto 40 %.
    - **Forma**: Círculo (por defecto) o Anillo (solo el contorno, de 4 px).
  - El resaltado se dibuja debajo del puntero, sin taparlo.
  - Una vista previa en el menú muestra el resultado.
  - Se puede activar y ajustar durante la grabación.
- **Criterios de aceptación:**
  - [ ] Con color rojo, 60 px y 50 %, el video muestra un círculo rojo semitransparente de 60 px (±2) centrado en la punta del puntero.
  - [ ] El puntero sigue viéndose encima del resaltado.
  - [ ] Elegir un color con el efecto desactivado lo activa.
- **Fuentes:** research/03-video.md §G (Highlight Cursor), §S (Highlight Cursor sub-options)

### VID-046 · Animación de clic
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Click Animation con color personalizable y color distinto para el clic derecho (Mac); conmutable en vivo desde la barra
- **Comportamiento:**
  - Efecto **Animación de clic** (por defecto: desactivado): en cada clic aparece en el video una onda que se expande desde la punta del puntero y se desvanece en unos 400 ms (decisión de diseño).
  - Opciones (decisión de diseño sobre los valores):
    - **Color del clic principal**: azul por defecto;
    - **Color del clic secundario**: naranja por defecto; casilla **Usar el mismo color** para igualarlos;
    - **Tamaño**: Pequeño (40 px), **Mediano** (64 px) o Grande (96 px), diámetro final de la onda.
  - El doble clic muestra dos ondas seguidas. Mantener pulsado y arrastrar muestra un punto fijo de ese color bajo el puntero mientras dura el arrastre (decisión de diseño).
  - Se puede activar y desactivar en vivo desde la barra.
- **Criterios de aceptación:**
  - [ ] Con el efecto activo, cada clic principal produce una onda azul y cada clic secundario una naranja en el fotograma del clic (±1 fotograma).
  - [ ] Con **Usar el mismo color**, los dos tipos de clic producen el mismo color.
  - [ ] La onda desaparece por completo en ≤ 500 ms.
- **Mejora sobre Snagit:** color distinto para el clic secundario en ambas plataformas, y tamaño elegible.
- **Fuentes:** research/03-video.md §G (Click Animation, Right-click animation color), §S; research/05-plataformas-versiones.md §1A (2024.0.0, 2024.0.1), §1B (2024.0.0, 2025.3.0)

### VID-047 · Mostrar teclas pulsadas
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Capture Hotkeys in Video (experimento de Mac): muestra combinaciones y teclas pulsadas en el video
- **Comportamiento:**
  - Efecto **Teclas pulsadas** (por defecto: desactivado), en el menú del cursor y en la pestaña Video.
  - Muestra en el video, en una etiqueta, las teclas que pulsa el usuario. Opciones (decisión de diseño):
    - **Qué mostrar**: **Solo combinaciones** (por defecto; teclas con `Ctrl`, `Alt`, `Cmd`, `Win`, más las teclas especiales: `Enter`, `Esc`, `Tab`, flechas, teclas de función) o **Todas las teclas** (también letras y números).
    - **Posición**: **Abajo en el centro** (por defecto), abajo a la izquierda o abajo a la derecha, dentro del área.
    - **Tamaño**: Pequeño, **Mediano** o Grande.
  - Las combinaciones se muestran con los símbolos de la plataforma (por ejemplo `Ctrl + Shift + S` en Windows; `⌘ ⇧ S` en macOS). Cada etiqueta dura 1,5 s después de soltar la última tecla; si se pulsan varias seguidas, se muestran en fila (máximo 3).
  - **Privacidad:** nunca se muestran las teclas mientras el foco está en un campo de contraseña. Las teclas no se guardan en ningún sitio aparte del video ni en los registros (APP-046).
  - Los atajos de la propia grabación (pausar, detener, Dibujo en vivo) no se muestran.
  - Observar las teclas nunca las bloquea ni las retrasa: si XcreenShot deja de responder, el teclado sigue funcionando con normalidad (como en APP-020).
- **Criterios de aceptación:**
  - [ ] Con Solo combinaciones, pulsar `Ctrl+C` (o `⌘C`) muestra la etiqueta en el video y escribir una palabra no muestra nada.
  - [ ] Con Todas las teclas, escribir "hola" muestra las letras en el video.
  - [ ] Caso límite: escribir en un campo de contraseña no muestra ninguna tecla, ni siquiera con Todas las teclas.
- **Notas de plataforma:** en macOS hace falta el permiso de Accesibilidad para ver las teclas pulsadas en otras apps (APP-024, VID-077); sin él, la opción aparece desactivada con el enlace a Reparar permisos.
- **Mejora sobre Snagit:** función estable en ambas plataformas, con protección de contraseñas.
- **Fuentes:** research/03-video.md §G (Capture Hotkeys in Video (keystroke display)), §S; research/05-plataformas-versiones.md §1B (2024.2.0), §5

---

## 7. Dibujo en vivo

### VID-048 · Modo Dibujo en vivo
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Screen Draw (botón y atajo; las formas se desvanecen; durante el dibujo se puede desplazar con la rueda pero no hacer clic, escribir ni cambiar de app)
- **Comportamiento:**
  - **Dibujo en vivo** permite dibujar formas sobre la pantalla mientras se graba; las formas quedan en el video y desaparecen solas.
  - Se activa y desactiva con el botón de la barra, con el botón de los controles minimizados (VID-016), con el menú del ícono o con el atajo **Dibujo en vivo** (VID-075).
  - Al activarlo:
    - aparece una barra de herramientas pequeña de Dibujo en vivo junto a la barra de grabación (VID-049, VID-050), que no sale en el video;
    - el puntero cambia al de la herramienta activa;
    - los clics y arrastres dibujan en lugar de llegar a las apps; el teclado solo responde a las teclas de Dibujo en vivo y a los atajos globales;
    - la rueda del ratón sigue desplazando la app que hay debajo.
  - Al desactivarlo, el ratón y el teclado vuelven a las apps y las formas que quedan se desvanecen (VID-050).
  - `Esc` desactiva el modo (sin cancelar la grabación).
  - Si se pausa la grabación con el modo activo, se puede seguir dibujando, pero no queda en el video (la pausa no graba).
- **Criterios de aceptación:**
  - [ ] Con el modo activo, una flecha dibujada aparece en el video y la app que está debajo no recibe el clic.
  - [ ] Con el modo activo, la rueda desplaza la página del navegador que hay debajo.
  - [ ] `Esc` desactiva el modo y la grabación sigue.
  - [ ] La barra de herramientas de Dibujo en vivo no aparece en ningún fotograma.
- **Fuentes:** research/03-video.md §H (Screen Draw mode); research/05-plataformas-versiones.md §3 (Video: Screen Draw)

### VID-049 · Herramientas de Dibujo en vivo
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Arrow (A), Square (S), Step (1, reinicia la numeración), Pen (P, Mac), Oval (Mac, experimental), Magnifier (M, dudoso)
- **Comportamiento:**
  - Herramientas, con su tecla mientras el modo está activo:

    | Herramienta | Tecla | Uso |
    |---|---|---|
    | Flecha | `A` | Arrastrar desde la cola a la punta |
    | Rectángulo | `R` (también `S`) | Arrastrar; `Shift` lo hace cuadrado |
    | Óvalo | `O` | Arrastrar; `Shift` lo hace círculo |
    | Lápiz | `P` | Trazo libre |
    | Paso | `1` | Clic: círculo numerado 1, 2, 3…; pulsar `1` otra vez reinicia en 1 |
    | Lupa | `M` | Muestra una lupa circular ×2 que sigue al puntero mientras está elegida (decisión de diseño) |

  - Herramienta por defecto al activar el modo: la última usada (la primera vez, Flecha).
  - `Z` / `Ctrl+Z` / `Cmd+Z` quita la última forma dibujada; `Supr` / `Retroceso` las quita todas al momento (decisión de diseño).
  - La numeración de pasos se mantiene durante toda la grabación hasta que se reinicia.
- **Criterios de aceptación:**
  - [ ] Pulsar `1` y hacer tres clics deja en el video los círculos 1, 2 y 3; pulsar `1` y otro clic muestra de nuevo el 1.
  - [ ] Cada herramienta de la tabla se elige con su tecla y produce su forma en el video.
  - [ ] `Ctrl+Z`/`Cmd+Z` quita solo la última forma.
- **Mejora sobre Snagit:** Lápiz y Óvalo en ambas plataformas; deshacer.
- **Fuentes:** research/03-video.md §H (Tools), §P; research/05-plataformas-versiones.md §1B (2024.1.0, 2025.0.0), §3 (Video: Screen Draw Pen tool), §5

### VID-050 · Color, grosor y desvanecimiento
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** selector de color de Screen Draw (paleta del tema activo); tiempo de desvanecimiento (3 s por defecto) en las opciones de Screen Draw; las formas se desvanecen al salir del modo
- **Comportamiento:**
  - **Color:** paleta con los colores del tema activo del editor (`EDT-`) y un selector de cualquier color. Teclas rápidas de `2` a `7` para los seis primeros colores de la paleta (decisión de diseño). Por defecto, el primer color del tema.
  - **Grosor:** Fino, **Medio** o Grueso (decisión de diseño).
  - **Desvanecimiento:** opción **Las formas desaparecen tras**: de 1 a 30 s, o **Nunca mientras el modo esté activo**; por defecto 3 s. El tiempo cuenta desde que se termina de dibujar cada forma; la forma se desvanece en 0,5 s.
  - Al desactivar el modo, todas las formas que queden se desvanecen en 0,5 s, aunque esté elegido Nunca.
  - Estas opciones están en la barra de Dibujo en vivo y en la pestaña Video (opciones de Dibujo en vivo), y se recuerdan.
- **Criterios de aceptación:**
  - [ ] Con 5 s, una flecha dibujada empieza a desvanecerse a los 5 s (±0,2) y ha desaparecido a los 5,5 s.
  - [ ] Con Nunca, la forma permanece hasta desactivar el modo y entonces desaparece.
  - [ ] Cambiar el color con `3` hace que la siguiente forma tenga el tercer color de la paleta.
- **Fuentes:** research/03-video.md §H (Color, Fade time); research/05-plataformas-versiones.md §1A (2024.0.0)

### VID-051 · Disponibilidad de Dibujo en vivo
- **Hito:** M4 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** interruptor de Screen Draw en la pestaña Video; Screen Draw disponible en cualquier momento (experimento de Mac, útil al compartir pantalla)
- **Comportamiento:**
  - Interruptor **Dibujo en vivo** en la pestaña Video (por defecto: activado): si se desactiva, el botón no aparece en la barra y su atajo no hace nada durante las grabaciones.
  - Preferencia **Dibujo en vivo en cualquier momento** (por defecto: no). Si se activa, el atajo **Dibujo en vivo** activa el modo también sin grabación, sobre el monitor donde está el puntero. Sirve para señalar cosas al compartir pantalla en una videollamada o al presentar.
  - Fuera de una grabación, el modo funciona igual (herramientas, colores, desvanecimiento, `Esc` para salir), con una barra de herramientas flotante que se puede mover. Las formas no se guardan en ningún sitio.
  - Si se inicia una grabación con el modo activo, las formas siguientes ya quedan en el video.
- **Criterios de aceptación:**
  - [ ] Con la preferencia activada y sin grabar, el atajo permite dibujar una flecha sobre cualquier app y `Esc` sale del modo.
  - [ ] Con el interruptor de la pestaña Video desactivado, la barra de grabación no muestra el botón y el atajo no hace nada al grabar.
  - [ ] Con la preferencia desactivada y sin grabar, el atajo no hace nada.
- **Mejora sobre Snagit:** función estable en ambas plataformas.
- **Fuentes:** research/03-video.md §H (Screen Draw on/off at capture start, Screen Draw always available); research/05-plataformas-versiones.md §1B (2026.3.0), §5

---

## 8. Después de detener

### VID-052 · Abrir en
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Open In (Snagit Editor / None); antes "Preview in Editor"
- **Comportamiento:**
  - Opción **Abrir en** de la pestaña Video (y de los presets de video):
    - **Editor** (por defecto): el video terminado se abre en el editor, listo para reproducir y editar (sección 9).
    - **Ninguno (solo biblioteca)**: el video se guarda en la biblioteca y aparece en la bandeja reciente (`TRY-`), sin abrir el editor. Una notificación "Grabación guardada" (APP-016) lo abre al hacer clic.
  - El resumen (VID-006) muestra la opción elegida.
  - En ambos casos se aplican antes los destinos de Compartir (VID-053).
- **Criterios de aceptación:**
  - [ ] Con Abrir en = Editor, al detener aparece el editor con el video en su primer fotograma, listo para reproducir.
  - [ ] Con Abrir en = Ninguno, al detener no se abre ninguna ventana, el video está en la biblioteca y aparece la notificación.
- **Fuentes:** research/03-video.md §I (Open In); research/05-plataformas-versiones.md §1A (2026.1.0), §1B (2025.3.0)

### VID-053 · Compartir al terminar
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** destino de Share en la pestaña Video (se ejecuta solo tras la captura)
- **Comportamiento:**
  - Opción **Compartir** de la pestaña Video: **Ninguno** (por defecto) o uno o varios destinos que acepten video (`SHR-`). Funciona como CAP-052 para imágenes.
  - Al detener, el video se guarda primero en la biblioteca (VID-054) y después se envía a cada destino elegido. El envío se hace en segundo plano y su resultado se notifica (APP-016).
  - Si un destino falla, el video sigue en la biblioteca y la notificación ofrece **Reintentar**.
  - Los destinos y sus opciones (incluido el Enlace para compartir) los define `SHR-`. Los destinos que solo aceptan imágenes no aparecen en la lista.
- **Criterios de aceptación:**
  - [ ] Con Compartir = un destino de carpeta, al detener el MP4 aparece en esa carpeta y en la biblioteca.
  - [ ] Si el destino falla (carpeta sin permiso), el video está igualmente en la biblioteca y la notificación ofrece Reintentar.
- **Fuentes:** research/03-video.md §I (Share destination on the Video tab), §O (Share destinations that accept video)

### VID-054 · Guardado en la biblioteca y metadatos
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** videos `.mp4` en la biblioteca; preferencia de guardar automáticamente los videos nuevos; metadatos de aplicación y sitio web
- **Comportamiento:**
  - Todo video terminado se guarda como `.mp4` en la carpeta de la biblioteca (`LIB-`), con el nombre que define `LIB-`, y se añade al índice. La preferencia de autoguardado de videos la define `LIB-`.
  - Metadatos que guarda (CAP-056 y SPEC §6): fecha y hora, duración, tamaño en píxeles, fps, códec, tamaño del archivo, origen (región, ventana, pantalla, cámara, diapositivas), dispositivos usados, aplicación en primer plano al empezar y, si era un navegador, la URL de la página. Con varias aplicaciones durante la grabación, se guardan todas las que estuvieron en primer plano al menos 5 s (decisión de diseño).
  - Con esos metadatos, la biblioteca puede filtrar por tipo Video, por aplicación y por sitio web (`LIB-`).
  - La miniatura es el fotograma del primer segundo (o el primero si dura menos) (decisión de diseño).
  - El video aparece en la bandeja reciente, donde se puede fijar y etiquetar como cualquier captura (`TRY-`).
- **Criterios de aceptación:**
  - [ ] Grabar un navegador con una página abierta deja en la biblioteca un video con la aplicación y la URL en sus metadatos, y aparece al filtrar por ese sitio.
  - [ ] El video aparece en la bandeja reciente y se puede etiquetar.
  - [ ] Borrar el índice y reconstruirlo (APP-052) conserva la duración y el tipo de los videos.
- **Fuentes:** research/03-video.md §I (Library storage and autosave, Recent Captures Tray)

### VID-055 · Importar videos
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** abrir grabaciones del sistema y videos recibidos por AirDrop; importar `.mov` sin conversión (Mac); arrastrar o abrir MP4 (Windows)
- **Comportamiento:**
  - Se pueden abrir en XcreenShot videos `.mp4`, `.m4v` y `.mov` (H.264 o H.265, con audio AAC u otros que el sistema pueda leer): con Archivo > Abrir, arrastrándolos al editor o a la bandeja, con "Abrir con" del sistema o con el CLI.
  - El video importado se añade a la biblioteca: se copia a su carpeta, sin convertirlo, y conserva su formato (decisión de diseño). Los metadatos que no se conocen quedan vacíos.
  - Con los videos importados funciona toda la edición de la sección 9 y la creación de GIF. Las operaciones que crean un video nuevo producen siempre `.mp4` (VID-026).
  - Las grabaciones de la herramienta del sistema y lo recibido por AirDrop o desde el iPhone se gestionan con CAP-042, CAP-043 y CAP-044 cuando son videos.
  - Si un video no se puede leer (códec no disponible en el sistema), se rechaza con un mensaje que dice el códec.
- **Criterios de aceptación:**
  - [ ] Arrastrar un `.mov` grabado con la herramienta del sistema de macOS al editor lo abre, lo añade a la biblioteca como `.mov` y se puede reproducir y cortar.
  - [ ] En Windows, un `.mp4` de otra app se abre, se reproduce y se puede convertir en GIF.
  - [ ] Un archivo de video con un códec que el sistema no puede leer muestra un mensaje con el nombre del códec y no cierra la app.
- **Mejora sobre Snagit:** importar `.mov` también en Windows (si el sistema puede leerlo).
- **Fuentes:** research/03-video.md §I (Import of external videos), §O (MP4 output)

---

## 9. Edición de video en el editor

### VID-056 · Reproductor de video
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** barra de reproducción (tiempo transcurrido y total, cabezal arrastrable, reproducir/pausar, fotograma anterior y siguiente, silenciar y volumen, Crear GIF, Guardar fotograma, Añadir videos); menú Video con Jump To y atajos; control de audio limitado a silencio y volumen
- **Comportamiento:**
  - Al abrir un video en el editor, el lienzo muestra el video y, debajo, la **barra de reproducción**:
    - tiempo transcurrido y duración total (`hh:mm:ss.f` si dura más de 1 h; `mm:ss.f` si no);
    - línea de tiempo con un **cabezal** que se arrastra para buscar; mientras se arrastra, el lienzo muestra el fotograma de esa posición;
    - **Reproducir/Pausar**;
    - **Fotograma anterior** y **Fotograma siguiente**; mantenerlos pulsados retrocede o avanza de forma continua;
    - **Silenciar** con control de **volumen** (solo afecta a la reproducción, no al archivo);
    - **Velocidad** (VID-057);
    - botones **Crear GIF** (VID-068), **Guardar fotograma** (VID-061) y **Añadir videos** (VID-067).
  - **Menú Video** en la barra de menús del editor, con todas las acciones de esta sección y sus atajos (VID-076): Reproducir/Pausar, Fotograma anterior/siguiente, Ir al inicio, Ir al final, **Ir a…** (escribir un tiempo), Cortar selección, Guardar selección como video, Guardar fotograma, Extraer texto del fotograma, Crear GIF, Combinar videos, Copia sin sonido e Información.
  - La edición de sonido se limita a silenciar o ajustar el volumen al reproducir y a la Copia sin sonido (VID-063).
  - El video se muestra ajustado al lienzo; el zoom del editor (`EDT-`) también funciona para ver detalles.
  - Al abrir un video no se reproduce solo: aparece pausado en el primer fotograma (decisión de diseño).
  - Mientras se reproduce, el equipo no se suspende (VID-021).
- **Criterios de aceptación:**
  - [ ] Arrastrar el cabezal hasta la mitad muestra en el lienzo el fotograma de ese instante mientras se arrastra.
  - [ ] Pulsar **Fotograma siguiente** 30 veces en un video de 30 fps avanza 1 s (±1 fotograma).
  - [ ] **Ir a…** con "01:15" coloca el cabezal en 1 min 15 s.
  - [ ] Silenciar la reproducción no cambia el sonido del archivo exportado.
- **Fuentes:** research/03-video.md §L (Playback toolbar, Video menu), §F (Audio editing in Snagit); research/05-plataformas-versiones.md §1A (2024.0.0), §1B (2024.0.0)

### VID-057 · Velocidad de reproducción y clic para reproducir
- **Hito:** M4 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** velocidad de reproducción solo para revisar (Mac); clic en el video para reproducir o pausar (Mac)
- **Comportamiento:**
  - Selector **Velocidad**: 0,5×, 0,75×, **1×** (por defecto), 1,25×, 1,5× y 2× (decisión de diseño sobre los valores). Solo cambia la reproducción para revisar: el archivo, los cortes y los GIF no cambian. Al cambiar la velocidad, la voz mantiene su tono.
  - **Clic para reproducir:** un clic sobre el video en el lienzo reproduce o pausa. Un clic fuera del video no hace nada. Con el zoom ampliado, arrastrar sigue desplazando la vista en lugar de reproducir.
- **Criterios de aceptación:**
  - [ ] A 2×, un video de 60 s se reproduce en 30 s (±1 s) y exportarlo produce un video de 60 s.
  - [ ] Un clic sobre el video lo reproduce y otro lo pausa.
- **Mejora sobre Snagit:** también en Windows.
- **Fuentes:** research/03-video.md §L (Playback speed, Click to play); research/05-plataformas-versiones.md §1B (2026.1.0)

### VID-058 · Selección en la línea de tiempo
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** tiradores de inicio y fin de la selección; se ajustan al cabezal; vista previa del fotograma al arrastrarlos
- **Comportamiento:**
  - La línea de tiempo tiene un **tirador de inicio** y un **tirador de fin**, con colores y formas distintos (identidad propia), que marcan un tramo. Por defecto el tramo es todo el video y los tiradores están en los extremos.
  - Al arrastrar un tirador, el lienzo muestra el fotograma de su posición y un rótulo indica el tiempo y la duración del tramo.
  - Los tiradores se ajustan al cabezal cuando pasan a 8 px o menos de él (decisión de diseño sobre la distancia).
  - Teclas (decisión de diseño): `I` fija el inicio en el cabezal y `O` el fin; `Esc` vuelve a seleccionar todo el video. Si ya está seleccionado todo el video, `Esc` hace lo que el editor tenga definido para esa tecla (`EDT-`).
  - El tramo seleccionado se resalta en la línea de tiempo y es el que usan Cortar (VID-059), Guardar selección como video (VID-060) y Crear GIF > Selección (VID-068).
  - La selección mínima es de un fotograma.
- **Criterios de aceptación:**
  - [ ] Arrastrar el tirador de inicio cerca del cabezal lo deja exactamente en el tiempo del cabezal.
  - [ ] Con el cabezal en 0:10, `I`, y en 0:20, `O`, el tramo es de 0:10 a 0:20 y el rótulo dice 10 s.
  - [ ] `Esc` vuelve a seleccionar todo el video.
- **Fuentes:** research/03-video.md §L (Selection handles); research/05-plataformas-versiones.md §1B (2024.0.0, 2024.2.0)

### VID-059 · Cortar
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Cut Out (quitar el tramo seleccionado; marca en cada corte; recortar inicio y fin con la misma herramienta; deshacer y rehacer; los cortes se respetan al compartir y al copiar)
- **Comportamiento:**
  - **Cortar selección** (botón con tijeras en la barra de reproducción, menú Video o `Supr`/`Retroceso` con un tramo seleccionado) quita el tramo seleccionado. Las dos partes quedan unidas y una marca en la línea de tiempo señala el punto de corte.
  - Para recortar el principio o el final, se selecciona desde el extremo hasta el punto deseado y se corta.
  - Se pueden hacer varios cortes. **Deshacer** y **Rehacer** (`Ctrl/Cmd+Z`, `Ctrl+Y`/`Cmd+Shift+Z`) funcionan con cada corte, sin límite en la sesión.
  - Los cortes dejan el video **modificado**:
    - todo lo que sale del editor usa la versión cortada: compartir, copiar, arrastrar, exportar, Crear GIF, Combinar, Guardar selección y Copia sin sonido;
    - **Guardar** (`Ctrl/Cmd+S`) aplica los cortes al archivo de la biblioteca; **Guardar como…** crea otro archivo y deja el original;
    - al cerrar o cambiar de elemento con cortes sin guardar, se aplica la misma regla que a un documento de imagen con cambios sin guardar (`EDT-`).
  - Guardar los cortes no cambia la calidad del video y es rápido: ≤ 5 s para un video de 10 min en el equipo de referencia (decisión de diseño).
  - No se puede cortar todo el video: si la selección es el video entero, Cortar aparece desactivado.
  - Mientras hay un tramo seleccionado, los efectos de imagen del editor no están disponibles (VID-065).
- **Criterios de aceptación:**
  - [ ] Cortar 0:10–0:20 de un video de 60 s y guardar produce un video de 50 s en el que el segundo 0:10 muestra lo que antes estaba en 0:20.
  - [ ] Tras tres cortes, deshacer tres veces devuelve el video original.
  - [ ] Con cortes sin guardar, arrastrar el video desde la bandeja a una carpeta produce un archivo con los cortes aplicados.
  - [ ] Caso límite: con todo el video seleccionado, Cortar está desactivado.
- **Fuentes:** research/03-video.md §L (Cut Out (cut and trim), Fast handling of edited videos)

### VID-060 · Guardar la selección como nuevo video
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Save As New Video (guardar el tramo como otro video; repetir para dividir un video largo)
- **Comportamiento:**
  - **Guardar selección como video** (menú Video, menú contextual de la línea de tiempo) crea un video nuevo con el tramo seleccionado. El original no cambia.
  - El nuevo video se guarda en la biblioteca con los metadatos del original más su tramo de origen, y aparece en la bandeja reciente junto al original, sin cambiar lo que se ve en el editor (decisión de diseño).
  - Repetirlo con varios tramos divide un video largo en clips.
  - Si el tramo incluye cortes sin guardar, el nuevo video ya los lleva.
- **Criterios de aceptación:**
  - [ ] Seleccionar 0:30–0:45 y guardarlo crea en la biblioteca un video de 15 s; el original sigue con su duración.
  - [ ] Tres tramos guardados uno tras otro crean tres videos en la bandeja.
- **Fuentes:** research/03-video.md §L (Save As New Video (clip or split)); research/05-plataformas-versiones.md §1A (2024.2.0), §1B (2024.1.0)

### VID-061 · Guardar fotograma como imagen
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Save Frame as Image (el fotograma bajo el cabezal pasa a ser una imagen que se abre en el lienzo)
- **Comportamiento:**
  - **Guardar fotograma** (botón de la barra de reproducción, menú Video o atajo, VID-076) crea una captura de imagen con el fotograma que está bajo el cabezal, al tamaño del video.
  - La imagen se guarda en la biblioteca como un documento de imagen normal (`.xshot`, `EDT-`), con los metadatos del video (aplicación, URL) y el tiempo del fotograma, y se abre en el editor para anotarla.
  - Funciona con el video pausado o en reproducción (en ese caso se pausa primero).
  - Si el video tiene cortes sin guardar, el fotograma corresponde a la versión cortada.
- **Criterios de aceptación:**
  - [ ] Con el cabezal en 0:12, Guardar fotograma abre en el editor una imagen igual a lo que se ve en 0:12, del tamaño del video.
  - [ ] La imagen aparece en la biblioteca con la aplicación de origen del video.
- **Fuentes:** research/03-video.md §L (Save Frame as Image)

### VID-062 · Extraer texto de un fotograma
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Grab Text desde un fotograma de video (Mac), durante la reproducción o en pausa
- **Comportamiento:**
  - **Extraer texto del fotograma** (menú Video, menú contextual del lienzo o atajo de Extraer texto) aplica Extraer texto (SMT- (Extraer texto)) al fotograma bajo el cabezal, sin crear una imagen en la biblioteca.
  - Si el video se está reproduciendo, se pausa primero.
  - El resultado, sus opciones (texto plano, formato, idioma) y el procesamiento en el equipo los define `SMT-`.
- **Criterios de aceptación:**
  - [ ] Con un video pausado en un fotograma con texto, la acción muestra el texto reconocido y no añade ninguna imagen a la biblioteca.
  - [ ] Lanzarla durante la reproducción pausa el video en ese fotograma.
- **Mejora sobre Snagit:** también en Windows.
- **Fuentes:** research/03-video.md §L (Grab Text from a video frame (OCR)); research/05-plataformas-versiones.md §2 (Grab Text), §3 (Editor: Grab Text from a paused video frame)

### VID-063 · Copia sin sonido
- **Hito:** M4 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Make a Silent Copy (duplica el video sin ningún audio; no se puede quitar el sonido de una parte)
- **Comportamiento:**
  - **Copia sin sonido** (menú Video y menú contextual de la bandeja) crea un video nuevo igual al actual, con los cortes aplicados, pero sin ninguna pista de audio. El original no cambia.
  - El nuevo video se guarda en la biblioteca con el mismo nombre y el sufijo "(sin sonido)", y aparece en la bandeja.
  - Si el video no tiene sonido, la acción aparece desactivada.
  - No hay forma de quitar el sonido de un solo tramo; para eso se puede dividir el video (VID-060), hacer la copia sin sonido de una parte y combinarlas (VID-067).
- **Criterios de aceptación:**
  - [ ] La copia sin sonido de un video con micrófono y audio del sistema (también con pistas separadas) no tiene ninguna pista de audio y dura lo mismo.
  - [ ] Con un video sin audio, la acción aparece desactivada.
- **Mejora sobre Snagit:** también en Windows.
- **Fuentes:** research/03-video.md §L (Make a Silent Copy), §F (Audio editing in Snagit); research/05-plataformas-versiones.md §1B (2024.3.2), §3 (Video)

### VID-064 · Información del video
- **Hito:** M4 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Get Info con la frecuencia de fotogramas (Mac)
- **Comportamiento:**
  - **Información** (menú Video, menú contextual de la bandeja y biblioteca, atajo de VID-076) muestra:
    - nombre, ubicación y tamaño del archivo;
    - duración, tamaño en píxeles, fps (media y máximo), códec y tasa de bits;
    - pistas de audio (número, nombre, canales y frecuencia);
    - fecha de creación y metadatos de captura (aplicación, URL, origen, dispositivos) (VID-054).
  - Los valores se pueden copiar.
  - El comando `video.info` devuelve lo mismo en JSON (VID-024).
- **Criterios de aceptación:**
  - [ ] Para un video grabado en Muy alta, Información muestra un fps máximo de 60 (±2).
  - [ ] Para un video con pistas separadas, Información lista dos pistas de audio con sus nombres.
- **Mejora sobre Snagit:** también en Windows.
- **Fuentes:** research/03-video.md §L (Get Info shows frame rate)

### VID-065 · Sin anotaciones sobre el video
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** las herramientas de anotación y los efectos solo valen para imágenes; la cámara queda grabada en el video
- **Comportamiento:**
  - Con un video abierto en el editor, las herramientas de anotación, los efectos, el recorte de lienzo y las demás operaciones de imagen (`EDT-`) aparecen desactivadas. Su tooltip explica la alternativa:
    - para anotar un instante, **Guardar fotograma** (VID-061) y anotar la imagen;
    - para señalar cosas en el video, usar **Dibujo en vivo** al grabar (VID-048).
  - La cámara, los efectos de cursor, las teclas pulsadas y el Dibujo en vivo forman parte de la imagen del video: no se pueden mover, cambiar ni quitar después de grabar.
  - Las únicas ediciones de video son las de esta sección: cortar, guardar tramos, combinar, copia sin sonido y crear GIF.
- **Criterios de aceptación:**
  - [ ] Con un video abierto, la herramienta Flecha está desactivada y su tooltip menciona Guardar fotograma.
  - [ ] Con un video abierto, el menú de efectos del editor no se puede usar.
- **Fuentes:** research/03-video.md §L (No annotations or effects on video), §E (Webcam is burned in)

### VID-066 · Guardar, exportar, copiar y arrastrar videos
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** File > Export para videos (Mac); Copy All de un video; arrastrar videos desde la bandeja; formato por defecto al compartir; sin espera al compartir o duplicar videos editados
- **Comportamiento:**
  - **Guardar como…** y **Exportar…** ofrecen para un video: **MP4** (mismo códec), **MP4 H.264** (convierte un video H.265, VID-027) y **GIF** (abre el diálogo de VID-068).
  - **Copiar todo** (`EDT-`) con un video copia al portapapeles el archivo de video (con los cortes aplicados), para pegarlo en apps que aceptan archivos (chat, correo, carpetas).
  - **Arrastrar** un video desde la bandeja o desde el editor a otra app o a una carpeta entrega el MP4 con los cortes aplicados. El formato por defecto al arrastrar o compartir videos es MP4 (`SHR-`).
  - **Duplicar** (`TRY-`) un video crea una copia en la biblioteca.
  - Compartir, copiar, arrastrar o duplicar un video **sin cortes** es inmediato: no hay espera de procesamiento. Con cortes sin guardar, se genera la versión cortada con una indicación de progreso.
  - Los destinos de compartir y el Enlace para compartir los define `SHR-`.
- **Criterios de aceptación:**
  - [ ] Copiar todo con un video y pegar en una carpeta del sistema crea el MP4.
  - [ ] Exportar un video H.265 como MP4 H.264 produce un archivo H.264 de la misma duración.
  - [ ] Arrastrar desde la bandeja un video sin cortes de 500 MB empieza la copia al momento, sin pantalla de procesamiento.
- **Fuentes:** research/03-video.md §L (File → Export for videos, Fast handling of edited videos), §O (Copy All / drag-out / Default sharing format); research/05-plataformas-versiones.md §1B (2024.2.0, 2024.2.1, 2025.1.0)

### VID-067 · Combinar videos
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Combine Videos (desde Add Videos, menú Video o selección múltiple en la bandeja; lista ordenable; bandeja de orden; cortes aplicados; bordes negros con tamaños distintos; el resultado se abre y los originales se conservan)
- **Comportamiento:**
  - Se inicia con el botón **Añadir videos** de la barra de reproducción, con Video > **Combinar videos** o seleccionando varios videos en la bandeja o la biblioteca y eligiendo **Combinar videos** en el menú contextual.
  - Ventana **Combinar videos**:
    - arriba, los videos de la biblioteca, con miniatura, nombre, duración y tamaño; se ordenan por fecha de modificación (por defecto), fecha de creación, nombre o tamaño, y se filtran por texto;
    - `Espacio` muestra una vista previa del video resaltado;
    - abajo, una **tira de orden** con los videos elegidos (el video abierto, si lo hay, ya está el primero); se añaden con doble clic o arrastrando, se reordenan arrastrando o con `Ctrl/Cmd+←/→`, y se quitan con su ✕;
    - la duración total y el tamaño del resultado se muestran al momento.
  - **Combinar** crea un video nuevo con los elegidos uno tras otro:
    - se aplican los cortes sin guardar de cada video;
    - **tamaño del resultado** (decisión de diseño): el ancho mayor y el alto mayor de los videos; los más pequeños se centran, sin ampliarse, con bandas negras alrededor;
    - fps: el mayor de los videos; códec: el común, o H.264 si son distintos (VID-027);
    - los videos sin sonido aportan silencio; las pistas se combinan según VID-042.
  - La combinación se hace en segundo plano con progreso y **Cancelar**. El resultado se guarda en la biblioteca y se abre en el editor; los originales no cambian.
  - Se necesitan al menos 2 videos; los GIF no se pueden combinar.
- **Criterios de aceptación:**
  - [ ] Combinar un video de 1920×1080 de 10 s y uno de 1280×720 de 5 s produce un video de 1920×1080 de 15 s en el que el segundo aparece centrado con bandas negras.
  - [ ] Reordenar en la tira cambia el orden en el resultado.
  - [ ] Un video con un corte sin guardar entra en la combinación ya cortado.
  - [ ] Cancelar la combinación no deja ningún archivo en la biblioteca.
- **Fuentes:** research/03-video.md §M (Combine Videos); research/05-plataformas-versiones.md §1A (2024.0.0, 2024.2.0), §1B (2024.0.0, 2024.2.5)

---

## 10. GIF animado

### VID-068 · Crear GIF animado
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Create / Save As Animated GIF desde el video (selección o video completo; se abre en el lienzo o al principio de la bandeja); guía de duración y tamaño; desactivado si ya es un GIF; la app sigue usable mientras se crea; cancelar sin fallos
- **Comportamiento:**
  - **Crear GIF** (botón de la barra de reproducción, menú Video, Exportar > GIF o comando `video.gif`) abre un diálogo:
    - **Qué convertir:** **Selección actual** (si hay un tramo seleccionado, VID-058; por defecto en ese caso) o **Video completo**.
    - **Optimizar para** (VID-069).
    - **Estimación** del tamaño del archivo, de la duración y de las dimensiones del GIF, que se actualiza al cambiar cualquier opción.
    - Una vista previa reproduce los primeros segundos con los ajustes elegidos.
  - **Guía de tamaño** (decisión de diseño sobre los límites): si el GIF durará más de 30 s o pesará más de 10 MB, aparece un aviso no bloqueante que sugiere acortar el tramo o reducir el tamaño, porque muchos servicios limitan el tamaño de los GIF y un GIF largo pesa mucho más que el MP4.
  - **Crear** genera el GIF en segundo plano con progreso y **Cancelar**; mientras tanto, la app se puede seguir usando. Cancelar no deja ningún archivo.
  - El GIF se guarda en la biblioteca con los metadatos del video y se abre en el editor (o queda al principio de la bandeja si Abrir en = Ninguno, decisión de diseño). En el editor se reproduce en bucle con el reproductor (VID-056); no se puede cortar ni volver a convertir en GIF.
  - Con un GIF abierto, Crear GIF aparece desactivado. Con varios elementos seleccionados en la bandeja, si alguno no es video, la acción también.
- **Criterios de aceptación:**
  - [ ] Con un tramo de 5 s seleccionado, Crear GIF > Selección actual produce un GIF de 5 s (±0,1 s) que se abre en el editor.
  - [ ] Mientras se crea un GIF de un video de 2 min, se puede hacer una captura de región.
  - [ ] Cancelar a mitad no deja ningún GIF en la biblioteca.
  - [ ] Con un tramo de 45 s, el diálogo muestra el aviso de duración antes de crear.
- **Fuentes:** research/03-video.md §N (Create Animated GIF from video, GIF engine and housekeeping)

### VID-069 · Optimizar para
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Optimize For: Screen Video (por defecto), High Motion Video, Reduced File Size (lado mayor ≤ 200 px), Custom; receta recomendada de alta calidad
- **Comportamiento:**
  - Lista **Optimizar para**, con valores fijados por este spec (decisión de diseño; Snagit no los documenta):

    | Opción | FPS | Tamaño | Difuminado | Colores dinámicos | Repetición |
    |---|---|---|---|---|---|
    | **Grabación de pantalla** (por defecto) | 15 | 100 %, lado mayor ≤ 1280 px | No | Sí | Siempre |
    | Mucho movimiento (cámara, juegos) | 20 | lado mayor ≤ 800 px | Sí | Sí | Siempre |
    | Archivo pequeño | 8 | lado mayor ≤ 200 px | No | No | Siempre |
    | Máxima calidad | 30 | 100 % | No | Sí | Siempre |
    | Personalizado | VID-070 | | | | |

  - Ningún preset sube los fps por encima de los del video ni amplía el video.
  - Al elegir un preset, los campos de Personalizado se rellenan con sus valores, para usarlos como punto de partida.
  - El diálogo recuerda la última opción elegida.
- **Criterios de aceptación:**
  - [ ] Con Archivo pequeño, un video de 1920×1080 produce un GIF de 200×112 px (±1).
  - [ ] Con Grabación de pantalla, un video de 1280×720 a 30 fps produce un GIF de 1280×720 a 15 fps.
  - [ ] Con Máxima calidad, un video de 10 fps produce un GIF de 10 fps (no 30).
- **Mejora sobre Snagit:** la receta de alta calidad está incluida como opción.
- **Fuentes:** research/03-video.md §N ("Optimize For" presets, Recommended high-quality recipe), §S (GIF default values)

### VID-070 · Ajustes personalizados del GIF
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Custom: Frame Rate, Dimensions (unidad, ancho, alto, Lock Aspect Ratio), Enable Dithering, Looping (continuo, una vez, número de veces), Dynamic Colors (Mac), Fade to Black
- **Comportamiento:**
  - Con **Personalizado**, el diálogo muestra:
    - **Fotogramas por segundo:** de 1 a 30 (no más que el video).
    - **Dimensiones:** unidad **Píxeles** o **Porcentaje**, **Ancho**, **Alto** y **Bloquear proporción** (por defecto: activado). Mínimo 16 px por lado; nunca más que el video.
    - **Difuminado** (por defecto: no): mezcla píxeles para suavizar los degradados; reduce las bandas de color, pero aumenta el tamaño.
    - **Colores dinámicos** (por defecto: sí): cada fotograma usa su propia paleta de colores; mejora los colores en videos con escenas distintas, con algo más de tamaño. Desactivado, todo el GIF comparte una paleta.
    - **Repetición:** **Siempre** (por defecto), **Una vez** o **Número de veces** (2 a 100).
    - **Fundido a negro al final** (por defecto: no), con duración de 0,5 a 3 s (por defecto 1 s).
  - Los valores personalizados se recuerdan para el siguiente GIF.
  - La estimación de tamaño (VID-068) se actualiza con cada cambio.
- **Criterios de aceptación:**
  - [ ] Con Porcentaje = 50 % y proporción bloqueada, un video de 1600×900 produce un GIF de 800×450.
  - [ ] Con Repetición = 3 veces, un navegador reproduce el GIF tres veces y se detiene en el último fotograma.
  - [ ] Con Fundido a negro de 1 s, el último fotograma del GIF es negro y el segundo final se oscurece progresivamente.
  - [ ] Activar Difuminado aumenta la estimación de tamaño mostrada.
- **Mejora sobre Snagit:** colores dinámicos en ambas plataformas; número de repeticiones elegible.
- **Fuentes:** research/03-video.md §N (Custom GIF settings, Recommended high-quality recipe), §S (GIF default values)

---

## 11. Diapositivas narradas

### VID-071 · Crear diapositivas narradas
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Video from Images (varias imágenes, narrar mientras se pasan, resultado MP4 o GIF) y Add Narration (Mac: narración sobre una o varias imágenes de la bandeja)
- **Comportamiento:**
  - **Diapositivas narradas** convierte una o varias imágenes en un video: el usuario las va pasando mientras habla, con micrófono y, si quiere, cámara.
  - Se inicia seleccionando una o más imágenes en la bandeja o en la biblioteca (`Ctrl/Cmd`+clic o `Shift`+clic) y eligiendo **Crear > Diapositivas narradas** (menú del editor, menú contextual o atajo, VID-075). Con una sola imagen abierta en el editor también funciona.
  - Valen las imágenes y los documentos de imagen (`.xshot`, con sus anotaciones tal como se ven); no los videos ni las guías de pasos.
  - El editor entra en **modo diapositivas**:
    - el lienzo muestra la diapositiva actual tal como saldrá en el video (VID-074);
    - una tira inferior muestra las diapositivas en orden; se reordenan arrastrando o con `Ctrl/Cmd+←/→`, y se añaden (arrastrando desde la bandeja) o quitan antes y durante la pausa;
    - la barra de diapositivas (VID-072) sustituye a la barra de herramientas del editor.
  - Lo que se graba es el contenido del lienzo, no la pantalla: otras ventanas, notificaciones o el resto del editor no salen en el video (decisión de diseño).
  - Al **Detener**, el video se guarda como MP4 en la biblioteca (VID-054; origen = Diapositivas) y se abre según Abrir en (VID-052). Desde ahí se puede editar, compartir o convertir en GIF (VID-068) como cualquier otro video.
  - Las grabaciones interrumpidas se recuperan como en VID-022.
  - **Salir** del modo sin grabar no cambia ninguna imagen.
- **Criterios de aceptación:**
  - [ ] Seleccionar 3 imágenes en la bandeja y elegir Diapositivas narradas abre el modo con las 3 en la tira, en el orden de selección.
  - [ ] Grabar narrando sobre las 3 y detener deja en la biblioteca un MP4 con las 3 diapositivas y la voz, y una notificación que aparece durante la grabación no sale en el video.
  - [ ] El MP4 resultante se puede convertir en GIF con VID-068.
  - [ ] Con una sola imagen abierta en el editor, la acción también está disponible.
- **Fuentes:** research/03-video.md §K (Create Video from Images, Add Narration), §N (GIF from Video from Images); research/05-plataformas-versiones.md §1B (2024.4.0), §3 (Video: Add Narration)

### VID-072 · Barra y navegación de las diapositivas
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** barra de Video from Images (color de fondo, cursor, micrófono y dispositivo, cámara y dispositivo; Record, Pause, Stop con los mismos atajos); botones Next/Previous Image y flechas (Mac también Cmd+] y Cmd+[)
- **Comportamiento:**
  - **Barra de diapositivas**, de izquierda a derecha:
    - **Fondo** (color, VID-074) y **Tamaño del video** (VID-074);
    - **Cursor**: si está activado, el puntero se ve en el video cuando está sobre la diapositiva, con los efectos de VID-045 y VID-046;
    - **Micrófono** con dispositivo, medidor y reducción de ruido (VID-039, VID-041);
    - **Cámara** con dispositivo y opciones de imagen en imagen, forma, espejo, borde y fondo (VID-031 a VID-037); la cámara se coloca sobre la diapositiva y empieza activada o no según **Cámara activada por defecto en videos nuevos** (VID-031);
    - **Anotar** (VID-073);
    - **Anterior** / **Siguiente** y el número de diapositiva ("3 / 8");
    - tiempo transcurrido, **Grabar** / **Pausar** / **Reanudar**, **Detener**, y en el menú Más ⋯ **Reiniciar** y **Cancelar** (como VID-014).
  - **Grabar**, **Pausar/Reanudar** y **Detener** usan los mismos atajos globales que la grabación de pantalla (VID-075), más la cuenta atrás (VID-008).
  - **Navegación**, en *Lista*, *Grabando* y *En pausa*:
    - **Siguiente**: `→`, `↓`, `Av Pág`, `Espacio` o `Ctrl/Cmd+]`;
    - **Anterior**: `←`, `↑`, `Re Pág` o `Ctrl/Cmd+[`;
    - **Primera** / **Última**: `Inicio` / `Fin`;
    - clic en una miniatura de la tira.
    - Los mandos de presentación que envían `Av Pág` / `Re Pág` funcionan sin configurar nada (decisión de diseño).
  - El cambio de diapositiva es un corte directo, sin transición (decisión de diseño). Cada diapositiva aparece en el video el tiempo que el usuario la deja en pantalla mientras graba; navegar en pausa no queda en el video.
  - En la última diapositiva, **Siguiente** no hace nada; no detiene la grabación (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Durante la grabación, `→` pasa a la siguiente diapositiva y el video muestra el cambio en ese instante.
  - [ ] Un mando de presentación (que envía `Av Pág`) pasa las diapositivas.
  - [ ] Pausar, ir a la diapositiva 5 y reanudar produce un video que salta directamente a la diapositiva 5, sin mostrar las intermedias.
  - [ ] El atajo global Grabar/Pausar funciona en el modo diapositivas igual que en una grabación de pantalla.
- **Mejora sobre Snagit:** las mismas teclas en ambas plataformas y compatibilidad con mandos de presentación.
- **Fuentes:** research/03-video.md §K (Recording toolbar options, Image navigation and reordering), §P

### VID-073 · Anotar las diapositivas en vivo
- **Hito:** M4 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** anotar desde Favoritos durante la grabación o en pausa; las anotaciones se añaden a las imágenes y se pueden quitar después
- **Comportamiento:**
  - El botón **Anotar** de la barra de diapositivas abre un panel con los **Favoritos** del editor (EDT- (Favoritos)).
  - Durante la grabación o en pausa, el usuario elige un favorito y dibuja sobre la diapositiva (arrastrar para flechas y formas; clic para pasos; clic y escribir para texto). La anotación aparece en el video en ese momento y se queda en la diapositiva mientras se sigue en ella y al volver a ella.
  - Las anotaciones se añaden como objetos normales al documento de la imagen (`EDT-`), marcadas como "añadidas al narrar".
  - Al detener, un aviso en el editor ofrece **Quitar las anotaciones añadidas al narrar** (las quita de todas las imágenes de una vez) o **Conservarlas** (decisión de diseño). Después también se pueden quitar una a una o deshacer.
  - Las herramientas de Dibujo en vivo (VID-049, VID-050) también están disponibles en este modo, con desvanecimiento; esas formas no se añaden a las imágenes.
  - `Ctrl/Cmd+Z` deshace la última anotación también durante la grabación; el video muestra cómo desaparece.
- **Criterios de aceptación:**
  - [ ] Durante la grabación, dibujar una flecha con un favorito la muestra en el video desde ese instante y sigue visible al volver a esa diapositiva.
  - [ ] Tras detener, **Quitar las anotaciones añadidas al narrar** deja las imágenes como estaban antes de narrar.
  - [ ] Una forma de Dibujo en vivo en este modo aparece en el video y se desvanece, y no queda en la imagen.
- **Mejora sobre Snagit:** se puede elegir conservar o quitar todas las anotaciones de una vez, y también usar Dibujo en vivo.
- **Fuentes:** research/03-video.md §K (Live annotation from Favorites)

### VID-074 · Tamaño y fondo del video de diapositivas
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Video Background Color; zoom y dimensiones de la imagen en el lienzo (Windows); dimensiones de salida fijas (2240×1260 en Windows, 1150×647 en Mac); sin reducción Retina en este modo (Mac)
- **Comportamiento:**
  - **Tamaño del video** (decisión de diseño; Snagit usa tamaños fijos distintos por plataforma): **1920×1080** (por defecto), 1280×720, 2560×1440, 3840×2160, 1080×1080 (cuadrado), 1080×1920 (vertical) o **Personalizado** (ancho y alto pares, de 320 a 3840). El tamaño se fija antes de grabar y no cambia durante la grabación.
  - El video se genera siempre a ese tamaño en píxeles, sin depender de la pantalla ni de su densidad.
  - **Encaje de cada diapositiva:** la imagen se muestra entera y centrada. Si es más grande que el video, se reduce hasta caber. Si es más pequeña, se muestra a su tamaño real, para no perder nitidez; la opción **Ampliar las imágenes pequeñas** (por defecto: no) las amplía hasta llenar.
  - **Zoom y encuadre por diapositiva:** antes y durante la grabación, la rueda con `Ctrl/Cmd` (o el gesto de pellizcar) acerca o aleja la diapositiva y arrastrar con el botón central o con `Espacio` mantenido la desplaza; `0` vuelve al encaje inicial. El video muestra el zoom con un movimiento suave. Cada diapositiva recuerda su zoom.
  - **Fondo:** color que rellena el espacio alrededor de la imagen; por defecto, negro; cualquier color del selector de la app.
  - La vista del lienzo muestra con una línea el borde del video, para que el usuario vea exactamente lo que se graba.
- **Criterios de aceptación:**
  - [ ] Con 1920×1080 y fondo blanco, una imagen de 800×600 aparece centrada a 800×600 px sobre blanco.
  - [ ] Con 1280×720, una captura de 2560×1600 se reduce a 1152×720 y queda centrada.
  - [ ] En un Mac Retina y en un PC al 100 %, el mismo conjunto de diapositivas produce videos del mismo tamaño en píxeles.
  - [ ] Acercar con `Ctrl/Cmd`+rueda durante la grabación muestra el zoom en el video, y `0` vuelve al encaje inicial.
- **Mejora sobre Snagit:** tamaño elegible e igual en ambas plataformas; zoom en ambas plataformas.
- **Fuentes:** research/03-video.md §K (Recording toolbar options, Canvas zoom and dimensions during Video from Images, Output dimensions); research/05-plataformas-versiones.md §1B (2026.1.0)

---

## 12. Atajos y permisos

### VID-075 · Atajos globales de video
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** atajos de video por defecto: Shift+F9 / Shift+F10 (Windows); Control+Shift+V, Control+Shift+Space, Control+Shift+F, Control+Shift+D (Mac); Ctrl+Shift+D (Windows); Ctrl+Alt+V / Option+V
- **Comportamiento:**
  - Comandos de video con atajo y sus valores por defecto en cada perfil (APP-021, D-08). La gestión (reasignar, conflictos, tomar atajos ocupados) es de APP-017 a APP-023. Todos son globales salvo los marcados "app".

    | Comando | Coexistencia · Windows | Coexistencia · macOS | Compatibilidad con Snagit · Windows | Compatibilidad con Snagit · macOS |
    |---|---|---|---|---|
    | Captura de video (inicia; durante una grabación, la detiene) | `Ctrl+Alt+Shift+V` | `⌃⌥⇧V` | — | `⌃⇧V` |
    | Grabar / Pausar / Reanudar | `Ctrl+Shift+F9` | `⌃⌥⇧Espacio` | `Shift+F9` | `⌃⇧Espacio` |
    | Detener grabación | `Ctrl+Shift+F10` | `⌃⌥⇧V` (la misma de Captura de video) | `Shift+F10` | `⌃⇧V` (la misma de Captura de video) |
    | Dibujo en vivo (activar/desactivar) | `Ctrl+Alt+Shift+D` | `⌃⌥⇧D` | `Ctrl+Shift+D` | `⌃⇧D` |
    | Cambiar vista de cámara (VID-035) | `Ctrl+Alt+Shift+F` | `⌃⌥⇧F` | — | `⌃⇧F` |
    | Barra de grabación (llevar el foco, VID-011) | `Ctrl+Alt+Shift+B` | `⌃⌥⇧B` | `Ctrl+Alt+Shift+B` | `⌃⌥⇧B` |
    | Reiniciar grabación | — | — | — | — |
    | Cancelar grabación | — | — | — | — |
    | Diapositivas narradas (app) | `Ctrl+Alt+V` | `⌥⌘V` | `Ctrl+Alt+V` | `⌥V` |

  - "—" significa sin atajo por defecto; el usuario puede asignar uno. Reiniciar y Cancelar no tienen atajo por defecto para evitar perder una grabación por accidente (decisión de diseño); en la barra siguen disponibles.
  - En macOS, Captura de video y Detener comparten combinación: fuera de una grabación inicia, durante una grabación detiene. En Windows son combinaciones distintas, pero Captura de video también detiene (VID-002).
  - Los atajos de grabación funcionan también en el modo diapositivas (VID-072) y con los controles minimizados (VID-016).
  - Los presets de video tienen su propio atajo (CAP-062).
  - Este archivo es el dueño de los valores de los comandos de video. Si la tabla de atajos de captura (CAP-065, CAP-066) lista también Captura de video, usa estos mismos valores.
  - Mientras la configuración administrada desactiva el video, estos atajos no se registran (APP-035).
- **Criterios de aceptación:**
  - [ ] En una instalación nueva (perfil Coexistencia) con Snagit en marcha, ningún atajo global de esta tabla choca con los de Snagit y todos responden.
  - [ ] Con Compatibilidad con Snagit en Windows, `Shift+F9` empieza a grabar y lo pausa, y `Shift+F10` detiene.
  - [ ] Con Compatibilidad con Snagit en macOS, `⌃⇧V` inicia una captura de video y, durante la grabación, la detiene.
  - [ ] Asignar un atajo a Reiniciar en Preferencias > Atajos hace que esa combinación reinicie la grabación.
- **Notas de plataforma:**
  - En algunos portátiles, las teclas de función (`F9`, `F10`) solo funcionan junto con `Fn` (APP-019).
  - En macOS, `⌥V` (perfil de compatibilidad) escribe un carácter en los campos de texto; cuando un campo de texto del editor tiene el foco, la tecla va al campo y no inicia las diapositivas. Por eso el perfil Coexistencia usa `⌥⌘V`.
- **Fuentes:** research/03-video.md §P, §A (Mac dedicated Video Capture hotkey), §C (Record / Pause / Resume / Stop), §E (Swap PiP and Fullscreen during recording), §H (Screen Draw mode), §K (Create Video from Images); research/05-plataformas-versiones.md §3 (Default hotkeys)

### VID-076 · Teclas contextuales de video
- **Hito:** M4 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** teclas durante la selección de video (G, F, Esc), de Screen Draw (A, S, 1, P, M), de navegación de imágenes y de reproducción (Espacio, flechas, Cmd+Return, Shift+Cmd+Return)
- **Comportamiento:**
  - Teclas que solo existen en un momento concreto (ámbito contextual, APP-017). Son iguales en ambas plataformas, aparecen en la ayuda en pantalla (CAP-020) y en la hoja de atajos (APP-023), y no se reasignan.

    | Momento | Teclas |
    |---|---|
    | Selección del área | Las de CAP-017 (incluidas `G` guías y `F` pantalla completa); `A` no aplica (VID-004) |
    | Estado *Lista* | `Enter` Grabar · `I` Capturar imagen (VID-010) · `Esc` Cancelar |
    | Cuenta atrás | `Esc` vuelve a *Lista* |
    | Barra de grabación con el foco | `Tab`/flechas recorren · `Enter`/`Espacio` activan · `Esc` Cancelar con confirmación (VID-014) |
    | Dibujo en vivo | `A` Flecha · `R`/`S` Rectángulo · `O` Óvalo · `P` Lápiz · `1` Paso (reinicia) · `M` Lupa · `2`–`7` Colores · `Ctrl/Cmd+Z` o `Z` deshacer · `Supr`/`Retroceso` borrar todo · `Esc` salir del modo |
    | Diapositivas narradas | VID-072 (navegación) · `Ctrl/Cmd+←/→` reordenar · `0` encaje inicial |
    | Reproductor del editor | `Espacio` Reproducir/Pausar · `←`/`→` fotograma anterior/siguiente · `Shift+←`/`Shift+→` 1 s atrás/adelante · `Inicio` o `Ctrl/Cmd+Enter` ir al inicio · `Fin` o `Shift+Ctrl/Cmd+Enter` ir al final · `I`/`O` inicio y fin de la selección · `Esc` seleccionar todo · `Supr`/`Retroceso` Cortar selección |

  - Atajos de la app del menú Video (reasignables en Preferencias > Atajos, grupo Video). Solo actúan cuando el elemento activo del editor es un video o un GIF:

    | Acción | Windows | macOS |
    |---|---|---|
    | Ir a… | `Ctrl+J` | `⌘J` |
    | Guardar fotograma | `Ctrl+Alt+F` | `⌥⌘F` |
    | Crear GIF | `Ctrl+Alt+G` | `⌥⌘G` |
    | Información | `Ctrl+Alt+I` | `⌥⌘I` |

  - Si `EDT-` asigna alguna de estas combinaciones a otra acción del editor, se resuelve con las reglas de APP-017 y se anota en `QUESTIONS.md`.
- **Criterios de aceptación:**
  - [ ] Con un video abierto, `Espacio` lo reproduce y lo pausa, y `→` avanza un fotograma.
  - [ ] En el estado *Lista*, `Enter` empieza la cuenta atrás.
  - [ ] Durante Dibujo en vivo, `O` cambia a Óvalo y la siguiente forma es un óvalo.
  - [ ] La hoja de atajos (APP-023) muestra las teclas del reproductor y de Dibujo en vivo.
- **Mejora sobre Snagit:** las mismas teclas en las dos plataformas.
- **Fuentes:** research/03-video.md §P, §B (Video selection guides), §H (Tools), §K (Image navigation and reordering), §L (Playback hotkeys)

### VID-077 · Permisos que necesita el video
- **Hito:** M4 · **Prioridad:** Must
- **Plataformas:** Ambas (con detalle en macOS)
- **Paridad con Snagit:** permisos de macOS para grabación de pantalla y audio del sistema, cámara y micrófono; la captura se reanuda al conceder el permiso; mensajes de privacidad de cámara y micrófono en Windows; contenido protegido
- **Comportamiento:**
  - El flujo de permisos (explicación, solicitud, Reparar permisos, reanudar) es de APP-024 a APP-026. Esta ficha fija qué necesita cada función de video:

    | Función | macOS | Windows |
    |---|---|---|
    | Grabar pantalla, región, ventana (VID-003 a VID-005); efectos de cursor (VID-044 a VID-046); Dibujo en vivo (VID-048) | Grabación de pantalla | — |
    | Audio del sistema (VID-040) | Grabación de pantalla (incluye el audio del sistema) | — |
    | Micrófono (VID-039); aviso de micrófono silenciado (VID-019); narración de diapositivas (VID-071) | Micrófono | Privacidad > Micrófono (acceso de las apps de escritorio) |
    | Cámara (VID-031 a VID-038) | Cámara | Privacidad > Cámara (acceso de las apps de escritorio) |
    | Teclas pulsadas (VID-047) | Accesibilidad (APP-024) | — |
    | Diapositivas narradas sin micrófono ni cámara; edición de video; GIF | — | — |

  - **Antes de grabar:** si falta un permiso que necesita la grabación tal como está configurada, el resumen (VID-006) lo avisa y ofrece **Reparar permisos**. Sin Grabación de pantalla no se puede grabar la pantalla; sin Micrófono o Cámara se puede grabar sin esa fuente, con el aviso.
  - **Reanudar:** al conceder el permiso que faltaba, la grabación pendiente vuelve al estado *Lista* sin repetir la selección (APP-024).
  - **Durante la grabación:** si el sistema retira un permiso, la grabación se detiene y se guarda lo grabado (VID-013) si falta Grabación de pantalla; si falta Micrófono o Cámara, esa fuente se deja de grabar y la grabación sigue, con un aviso.
  - **Contenido protegido:** si el sistema entrega en negro una ventana o un video protegido contra copia, la grabación sigue y, al terminar, se avisa de que esa zona puede verse en negro (APP-026).
  - En ningún caso se graba un video que solo muestra el fondo de escritorio por falta de permiso.
- **Criterios de aceptación:**
  - [ ] (macOS) Con Grabación de pantalla denegada, Captura de video muestra el aviso y no crea ningún video; tras conceder el permiso, aparece el estado *Lista* con la misma área (o tras el reinicio que pida el sistema).
  - [ ] (macOS) Con Micrófono denegado, el resumen avisa antes de grabar, y la grabación se puede hacer sin micrófono.
  - [ ] (Windows) Con la cámara bloqueada en la privacidad de Windows, activar la cámara muestra el aviso con el botón que abre esa página de Configuración.
  - [ ] (macOS) Sin Accesibilidad, Teclas pulsadas aparece desactivado con el enlace a Reparar permisos, y el resto de la grabación funciona.
- **Notas de plataforma:** en versiones de macOS que exijan, además de Accesibilidad, un permiso específico para leer el teclado, Teclas pulsadas lo pide igual que los demás, por el flujo de APP-024.
- **Fuentes:** research/03-video.md §Q (macOS permissions, macOS-version-specific behavior, Windows requirements for video, Protected content), §E (Camera-in-use and permission messaging); research/05-plataformas-versiones.md §1B (2024.1.0)

---

## Relacionado

| Prefijo / archivo | Relación con este archivo |
|---|---|
| `SPEC.md` | Principios §2 (una app, un comportamiento; todo es un comando; local primero), alcance §3.2 (Camtasia, Screencast fuera), hitos §4 (M4), formatos §7 (`.mp4`, `.gif`), nombres §8 (Dibujo en vivo, Diapositivas narradas), decisiones D-04 (codificadores nativos) y D-08 (atajos) |
| `CAP-` (`01-captura.md`) | Panel de captura (CAP-001), Restablecer ajustes (CAP-003), Lanzador de borde (CAP-004), menú del ícono (CAP-005), atajo Capturar (CAP-006), Repetir última captura (CAP-007), captura universal y botón Video (CAP-009, CAP-012), cruceta, lupa, detección y teclado (CAP-013 a CAP-017), proporciones (CAP-018), guías (CAP-019), ayuda en pantalla (CAP-020), multimonitor y DPI (CAP-021), HDR (CAP-022), Ocultar XcreenShot (CAP-023), región fija (CAP-026), ventana (CAP-027), pantalla completa (CAP-028), capturas del sistema, AirDrop e iPhone (CAP-042 a CAP-044), Compartir al capturar (CAP-052), ruta de la captura (CAP-054), metadatos (CAP-056), presets (CAP-058 a CAP-064) |
| `EDT-` (`02-editor.md`) | Editor que aloja el reproductor y la edición de video; herramientas desactivadas con video; selector de color; tema y Favoritos (Dibujo en vivo y Diapositivas narradas); documentos de imagen de los fotogramas guardados; Copiar todo; cambios sin guardar |
| `LIB-`, `TRY-`, `SHR-` (`04-biblioteca-compartir.md`) | Carpeta y nombre de los videos, autoguardado, índice y filtros por aplicación y sitio web; bandeja reciente (fijar, etiquetar, duplicar, selección múltiple); destinos que aceptan video, Enlace para compartir, formato al arrastrar |
| `APP-` (`05-app.md`) | Ícono y menú rápido con el estado de grabación (APP-002), Salir durante una grabación (APP-004), sección Video de Preferencias (APP-008), sección Avanzado (APP-009), notificaciones (APP-016), atajos y perfiles (APP-017 a APP-023), permisos (APP-024 a APP-026), comandos `record.*` y eventos (APP-030, APP-031), indicadores obligatorios (APP-032), configuración administrada `disableVideoCapture` (APP-035), requisitos del sistema (APP-040), accesibilidad (APP-043 a APP-045), Recuperar trabajo (APP-047), requisitos no funcionales (APP-048 a APP-055) |
| `SMT-` (`06-inteligentes.md`) | Extraer texto aplicado a un fotograma (VID-062) |
| `PARITY.md` | Usa los IDs VID- de este archivo |

---

## Excluido

| Elemento de la investigación | Motivo |
|---|---|
| Recorrido guiado "Record a Video" del Learning Center (research/03 §A) | Fuera del alcance: tours y Learning Center (SPEC §3.2). Ayuda propia en `APP-015` |
| Preset incorporado "abrir en Camtasia Editor" (research/03 §A) | Fuera del alcance: Camtasia (SPEC §3.2) |
| Abrir en = Camtasia Editor y Camtasia Online (research/03 §I) | Fuera del alcance: Camtasia (SPEC §3.2). Abrir en queda en Editor / Ninguno (VID-052) |
| Grabación multipista hacia Camtasia Editor y Camtasia Rev, con sus formatos de exportación y marca de agua (research/03 §J) | Fuera del alcance (SPEC §3.2). Candidato futuro: exportar pistas separadas. Hoy solo se pueden separar las pistas de audio (VID-042) |
| Grabar directamente a Camtasia Online (research/03 §J) | Fuera del alcance (SPEC §3.2) |
| Botón Enviar / Editar en Camtasia del editor, también con varios archivos (research/03 §J) | Fuera del alcance (SPEC §3.2) |
| Compartir videos a Camtasia Online (research/03 §J) | Fuera del alcance (SPEC §3.2) |
| Imágenes a video narrado en Camtasia Online, con sus límites (research/03 §J) | Fuera del alcance (SPEC §3.2). La narración local de imágenes está en Diapositivas narradas (VID-071 a VID-074) |
| Grabador de pasos hacia video en Camtasia (research/03 §J) | Fuera del alcance (SPEC §3.2). El grabador de pasos es de `SMT-` |
| Editar imágenes desde Camtasia (research/03 §J) | Fuera del alcance (SPEC §3.2) |
| Cursor como datos separados para Camtasia (research/03 §G) | Fuera del alcance (SPEC §3.2). El cursor queda grabado en el video (VID-044, VID-065) |
| Recuperación de grabaciones enviadas a Camtasia (research/03 §D) | Fuera del alcance (SPEC §3.2). La recuperación local es VID-022 |
| Screencast como alojamiento (comentarios, reacciones, límites de plan, subtítulos, títulos y capítulos con IA, optimización antes de subir) (research/03 §O) | Fuera del alcance (SPEC §3.2). El Enlace para compartir con proveedor configurable es de `SHR-` (D-02) |
| Subtítulos o transcripciones con IA (research/03 §O, §S) | No existen en Snagit (solo en Screencast); funciones generativas o en la nube fuera del alcance (SPEC §3.2) |
| Destinos X/Twitter, Knowmia y Panopto (research/03 §O) | Retirados por TechSmith (SPEC §3.2) |
| Ajustes de registro de Windows para el método de captura de pantalla y para el audio (research/03 §D) | No se replican como ajustes ocultos; la necesidad se cubre con una opción visible de método alternativo (VID-029) |
| Soporte de cámara en Citrix mediante un ajuste de registro (research/03 §E) | No se replica el ajuste. XcreenShot debe funcionar en escritorios remotos y virtuales sin ajustes ocultos (APP-026); la cámara usa lo que el sistema ofrezca en esa sesión |
| Punto negro o ícono de cámara tachada en el video cuando la cámara está bloqueada (research/03 §E) | Es un defecto, no una función: XcreenShot no graba la cámara si no tiene imagen y lo avisa (VID-038) |
| Compatibilidad con sistemas antiguos: audio del sistema en XP/Vista y en macOS 10.14 o anteriores, componente de captura de audio de terceros antes de macOS 13 (research/03 §F) | Sistemas por debajo del mínimo de XcreenShot (APP-040). El audio del sistema no necesita componentes adicionales (VID-040) |
| Correcciones de errores y de versión sin función nueva (por ejemplo: campos de tamaño que fallaban, menús que se colaban en el video, barra en Tahoe y macOS 27, rendimiento de la lupa, cancelar un GIF sin fallo) (research/03 §B, §C, §N, §Q) | Solo son correcciones. Sus resultados esperados ya están en los criterios de aceptación de las fichas correspondientes |
| Actualización de la biblioteca de GIF por una vulnerabilidad (research/03 §N) | Corrección de seguridad (CVE) de un componente de Snagit; no es una función |
| Desactivar la grabación de video por despliegue (research/03 §R) | Cubierto por la configuración administrada (`policy.disableVideoCapture`, APP-035). VID-001 y VID-075 indican su efecto |
| Directiva de cámara activada por defecto (research/03 §R) | Se fija con `defaults`/`locked` sobre el ajuste de VID-031 (APP-035) |
| Bloqueo de destinos y personalización del Enlace para compartir, funciones en prueba y controles de IA, canal de actualizaciones (research/03 §R) | Son de `APP-034`, `APP-035`, `APP-036` y `SHR-` |
| Requisitos de procesador y memoria para grabar, y programas de seguridad que bloquean la captura (research/03 §Q) | Son de `APP-040` y `APP-026` |
| Aviso periódico de macOS 15 para acceder a la pantalla (research/03 §Q) | Es de `APP-025` |
| Entornos virtuales y remotos (research/03 §Q) | Es de `APP-026`; VID-016 garantiza que la barra quede visible |
| Pestaña "Experiments" como vía para activar Teclas pulsadas y Dibujo en vivo en cualquier momento (research/03 §G, §H) | XcreenShot las ofrece como funciones estables (VID-047, VID-051); el mecanismo de funciones en prueba es de `APP-034` |

---

## Cobertura

| Sección de la investigación (research/03-video.md) | Cubierta por |
|---|---|
| §A — Video tab (Capture Window) | VID-001 |
| §A — All-in-One → Video Capture button (incluye recordar ajustes) | VID-002, VID-001, VID-007; CAP-009 |
| §A — Capture Widget | VID-002; CAP-004 |
| §A — Global capture hotkey | VID-002, VID-075; CAP-006 |
| §A — Mac dedicated Video Capture hotkey | VID-002, VID-013, VID-075 |
| §A — Capture presets for video | VID-009; CAP-058 a CAP-064; preset de Camtasia: Excluido |
| §A — Repeat Last Capture | VID-002; CAP-007 |
| §A — Ready to Record window | VID-006 |
| §A — Automatically start video recording | VID-007 |
| §A — Show video countdown | VID-008 |
| §A — Switch an accidental video capture to an image capture | VID-010; CAP-012 |
| §A — Learning Center tour "Record a Video" | Excluido |
| §B — Region selection | VID-003; CAP-011, CAP-013, CAP-015, CAP-017 |
| §B — Window selection | VID-005 |
| §B — Fullscreen | VID-004 |
| §B — Selection gear (more options) | VID-001; CAP-026 |
| §B — Aspect-ratio lock while dragging | VID-003; CAP-018 |
| §B — Video selection guides (G key) | VID-003, VID-076; CAP-019 |
| §B — Width/Height fields on the recording toolbar | VID-003, VID-011 |
| §B — Move the recording area and toolbar before recording | VID-003, VID-016 |
| §B — Scale captures down to 1920×1080 / Downsample Retina | VID-028 |
| §B — Dimension limits in hardware mode | VID-028, VID-003, VID-029 |
| §B — HDR | VID-026; CAP-022 |
| §C — Video Recording toolbar | VID-011 |
| §C — Record / Pause / Resume / Stop | VID-012, VID-013, VID-075; comandos: VID-024 |
| §C — Restart / cancel a recording | VID-014 |
| §C — Timer (auto-stop) | VID-015 |
| §C — Elapsed recording time display | VID-011 |
| §C — Mini or minimized toolbar for fullscreen on one monitor | VID-016 |
| §C — Hide Snagit during capture / remove Snagit controls | VID-017; CAP-023 |
| §C — Hide desktop icons during video capture | VID-018, VID-020 |
| §C — Muted Mic Alert | VID-019 |
| §C — Recording status indicator | VID-020; APP-002 |
| §C — Sleep prevention | VID-021 |
| §C — Zoom meeting border removal | VID-017 |
| §C — System screenshots taken during a recording | VID-017; CAP-042 |
| §D — Video Quality (Win) | VID-025 |
| §D — Frame Rate (Mac) | VID-025 |
| §D — Variable frame rate | VID-025 |
| §D — Encoding: H.264 (default) or H.265 | VID-026, VID-027 |
| §D — Disable hardware accelerated recording | VID-029 |
| §D — Recording engines | VID-025, VID-030 |
| §D — Screen-capture method fallback (registry) | VID-029; ajustes de registro: Excluido |
| §D — Crash or interruption recovery | VID-022; APP-047; recuperación hacia Camtasia: Excluido |
| §D — Maximum length | VID-023 |
| §E — Camera toggle and device selection | VID-031 |
| §E — Picture-in-Picture (PiP) | VID-032 |
| §E — PiP aspect ratio 4:3 or 16:9 | VID-033 |
| §E — PiP shapes | VID-033 |
| §E — Fullscreen camera (webcam only) | VID-034 |
| §E — Swap PiP and Fullscreen during recording | VID-035, VID-075 |
| §E — Mirror camera | VID-036 |
| §E — Camera border | VID-036 |
| §E — Blur camera background | VID-037 |
| §E — Virtual backgrounds | VID-037 |
| §E — Virtual camera devices | VID-031 |
| §E — "Enable webcam" / "Enable Camera" preference | VID-031, VID-038 |
| §E — Camera-in-use and permission messaging | VID-038, VID-077 |
| §E — Webcam indicator in video | VID-038; Excluido (como defecto) |
| §E — Citrix webcam support | Excluido; APP-026 |
| §E — Webcam is burned in | VID-032, VID-065 |
| §F — Microphone | VID-039 |
| §F — System Audio | VID-040; versiones antiguas: Excluido |
| §F — Remove background noise | VID-041 |
| §F — Automatically adjust input (mic gain leveling) | VID-039 |
| §F — Combine audio tracks | VID-042 |
| §F — Audio level meters | VID-011, VID-039, VID-040, VID-006 |
| §F — Audio format | VID-026 |
| §F — Device hot-plug handling | VID-043 |
| §F — Mic + call audio (webinar recording) | VID-040, VID-039 |
| §F — Audio editing in Snagit | VID-056, VID-063 |
| §G — Capture Cursor | VID-044 |
| §G — Highlight Cursor | VID-045 |
| §G — Click Animation | VID-046 |
| §G — Right-click animation color | VID-046 |
| §G — Capture Hotkeys in Video (keystroke display) | VID-047 |
| §G — Cursor in Camtasia multitrack | Excluido |
| §H — Screen Draw mode | VID-048, VID-075 |
| §H — Tools | VID-049, VID-076 |
| §H — Color | VID-050 |
| §H — Fade time | VID-050 |
| §H — Screen Draw on/off at capture start | VID-051 |
| §H — Screen Draw always available | VID-051 |
| §I — Open In | VID-052; opciones de Camtasia: Excluido |
| §I — Share destination on the Video tab | VID-053; `SHR-` |
| §I — Library storage and autosave | VID-054; `LIB-` |
| §I — Recent Captures Tray | VID-054; `TRY-` |
| §I — Import of external videos | VID-055; CAP-042, CAP-043 |
| §J — Record to Camtasia Editor (multitrack) | Excluido |
| §J — Record to Camtasia Online | Excluido |
| §J — Send / Edit in Camtasia | Excluido |
| §J — Share videos to Camtasia Online | Excluido |
| §J — Images → Camtasia Online narrated video | Excluido (alternativa local: VID-071) |
| §J — Step Capture → video | Excluido; grabador de pasos: `SMT-` |
| §J — Edit Images from Camtasia | Excluido |
| §K — Create Video from Images | VID-071, VID-075 |
| §K — Recording toolbar options | VID-072, VID-074 |
| §K — Image navigation and reordering | VID-072, VID-071 |
| §K — Live annotation from Favorites | VID-073 |
| §K — Canvas zoom and dimensions during Video from Images | VID-074 |
| §K — Output dimensions | VID-074 |
| §K — Add Narration | VID-071 |
| §L — Playback toolbar | VID-056 |
| §L — Video menu | VID-056, VID-076 |
| §L — Playback speed | VID-057 |
| §L — Click to play | VID-057 |
| §L — Playback hotkeys | VID-076 |
| §L — Selection handles | VID-058 |
| §L — Cut Out (cut and trim) | VID-059 |
| §L — Save As New Video (clip or split) | VID-060 |
| §L — Save Frame as Image | VID-061 |
| §L — Grab Text from a video frame (OCR) | VID-062; `SMT-` |
| §L — Make a Silent Copy | VID-063 |
| §L — Get Info shows frame rate | VID-064 |
| §L — No annotations or effects on video | VID-065 |
| §L — File → Export for videos | VID-066 |
| §L — Fast handling of edited videos | VID-066, VID-059 |
| §M — Combine Videos | VID-067 |
| §N — Create Animated GIF from video | VID-068 |
| §N — "Optimize For" presets | VID-069 |
| §N — Custom GIF settings | VID-070 |
| §N — Recommended high-quality recipe | VID-069, VID-070 |
| §N — GIF from Video from Images | VID-071, VID-068 |
| §N — GIF engine and housekeeping | VID-068; actualización por CVE: Excluido |
| §O — MP4 output | VID-026, VID-055 |
| §O — Share Link | `SHR-`; Screencast: Excluido |
| §O — Screencast (TechSmith hosting) | Excluido |
| §O — Share destinations that accept video | VID-053; `SHR-`; destinos retirados y de Camtasia: Excluido |
| §O — Copy All / drag-out / Default sharing format | VID-066; `SHR-` |
| §O — Descriptive share titles | `SHR-` (nombres); el origen viene de VID-054 |
| §O — AI captions or transcripts | Excluido |
| §P — Video-related hotkeys (defaults) | VID-075, VID-076; gestión: APP-017 a APP-023 |
| §Q — macOS permissions | VID-077; APP-024 |
| §Q — macOS 15 Sequoia periodic prompt | APP-025 |
| §Q — macOS-version-specific behavior | VID-077; APP-040 |
| §Q — Windows requirements for video | VID-027 (H.265 en Windows 10), VID-029; APP-040, APP-026 |
| §Q — Protected content | VID-077; APP-026 |
| §Q — Virtual and remote environments | VID-016; APP-026; Citrix: Excluido |
| §R — Deploy with video capture disabled | APP-035; VID-001, VID-075 |
| §R — Webcam-on-by-default policy | VID-031; APP-035 |
| §R — Share destination lockdown and Share Link customization | APP-035; `SHR-` |
| §R — Experiments and AI controls | APP-034, APP-035 |
| §R — Update channel | APP-036 |
| §S — Keystroke display on Windows | VID-047 (ambas plataformas) |
| §S — Webcam background, blur, border or mirror on Windows | VID-036, VID-037 (ambas plataformas) |
| §S — In-app AI captions or transcripts | Excluido |
| §S — Recording Timer on Windows | VID-015 (ambas plataformas) |
| §S — H.265 on Windows | VID-027 (ambas plataformas) |
| §S — Restart/Delete button names | VID-014 (decisión de diseño) |
| §S — GIF default values | VID-069, VID-070 (decisión de diseño) |
| §S — Highlight Cursor and Click Animation sub-options | VID-045, VID-046 (decisión de diseño) |
| §S — Countdown length other than 3 s | VID-008 (decisión de diseño) |
| research/05-plataformas-versiones.md §3 — Matriz de video (temporizador, H.265, Retina, 1080p, íconos del escritorio, efectos de cámara, nivel automático, 4:3, Lápiz, clic derecho, copia sin sonido, narración, ruido, 60 fps, cambio de vista, multipista) | VID-015, VID-027, VID-028, VID-018, VID-036, VID-037, VID-039, VID-003, VID-049, VID-046, VID-063, VID-071, VID-041, VID-025, VID-035; multipista: Excluido |
| research/05 §5 — Experimentos de Mac relacionados con video (Dibujo en vivo en cualquier momento, teclas pulsadas, grabar una ventana que se mueve, Lápiz y Óvalo) | VID-051, VID-047, VID-005, VID-049 |
