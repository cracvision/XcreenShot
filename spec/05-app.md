# 05 · Aplicación

**Archivo:** `05-app.md` · **Prefijo de IDs:** `APP-` · **Versión:** 0.1 (borrador) · **Fecha:** 2026-10-12

Este archivo especifica cómo se comporta XcreenShot como aplicación, por encima de cada función concreta. Cubre el proceso en segundo plano y su ícono, la ventana de Preferencias, el tema, el idioma, la ayuda y las notificaciones, el sistema de atajos de teclado, los permisos del sistema operativo, la automatización por CLI y servidor MCP (que reemplaza la API COM de Snagit), las funciones en prueba, la configuración administrada (que reemplaza el despliegue empresarial), las actualizaciones, la distribución, la accesibilidad, los registros y los requisitos no funcionales. Las funciones de captura (`CAP-`), editor (`EDT-`), video (`VID-`), biblioteca, bandeja reciente y compartir (`LIB-`, `TRY-`, `SHR-`) y funciones inteligentes (`SMT-`) se definen en sus archivos. Aquí solo se citan por prefijo y nombre, por ejemplo "CAP- (Ocultar XcreenShot al capturar)"; una pasada posterior pondrá los IDs exactos. Salvo que una función diga otra cosa, todo vale igual en Windows y macOS (SPEC §2.1).

**Convenciones de este archivo**
- **Teclas.** `Ctrl`/`Cmd` significa `Ctrl` en Windows y `Command` (⌘) en macOS; `Alt`/`Option` igual. `PrtScn` es la tecla Imprimir pantalla.
- **(decisión de diseño)** marca un comportamiento que la documentación pública de Snagit no describe, o que XcreenShot define a propósito de otra forma. Lo fija este spec.
- **"Por defecto"** es el valor en una instalación nueva, sin configuración administrada.
- **Ícono de XcreenShot.** El ícono de la app en el área de notificación de Windows o en la barra de menús de macOS. No confundir con la **Bandeja reciente** (`TRY-`).
- **Clave de ajuste.** Identificador estable en inglés de cada preferencia, con la forma `área.nombre` (por ejemplo `general.theme`). La usan la exportación de ajustes, el CLI y la configuración administrada. Las claves de los ajustes que define este archivo se fijan aquí; las de los demás archivos las asigna el equipo de construcción con el mismo patrón.
- **Identificadores de máquina.** Los IDs de comandos, nombres de parámetros, valores enumerados, códigos de error y claves JSON se escriben en inglés y nunca se traducen. Los textos para personas sí se traducen (APP-013).
- **Referencias a otros archivos.** `PREFIJO- (nombre de la función)`.

---

## Índice

1. **Ciclo de vida** — APP-001 Proceso en segundo plano e instancia única · APP-002 Ícono de XcreenShot y menú rápido · APP-003 Iniciar al iniciar sesión · APP-004 Cerrar ventanas y salir · APP-005 Primer arranque
2. **Preferencias** — APP-006 Ventana de Preferencias · APP-007 Sección General · APP-008 Secciones Captura, Video, Editor, Biblioteca y Compartir · APP-009 Sección Avanzado · APP-010 Restablecer ajustes y avisos · APP-011 Exportar e importar ajustes
3. **Interfaz, idioma y ayuda** — APP-012 Tema de la interfaz · APP-013 Idioma de la interfaz · APP-014 Tooltips y pistas · APP-015 Ayuda, Acerca de y comentarios · APP-016 Notificaciones
4. **Atajos de teclado** — APP-017 Ámbitos y reglas de los atajos · APP-018 Sección Atajos · APP-019 Detección de conflictos · APP-020 Tomar atajos ocupados · APP-021 Perfiles de atajos · APP-022 Atajos de presets, destinos y otros comandos · APP-023 Hoja de atajos
5. **Permisos** — APP-024 Permisos de macOS y Reparar permisos · APP-025 Aviso periódico de acceso a la pantalla (macOS) · APP-026 Privilegios y privacidad en Windows
6. **Automatización** — APP-027 Registro de comandos · APP-028 CLI `xcreenshot` · APP-029 Servidor MCP · APP-030 Catálogo mínimo de comandos · APP-031 Eventos · APP-032 Seguridad de la automatización · APP-033 `AGENTS.md`
7. **Funciones en prueba y configuración administrada** — APP-034 Funciones en prueba · APP-035 Configuración administrada
8. **Actualizaciones y distribución** — APP-036 Actualizaciones · APP-037 Instalador de Windows · APP-038 Modo portable (Windows) · APP-039 Paquete de macOS · APP-040 Requisitos del sistema · APP-041 Windows en ARM · APP-042 Desinstalación
9. **Accesibilidad** — APP-043 Navegación con teclado · APP-044 Lectores de pantalla · APP-045 Contraste, escalado y movimiento
10. **Registros y diagnóstico** — APP-046 Registros e información de diagnóstico · APP-047 Recuperación tras un cierre inesperado
11. **Requisitos no funcionales** — APP-048 Del atajo a la cruceta · APP-049 De la captura al editor · APP-050 Editor con imágenes muy grandes · APP-051 Consumo en reposo y arranque · APP-052 Escala de la biblioteca · APP-053 Cruceta con varios monitores 4K · APP-054 Local y privado · APP-055 Portabilidad de datos y migración entre equipos
12. [Relacionado](#relacionado) · [Excluido](#excluido) · [Cobertura](#cobertura)

---

## 1. Ciclo de vida

### APP-001 · Proceso en segundo plano e instancia única
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** proceso de captura en segundo plano que posee los atajos globales (Snagit Capture en Windows; app de la barra de menús en Mac)
- **Comportamiento:**
  - XcreenShot tiene un proceso principal que sigue activo aunque no haya ninguna ventana abierta. Ese proceso:
    - registra los atajos globales (APP-017);
    - mantiene el ícono de XcreenShot (APP-002);
    - atiende al CLI y al servidor MCP (APP-028, APP-029);
    - ejecuta las capturas por intervalo y programadas (CAP- (Intervalo), CAP- (Programada)).
  - Hay **una sola instancia** por usuario y sesión del sistema. Si el usuario vuelve a abrir XcreenShot (menú Inicio, Launchpad, Dock o un acceso directo), no arranca otra copia: la instancia en marcha muestra la ventana principal (editor y biblioteca, `EDT-`).
  - Abrir un archivo con XcreenShot (doble clic, "Abrir con", soltarlo sobre el ícono del Dock) lo envía a la instancia en marcha:
    - `.xshot` y las imágenes se abren en el editor (`EDT-`);
    - `.xpreset`, `.xtheme`, `.xstamps` y `.xtemplate` inician la importación que define su archivo dueño (`CAP-`, `EDT-`);
    - `.xsettings` inicia la importación de ajustes (APP-011).
  - Cada usuario del sistema tiene su propia instancia, sus ajustes y sus atajos. Con varias sesiones abiertas a la vez (cambio rápido de usuario, escritorios remotos), cada sesión tiene la suya y no se interfieren.
  - Las ventanas (panel de captura, editor, preferencias, ayuda) se crean cuando hacen falta y se pueden cerrar sin que el proceso termine (APP-004).
  - En Windows, la app tiene botón en la barra de tareas solo mientras hay alguna ventana abierta. En macOS, el ícono del Dock aparece solo mientras hay una ventana principal abierta; en segundo plano la app vive solo en la barra de menús (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con todas las ventanas cerradas, el atajo Capturar funciona y el ícono sigue visible.
  - [ ] Abrir XcreenShot por segunda vez no crea otro proceso ni otro ícono: aparece la ventana principal de la instancia que ya estaba en marcha.
  - [ ] Con XcreenShot en marcha, doble clic en un `.xshot` lo abre en el editor existente en ≤ 1 s.
  - [ ] Con dos usuarios con sesión abierta en el mismo equipo, el atajo de uno no dispara capturas en la sesión del otro.
- **Fuentes:** research/01-captura.md §1.1; research/04-biblioteca-compartir.md §5.3 (General)

### APP-002 · Ícono de XcreenShot y menú rápido
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** ícono del área de notificación (Windows) y de la barra de menús (Mac); menú rápido del ícono; ícono rojo durante la grabación; opción de ícono en blanco y negro
- **Comportamiento:**
  - Mientras la app está en marcha, el ícono de XcreenShot está en el área de notificación (Windows) o en la barra de menús (macOS).
  - **Clic en el ícono:**
    - Windows: el clic izquierdo ejecuta la acción elegida en Preferencias > General > **Clic en el ícono**: Abrir el panel de captura (por defecto), Captura universal o Abrir el editor. El clic derecho abre el menú rápido.
    - macOS: el clic y el clic secundario abren el menú rápido, como el resto de íconos de la barra de menús.
  - **Menú rápido**, de arriba abajo:
    1. **Estado**, solo cuando hay algo en curso. Cada dueño define sus entradas: grabación, con Detener y Pausar/Reanudar (`VID-`); captura por intervalo o programada, con Detener captura (CAP- (Menú de captura del ícono)); grabador de pasos, con Pausar y Terminar (SMT- (Grabador de pasos)); actualización descargada, con Reiniciar para actualizar (APP-036); y avisos de atención (permiso que falta, atajos que no se pudieron activar), cada uno con un enlace para resolverlo.
    2. **Entradas de captura**, definidas en CAP- (Menú de captura del ícono).
    3. **Recientes ▸**: las 10 últimas capturas, con miniatura, nombre y hora. Elegir una la abre en el editor. Al final del submenú, **Abrir biblioteca** (`LIB-`).
    4. **Abrir editor**.
    5. **Pausar atajos globales** (casilla). Mientras está marcada, ningún atajo global de XcreenShot responde, salvo el atajo de la propia acción Pausar atajos, si tiene uno (decisión de diseño; útil en juegos, escritorios remotos o presentaciones).
    6. **Preferencias…** y **Ayuda ▸** (APP-015).
    7. **Salir de XcreenShot** (APP-004).
  - Cada entrada del menú muestra su atajo, si tiene uno.
  - **Estados del ícono.** Cada estado se distingue por la forma y no solo por el color:
    - normal;
    - grabando: punto rojo sobre el ícono; en pausa: dos barras verticales;
    - captura por intervalo o programada activa: pequeño reloj;
    - atajos en pausa: ícono atenuado con una barra diagonal;
    - atención: punto pequeño en una esquina cuando falta un permiso esencial, algún atajo no se pudo activar o hay una actualización lista.
  - En Windows, el texto emergente del ícono indica el estado, por ejemplo "XcreenShot · Grabando 00:03:12".
  - Preferencias > General > **Mostrar el ícono** (por defecto: sí). Si se oculta, la app sigue en segundo plano y se vuelve a mostrar abriendo XcreenShot desde el sistema. Mientras haya una grabación o una captura por intervalo en curso, el ícono aparece siempre, aunque esté oculto (decisión de diseño).
  - Preferencias > General > **Ícono monocromo** (por defecto: no): dibuja el ícono en un solo color que se adapta a una barra clara u oscura. Los estados se siguen distinguiendo por la forma.
  - El ícono se adapta solo al color de la barra de tareas o de menús, con independencia del tema de la app (APP-012).
- **Criterios de aceptación:**
  - [ ] (Windows) Con Clic en el ícono = Captura universal, un clic izquierdo muestra la cruceta.
  - [ ] Al iniciar una grabación, el ícono pasa al estado de grabación en ≤ 500 ms y el menú rápido muestra Detener en la primera línea; al detenerla, el ícono vuelve al estado normal.
  - [ ] Con Pausar atajos globales marcada, el atajo Capturar no hace nada; al desmarcarla vuelve a funcionar sin reiniciar.
  - [ ] Con el ícono monocromo, los estados de grabación y pausa se distinguen en una captura de pantalla en escala de grises.
  - [ ] Recientes lista las 10 últimas capturas, de la más nueva a la más antigua, y elegir una la abre en el editor.
- **Notas de plataforma:** en Windows 11, el sistema decide si el ícono queda visible o dentro del menú de íconos ocultos. La primera vez, XcreenShot muestra una pista que explica cómo dejarlo siempre visible (APP-014).
- **Fuentes:** research/01-captura.md §1.1, §11.2; research/04-biblioteca-compartir.md §5.3 (Advanced/Editor); research/05-plataformas-versiones.md §1B (2024.3.0)

### APP-003 · Iniciar al iniciar sesión
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** "Run Snagit when Windows starts" (Windows); mantener Snagit en la barra de menús e iniciarlo al iniciar sesión (Mac)
- **Comportamiento:**
  - Preferencia **Iniciar XcreenShot al iniciar sesión** (clave `general.launchAtLogin`). El primer arranque la propone activada (APP-005).
  - Al iniciar sesión, la app arranca sin abrir ventanas: solo aparecen el ícono y los atajos.
  - El arranque se registra para el usuario actual y aparece donde el sistema lista las apps de inicio (Windows: Configuración > Aplicaciones > Inicio; macOS: Ajustes del Sistema > General > Ítems de inicio). Si el usuario lo desactiva desde el sistema, la preferencia muestra ese estado.
  - Arrancar con el sistema no retrasa el inicio de sesión. Primero se activan los atajos y el ícono; el trabajo pesado (comprobar el índice de la biblioteca, buscar actualizaciones) se hace después.
  - Si el arranque automático falla (por ejemplo, porque la carpeta portable cambió de sitio, APP-038), XcreenShot lo avisa la siguiente vez que se abre a mano.
- **Criterios de aceptación:**
  - [ ] Con la opción activada, tras reiniciar el equipo e iniciar sesión, el ícono aparece y el atajo Capturar funciona en ≤ 3 s desde que arranca el proceso, sin abrir ninguna ventana (APP-051).
  - [ ] Al desactivar la opción, XcreenShot desaparece de la lista de apps de inicio del sistema.
  - [ ] Si se desactiva desde la lista del sistema, la preferencia aparece desactivada al abrir Preferencias.
- **Fuentes:** research/01-captura.md §1.1, §11.1, §11.2; research/04-biblioteca-compartir.md §5.1 (General), §5.3 (General)

### APP-004 · Cerrar ventanas y salir
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** "Keep Editor running in background" (Windows); mantener Snagit en la barra de menús (Mac); salir de la captura en segundo plano
- **Comportamiento:**
  - Cerrar una ventana (botón de cerrar, `Ctrl+W`/`Cmd+W`) no cierra la app: el proceso sigue en segundo plano con el ícono y los atajos.
  - Preferencia **Seguir en segundo plano al cerrar todas las ventanas** (clave `general.keepRunning`; por defecto: sí). Si se desactiva, cerrar la última ventana equivale a Salir.
  - Preferencia **Mantener el editor cargado en segundo plano** (sección Avanzado; clave `advanced.keepEditorLoaded`; por defecto: sí). Al cerrar el editor, su ventana se oculta en lugar de destruirse, para que la siguiente captura se abra más rápido (APP-049). Desactivarla reduce la memoria en reposo (APP-051).
  - **Salir** (menú rápido, menú de la app, `Ctrl+Q`/`Cmd+Q` o `xcreenshot app quit`) termina el proceso, libera los atajos globales y quita el ícono.
  - Antes de salir, XcreenShot solo pide confirmación si hay algo en curso, y lo nombra:
    - una grabación o un grabador de pasos activos: **Detener y guardar** o **Cancelar** (`VID-`, `SMT-`);
    - capturas por intervalo o programadas activas: avisa de que se detendrán (`CAP-`);
    - documentos con cambios sin guardar fuera de la biblioteca: **Guardar**, **No guardar** o **Cancelar** (`EDT-`);
    - envíos a destinos en curso: **Esperar** o **Salir de todos modos** (`SHR-`).
  - Al cerrar sesión o apagar el equipo, XcreenShot no bloquea al sistema: guarda los documentos pendientes para poder recuperarlos (APP-047), cierra la grabación en curso como indique `VID-` y sale.
  - Al salir se guardan las posiciones de las ventanas y las pestañas activas, que se restauran la próxima vez.
- **Criterios de aceptación:**
  - [ ] Cerrar el editor con el botón de cerrar deja el ícono y los atajos activos.
  - [ ] `Ctrl+Q`/`Cmd+Q` sin nada en curso cierra la app sin preguntar, y el atajo Capturar deja de responder.
  - [ ] Salir durante una grabación muestra el diálogo; **Detener y guardar** deja el video en la biblioteca.
  - [ ] Apagar el equipo con un documento modificado no bloquea el apagado y, al volver, el documento aparece en Recuperar trabajo (APP-047).
- **Notas de plataforma:** en macOS, `Cmd+Q` sale siempre de la app completa, por convención del sistema; para cerrar solo el editor se usa `Cmd+W`.
- **Fuentes:** research/01-captura.md §1.1, §10, §11.1; research/04-biblioteca-compartir.md §5.2 (Advanced), §5.3 (General); research/02-editor.md §11

### APP-005 · Primer arranque
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** ventana de bienvenida (sin encuesta, sin tours y sin cuenta)
- **Comportamiento:**
  - La primera vez que un usuario abre XcreenShot aparece una ventana de bienvenida breve. Cada paso tiene **Omitir**:
    1. **Idioma**, preseleccionado según el sistema (APP-013).
    2. **Atajos.** Pregunta si Snagit sigue instalado en este equipo. Si la respuesta es sí, deja el perfil Coexistencia (por defecto); si es no, ofrece Compatibilidad con Snagit (APP-021). Muestra los atajos principales y si se han podido activar (APP-019).
    3. **Permisos** (solo macOS). Explica para qué se necesita la Grabación de pantalla y lanza la solicitud del sistema. Los demás permisos se piden cuando hacen falta (APP-024).
    4. **Inicio y biblioteca.** Casilla Iniciar al iniciar sesión, marcada (APP-003), y ubicación de la biblioteca (`LIB-`), con opciones para cambiarla o para usar una biblioteca que ya existe (APP-055).
    5. **Listo.** Muestra el atajo de captura y propone hacer una captura de prueba.
  - **Omitir** en cualquier paso aplica los valores por defecto de los pasos que faltan. Todo se puede cambiar después en Preferencias.
  - No pide cuenta, correo ni ningún dato. No hay encuesta ni consentimiento de datos, porque XcreenShot no recoge ninguno (APP-054).
  - La configuración administrada puede omitir la bienvenida o fijar sus valores (APP-035).
  - Ayuda > **Volver a mostrar la bienvenida** la abre de nuevo.
- **Criterios de aceptación:**
  - [ ] En una instalación nueva aparece la bienvenida; tras completarla u omitirla, no vuelve a aparecer en el siguiente arranque.
  - [ ] Si se responde que Snagit no está instalado y se acepta Compatibilidad con Snagit, el atajo Capturar queda con el valor de ese perfil.
  - [ ] (macOS) Si se omite el paso de permisos, el primer intento de captura muestra el aviso de permisos (APP-024) en lugar de fallar.
  - [ ] Con `policy.skipFirstRun` en la configuración administrada, la bienvenida no aparece.
- **Fuentes:** research/04-biblioteca-compartir.md §5.4 (Help / feedback), §9 (Admin deployment controls); research/05-plataformas-versiones.md §1A (2024.1.0)

---

## 2. Preferencias

### APP-006 · Ventana de Preferencias
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Capture Preferences y Editor Preferences (Windows); ventana Settings (Mac)
- **Comportamiento:**
  - Una sola ventana **Preferencias**, con la misma estructura en Windows y en macOS (decisión de diseño; Snagit separa en Windows las preferencias de captura y las del editor).
  - Se abre desde:
    - `Ctrl+,`/`Cmd+,` en cualquier ventana de XcreenShot;
    - el menú rápido del ícono (APP-002);
    - el menú de la app (Windows: Archivo > Preferencias; macOS: XcreenShot > Ajustes…);
    - el menú Más del panel de captura, que la abre en la sección Captura (CAP- (Panel de captura));
    - los enlaces "Configurar…" de otras pantallas, que la abren directamente en el ajuste concreto;
    - el CLI: `xcreenshot settings open [--section <id>] [--key <clave>]`.
  - Barra lateral con estas secciones, en este orden (entre paréntesis, el identificador para el CLI):

    | Sección | Identificador | Contenido |
    |---|---|---|
    | General | `general` | APP-007 |
    | Captura | `capture` | APP-008 |
    | Video | `video` | APP-008 |
    | Editor | `editor` | APP-008 |
    | Biblioteca | `library` | APP-008 |
    | Compartir | `share` | APP-008 |
    | Atajos | `shortcuts` | APP-018 |
    | Avanzado | `advanced` | APP-009 |
    | Funciones en prueba | `experimental` | APP-034 |

  - Un campo **Buscar**, encima de la barra lateral, filtra por nombre, descripción o clave del ajuste en todas las secciones. Los resultados se agrupan por sección; al elegir uno, la ventana va a ese ajuste y lo resalta (mejora).
  - Los cambios se aplican al momento: no hay botones Aceptar ni Aplicar. Un valor no válido se rechaza junto al campo, con el motivo. Si un cambio necesita reiniciar la app, aparece una franja con **Reiniciar ahora**.
  - Clic secundario sobre un ajuste > **Copiar clave** copia su clave de ajuste (útil para la configuración administrada y el CLI). `xcreenshot settings list --json` lista todas las claves con su tipo, valor por defecto, valor actual y si están bloqueadas.
  - Un ajuste bloqueado por la configuración administrada (APP-035) aparece desactivado, con un candado y el texto "Lo gestiona tu organización". Las funciones que la configuración administrada desactiva no aparecen; por ejemplo, la sección Video se oculta si la grabación está desactivada.
  - Cada sección termina con **Restablecer esta sección** (APP-010).
  - Solo hay una ventana de Preferencias: abrirla otra vez la trae al frente. No es modal: se puede seguir usando el editor con ella abierta. Recuerda la última sección, su tamaño y su posición.
- **Criterios de aceptación:**
  - [ ] En Windows y en macOS, la ventana muestra las nueve secciones en el mismo orden.
  - [ ] Buscar "lupa" muestra el ajuste de la lupa de la sección Captura; elegirlo lleva a ese ajuste y lo resalta.
  - [ ] Cambiar el tema en General cambia el aspecto de todas las ventanas abiertas sin pulsar ningún botón.
  - [ ] Un ajuste bloqueado por la configuración administrada aparece con candado y no se puede cambiar ni desde la ventana ni con `xcreenshot settings set` (código de salida 6, APP-028).
  - [ ] `xcreenshot settings open --section shortcuts` abre la ventana en la sección Atajos.
- **Notas de plataforma:** en macOS, la entrada del menú de la app se llama "Ajustes…" por convención del sistema y abre esta misma ventana.
- **Fuentes:** research/04-biblioteca-compartir.md §5.1, §5.2, §5.3, §5.4; research/05-plataformas-versiones.md §3 (Settings and deployment); research/01-captura.md §11.1, §11.2

### APP-007 · Sección General
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** pestaña General de Capture Preferences y de Editor Preferences (Windows); pestaña General de Settings (Mac)
- **Comportamiento:**
  - Contenido, en este orden:

    | Ajuste | Valores (por defecto) | Clave | Dueño |
    |---|---|---|---|
    | Iniciar XcreenShot al iniciar sesión | Sí / No (Sí desde el primer arranque) | `general.launchAtLogin` | APP-003 |
    | Mostrar el ícono | **Sí** / No | `general.showIcon` | APP-002 |
    | Ícono monocromo | Sí / **No** | `general.monochromeIcon` | APP-002 |
    | Clic en el ícono (solo Windows) | **Panel de captura** / Captura universal / Editor | `general.iconClickAction` | APP-002 |
    | Seguir en segundo plano al cerrar todas las ventanas | **Sí** / No | `general.keepRunning` | APP-004 |
    | Interfaz de captura | **Panel de captura** / Lanzador de borde / Ambos | (la fija el dueño) | CAP- (Panel de captura), CAP- (Lanzador de borde) |
    | Tema | **Sistema** / Claro / Oscuro | `general.theme` | APP-012 |
    | Idioma | **Sistema** / English / Español | `general.language` | APP-013 |
    | Detalle de los tooltips | Desactivados / Breves / **Detallados** | `general.tooltips` | APP-014 |
    | Mostrar pistas | **Sí** / No | `general.hints` | APP-014 |
    | Notificaciones | una casilla por categoría | `notifications.*` | APP-016 |
    | Permisos | estado de cada permiso y botón **Reparar permisos** | — | APP-024, APP-026 |
    | Actualizaciones | Buscar automáticamente, Descargar e instalar automáticamente, Canal y botón **Buscar ahora** | `updates.*` | APP-036 |
    | Volver a mostrar avisos y pistas | botón | — | APP-010 |

  - Como en APP-008, cada fila aparece cuando existe su función: por ejemplo, el bloque Actualizaciones llega con M6.
- **Criterios de aceptación:**
  - [ ] En una instalación nueva aparecen todas las filas de la tabla con los valores por defecto indicados.
  - [ ] Cambiar Idioma actualiza al momento los textos de la propia sección.
  - [ ] (macOS) Si se retira el permiso de Grabación de pantalla en Ajustes del Sistema, al volver a XcreenShot el bloque Permisos lo muestra como denegado.
- **Fuentes:** research/04-biblioteca-compartir.md §5.1 (General), §5.2 (General), §5.3 (General); research/01-captura.md §11.1, §11.2

### APP-008 · Secciones Captura, Video, Editor, Biblioteca y Compartir
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** pestaña Capture (Windows y Mac); pestañas Editor, Share, Library y Advanced (Windows); pestañas Share y Advanced (Mac)
- **Comportamiento:**
  - Estas secciones agrupan los ajustes de las funciones. El archivo dueño define cada ajuste (valores, valor por defecto y efecto); esta ficha solo fija **en qué sección** aparece y en qué orden.
  - Un archivo dueño puede añadir ajustes a su sección. Las tablas siguientes son la lista mínima y su orden.
  - Un ajuste aparece cuando su función existe, igual que los modos de captura (CAP- (Pestaña Imagen y lista Selección)): no se muestran ajustes de hitos futuros. Cada sección se completa con el hito de sus funciones (Video en M4, Compartir en M6).
  - **Captura:**

    | Ajuste | Dueño |
    |---|---|
    | Ocultar XcreenShot al capturar | CAP- (Ocultar XcreenShot al capturar) |
    | Mostrar la lupa al empezar | CAP- (Lupa) |
    | Cruceta simplificada | CAP- (Cruceta simplificada) |
    | Exigir el botón Iniciar en las capturas con desplazamiento | CAP- (Desplazamiento manual), CAP- (Desplazamiento automático) |
    | Corrección de color HDR | CAP- (Pantallas HDR) |
    | Abrir en XcreenShot las capturas de la herramienta del sistema (y conservar el original) | CAP- (Capturas del sistema) |
    | Abrir lo recibido por AirDrop (macOS) | CAP- (AirDrop) |
    | Confirmación de captura | CAP- (Confirmación de captura) |
    | Metadatos de captura | CAP- (Metadatos de captura) |
    | Anotación rápida | CAP- (Anotación rápida de imagen) |
    | Idioma del reconocimiento de texto | SMT- (Extraer texto) |
    | Opciones del grabador de pasos | SMT- (Grabador de pasos) |

  - **Video:**

    | Ajuste | Dueño |
    |---|---|
    | Cuenta atrás antes de grabar | VID- (cuenta atrás) |
    | Empezar a grabar sin esperar | VID- (inicio automático) |
    | Ventana Listo para grabar | VID- (Listo para grabar) |
    | Barra de grabación minimizada | VID- (barra de grabación) |
    | Avisar si el micrófono está silenciado | VID- (micrófono) |
    | Ocultar los íconos del escritorio al grabar | VID- (ocultar escritorio) |
    | Calidad y fotogramas por segundo | VID- (calidad y FPS) |
    | Códec (H.264 / H.265) | VID- (codificación) |
    | Reducir la resolución (a 1080p o de pantallas de alta densidad) | VID- (escalado) |
    | Combinar las pistas de audio | VID- (audio) |
    | Codificación por software | VID- (codificación) |
    | Cámara activada por defecto | VID- (cámara) |

  - **Editor:**

    | Ajuste | Dueño |
    |---|---|
    | Color del área de trabajo | EDT- (área de trabajo) |
    | Ampliar el lienzo automáticamente | EDT- (lienzo) |
    | `Esc` cierra el editor | EDT- (armazón del editor) |
    | Vista previa del estilo junto al puntero | EDT- (estilos guardados) |
    | Etiquetas de la barra de herramientas | EDT- (barra de herramientas) |
    | Seleccionar la anotación al terminar de dibujarla | EDT- (objetos) |
    | Barra de edición rápida | EDT- (objetos) |
    | Eliminar el contenido recortado | EDT- (Recortar) |
    | Aplanar automáticamente las imágenes pegadas | EDT- (pegar) |
    | Tamaño y color de una imagen nueva | EDT- (nueva imagen) |
    | Corrector ortográfico e idioma | EDT- (Texto) |

  - **Biblioteca:**

    | Ajuste | Dueño |
    |---|---|
    | Ubicación de la biblioteca | LIB- (ubicación) |
    | Guardar automáticamente las imágenes / los videos | LIB- (autoguardado) |
    | Confirmar antes de eliminar | LIB- (eliminar) |
    | Conservar una copia editable al guardar en otro formato | LIB- (guardar como) |
    | Reconstruir el índice | LIB- (mantenimiento) |
    | Número de elementos de la bandeja reciente (1–200) | TRY- (tamaño de la bandeja) |
    | Tamaño de las miniaturas | TRY- (miniaturas) |
    | Quitar de la bandeja los elementos no fijados al cerrar | TRY- (fijar) |

  - **Compartir:**

    | Ajuste | Dueño |
    |---|---|
    | Destinos: activar, ordenar, crear instancias con nombre y opciones de cada uno | SHR- (gestión de destinos) |
    | Proveedor del Enlace para compartir | SHR- (Enlace para compartir) |
    | No incluir el origen de la captura en nombres y títulos | SHR- (nombres) |
    | Formato al arrastrar fuera de XcreenShot | SHR- (arrastrar y soltar) |
    | Formato por defecto al compartir (PNG / JPG) | SHR- (formato) |
    | Reducir las imágenes de alta densidad al compartir | SHR- (escalado) |
    | Cuentas conectadas (cerrar sesión de cada servicio) | SHR- (autenticación) |

- **Criterios de aceptación:**
  - [ ] Cada ajuste de las tablas aparece en su sección y, al cambiarlo, la función dueña se comporta como dice su ficha (por ejemplo, con Ocultar XcreenShot al capturar desactivado, el panel de captura aparece en una captura de pantalla completa).
  - [ ] Ningún ajuste aparece en más de una sección: la búsqueda (APP-006) lo encuentra una sola vez.
  - [ ] Con la grabación desactivada por la configuración administrada, la sección Video no aparece.
- **Fuentes:** research/04-biblioteca-compartir.md §5.1 (Capture), §5.2 (Editor, Share, Library, Advanced), §5.3 (Capture, Share, Advanced/Editor); research/02-editor.md §11; research/01-captura.md §11.1, §11.2

### APP-009 · Sección Avanzado
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** pestaña Advanced (Windows y Mac)
- **Comportamiento:**
  - Contenido, en este orden:

    | Ajuste | Valores (por defecto) | Clave | Dueño |
    |---|---|---|---|
    | Mantener el editor cargado en segundo plano | **Sí** / No | `advanced.keepEditorLoaded` | APP-004 |
    | Permitir el control por CLI y agentes (MCP) | **Sí** / No | `automation.enabled` | APP-032 |
    | Avisar cuando la automatización capture o grabe | **Sí** / No | `automation.notifyOnCapture` | APP-032 |
    | Permitir que la automatización envíe a destinos en línea | Sí / **No** | `automation.allowOnlineShare` | APP-032 |
    | Comando de terminal | estado y botón Instalar / Quitar | — | APP-028 |
    | Configuración para clientes MCP | botón **Copiar** | — | APP-029 |
    | Registro detallado | Sí / **No** | `diagnostics.verbose` | APP-046 |
    | Abrir la carpeta de registros · Copiar información de diagnóstico | botones | — | APP-046 |
    | Carpeta de datos de la app | ruta y botón **Abrir** | — | APP-046 |
    | Exportar ajustes… · Importar ajustes… | botones | — | APP-011 |
    | Restablecer todos los ajustes… | botón | — | APP-010 |
    | Configuración administrada | estado (Inactiva / Activa / Con errores) y botón **Ver** | — | APP-035 |
    | Quitar XcreenShot de este Mac (solo macOS) | botón | — | APP-042 |

  - Otros archivos solo añaden aquí ajustes técnicos que el usuario medio no necesita; por ejemplo, un método de captura alternativo para equipos donde el normal falla (`VID-`).
- **Criterios de aceptación:**
  - [ ] Con Mantener el editor cargado desactivado, el consumo de memoria en reposo queda por debajo del límite que fija APP-051 para ese caso.
  - [ ] Con Permitir el control por CLI y agentes desactivado, `xcreenshot capture fullscreen` termina con código 6 y un mensaje que nombra este ajuste.
  - [ ] **Copiar** en Configuración para clientes MCP deja en el portapapeles un bloque JSON válido que lanza `xcreenshot mcp`.
- **Fuentes:** research/04-biblioteca-compartir.md §5.2 (Advanced), §5.3 (Advanced/Editor); research/02-editor.md §11

### APP-010 · Restablecer ajustes y avisos
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** "Reset to Default Settings" (Windows, pestaña Advanced); "Reset Don't Ask Me warnings" (Mac)
- **Comportamiento:**
  - **Restablecer esta sección**, al final de cada sección, devuelve sus ajustes al valor por defecto. En la sección Atajos devuelve todos los atajos al valor del perfil activo (APP-021).
  - **Restablecer todos los ajustes…** (sección Avanzado) devuelve al valor por defecto todas las preferencias, los atajos y el perfil de atajos, y la personalización de la barra de herramientas del editor.
  - No se tocan nunca los contenidos del usuario: capturas y biblioteca, presets, temas, estilos y efectos guardados, sellos, plantillas, ni las sesiones iniciadas en los destinos.
  - El diálogo de confirmación lista lo que se va a restablecer y ofrece **Exportar mis ajustes antes** (APP-011).
  - Si hay configuración administrada (APP-035), restablecer lleva cada ajuste a su valor administrado por defecto, si existe, y deja intactos los bloqueados.
  - **Volver a mostrar avisos y pistas** (sección General) anula todas las respuestas "No volver a preguntar" de los diálogos de XcreenShot y vuelve a activar las pistas (APP-014). El botón indica cuántos avisos están ocultos.
  - CLI: `xcreenshot settings reset [--section <id>] --yes`.
- **Criterios de aceptación:**
  - [ ] Tras cambiar cinco ajustes de Captura y uno de Editor, Restablecer esta sección en Captura devuelve los cinco a su valor por defecto y no toca el de Editor.
  - [ ] Restablecer todos los ajustes no borra ni modifica ninguna captura, preset, tema ni estilo guardado.
  - [ ] Tras marcar "No volver a preguntar" en un diálogo, Volver a mostrar avisos y pistas hace que el diálogo reaparezca la próxima vez.
  - [ ] Con un ajuste bloqueado por la configuración administrada, restablecer lo deja en el valor administrado.
- **Fuentes:** research/04-biblioteca-compartir.md §5.2 (Advanced), §5.3 (General), §5.4 (Reset to defaults); research/05-plataformas-versiones.md §1A (2024.0.1)

### APP-011 · Exportar e importar ajustes
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (Snagit exporta presets y temas por separado, pero no las preferencias)
- **Comportamiento:**
  - Avanzado > **Exportar ajustes…** crea un archivo `.xsettings`: JSON legible, con número de versión de formato, versión de la app y plataforma de origen.
  - Grupos que se pueden incluir (todos marcados por defecto):
    - preferencias;
    - atajos y perfil de atajos;
    - presets de captura (`CAP-`);
    - temas, estilos y efectos guardados (`EDT-`);
    - personalización de la barra de herramientas (`EDT-`);
    - destinos de compartir y sus opciones (`SHR-`).
  - El archivo **nunca** incluye contraseñas, tokens de acceso ni sesiones iniciadas, ni el contenido de la biblioteca, ni los registros.
  - **Importar ajustes…** muestra primero un resumen por grupo (cuántos ajustes, atajos, presets o temas cambiarán) y deja desmarcar grupos. Si ya existe un preset o tema con el mismo nombre, se elige entre **Reemplazar**, **Conservar ambos** u **Omitir**.
  - **Entre plataformas.** Un archivo de Windows se importa en macOS y al revés:
    - los atajos se pueden convertir (`Ctrl` ↔ `Cmd`, `Alt` ↔ `Option`, `Win` ↔ `Control`) o dejar los de este equipo. Los que no tienen equivalente (por ejemplo `PrtScn` en macOS) quedan sin asignar y se listan; los que entran en conflicto se resuelven como en APP-019;
    - los ajustes que solo existen en la otra plataforma se conservan en el archivo, pero no se aplican;
    - las rutas que no existen en este equipo (biblioteca, carpetas de destino) se omiten y se listan.
  - Los ajustes bloqueados por la configuración administrada no se importan; el resumen lo indica.
  - CLI: `xcreenshot settings export --out <archivo> [--groups …]` y `xcreenshot settings import <archivo> [--groups …] --yes`.
  - El formato `.xsettings` se propone para la tabla de formatos de SPEC §7.
- **Criterios de aceptación:**
  - [ ] Exportar en un equipo e importar en otro recién instalado deja iguales las preferencias, los atajos, los presets y los temas.
  - [ ] El `.xsettings` no contiene ningún token ni contraseña, aunque haya destinos con sesión iniciada (se comprueba buscando esos valores en el archivo).
  - [ ] Un archivo de Windows con Capturar = `Ctrl+Shift+PrtScn`, importado en macOS con conversión, deja ese atajo sin asignar y lo lista como no convertible.
  - [ ] Importar solo el grupo Atajos no cambia ningún otro ajuste.
- **Mejora sobre Snagit:** copia completa y legible de la configuración, que también sirve para pasar a otro equipo (APP-055).
- **Fuentes:** research/04-biblioteca-compartir.md §4 (Preset export/import, Themes), §5.4

---

## 3. Interfaz, idioma y ayuda

### APP-012 · Tema de la interfaz
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** tema de color Claro / Oscuro / Sistema
- **Comportamiento:**
  - Preferencia **Tema** (clave `general.theme`): **Sistema** (por defecto), Claro u Oscuro.
  - Con Sistema, XcreenShot sigue al sistema y cambia en el momento en que este cambia (por ejemplo, con el modo oscuro automático al anochecer).
  - El tema se aplica al momento a todas las ventanas, diálogos y barras de XcreenShot, incluidas las barras que aparecen durante una captura o grabación, y a las barras de título nativas.
  - Color de acento: XcreenShot tiene un color de acento propio (identidad visual propia, SPEC §2.4), con una variante para tema claro y otra para oscuro que mantienen el contraste. Se usa en el foco, la selección, los botones principales y la cruceta y el borde de selección (CAP- (Pantalla congelada y cruceta)).
  - Con el modo de alto contraste del sistema activo, XcreenShot usa los colores del sistema en lugar de su paleta (APP-045).
  - El color del área de trabajo del editor es un ajuste aparte (`EDT-`). El ícono de XcreenShot sigue el color de la barra del sistema, no el tema de la app (APP-002).
- **Criterios de aceptación:**
  - [ ] Con Tema = Sistema, cambiar el sistema de claro a oscuro cambia todas las ventanas abiertas de XcreenShot en ≤ 1 s, sin reiniciar.
  - [ ] Con Tema = Oscuro y el sistema en claro, todas las ventanas, menús propios y diálogos de XcreenShot se ven oscuros.
  - [ ] En ambos temas, el texto normal tiene un contraste de al menos 4,5:1 con su fondo (se comprueba con una herramienta de contraste sobre capturas de cada ventana).
- **Fuentes:** research/04-biblioteca-compartir.md §5.1 (General), §5.2 (General), §5.3 (General); research/02-editor.md §1 (Color theme)

### APP-013 · Idioma de la interfaz
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** selección de idioma de la interfaz
- **Comportamiento:**
  - Preferencia **Idioma** (clave `general.language`): **Sistema** (por defecto), English o Español (D-07). Con Sistema, la app usa español si el idioma preferido del sistema es cualquier variante de español, e inglés en los demás casos.
  - El cambio se aplica **sin reiniciar**: ventanas abiertas, menús, menú rápido del ícono, tooltips, ayuda, hoja de atajos y las notificaciones nuevas.
  - El español es neutro y trata de tú al usuario (decisión de diseño). Los nombres propios de XcreenShot siguen la tabla de SPEC §8.
  - No se traducen: el contenido del usuario, los identificadores de máquina (comandos, claves, códigos, nombres de formato) ni los mensajes de los registros, que están siempre en inglés para facilitar el soporte (decisión de diseño).
  - Fechas, horas y números siguen el formato regional del sistema, no el idioma de la interfaz.
  - Si falta un texto en español, se muestra en inglés y se anota en el registro. Una versión publicada no puede tener textos sin traducir.
  - Las ventanas se adaptan a textos más largos: nada se corta sin mostrar el texto completo en un tooltip.
  - El instalador usa el idioma del sistema (APP-037).
- **Criterios de aceptación:**
  - [ ] Cambiar de English a Español con el editor y las Preferencias abiertos traduce ambas ventanas y el menú rápido sin reiniciar.
  - [ ] Con el sistema en español de México y Idioma = Sistema, la interfaz aparece en español.
  - [ ] Una comprobación automática sobre los archivos de textos de una versión publicada no encuentra ninguna clave sin traducción al español.
  - [ ] En español, ningún botón ni etiqueta de las ventanas principales aparece cortado a 100 % y 200 % de escala.
- **Notas de plataforma:** algunos menús que dibuja el sistema (por ejemplo, Servicios en macOS) siguen el idioma del sistema.
- **Fuentes:** research/04-biblioteca-compartir.md §5.2 (General), §5.4 (Language); research/05-plataformas-versiones.md §4 (UI languages)

### APP-014 · Tooltips y pistas
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** nivel de detalle de los tooltips; ventanas emergentes de introducción
- **Comportamiento:**
  - Todos los botones con solo ícono y los controles tienen tooltip. Preferencia **Detalle de los tooltips** (clave `general.tooltips`):
    - **Desactivados**: no se muestran (los lectores de pantalla siguen recibiendo los nombres, APP-044);
    - **Breves**: nombre de la acción y su atajo;
    - **Detallados** (por defecto): nombre, atajo y una frase que explica qué hace.
  - Un tooltip aparece tras ~0,6 s sobre el control, o al llegar al control con el teclado; `Esc` lo oculta. Los tooltips nunca aparecen en una captura (CAP- (Ocultar XcreenShot al capturar)).
  - **Pistas** (clave `general.hints`; por defecto: sí): mensajes breves y propios que aparecen **una sola vez**, la primera vez que se usa algo. Ejemplos:
    - primera cruceta: "H muestra las teclas disponibles";
    - primer uso del editor: "Arrastra una captura de la bandeja al lienzo para combinarlas";
    - primer atajo que no se pudo activar: dónde cambiarlo;
    - (Windows) primer arranque: cómo dejar el ícono siempre visible.
  - Cada pista tiene **Entendido** y **No mostrar más pistas**. Nunca aparece más de una pista a la vez, ni durante una captura o grabación.
  - Todo el contenido es local y de texto: no hay videos, tours guiados, encuestas, noticias ni mensajes promocionales.
- **Criterios de aceptación:**
  - [ ] Con Detalle = Breves, el tooltip del botón Flecha muestra solo "Flecha" y su atajo.
  - [ ] Una pista vista y cerrada no vuelve a aparecer, salvo tras Volver a mostrar avisos y pistas (APP-010).
  - [ ] Con Mostrar pistas desactivado, no aparece ninguna pista en un recorrido completo por captura, editor y biblioteca.
- **Fuentes:** research/02-editor.md §1 (Enhanced tooltips, Onboarding pop-ups); research/04-biblioteca-compartir.md §5.2 (General), §5.3 (General)

### APP-015 · Ayuda, Acerca de y comentarios
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** menú de ayuda "?" (soporte, tutoriales, comentarios); ayuda sin conexión
- **Comportamiento:**
  - El menú **Ayuda** está en la barra de menús de la ventana principal, en el botón **?** de su barra de herramientas, en el menú Más del panel de captura y en el menú rápido del ícono. Entradas:
    - **Ayuda de XcreenShot** (`F1` en Windows, `Cmd+?` en macOS): páginas propias, locales y en el idioma de la interfaz, una por área (captura, editor, video, biblioteca, compartir, atajos, permisos, automatización). Funcionan sin conexión.
    - **Atajos de teclado** (APP-023).
    - **Novedades**: notas de la versión instalada y de las anteriores (APP-036).
    - **Buscar actualizaciones** (APP-036).
    - **Reparar permisos** (APP-024; en Windows, el estado de privacidad de cámara y micrófono, APP-026).
    - **Recuperar trabajo** (APP-047).
    - **Abrir la carpeta de registros** y **Copiar información de diagnóstico** (APP-046).
    - **Mover XcreenShot a otro equipo…** (APP-055).
    - **Volver a mostrar la bienvenida** (APP-005).
    - **Enviar comentarios…**
    - **Acerca de XcreenShot** (en macOS, en el menú de la app).
  - **Acerca de XcreenShot** muestra: versión, compilación y fecha, canal de actualizaciones, arquitectura (x64, ARM64 o Apple Silicon/Intel), licencias de los componentes de terceros, estado de la configuración administrada, ubicación de los datos de la app y una declaración breve de privacidad con las únicas conexiones de red que hace la app (APP-054). Tiene un botón **Comprobar comunicación** que ejecuta el comando de prueba `app.ping` (APP-027).
  - **Enviar comentarios…** abre en el navegador la página para crear una incidencia en el repositorio del proyecto, o un correo nuevo en el cliente de correo predeterminado, según se haya configurado al compilar (decisión de diseño). El texto llega rellenado con la versión, el sistema y una plantilla. XcreenShot no envía nada por sí mismo: el usuario revisa el texto y decide. La información de diagnóstico solo se añade si el usuario la pega.
  - La configuración administrada puede ocultar Enviar comentarios o cambiar su destino (APP-035).
- **Criterios de aceptación:**
  - [ ] Sin conexión de red, Ayuda de XcreenShot abre la página de ayuda en el idioma de la interfaz.
  - [ ] Acerca de muestra la versión, la arquitectura y el canal, y Comprobar comunicación responde con la versión en ≤ 1 s.
  - [ ] Enviar comentarios abre un borrador (incidencia o correo) con la versión y el sistema rellenados, y XcreenShot no hace ninguna conexión de red por su cuenta.
  - [ ] Todas las entradas del menú Ayuda se pueden usar solo con el teclado.
- **Fuentes:** research/04-biblioteca-compartir.md §5.4 (Help / feedback); research/02-editor.md §1 (Help / feedback menu, Learning Center / offline help PDF); research/05-plataformas-versiones.md §1A (2025.3.0, 2026.4.0), §1B (2025.4.0)

### APP-016 · Notificaciones
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** notificación "Capture Copied" (Mac); avisos de compartir completado con notificaciones de Windows; ajustes de notificaciones
- **Comportamiento:**
  - XcreenShot usa el centro de notificaciones del sistema. Cada archivo dueño define el texto y las acciones de sus notificaciones; esta ficha define el mecanismo y las categorías.
  - Categorías, cada una con su casilla en Preferencias > General (todas activadas por defecto):

    | Categoría | Ejemplos | Clave | Dueño de los mensajes |
    |---|---|---|---|
    | Capturas | "Captura copiada" cuando la captura va al portapapeles sin abrir el editor; grabación guardada | `notifications.captures` | CAP-, VID- |
    | Compartir | envío terminado (con **Copiar enlace** o **Abrir**); envío fallido (con **Reintentar**) | `notifications.share` | SHR- |
    | Problemas | atajos que no se pudieron activar; permiso que falta; error al guardar | `notifications.problems` | APP-019, APP-024, LIB- |
    | Actualizaciones | actualización disponible o instalada | `notifications.updates` | APP-036 |
    | Automatización | captura o grabación pedida desde el CLI o un agente | `notifications.automation` | APP-032 |

  - Al hacer clic en una notificación se abre lo que nombra: la captura en el editor, la carpeta del archivo, la sección de Preferencias correspondiente, etc.
  - Durante una captura en pantalla o una grabación, XcreenShot no muestra notificaciones propias: las guarda y las muestra al terminar, salvo un error que detenga la grabación (decisión de diseño).
  - Se respetan los modos No molestar o Concentración del sistema.
  - Si el sistema tiene bloqueadas las notificaciones de XcreenShot, la sección General lo indica con un botón que abre el ajuste del sistema. Los problemas se siguen señalando con el punto de atención del ícono (APP-002).
- **Criterios de aceptación:**
  - [ ] Una captura con Abrir en = Ninguno y Copiar al portapapeles activado muestra "Captura copiada"; al hacer clic, la captura se abre en el editor.
  - [ ] Con la categoría Compartir desactivada, un envío terminado no muestra notificación.
  - [ ] Durante una grabación de pantalla completa no aparece ninguna notificación de XcreenShot en el video.
  - [ ] Con las notificaciones de XcreenShot bloqueadas en el sistema, la sección General muestra el aviso y el botón para abrir el ajuste.
- **Fuentes:** research/01-captura.md §11.3; research/04-biblioteca-compartir.md §5.3 (General), §5.4 (Telemetry and notifications); research/05-plataformas-versiones.md §1A (2025.0.0)

---

## 4. Atajos de teclado

### APP-017 · Ámbitos y reglas de los atajos
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** atajos globales y atajos dentro de la app; hotkeys configurables
- **Comportamiento:**
  - Un atajo siempre ejecuta un **comando** del registro (APP-027). Hay tres ámbitos:
    - **Global**: funciona en todo el sistema, aunque XcreenShot no tenga el foco ni ventanas abiertas. Ejemplos: Capturar, Captura universal, Repetir última captura, Iniciar/Pausar y Detener grabación, Dibujo en vivo, atajos de presets.
    - **De la app**: funciona cuando una ventana de XcreenShot tiene el foco (editor, biblioteca). Ejemplos: herramientas del editor, Copiar todo, Enlace para compartir.
    - **Contextual**: teclas que solo existen en un momento concreto (durante la selección, la grabación o la reproducción). Los define cada archivo dueño (por ejemplo, CAP- (Teclado durante la selección)) y aparecen en la hoja de atajos, pero no se reasignan en Preferencias (decisión de diseño).
  - **Combinaciones válidas:**
    - Global: al menos un modificador (`Ctrl`, `Alt`, `Shift`, `Win` en Windows; `Control`, `Option`, `Shift`, `Command` en macOS) más una tecla; o una tecla de función (`F1`–`F24`) o `PrtScn` solas o con modificadores. Una letra o un número sin modificador no se aceptan como atajo global.
    - De la app: cualquier combinación, incluidas las teclas solas que el dueño permita (por ejemplo, la letra de una herramienta).
    - No se aceptan como atajo: `Esc`, `Tab`, `Enter`, ni un modificador solo.
  - **Distribución de teclado.** Los atajos se guardan por posición física de tecla y se muestran con el carácter de la distribución activa (por ejemplo, la misma tecla aparece como `Ñ` en un teclado español). Cambiar de distribución de teclado no rompe los atajos.
  - Un atajo global no puede repetirse dentro de XcreenShot. Un atajo de la app no puede coincidir con un atajo global de XcreenShot.
  - Los atajos globales se registran al arrancar la app y se liberan al salir o al pausarlos (APP-002).
  - Si el sistema deja de entregar un atajo global (por ejemplo, tras suspender el equipo o cambiar de sesión), XcreenShot lo vuelve a registrar solo; si no puede, lo señala como en APP-019.
  - Los atajos están disponibles para el CLI: `xcreenshot shortcuts list --json` y `xcreenshot shortcuts set <comando> <combinación>`.
- **Criterios de aceptación:**
  - [ ] Asignar `A` sola como atajo global se rechaza con el motivo; asignar `Ctrl+Alt+A` se acepta.
  - [ ] Tras suspender y reanudar el equipo, el atajo Capturar responde a la primera pulsación.
  - [ ] Con un teclado español y otro estadounidense, el mismo atajo funciona en ambos y se muestra con el carácter de cada distribución.
  - [ ] Asignar a una herramienta del editor una combinación que ya es atajo global de XcreenShot se rechaza indicando qué comando la usa.
- **Fuentes:** research/01-captura.md §10; research/04-biblioteca-compartir.md §5.4 (Hotkeys (app level)); research/03-video.md §P

### APP-018 · Sección Atajos
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** pestaña Hotkeys de Capture Preferences (Windows); pestaña Keyboard de Settings (Mac)
- **Comportamiento:**
  - Arriba: el selector **Perfil** (APP-021), un campo **Buscar** (por nombre de comando o por combinación) y un filtro: Todos / Globales / De la app / Con conflicto / Personalizados.
  - Una tabla agrupada por área (Captura, Video, Editor, Biblioteca y bandeja, Compartir, Presets, App). Cada fila tiene:
    - el nombre del comando y su ámbito (Global / De la app);
    - el campo del atajo;
    - el estado: Activo, **En conflicto** (con el motivo) o **Tomado de otra app** (APP-020);
    - las acciones **Restaurar** (vuelve al valor del perfil; flecha circular) y **Quitar** (deja la acción sin atajo; ✕).
  - **Grabar un atajo:** al hacer clic en el campo (o al pulsar `Enter` con el foco en él), muestra "Pulsa la combinación…". La combinación se acepta al soltar las teclas. `Esc` cancela la grabación y deja el valor anterior; `Retroceso` o `Supr` lo quitan. Mientras se graba, los atajos globales de XcreenShot están en pausa para poder grabarlos.
  - El cambio se aplica al momento: el atajo anterior deja de funcionar y el nuevo empieza a funcionar, sin reiniciar.
  - Si la combinación ya la usa otro comando de XcreenShot, se pregunta: **Reasignar** (el otro comando queda sin atajo) o **Cancelar**.
  - Si la combinación está ocupada por el sistema o por otra app, se aplica APP-019.
  - Los atajos que se asignan en el campo Atajo del panel de captura, de un preset o de un destino (APP-022) son los mismos que aparecen aquí: cambiarlos en un sitio los cambia en el otro.
  - Los atajos contextuales se muestran en un grupo aparte, de solo lectura, con un enlace a la hoja de atajos (APP-023).
  - Botones al pie: **Imprimir o guardar hoja de atajos** (APP-023) y **Restablecer esta sección** (APP-010).
- **Criterios de aceptación:**
  - [ ] Grabar `Ctrl+Shift+5` en Capturar hace que esa combinación capture al momento, y la combinación anterior deja de capturar.
  - [ ] `Esc` durante la grabación deja el atajo como estaba.
  - [ ] Asignar a Captura universal la combinación de Capturar ofrece Reasignar y, al aceptar, Capturar queda sin atajo.
  - [ ] Cambiar el atajo de un preset en el panel de captura se refleja en la sección Atajos sin cerrarla.
  - [ ] Filtrar por Con conflicto muestra solo las filas en conflicto.
- **Fuentes:** research/01-captura.md §1.4, §10, §11.1, §11.2; research/04-biblioteca-compartir.md §5.1 (Hotkeys), §5.3 (Keyboard); research/03-video.md §P

### APP-019 · Detección de conflictos
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** aviso de atajo en uso; problemas de propiedad de `PrtScn` en Windows 11 con la Herramienta Recortes
- **Comportamiento:**
  - Al arrancar, y cada vez que se asigna un atajo global, XcreenShot comprueba si lo puede registrar. Distingue y explica, en lenguaje claro, estos casos:
    - **Reservado por el sistema.** Atajos que el sistema operativo no permite usar (por ejemplo, `Win+L` en Windows; `Cmd+Tab` en macOS). No se pueden asignar; el mensaje lo dice.
    - **Asignado por el sistema a otra función.** Por ejemplo:
      - en Windows 11, `PrtScn` puede estar asignada a la Herramienta Recortes;
      - en macOS, `Cmd+Shift+3`, `4` y `5` pertenecen a las capturas del sistema.
      El mensaje nombra el ajuste del sistema que hay que cambiar e incluye un botón que lo abre (Windows: Configuración > Accesibilidad > Teclado; macOS: Ajustes del Sistema > Teclado > Funciones rápidas de teclado).
    - **Ocupado por otra app.** Otra app ya registró la combinación (por ejemplo, Snagit, OneDrive, Dropbox, o el software del teclado o del ratón). XcreenShot dice el nombre de la app cuando el sistema lo permite saber y, si no, "otra aplicación".
  - **Al arrancar**, si algún atajo global no se pudo activar, aparece **un solo** aviso (notificación de la categoría Problemas, más el punto de atención del ícono) que lista los atajos afectados con un enlace a Preferencias > Atajos filtrado por Con conflicto. No se muestra un diálogo por cada atajo (decisión de diseño).
  - **Al asignar** un atajo en conflicto, el campo queda marcado (borde de error, ícono y texto; nunca solo color) con el motivo, y se ofrecen:
    - **Elegir otro**;
    - **Intentar tomarlo mientras XcreenShot esté abierto** (APP-020), solo si el sistema lo permite;
    - **Dejarlo así**: el atajo queda guardado pero inactivo; XcreenShot lo vuelve a intentar en cada arranque y cuando el usuario pulsa **Reintentar** en la fila.
  - Los atajos de la app (no globales) no entran en conflicto con otras apps, porque solo funcionan con XcreenShot en primer plano. Solo se comprueban contra los atajos de XcreenShot y del sistema.
  - Un aviso informa si una tecla de función depende de la tecla `Fn` del portátil o de software del fabricante del teclado (Notas de plataforma).
- **Criterios de aceptación:**
  - [ ] (Windows 11) Con la Herramienta Recortes asignada a `PrtScn` y el perfil Compatibilidad con Snagit, el aviso de arranque explica el ajuste del sistema y su botón abre esa página de Configuración.
  - [ ] Con otra app que ya usa `Ctrl+Shift+X`, asignar esa combinación marca el campo con el motivo "Ocupado por otra app".
  - [ ] Si tres atajos fallan al arrancar, aparece una sola notificación que lista los tres.
  - [ ] (macOS) Asignar `Cmd+Shift+4` muestra que el sistema lo usa para sus capturas y ofrece abrir Funciones rápidas de teclado.
- **Notas de plataforma:**
  - En Windows, algunos teclados de portátil envían `PrtScn` solo junto con `Fn`, y algunas utilidades de teclado o ratón reasignan teclas. XcreenShot no puede detectarlo; la ayuda de Atajos lo explica.
  - En macOS, el sistema no informa de qué app ocupa un atajo; el mensaje dice "otra aplicación".
- **Fuentes:** research/01-captura.md §1.4, §1.5, §10; research/04-biblioteca-compartir.md §5.1 (Hotkeys)

### APP-020 · Tomar atajos ocupados
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas (con detalle en Windows)
- **Paridad con Snagit:** "Allow Snagit to override other hotkey assignments" (Windows); oferta de quedarse con `PrtScn` mientras Snagit está abierto
- **Comportamiento:**
  - En Preferencias > Atajos, la opción **Tomar los atajos ocupados por otras apps mientras XcreenShot esté abierto** (clave `shortcuts.takeOver`; por defecto: no) y, para cada atajo en conflicto, la acción **Intentar tomarlo**.
  - Cuando se toma un atajo:
    - XcreenShot recibe la combinación y la otra app deja de recibirla **solo mientras XcreenShot está en marcha**. Al salir o pausar los atajos, la otra app vuelve a recibirla. XcreenShot nunca cambia la configuración de otra app ni del sistema;
    - la fila del atajo muestra **Tomado de otra app**.
  - **`PrtScn` en Windows 11.** Si el sistema tiene `PrtScn` asignada a la Herramienta Recortes, XcreenShot explica que puede tomarla mientras está abierto, o que se puede cambiar el ajuste del sistema para que no la reclame (con botón para abrirlo). El usuario elige.
  - Hay atajos que no se pueden tomar porque el sistema los reserva (APP-019). En ese caso la acción no aparece y el mensaje dice por qué.
  - Tomar un atajo nunca bloquea el uso normal del teclado: si XcreenShot deja de responder, el sistema recupera el atajo (no se usan capturas de teclado de bajo nivel que puedan bloquear la entrada; decisión de diseño).
- **Criterios de aceptación:**
  - [ ] (Windows 11) Con la Herramienta Recortes asignada a `PrtScn` y la tecla tomada, pulsar `PrtScn` abre la cruceta de XcreenShot y no la Herramienta Recortes.
  - [ ] (Windows 11) Tras salir de XcreenShot, `PrtScn` vuelve a abrir la Herramienta Recortes, sin cambiar ningún ajuste del sistema.
  - [ ] Si XcreenShot se cierra de forma inesperada con un atajo tomado, la otra app vuelve a recibir la combinación.
  - [ ] Un atajo reservado por el sistema no ofrece Intentar tomarlo.
- **Notas de plataforma:** en macOS, una combinación que ya tiene otra app no se puede tomar en general. La opción solo aparece donde el sistema lo permite, y la ayuda explica cómo liberar la combinación en la otra app.
- **Fuentes:** research/01-captura.md §1.4, §10, §11.1; research/04-biblioteca-compartir.md §5.1 (Hotkeys); research/03-video.md §P

### APP-021 · Perfiles de atajos
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** atajos por defecto de Snagit (para el perfil de compatibilidad)
- **Comportamiento:**
  - Dos perfiles incorporados (D-08), elegibles en Preferencias > Atajos > **Perfil** y en la bienvenida (APP-005):
    - **Coexistencia** (por defecto). Atajos globales que no chocan con los de Snagit ni con los del sistema, para que las dos apps convivan. Ejemplo: Capturar = `Ctrl+Shift+PrtScn` en Windows.
    - **Compatibilidad con Snagit.** Los mismos atajos que Snagit en cada plataforma, para conservar la memoria muscular. Ejemplo: Capturar = `PrtScn` en Windows y `Control+Shift+C` en macOS.
  - Las tablas de valores por defecto de cada perfil, para cada atajo, las fija el archivo dueño del comando: captura en CAP- (Acciones de captura y atajos por defecto) y CAP- (Perfil de compatibilidad); video en `VID-`; editor en `EDT-`; biblioteca y bandeja en `LIB-`/`TRY-`; compartir en `SHR-`. Los comandos de este archivo (Abrir editor, Preferencias, Pausar atajos globales, Salir) no tienen atajo global por defecto en Coexistencia. En Compatibilidad con Snagit, en Windows, `Ctrl+Shift+X` = Salir de XcreenShot. En macOS, Salir es `Cmd+Q` (atajo de la app) en ambos perfiles.
  - El perfil solo cambia los **valores por defecto**. Los atajos que el usuario personalizó se conservan al cambiar de perfil, salvo que elija **Cambiar y restablecer mis atajos**. El diálogo de cambio de perfil lista los atajos que cambiarán y los que entrarán en conflicto.
  - Si el usuario cambia a Compatibilidad con Snagit mientras Snagit está en marcha, el diálogo avisa de que los atajos chocarán. XcreenShot detecta que Snagit está en marcha por el nombre de su proceso y solo para este aviso (decisión de diseño).
  - Las diferencias de comportamiento que conlleva un perfil (por ejemplo, los modificadores de proporción durante la selección en Windows) las define el archivo dueño; ver CAP- (Perfil de compatibilidad).
  - Nombre en la interfaz: "Compatibilidad con Snagit" ("Snagit compatibility"). Nombra la otra app solo para describir con qué es compatible el perfil, sin usar su logotipo ni su identidad visual (SPEC §2.4). Si Carlos prefiere no nombrarla, el perfil se llama "Compatibilidad" y la descripción lo explica (pendiente de confirmar en `QUESTIONS.md`).
- **Criterios de aceptación:**
  - [ ] En una instalación nueva, el perfil es Coexistencia y ningún atajo global de XcreenShot coincide con un atajo por defecto de Snagit en esa plataforma.
  - [ ] Al pasar a Compatibilidad con Snagit, Capturar queda en `PrtScn` (Windows) o `Control+Shift+C` (macOS).
  - [ ] Un atajo personalizado se conserva al cambiar de perfil, salvo si se elige Cambiar y restablecer mis atajos.
  - [ ] Con Snagit en marcha, elegir Compatibilidad con Snagit muestra el aviso de choque antes de aplicar el cambio.
- **Fuentes:** research/01-captura.md §10; research/05-plataformas-versiones.md §3 (Default hotkeys); research/03-video.md §P; research/02-editor.md §12

### APP-022 · Atajos de presets, destinos y otros comandos
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** atajo por preset ("Add Hotkey" / "+ Shortcut"); atajos para destinos de compartir y para Share Link (Mac)
- **Comportamiento:**
  - Cualquier comando del registro (APP-027) que el dueño marque como asignable puede tener un atajo. Esto incluye los que se crean al usarse la app:
    - cada **preset de captura**: atajo global (CAP- (Atajo por preset));
    - cada **destino de compartir** y cada instancia con nombre: atajo de la app (SHR- (gestión de destinos));
    - el **Enlace para compartir**: atajo de la app (`SHR-`).
  - El archivo dueño define dónde se asigna el atajo en su propia interfaz (por ejemplo, el campo de atajo del preset). Esta ficha garantiza que:
    - todos aparecen también en Preferencias > Atajos, en su grupo, con las mismas reglas de validación y conflicto (APP-017, APP-019);
    - al borrar un preset o destino, su atajo se libera;
    - al renombrarlo, el atajo se conserva;
    - los presets importados (`.xpreset`) traen su atajo solo si no choca con ninguno existente; si choca, se importan sin atajo y se avisa.
- **Criterios de aceptación:**
  - [ ] Un atajo asignado a un preset desde el panel de captura aparece en la sección Atajos, en el grupo Presets.
  - [ ] Al eliminar el preset, su combinación queda libre y se puede asignar a otro comando sin aviso de conflicto.
  - [ ] Importar un preset cuyo atajo ya usa otro comando lo importa sin atajo y muestra el aviso.
- **Fuentes:** research/04-biblioteca-compartir.md §4 (Preset hotkeys), §5.3 (Share); research/01-captura.md §10

### APP-023 · Hoja de atajos
- **Hito:** M2 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** guías de atajos imprimibles (PDF) para Windows y Mac
- **Comportamiento:**
  - Ayuda > **Atajos de teclado** (y `Ctrl+/`/`Cmd+/` en cualquier ventana de XcreenShot) abre una ventana con todos los atajos **actuales** del usuario: globales, de la app y contextuales, agrupados por área y con un campo de búsqueda.
  - Refleja el perfil, los cambios del usuario y la distribución de teclado activa. Los atajos inactivos por conflicto aparecen marcados.
  - **Imprimir…** y **Guardar como PDF…** generan una hoja de una o dos páginas A4/Carta, en el idioma de la interfaz, con el nombre del perfil, la plataforma y la fecha.
  - También se puede generar con `xcreenshot shortcuts export --format pdf|html|json --out <archivo>`.
- **Criterios de aceptación:**
  - [ ] Tras cambiar el atajo de Capturar, la hoja muestra el nuevo valor.
  - [ ] Guardar como PDF produce un archivo de 1–2 páginas con todos los atajos globales y los de las herramientas del editor.
  - [ ] Buscar "flecha" en la hoja muestra el atajo de la herramienta Flecha.
- **Fuentes:** research/01-captura.md §10; research/02-editor.md §12

---

## 5. Permisos

### APP-024 · Permisos de macOS y Reparar permisos
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** macOS
- **Paridad con Snagit:** permiso de Grabación de pantalla y audio del sistema; Accesibilidad para el desplazamiento automático; Cámara y Micrófono; botón "Fix Capture Permissions"; la captura se reanuda al conceder el permiso
- **Comportamiento:**
  - Permisos que usa XcreenShot, cuándo los pide y qué pasa sin ellos:

    | Permiso del sistema | Para qué | Cuándo se pide | Sin el permiso |
    |---|---|---|---|
    | Grabación de pantalla (y audio del sistema) | Toda captura de pantalla o ventana y toda grabación; también el audio del sistema | En la bienvenida, o en la primera captura | No se captura. Se muestra el aviso; nunca se produce una imagen que solo contiene el escritorio (CAP- (Permisos que faltan al capturar)) |
    | Accesibilidad | Desplazamiento automático, captura de menús, grabador de pasos, detección de elementos de la interfaz, mostrar las teclas pulsadas en video | La primera vez que se usa una de esas funciones | Esas funciones se desactivan, con una explicación; el resto funciona |
    | Cámara | Cámara en video y captura de cámara como imagen | La primera vez que se activa la cámara | La cámara aparece como no disponible, con un enlace para resolverlo (`VID-`) |
    | Micrófono | Audio del micrófono en grabaciones y diapositivas narradas | La primera vez que se activa el micrófono | Se graba sin micrófono y se avisa antes de empezar (`VID-`) |

  - XcreenShot no pide ningún permiso antes de necesitarlo, salvo Grabación de pantalla en la bienvenida (APP-005). Antes de la solicitud del sistema, una ventana propia explica para qué sirve el permiso.
  - **Reparar permisos** (Preferencias > General > Permisos, menú Ayuda y botón de cada aviso) abre un panel que:
    - muestra el estado de cada permiso: Concedido / Denegado / Sin pedir;
    - para cada permiso no concedido, ofrece **Pedir** (si el sistema todavía permite pedirlo) o **Abrir Ajustes del Sistema** en la página exacta;
    - explica, con texto propio, qué hacer en el sistema: activar XcreenShot en la lista o, si aparece activado pero no funciona, quitarlo y volver a añadirlo;
    - si el sistema exige reiniciar la app para aplicar el permiso, ofrece **Reiniciar XcreenShot**, que vuelve a abrir lo que estaba abierto.
  - XcreenShot vuelve a comprobar los permisos cada vez que la app pasa a primer plano y antes de cada captura. Si un permiso cambia, lo refleja sin reiniciar.
  - **La captura se reanuda.** Si una captura o grabación se detuvo por falta de permiso, al concederlo continúa sola, sin volver a pulsar el atajo. Si el sistema exige reiniciar, la retoma después del reinicio (CAP- (Permisos que faltan al capturar)).
  - Si una actualización del sistema retira los permisos, el siguiente intento de captura muestra el aviso, no un fallo silencioso.
  - Los permisos se conceden a una sola app firmada, sin procesos auxiliares con nombre propio que el usuario tenga que autorizar por separado (decisión de diseño). La firma estable (APP-039) hace que las actualizaciones no pierdan los permisos.
- **Criterios de aceptación:**
  - [ ] Con Grabación de pantalla denegada, el atajo Capturar muestra el aviso con **Reparar permisos** y no crea ninguna imagen.
  - [ ] Tras conceder Grabación de pantalla desde Reparar permisos, la captura pendiente continúa sin volver a pulsar el atajo (o tras el reinicio que pida el sistema).
  - [ ] Sin el permiso de Accesibilidad, el desplazamiento automático aparece no disponible y su tooltip explica cómo activarlo; la captura de región funciona.
  - [ ] Tras actualizar XcreenShot a una versión nueva, los permisos concedidos siguen concedidos.
  - [ ] Con Micrófono denegado, al iniciar una grabación con micrófono aparece el aviso antes de empezar, no después.
- **Fuentes:** research/01-captura.md §14, §11.2; research/03-video.md §Q (macOS permissions); research/05-plataformas-versiones.md §1B (2024.1.0)

### APP-025 · Aviso periódico de acceso a la pantalla (macOS)
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** macOS
- **Paridad con Snagit:** aviso periódico de macOS 15 para permitir que la app use la pantalla sin el selector del sistema ("Allow For One Month")
- **Comportamiento:**
  - Las versiones recientes de macOS piden de vez en cuando que el usuario confirme que una app puede seguir capturando la pantalla sin usar el selector de ventanas del sistema. La confirmación puede valer un tiempo limitado.
  - XcreenShot:
    - explica este aviso en la ayuda de permisos, con texto propio, y recomienda la opción de mayor duración que ofrezca el sistema;
    - si el aviso aparece durante una captura, espera a que el usuario responda y luego continúa la captura sin perderla;
    - si el usuario elige no permitir, cancela la captura con un mensaje claro que enlaza a Reparar permisos, en lugar de fallar en silencio.
  - XcreenShot no intenta evitar ni automatizar la respuesta a este aviso.
- **Criterios de aceptación:**
  - [ ] Si el aviso del sistema aparece al pulsar Capturar, la cruceta aparece después de aceptar, sin volver a pulsar el atajo.
  - [ ] Si se rechaza el aviso, aparece un mensaje de XcreenShot con enlace a Reparar permisos y no se crea ninguna imagen.
- **Fuentes:** research/03-video.md §Q (macOS 15 Sequoia periodic prompt)

### APP-026 · Privilegios y privacidad en Windows
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Windows
- **Paridad con Snagit:** captura de ventanas elevadas; mensajes de privacidad de cámara y micrófono de Windows 10
- **Comportamiento:**
  - **Ventanas con más privilegios.** XcreenShot se ejecuta sin privilegios de administrador. Las ventanas de apps que se ejecutan como administrador (por ejemplo, el Administrador de tareas):
    - se pueden capturar como imagen o video, porque la captura de pantalla no depende de los privilegios;
    - pero algunas funciones que interactúan con la ventana pueden no responder en ellas: desplazamiento automático, detección de subáreas, el grabador de pasos y la captura de menús.
    - Cuando pasa, XcreenShot avisa y explica la opción de abrir XcreenShot como administrador para esa sesión (menú del ícono > **Reiniciar como administrador**). En ese modo, la barra de título indica "(Administrador)" (decisión de diseño). Al reiniciar sin privilegios, todo vuelve a la normalidad.
    - La forma en que cada función maneja las ventanas elevadas la define su dueño (por ejemplo, SMT- (Grabador de pasos)).
  - **Cámara y micrófono.** Si la privacidad de Windows bloquea la cámara o el micrófono para las aplicaciones de escritorio, XcreenShot lo detecta al activarlos y muestra un mensaje que explica el ajuste, con un botón que abre Configuración > Privacidad y seguridad > Cámara (o Micrófono).
  - **Contenido protegido.** Si el sistema devuelve en negro el contenido protegido (video con protección anticopia, ventanas que se excluyen de la captura), la captura termina igual y se avisa (CAP- (Permisos que faltan al capturar)).
  - **Programas de seguridad.** Si un programa de seguridad (por ejemplo, contra registradores de teclas) bloquea la captura o los atajos, el mensaje de error lo sugiere como posible causa.
  - El estado de privacidad de cámara y micrófono aparece en Preferencias > General > Permisos, igual que en macOS.
  - Escritorios remotos y máquinas virtuales: XcreenShot funciona dentro de una sesión remota o virtual. Las barras flotantes no deben quedar fuera de la pantalla visible de la sesión.
- **Criterios de aceptación:**
  - [ ] Una captura de región sobre el Administrador de tareas (elevado) produce una imagen con su contenido.
  - [ ] Si el desplazamiento automático no puede actuar sobre una ventana elevada, aparece el aviso con la opción de reiniciar como administrador.
  - [ ] Con el acceso al micrófono desactivado en la privacidad de Windows, activar el micrófono en el panel de captura muestra el mensaje con el botón que abre esa página de Configuración.
  - [ ] En una sesión de Escritorio remoto, el atajo Capturar y la cruceta funcionan y la barra de grabación aparece dentro de la pantalla.
- **Fuentes:** research/01-captura.md §14; research/03-video.md §Q (Windows requirements for video, Protected content, Virtual and remote environments)

---

## 6. Automatización

### APP-027 · Registro de comandos
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (base de la automatización que reemplaza la API COM; SPEC §2.2)
- **Comportamiento:**
  - Toda acción de XcreenShot es un **comando** registrado (SPEC §2.2). La interfaz, los atajos, el menú del ícono, el CLI y el servidor MCP llaman a los mismos comandos. Ninguna acción existe solo en la interfaz.
  - Cada comando tiene:
    - **ID** estable en inglés, con la forma `área.acción` (por ejemplo `capture.region`, `editor.export`, `library.search`);
    - **título y descripción** traducidos (APP-013);
    - **parámetros** con nombre, tipo, valor por defecto y si son obligatorios, descritos con un esquema JSON;
    - **resultado** descrito con un esquema JSON;
    - **ámbitos** en los que se puede usar: interfaz, atajo, CLI, MCP;
    - si es **interactivo** (necesita al usuario, por ejemplo una selección en pantalla) o no;
    - si **solo lee** o **cambia datos**, y si **sale a la red** (por ejemplo, un destino en línea);
    - si se le puede **asignar un atajo** (APP-022).
  - Áreas, cada una con su dueño: `app`, `settings`, `shortcuts` (este archivo); `capture`, `preset` (`CAP-`); `record` (`VID-`); `editor`, `document` (`EDT-`); `library`, `tray` (`LIB-`, `TRY-`); `share` (`SHR-`); `smart` (`SMT-`). Cada archivo dueño fija los parámetros y resultados de sus comandos; este archivo fija el mecanismo y el catálogo mínimo (APP-030).
  - **Errores.** Un comando que falla devuelve un error estructurado: `code` (estable, en inglés), `message` (en el idioma de la interfaz) y `details` opcionales. Códigos comunes: `invalid_params`, `not_found`, `busy`, `cancelled`, `permission_denied`, `disabled_by_policy`, `not_supported_on_platform`, `timeout`, `internal`.
  - **Comandos de prueba:** `app.ping` (devuelve versión, plataforma y estado) y `app.commands` (lista los comandos con sus esquemas).
  - El registro está versionado. Un cambio incompatible en un comando requiere un ID nuevo o una versión nueva del esquema; el anterior se mantiene al menos una versión menor con aviso de obsolescencia.
  - Una función que la configuración administrada desactiva no tiene comandos disponibles: se rechazan con `disabled_by_policy`.
- **Criterios de aceptación:**
  - [ ] `app.ping` responde igual desde un botón de la interfaz, un atajo de prueba y el CLI (criterio de salida de M0, SPEC §4) y, a partir de M2, desde el MCP.
  - [ ] `app.commands` devuelve todos los comandos del catálogo mínimo, cada uno con un esquema JSON de parámetros y de resultado válido.
  - [ ] Una prueba automática recorre los menús de la app y comprueba que cada entrada está asociada a un ID de comando registrado.
  - [ ] Un comando llamado con un parámetro desconocido falla con `invalid_params` y nombra el parámetro.
- **Fuentes:** research/04-biblioteca-compartir.md §8 (COM Server API); research/01-captura.md §1.9

### APP-028 · CLI `xcreenshot`
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** COM Server API (Windows), reemplazada por un CLI multiplataforma (SPEC §3.2)
- **Comportamiento:**
  - Forma general: `xcreenshot <área> <acción> [argumentos] [--parámetro valor …]`. Por ejemplo: `xcreenshot capture region --rect 0,0,800,600 --out ~/a.png`. También se acepta el ID completo: `xcreenshot run capture.region --params '<json>'`.
  - El CLI habla con la instancia en marcha (APP-001). Si la app no está en marcha, la arranca en segundo plano (sin ventanas), salvo con `--no-launch`, en cuyo caso falla con código 3.
  - **Salida:**
    - por defecto, texto legible en el idioma de la interfaz;
    - con `--json`, un único objeto JSON en la salida estándar: `{"ok": true, "result": {…}}` o `{"ok": false, "error": {"code": …, "message": …, "details": …}}`;
    - los mensajes de progreso y los avisos van a la salida de errores, para no mezclarse con el JSON.
  - **Códigos de salida:**

    | Código | Significado |
    |---|---|
    | 0 | Correcto |
    | 1 | Error general o interno |
    | 2 | Uso incorrecto (comando o parámetros no válidos) |
    | 3 | XcreenShot no está en marcha y no se pudo arrancar |
    | 4 | Cancelado por el usuario |
    | 5 | Ocupado (otra captura o grabación en curso) |
    | 6 | Bloqueado: falta permiso del sistema, la automatización está desactivada o lo impide la configuración administrada |
    | 7 | No encontrado (documento, preset, destino) |
    | 8 | Tiempo agotado |

  - **Opciones globales:** `--json`, `--quiet`, `--timeout <s>`, `--no-launch`, `--lang en|es` (idioma de los mensajes legibles), `--version`, `--help`.
  - **Nombres:** `xcreenshot <área> <acción>` corresponde al comando `<área>.<acción>`; en la acción se acepta `-` en lugar de `_` (`xcreenshot capture repeat-last` = `capture.repeat_last`). Los parámetros se escriben `--nombre-del-parámetro`.
  - **Subcomandos propios del CLI**, que no son comandos del registro porque solo tienen sentido en una terminal: `xcreenshot mcp` (APP-029), `xcreenshot events watch` (APP-031), `xcreenshot agents-md` (APP-033), `xcreenshot run` y `xcreenshot help`.
  - **Ayuda:** `xcreenshot --help`, `xcreenshot <área> --help` y `xcreenshot help <comando>` se generan del registro (APP-027), con parámetros, valores por defecto y ejemplos.
  - **Comandos interactivos** (selección en pantalla): el CLI espera a que el usuario termine, hasta `--timeout` (por defecto, sin límite). `Ctrl+C` en la terminal cancela la acción y devuelve el código 4.
  - **Entrada estándar:** `--params -` lee los parámetros en JSON de la entrada estándar. `--stdout` en los comandos que producen una imagen escribe sus bytes en la salida estándar, para encadenar con otras herramientas.
  - **Instalación en el PATH:** el instalador de Windows añade `xcreenshot` al PATH del usuario (APP-037). En macOS, Avanzado > Comando de terminal > **Instalar** crea un enlace en una carpeta del PATH del usuario, pidiendo permiso solo si hace falta (APP-009).
  - **Varias ejecuciones a la vez:** las de solo lectura (listar, buscar) se ejecutan en paralelo; las que capturan o graban responden `busy` (código 5) si ya hay una en curso (CAP- (Comandos de captura)).
- **Criterios de aceptación:**
  - [ ] `xcreenshot capture fullscreen --out <archivo> --json` crea el archivo, sale con 0 y escribe un JSON válido con la ruta.
  - [ ] Con la app cerrada, el mismo comando la arranca en segundo plano y termina con éxito; con `--no-launch` sale con código 3.
  - [ ] Pulsar `Esc` en una captura de región lanzada desde el CLI hace que este salga con código 4 y `"code": "cancelled"`.
  - [ ] `xcreenshot help capture.region` muestra todos los parámetros del esquema registrado.
  - [ ] Con `--json`, la salida estándar contiene solo el objeto JSON, aunque haya avisos.
- **Mejora sobre Snagit:** el mismo control en Windows y macOS, utilizable desde scripts y desde cualquier lenguaje, con resultados en JSON.
- **Fuentes:** research/04-biblioteca-compartir.md §8 (COM Server API, AppleScript / Shortcuts); research/01-captura.md §1.9

### APP-029 · Servidor MCP
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (reemplaza la API COM para su uso por agentes; SPEC §3.2)
- **Comportamiento:**
  - `xcreenshot mcp` inicia un servidor MCP (Model Context Protocol) que se comunica por **entrada y salida estándar**. Lo lanza el cliente MCP (un agente o asistente) como proceso hijo; ese proceso hace de puente con la instancia en marcha (APP-001), igual que el CLI.
  - **Herramientas MCP:** cada comando del registro con ámbito MCP es una herramienta. Su nombre es el ID con el punto sustituido por un guion bajo (`capture.fullscreen` → `capture_fullscreen`), para que sea válido en cualquier cliente. Lleva su descripción y el esquema de parámetros y resultado (APP-027). Las herramientas que cambian datos o salen a la red llevan esa indicación, para que el cliente pueda pedir confirmación.
  - **Imágenes en el resultado:** las herramientas que producen o leen una imagen (capturar, abrir un documento, exportar) devuelven la ruta del archivo y, si se pide con `include_image: true`, la imagen en línea, reducida a 1568 px en el lado mayor (decisión de diseño), para que el agente pueda verla.
  - **Recursos MCP** (solo lectura): la lista de presets, los destinos configurados, las capturas recientes y la ayuda de comandos de `AGENTS.md` (APP-033).
  - **Eventos:** el servidor envía como notificaciones MCP los eventos de APP-031 a los clientes que se suscriben.
  - **Configuración:** Avanzado > Configuración para clientes MCP > **Copiar** (APP-009) copia un bloque JSON con la ruta absoluta del ejecutable y el argumento `mcp`, listo para pegar en la configuración de un cliente.
  - Se aplican todas las reglas de seguridad de APP-032.
- **Criterios de aceptación:**
  - [ ] Un cliente MCP configurado con el bloque copiado lista las herramientas del catálogo mínimo (APP-030) con sus esquemas.
  - [ ] La herramienta `capture_fullscreen` con `include_image: true` devuelve la ruta y una imagen cuyo lado mayor mide 1568 px o menos.
  - [ ] Con la automatización desactivada en Avanzado, el servidor arranca pero cada herramienta responde `disabled_by_policy`, con un mensaje que nombra el ajuste.
  - [ ] El servidor no abre ningún puerto de red (se comprueba con las herramientas del sistema mientras está en marcha).
- **Notas de plataforma:** el bloque de configuración usa la ruta del ejecutable propia de cada plataforma y de cada tipo de instalación (instalada o portable).
- **Mejora sobre Snagit:** XcreenShot se puede operar desde agentes de IA en ambas plataformas, sin código específico.
- **Fuentes:** research/04-biblioteca-compartir.md §8 (COM Server API); research/01-captura.md §1.9

### APP-030 · Catálogo mínimo de comandos
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** entradas, salidas, selección de ventana y región, desplazamiento automático, retardo, nombres de archivo automáticos y filtros de imagen de la API COM
- **Comportamiento:**
  - Cada comando llega con su función y su hito. Esta tabla fija la **cobertura mínima** que debe exponerse por CLI y MCP. Los parámetros y resultados los define el dueño.

    | Necesidad | Comandos (ID) | Dueño |
    |---|---|---|
    | Capturar con cualquier modo, selección y opciones (cursor, retardo, efectos, abrir en, portapapeles, destinos, archivo) | `capture.<modo>` (por ejemplo `capture.region`, `capture.window`, `capture.fullscreen`, `capture.scrolling`, `capture.menu`, `capture.webpage`, `capture.clipboard`, `capture.file`), `capture.repeat_last` | CAP- (Comandos de captura) |
    | Ejecutar, listar y exportar presets | `preset.run`, `preset.list`, `preset.export`, `preset.import` | CAP- (Lista de presets), CAP- (Exportar e importar `.xpreset`) |
    | Grabar video | `record.start` (área, cámara, micrófono, audio del sistema, límite de tiempo), `record.pause`, `record.resume`, `record.stop`, `record.cancel`, `record.status` | `VID-` |
    | Abrir, anotar y exportar un documento | `document.open`, `document.info`, `document.annotate` (añade objetos descritos en JSON: flecha, texto, forma, paso, desenfoque, resaltado, con posición y estilo), `document.apply_effect`, `document.crop`, `document.resize`, `document.save`, `document.export` (formato, calidad, ruta) | `EDT-` |
    | Funciones inteligentes | `smart.extract_text`, `smart.redact` (categorías), `smart.steps_start`, `smart.steps_stop` | `SMT-` |
    | Biblioteca | `library.list`, `library.search` (texto, etiquetas, aplicación, sitio web, fecha, tipo), `library.get`, `library.tag`, `library.reveal`, `library.delete` | `LIB-` |
    | Compartir | `share.destinations` (lista), `share.send` (documento, destino o instancia, opciones), `share.link` | `SHR-` |
    | App | `app.ping`, `app.commands`, `app.show`, `app.quit`, `app.check_update`, `app.diagnostics`, `settings.open`, `settings.get`, `settings.set`, `settings.list`, `settings.schema`, `settings.export`, `settings.import`, `settings.reset`, `shortcuts.list`, `shortcuts.set`, `shortcuts.export` | APP- (este archivo) |

  - Los comandos que borran datos (`library.delete`, `settings.reset`, `settings.import`) exigen `--yes` en el CLI y llevan la indicación de destructivo en MCP.
  - `document.annotate` permite que un agente anote sin interfaz: recibe una lista de objetos con coordenadas en píxeles de la imagen y estilo (o el nombre de un estilo guardado) y devuelve los identificadores de los objetos creados, que se pueden seguir editando en el editor.
- **Criterios de aceptación:**
  - [ ] Desde el CLI, una secuencia `capture.fullscreen` → `document.annotate` (una flecha y un texto) → `document.export --format png` produce un PNG con la flecha y el texto en las posiciones pedidas, sin abrir ninguna ventana.
  - [ ] `library.search --app "Visual Studio Code" --json` devuelve solo capturas de esa aplicación.
  - [ ] `record.start` seguido de `record.stop` deja un `.mp4` en la biblioteca y devuelve su ruta y su duración.
  - [ ] `preset.run --name "<preset>"` produce el mismo resultado que el atajo del preset.
  - [ ] `share.send` a un destino local (carpeta) escribe el archivo y devuelve la ruta; a un destino en línea, se rechaza con `disabled_by_policy` mientras `automation.allowOnlineShare` esté desactivado.
- **Mejora sobre Snagit:** también cubre grabación, edición, biblioteca y compartir, que la API COM no ofrecía o cubría solo en parte.
- **Fuentes:** research/01-captura.md §1.9; research/04-biblioteca-compartir.md §8 (COM Server API)

### APP-031 · Eventos
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** estados (inactivo, correcto, fallido, ocupado) y eventos (error, cambio de estado, archivo escrito) de la API COM
- **Comportamiento:**
  - XcreenShot publica eventos que pueden seguir los scripts y los agentes:
    - `capture.started`, `capture.succeeded`, `capture.failed`, `capture.cancelled`;
    - `file.written` (ruta, tipo, tamaño), cada vez que se escribe un archivo de salida;
    - `record.started`, `record.paused`, `record.resumed`, `record.stopped`, `record.failed`;
    - `share.succeeded`, `share.failed`;
    - `app.state_changed` (inactiva / capturando / grabando / ocupada).
  - Cada evento es un objeto JSON con `event`, `time` (ISO 8601), `source` (interfaz, atajo, CLI, MCP o programada) y datos propios (por ejemplo, el identificador de la captura en la biblioteca). Se publican también los eventos de las acciones que hace el usuario en la interfaz, no solo las del CLI.
  - **CLI:** `xcreenshot events watch [--filter capture.*,file.written]` escribe un evento JSON por línea hasta que se interrumpe. **MCP:** notificaciones a los clientes suscritos (APP-029).
  - Los eventos no incluyen el contenido de las imágenes ni el texto reconocido; solo metadatos y rutas.
- **Criterios de aceptación:**
  - [ ] Con `events watch` activo, una captura hecha con el atajo produce `capture.started`, `capture.succeeded` y `file.written`, en ese orden, con `source: "shortcut"`.
  - [ ] Una captura cancelada con `Esc` produce `capture.cancelled` y ningún `file.written`.
  - [ ] Cada línea que escribe `events watch` es un JSON válido.
- **Fuentes:** research/01-captura.md §1.9; research/04-biblioteca-compartir.md §8 (COM Server API)

### APP-032 · Seguridad de la automatización
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (decisión de diseño)
- **Comportamiento:**
  - **Solo local.** El CLI y el servidor MCP se comunican con la app por un canal local del sistema (tubería con nombre en Windows; socket de dominio Unix en macOS), accesible solo por el **mismo usuario** del sistema. XcreenShot **no abre ningún puerto de red** para la automatización, ni en `localhost`. El MCP usa solo entrada y salida estándar.
  - **Interruptor general:** Avanzado > Permitir el control por CLI y agentes (clave `automation.enabled`; por defecto: sí). Desactivado, todo comando que llegue por CLI o MCP se rechaza con `disabled_by_policy` (código 6), salvo `app.ping`, que responde y dice que la automatización está desactivada. La configuración administrada puede bloquearlo.
  - **Visibilidad:** con `automation.notifyOnCapture` activado (por defecto), cada captura o grabación que empieza por CLI o MCP muestra una notificación (categoría Automatización) y la grabación muestra sus indicadores normales (ícono y barra). La automatización nunca puede grabar ni capturar de forma oculta: los indicadores de grabación no se pueden desactivar por comando (decisión de diseño).
  - **Salida a la red:** los comandos que envían a un destino en línea (`share.send` a un servicio, `share.link`) se rechazan desde CLI y MCP mientras `automation.allowOnlineShare` esté desactivado (por defecto). Los destinos locales (archivo, carpeta, portapapeles, aplicación) están siempre permitidos.
  - **Rutas:** los comandos que leen o escriben archivos aceptan cualquier ruta a la que el usuario tenga acceso. No siguen rutas de red de otros equipos salvo que el usuario las haya configurado como destino.
  - **Ajustes sensibles:** `settings.set` no puede cambiar `automation.*` ni la ubicación de la biblioteca. Esos ajustes solo se cambian desde la ventana de Preferencias o con la configuración administrada (decisión de diseño).
  - **Registro:** cada comando recibido por CLI o MCP se anota en el registro local (APP-046) con la hora, el ID del comando y el resultado, sin el contenido de los parámetros sensibles.
- **Criterios de aceptación:**
  - [ ] Con la app en marcha, ninguna herramienta del sistema muestra un puerto de red en escucha abierto por XcreenShot.
  - [ ] Otro usuario del mismo equipo no puede conectarse a la instancia de XcreenShot de un usuario (el CLI de ese otro usuario arranca o encuentra su propia instancia).
  - [ ] Una captura lanzada desde MCP muestra la notificación de automatización.
  - [ ] `xcreenshot settings set automation.enabled false` se rechaza con un mensaje que indica que ese ajuste solo se cambia en Preferencias.
- **Fuentes:** SPEC §2.3; research/04-biblioteca-compartir.md §8 (COM Server API)

### APP-033 · `AGENTS.md`
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (mejora)
- **Comportamiento:**
  - XcreenShot incluye un archivo `AGENTS.md` que explica a los agentes de IA cómo usar la app:
    - qué es XcreenShot y cómo conectarse (CLI y MCP);
    - la lista de comandos con una línea de descripción, sus parámetros y un ejemplo de cada área;
    - recetas: capturar y anotar una ventana; buscar en la biblioteca; grabar 30 segundos; extraer texto de una captura;
    - los códigos de error y de salida, y las reglas de seguridad (APP-032);
    - qué no puede hacer un agente (cambiar ajustes sensibles, grabar sin indicadores).
  - El archivo se **genera del registro de comandos** (APP-027) al compilar, más un texto introductorio escrito a mano. Así nunca queda desfasado respecto de la versión instalada.
  - Está en inglés (es para agentes; decisión de diseño). Se obtiene con `xcreenshot agents-md` (lo escribe en la salida estándar), como recurso MCP (APP-029) y en la carpeta de instalación.
- **Criterios de aceptación:**
  - [ ] `xcreenshot agents-md` produce un Markdown que menciona todos los comandos que lista `app.commands`.
  - [ ] Una prueba de compilación falla si un comando registrado no aparece en `AGENTS.md`.
  - [ ] Siguiendo solo la receta "capturar y anotar una ventana" de `AGENTS.md`, un agente obtiene un PNG anotado.
- **Fuentes:** research/04-biblioteca-compartir.md §8 (COM Server API, AppleScript / Shortcuts)

---

## 7. Funciones en prueba y configuración administrada

### APP-034 · Funciones en prueba
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** pestaña Experiments (sin recogida de datos ni botón de enviar comentarios)
- **Comportamiento:**
  - La sección **Funciones en prueba** de Preferencias lista las funciones que todavía no son definitivas. Cada una tiene:
    - un interruptor (todas **desactivadas** por defecto);
    - nombre, descripción breve y, si aplica, la plataforma;
    - **Más información**, que abre la página de ayuda local de la función.
  - Encabezado de la sección: aviso de que estas funciones pueden cambiar o desaparecer en una actualización y de que XcreenShot **no recoge ningún dato de uso** al activarlas.
  - Activar o desactivar una función se aplica al momento, sin reiniciar; si una función lo necesita, se indica.
  - Cuando una función pasa a ser definitiva, sale de la lista y queda activada en la interfaz normal. Si se retira, su interruptor desaparece y una nota en Novedades (APP-036) lo explica.
  - Los archivos dueños deciden qué funciones salen primero como prueba (por ejemplo, CAP- (Anotación rápida de imagen)). El mecanismo es el mismo para todas: una clave `experimental.<id>` que sus comandos y su interfaz consultan.
  - La configuración administrada puede ocultar la sección y forzar el estado de cada función (APP-035).
  - No hay botón de enviar comentarios por función; los comentarios van por Ayuda > Enviar comentarios (APP-015).
- **Criterios de aceptación:**
  - [ ] En una instalación nueva, todas las funciones en prueba aparecen desactivadas.
  - [ ] Activar una función en prueba la hace aparecer en su lugar de la interfaz sin reiniciar; desactivarla la oculta.
  - [ ] Con una función en prueba activada, la app no hace ninguna conexión de red nueva (se comprueba con un monitor de red durante su uso).
  - [ ] Con `policy.hideExperimental` en la configuración administrada, la sección no aparece y todas las funciones en prueba quedan desactivadas.
- **Fuentes:** research/04-biblioteca-compartir.md §5.2 (Experiments), §5.3 (Experiments), §7; research/05-plataformas-versiones.md §5; research/02-editor.md §11

### APP-035 · Configuración administrada
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** herramienta de despliegue empresarial (MST en Windows; claves de preferencias en Mac), en su parte visible para el usuario
- **Comportamiento:**
  - Un administrador puede colocar un archivo opcional **`managed.json`** que fija valores por defecto y bloquea ajustes, para todos los usuarios del equipo.
  - **Ubicación**, de solo lectura para usuarios normales:
    - Windows: `%ProgramData%\XcreenShot\managed.json`;
    - macOS: `/Library/Application Support/XcreenShot/managed.json`;
    - modo portable (APP-038): junto al ejecutable.
  - **Formato:**
    ```json
    {
      "version": 1,
      "defaults": { "general.language": "es" },
      "locked":   { "updates.channel": "deferred" },
      "policy":   { "disableVideoCapture": true }
    }
    ```
    - `defaults`: valor inicial de cualquier clave de ajuste; el usuario lo puede cambiar.
    - `locked`: valor fijo; el usuario no lo puede cambiar (APP-006).
    - `policy`: directivas que no son ajustes normales.
  - **Directivas mínimas:**

    | Directiva | Efecto | Dueño del detalle |
    |---|---|---|
    | `library.location` (en `defaults` o `locked`) | Ubicación de la biblioteca; con bloqueo, no se puede cambiar | LIB- (ubicación) |
    | `policy.disabledDestinations` (lista) | Esos destinos de compartir no aparecen en ningún sitio | SHR- (gestión de destinos) |
    | `policy.shareLinkProvider` | Proveedor y opciones del Enlace para compartir | SHR- (Enlace para compartir) |
    | `policy.disableVideoCapture` | Sin grabación de video: se ocultan la pestaña Video, la sección Video de Preferencias, los atajos y los comandos `record.*` | VID- |
    | `updates.channel` y `updates.autoCheck` / `updates.autoInstall` | Canal y comportamiento de las actualizaciones | APP-036 |
    | `policy.disableUpdates` | Sin búsqueda de actualizaciones ni la entrada Buscar actualizaciones | APP-036 |
    | `policy.updateSource` | Dirección alternativa (interna) del manifiesto de actualizaciones | APP-036 |
    | `policy.hideExperimental` y `policy.experimental` (estado por función) | Oculta la sección Funciones en prueba o fuerza funciones | APP-034 |
    | `policy.automation` (`enabled`, `allowOnlineShare`) | Bloquea la automatización | APP-032 |
    | `policy.skipFirstRun` | No muestra la bienvenida | APP-005 |
    | `policy.feedback` (`hidden` o dirección) | Oculta o redirige Enviar comentarios | APP-015 |
    | `policy.presets`, `policy.themes` (rutas a `.xpreset` / `.xtheme`) | Se instalan para cada usuario al primer arranque | CAP-, EDT- |

  - **Al arrancar**, XcreenShot lee el archivo y lo valida contra su esquema. Si tiene errores, aplica las partes válidas, ignora las demás, lo anota en el registro y lo muestra en Avanzado > Configuración administrada (estado Con errores y lista de errores). Nunca deja de arrancar por un archivo mal formado.
  - Avanzado > Configuración administrada > **Ver** muestra la ruta, la fecha y qué ajustes están fijados o bloqueados.
  - Los cambios en el archivo se aplican en el siguiente arranque de la app.
  - El esquema JSON del archivo se publica junto con la app y lo exporta `xcreenshot settings schema`, para que el administrador pueda validarlo.
- **Criterios de aceptación:**
  - [ ] Con `"locked": {"updates.channel": "deferred"}`, Preferencias muestra el canal Diferido con candado y `settings set updates.channel stable` falla con código 6.
  - [ ] Con `policy.disableVideoCapture: true`, no aparecen ni la pestaña Video del panel de captura ni la sección Video de Preferencias, y `xcreenshot record start` falla con `disabled_by_policy`.
  - [ ] Con `policy.disabledDestinations: ["youtube"]`, ese destino no aparece en el menú Compartir ni en Preferencias > Compartir.
  - [ ] Con un `managed.json` con un error de sintaxis, la app arranca, ignora el archivo y Avanzado muestra el estado Con errores con la línea del error.
- **Mejora sobre Snagit:** un único archivo legible y con el mismo formato en las dos plataformas, en lugar de transformaciones del instalador en Windows y claves de preferencias en Mac.
- **Fuentes:** research/04-biblioteca-compartir.md §9 (Admin deployment controls), §7; research/03-video.md §R; research/01-captura.md §14; research/05-plataformas-versiones.md §3 (Settings and deployment)

---

## 8. Actualizaciones y distribución

### APP-036 · Actualizaciones
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** actualizador integrado; buscar actualizaciones automáticamente (con frecuencia); descargar e instalar automáticamente; canal Standard/Delayed; Buscar actualizaciones
- **Comportamiento:**
  - Ajustes en Preferencias > General > Actualizaciones:

    | Ajuste | Valores (por defecto) | Clave |
    |---|---|---|
    | Buscar actualizaciones automáticamente | **Sí** / No | `updates.autoCheck` |
    | Frecuencia | **Diaria** / Semanal | `updates.frequency` |
    | Descargar e instalar automáticamente | Sí / **No** | `updates.autoInstall` |
    | Canal | **Estable** / Diferido | `updates.channel` |

  - **Canales:**
    - **Estable**: recibe cada versión en cuanto se publica;
    - **Diferido**: recibe una versión cuando lleva al menos 30 días publicada en Estable sin una corrección posterior, con sus correcciones ya incluidas (decisión de diseño). Las correcciones de seguridad llegan a los dos canales a la vez.
  - **Buscar actualizaciones** (menú Ayuda, menú rápido y `xcreenshot app check-update`) busca en el momento, con o sin la búsqueda automática activada.
  - Cuando hay una versión nueva, aparece una notificación (categoría Actualizaciones) y una ventana con:
    - versión actual y versión nueva;
    - **notas de la versión** en el idioma de la interfaz (inglés si no hay traducción);
    - los botones **Instalar y reiniciar**, **Instalar al salir** y **Ahora no**.
  - **Descargar e instalar automáticamente:** la descarga se hace en segundo plano y la instalación se aplica al salir de la app o en el siguiente arranque. Nunca interrumpe una captura, grabación o edición en curso.
  - **Firma:** cada paquete de actualización se verifica antes de instalarlo, con la firma de la plataforma (APP-037, APP-039) y además con una firma propia del manifiesto de actualizaciones. Si la verificación falla, la actualización se descarta, se anota en el registro y se avisa; la versión instalada sigue funcionando.
  - **Novedades** (menú Ayuda) muestra las notas de la versión instalada y de las anteriores, incluidas en la app, por lo que funciona sin conexión. Tras actualizar, la primera vez que se abre el editor aparece una línea "Actualizado a la versión X · Ver novedades" (descartable).
  - La actualización conserva ajustes, atajos, permisos de macOS y biblioteca, y no requiere privilegios de administrador para una instalación por usuario (APP-037).
  - Las únicas conexiones de red de esta función son: descargar el manifiesto de actualizaciones (sin enviar más datos que la versión, la plataforma, la arquitectura y el canal) y descargar el paquete (APP-054).
  - Usa el proxy configurado en el sistema.
  - La configuración administrada puede fijar el canal, desactivar las actualizaciones o apuntar a un servidor interno (APP-035). En modo portable, la actualización reemplaza los archivos en la carpeta portable (APP-038).
- **Criterios de aceptación:**
  - [ ] Con una versión nueva publicada en Estable, Buscar actualizaciones la muestra con sus notas y **Instalar y reiniciar** deja la app actualizada y abierta, con los mismos ajustes.
  - [ ] Con el canal Diferido, una versión publicada hace 10 días en Estable no se ofrece.
  - [ ] Un paquete con la firma alterada no se instala y aparece el aviso.
  - [ ] Con Descargar e instalar automáticamente activado y una grabación en curso, la actualización no se aplica hasta que la app se cierra.
  - [ ] Con la búsqueda automática desactivada, un monitor de red no registra ninguna conexión de XcreenShot durante un día de uso sin compartir.
- **Fuentes:** research/04-biblioteca-compartir.md §5.1 (General), §5.3 (General), §5.4 (Updates); research/01-captura.md §11.1, §11.2, §14; research/03-video.md §R (Update channel); research/05-plataformas-versiones.md §1A (2024.0.3, 2025.2.1, 2026.4.0), §1B (2026.4.0)

### APP-037 · Instalador de Windows
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Windows
- **Paridad con Snagit:** instalador de Windows (EXE/MSI), instalación silenciosa, selección de idioma en la instalación
- **Comportamiento:**
  - Un único instalador `.exe`, firmado con certificado de firma de código (D-06), que **no necesita privilegios de administrador**: instala para el usuario actual (en su carpeta de programas por usuario) y no muestra el aviso de control de cuentas.
  - El asistente es breve: licencia, carpeta (con un valor por defecto que no hace falta cambiar), casillas **Iniciar al iniciar sesión** y **Añadir `xcreenshot` al PATH** (ambas marcadas) e Instalar. Usa el idioma del sistema (inglés o español).
  - Crea un acceso directo en el menú Inicio, registra las asociaciones de archivo (`.xshot`, `.xpreset`, `.xtheme`, `.xstamps`, `.xtemplate`, `.xsettings`) para el usuario y aparece en Configuración > Aplicaciones, desde donde se desinstala (APP-042).
  - **Instalación desatendida:** `/silent` (sin interfaz) con opciones `/nolaunchatlogin` y `/nopath`. Termina con código 0 si la instalación fue correcta y distinto de 0 si falló.
  - **Instalación para todos los usuarios** (opcional, para administradores): `/allusers` instala en la carpeta de programas del equipo y sí pide privilegios. La configuración administrada se coloca aparte (APP-035).
  - Si XcreenShot está en marcha, el instalador le pide salir (guardando lo pendiente), instala y lo vuelve a abrir.
  - Instalar encima de una versión anterior conserva ajustes, atajos y biblioteca.
  - Si faltan componentes del sistema que la app necesita (por ejemplo, el motor web de Windows), el instalador los obtiene o explica cómo hacerlo. En las ediciones de Windows sin componentes multimedia, avisa de que el video necesita instalarlos.
  - Se publica un instalador por arquitectura (x64 y ARM64, APP-041) y un instalador que elige solo la arquitectura correcta.
- **Criterios de aceptación:**
  - [ ] En una cuenta de usuario estándar (sin privilegios de administrador), el instalador termina sin pedir credenciales y la app arranca.
  - [ ] `XcreenShot-Setup.exe /silent` instala sin mostrar ninguna ventana y sale con código 0.
  - [ ] Tras instalar, `xcreenshot --version` funciona en una terminal nueva.
  - [ ] Doble clic en un `.xshot` abre XcreenShot.
  - [ ] Windows no muestra el aviso de editor desconocido al ejecutar el instalador (firma válida).
- **Fuentes:** research/05-plataformas-versiones.md §1A (2024.1.4, 2024.2.0, 2025.0.0, 2025.2.1), §3 (Settings and deployment), §4 (Windows); research/04-biblioteca-compartir.md §5.4 (Language)

### APP-038 · Modo portable (Windows)
- **Hito:** M6 · **Prioridad:** Could
- **Plataformas:** Windows
- **Paridad con Snagit:** — (mejora)
- **Comportamiento:**
  - Se publica también como `.zip` portable. Si junto al ejecutable hay un archivo `portable.flag` (incluido en el `.zip`), XcreenShot funciona en modo portable:
    - guarda ajustes, registros y datos de la app en una carpeta `data` junto al ejecutable, no en el perfil del usuario;
    - la biblioteca por defecto es la carpeta `library` junto al ejecutable (se puede cambiar como en `LIB-`);
    - no registra asociaciones de archivo ni el PATH; Iniciar al iniciar sesión funciona mientras la carpeta no cambie de sitio (APP-003);
    - no deja nada en el registro de Windows ni en el perfil del usuario, salvo lo que el propio sistema guarda (por ejemplo, la lista de apps recientes).
  - La barra de título y Acerca de indican "(portable)".
  - Las actualizaciones (APP-036) reemplazan los archivos en la misma carpeta; si la carpeta no se puede escribir, se avisa y se ofrece descargar el `.zip` nuevo.
  - Al iniciar en modo portable en una ruta que no se puede escribir (por ejemplo, un medio de solo lectura), avisa y funciona con los datos en una carpeta temporal, sin guardar nada al salir.
- **Criterios de aceptación:**
  - [ ] Copiar la carpeta portable a una memoria USB, usarla en otro equipo y volver lleva consigo los ajustes, atajos y capturas.
  - [ ] Tras usar la versión portable, el perfil del usuario no tiene ninguna carpeta de XcreenShot.
  - [ ] Si en el equipo también está instalada la versión normal, cada una conserva sus propios ajustes. Si las dos se ejecutan a la vez, la segunda avisa de que ya hay una instancia en marcha y se cierra.
- **Mejora sobre Snagit:** se puede usar sin instalar, por ejemplo en equipos ajenos o de prueba.
- **Fuentes:** research/05-plataformas-versiones.md §3 (Settings and deployment), §4 (Windows)

### APP-039 · Paquete de macOS
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** macOS
- **Paridad con Snagit:** app universal nativa en Apple Silicon; distribución fuera de la App Store
- **Comportamiento:**
  - Se distribuye como imagen `.dmg` que contiene la app, **firmada con un certificado de desarrollador de Apple y notarizada** (D-06), para que macOS la abra sin avisos de seguridad.
  - La app es **universal**: nativa en Apple Silicon e Intel en un solo paquete (decisión de diseño). Si en algún momento mantener Intel deja de ser viable, se anunciará en Novedades con una versión de antelación. Las funciones que el sistema no ofrece en Intel quedan marcadas en sus fichas (`VID-`).
  - Instalar consiste en arrastrar la app a Aplicaciones, como indica el fondo del `.dmg`. Si se abre la app desde el propio `.dmg` o desde Descargas, ofrece moverse a Aplicaciones (necesario para que los permisos y las actualizaciones funcionen bien).
  - La identidad de firma es estable entre versiones, para que los permisos concedidos (APP-024) se conserven al actualizar.
  - No se distribuye por la Mac App Store en v1 (decisión de diseño: las restricciones de esa tienda impiden partes de la automatización y del control de otras apps).
  - Las asociaciones de archivo (`.xshot`, `.xpreset`, `.xtheme`, `.xstamps`, `.xtemplate`, `.xsettings`) se declaran en la app.
- **Criterios de aceptación:**
  - [ ] En un Mac recién configurado, abrir la app descargada no muestra el aviso de desarrollador no identificado.
  - [ ] La misma descarga se ejecuta de forma nativa en un Mac con Apple Silicon y en uno con Intel (Monitor de Actividad muestra la arquitectura nativa).
  - [ ] Abrir la app desde el `.dmg` ofrece moverla a Aplicaciones y, al aceptar, se relanza desde allí.
- **Fuentes:** research/05-plataformas-versiones.md §4 (macOS), §3 (Settings and deployment)

### APP-040 · Requisitos del sistema
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** requisitos del sistema publicados (sistemas, CPU, memoria, disco)
- **Comportamiento:**
  - **Sistemas compatibles** (decisión de diseño, a partir de los de Snagit):

    | Plataforma | Mínimo | Notas |
    |---|---|---|
    | Windows | Windows 10 22H2 y Windows 11 (23H2 o posterior), 64 bits (x64 o ARM64) | Ediciones N: el video necesita el paquete de componentes multimedia. Windows Server no se prueba (puede funcionar). |
    | macOS | macOS 14 Sonoma o posterior | Apple Silicon e Intel (APP-039). |

  - **Hardware recomendado:** 8 GB de RAM; procesador de los últimos ~8 años con codificación de video por hardware para grabar (si no la hay, se usa codificación por software, `VID-`); 500 MB libres para la app, más el espacio de la biblioteca.
  - **Otros requisitos:** sin requisitos de red para usar la app (APP-054).
  - En un sistema por debajo del mínimo, el instalador o la app explican que no es compatible y no se instalan o no arrancan a medias.
  - Las funciones que necesitan una versión concreta del sistema (por ejemplo, excluir las ventanas propias de la captura en Windows, o ciertos efectos de cámara en macOS) lo indican en su ficha, se desactivan en versiones anteriores y explican el motivo en su tooltip.
  - La lista de requisitos aparece en la ayuda y en las notas de la versión; se revisa en cada versión mayor.
- **Criterios de aceptación:**
  - [ ] La app se instala y pasa las pruebas de humo en Windows 10 22H2, Windows 11 (x64 y ARM64), macOS 14 y la versión más reciente de macOS.
  - [ ] En macOS 13, la app muestra un mensaje de sistema no compatible en lugar de cerrarse sin explicación.
  - [ ] Una función que necesita una versión del sistema más nueva aparece desactivada, con el motivo, en un sistema más antiguo dentro del mínimo.
- **Notas de plataforma:** Snagit 2026 exige macOS 15 o posterior. XcreenShot admite también macOS 14 porque las funciones de captura que usa ya existen en esa versión; si una función solo existe a partir de macOS 15, se aplica la regla de funciones que necesitan una versión concreta.
- **Fuentes:** research/05-plataformas-versiones.md §4 (Windows, macOS); research/04-biblioteca-compartir.md §9 (Platform requirements); research/03-video.md §Q (Windows requirements for video, macOS-version-specific behavior); research/01-captura.md §14

### APP-041 · Windows en ARM
- **Hito:** M6 · **Prioridad:** Should
- **Plataformas:** Windows
- **Paridad con Snagit:** Snagit se ejecuta emulado en Windows on ARM; no tiene compilación nativa y su impresora virtual no funciona en ARM
- **Comportamiento:**
  - XcreenShot publica una compilación **nativa ARM64** para Windows (mejora).
  - Todas las funciones son las mismas que en x64. Si alguna depende de un componente del sistema que no existe en ARM64, se marca en su ficha con el motivo.
  - El instalador detecta la arquitectura e instala la compilación correcta (APP-037). Las actualizaciones respetan la arquitectura.
  - Si la compilación nativa no estuviera lista para el gate de paridad, se distribuye temporalmente la versión x64 (que Windows ejecuta emulada) y se anota en `PARITY.md`.
- **Criterios de aceptación:**
  - [ ] En un equipo Windows 11 ARM64, el Administrador de tareas muestra XcreenShot como proceso ARM64.
  - [ ] Las pruebas de humo de captura, edición, grabación y compartir pasan en ARM64.
  - [ ] En ARM64, el tiempo del atajo a la cruceta cumple APP-048.
- **Mejora sobre Snagit:** ejecución nativa, sin emulación, en equipos Windows con ARM.
- **Fuentes:** research/05-plataformas-versiones.md §4 (Windows: ARM64), §3 (Capture)

### APP-042 · Desinstalación
- **Hito:** M6 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** desinstalación sin privilegios de administrador; la biblioteca se conserva
- **Comportamiento:**
  - **La desinstalación nunca borra la biblioteca** ni ningún archivo que el usuario haya guardado fuera de la carpeta de la app.
  - **Windows:** se desinstala desde Configuración > Aplicaciones, sin privilegios de administrador en una instalación por usuario. El desinstalador:
    - cierra la app;
    - quita los archivos del programa, el acceso directo, las asociaciones de archivo, la entrada del PATH y el arranque al iniciar sesión;
    - pregunta si se quieren borrar también los **ajustes y datos de la app** (no la biblioteca; por defecto: conservarlos). La pregunta dice dónde está la biblioteca y que no se tocará.
    - `/silent` desinstala sin preguntar y conserva los ajustes.
  - **macOS:** basta con mover la app a la Papelera; los ajustes y datos quedan en la carpeta de la app del usuario. Avanzado > **Quitar XcreenShot de este Mac…** (APP-009) hace una desinstalación completa: quita el arranque al iniciar sesión y el comando de terminal; opcionalmente borra los ajustes y datos de la app (nunca la biblioteca); y por último mueve la app a la Papelera y sale.
  - Una reinstalación posterior encuentra la biblioteca en su sitio y la usa (o la sugiere si no está en la ubicación por defecto, APP-055).
- **Criterios de aceptación:**
  - [ ] Tras desinstalar en Windows y elegir borrar los ajustes, la carpeta de la biblioteca sigue con todas sus capturas.
  - [ ] Tras desinstalar, el atajo global ya no responde y XcreenShot no aparece en las apps de inicio.
  - [ ] Reinstalar tras una desinstalación que conservó los ajustes recupera los ajustes, los atajos y la biblioteca.
- **Fuentes:** research/05-plataformas-versiones.md §1A (2024.1.4); research/04-biblioteca-compartir.md §1.3 (Export Library on exit)

---

## 9. Accesibilidad

### APP-043 · Navegación con teclado
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** navegación completa con teclado e indicador de foco; teclas de flecha en deslizadores y campos numéricos
- **Comportamiento:**
  - Todas las ventanas, paneles, menús y diálogos de XcreenShot se pueden usar **solo con el teclado**:
    - `Tab` / `Shift+Tab` recorren los controles en un orden lógico (de arriba abajo y de izquierda a derecha);
    - las teclas de flecha se mueven dentro de un grupo (barras de herramientas, listas, pestañas, cuadrículas de la biblioteca y la bandeja);
    - `Enter` / `Espacio` activan; `Esc` cierra menús y diálogos, o cancela;
    - `F6` (Windows) o `Ctrl+F6`/`Cmd+F6` mueve el foco entre las grandes zonas de la ventana: barra de herramientas, lienzo, panel lateral, bandeja (decisión de diseño).
  - El **foco es siempre visible**: un anillo de 2 px o más, con contraste de al menos 3:1 sobre el fondo, en ambos temas. Nunca se oculta el foco al usar el teclado.
  - **Deslizadores:** las flechas los mueven en un paso pequeño; `Re Pág` / `Av Pág` en un paso grande; `Inicio` / `Fin` al mínimo y al máximo.
  - **Campos numéricos:** flecha arriba y abajo suman o restan 1; con `Shift`, 10. El valor se aplica al momento con vista previa.
  - Al abrir un diálogo, el foco va a su primer control útil; al cerrarlo, vuelve a donde estaba.
  - Las acciones que en la interfaz se hacen arrastrando (reordenar presets o elementos de la bandeja, mover objetos en el lienzo) tienen una alternativa con el teclado, definida por cada dueño.
  - Las barras flotantes que aparecen durante una captura o grabación también se pueden usar con el teclado. Cada dueño define cómo se llega a ellas (por ejemplo, con una tecla).
- **Criterios de aceptación:**
  - [ ] Sin ratón, se puede abrir Preferencias, cambiar el tema, grabar un atajo nuevo y cerrar la ventana.
  - [ ] En cualquier punto de un recorrido con `Tab` por el editor, el control con foco se identifica visualmente en ambos temas.
  - [ ] En un deslizador de opacidad, cada pulsación de flecha cambia el valor y la vista previa.
  - [ ] Un campo numérico con valor 10 pasa a 20 con `Shift+↑`.
- **Fuentes:** research/04-biblioteca-compartir.md §5.4 (Accessibility); research/02-editor.md §1 (Accessibility); research/05-plataformas-versiones.md §1A (2024.3.5, 2026.0.0)

### APP-044 · Lectores de pantalla
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** etiquetas para lectores de pantalla y mejoras continuas en diálogos y paneles
- **Comportamiento:**
  - Todos los controles tienen un **nombre accesible**, un **rol** y, si aplica, un **estado** y un **valor**, en el idioma de la interfaz. Los botones que solo tienen ícono usan el nombre de la acción (por ejemplo, "Flecha", no "Botón 3").
  - Funciona con los lectores de pantalla del sistema: Narrador y NVDA en Windows; VoiceOver en macOS.
  - Se anuncian los cambios importantes que no mueven el foco, por ejemplo:
    - "Captura copiada al portapapeles";
    - "Grabación iniciada";
    - el número de resultados de una búsqueda en la biblioteca;
    - los errores de validación.
  - Las miniaturas de la biblioteca y la bandeja se anuncian con su nombre, tipo, fecha y aplicación de origen.
  - Los objetos del lienzo se anuncian con su tipo y su texto, si lo tienen (por ejemplo, "Texto: Paso 1"). El detalle lo define `EDT-`.
  - Los atajos de cada control se exponen al lector de pantalla.
  - La selección en pantalla durante una captura es visual por naturaleza, pero se anuncian el modo, el tamaño de la selección y las teclas disponibles, y las capturas no interactivas (pantalla completa, ventana activa, repetir última área) no dependen de la vista.
- **Criterios de aceptación:**
  - [ ] Con VoiceOver o Narrador, al recorrer la barra de herramientas del editor, cada herramienta se lee con su nombre y su atajo.
  - [ ] Al terminar una captura que va al portapapeles, el lector anuncia "Captura copiada".
  - [ ] Una auditoría automática de accesibilidad de las ventanas principales no encuentra controles sin nombre accesible.
  - [ ] Todas las secciones de Preferencias se pueden recorrer y cambiar con el lector de pantalla.
- **Fuentes:** research/04-biblioteca-compartir.md §5.4 (Accessibility); research/02-editor.md §1 (Accessibility); research/05-plataformas-versiones.md §1A (2024.0.2)

### APP-045 · Contraste, escalado y movimiento
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** tema oscuro para contraste; escalado de DPI del sistema
- **Comportamiento:**
  - **Alto contraste:** con los temas de contraste de Windows o el ajuste Aumentar contraste de macOS, XcreenShot usa los colores del sistema o refuerza bordes y separadores. Ninguna información depende solo del color: los estados llevan también ícono, forma o texto.
  - **Escalado:** la interfaz respeta el escalado del sistema (100 % a 300 % en Windows; pantallas Retina en macOS) y el tamaño de texto del sistema donde el sistema lo ofrezca. Con monitores con escalados distintos, cada ventana se dibuja nítida en el monitor donde está, también al moverla de uno a otro. La relación entre escalado y tamaño de las capturas la define CAP- (Varios monitores y DPI mixto).
  - **Reducir movimiento:** con el ajuste de reducir movimiento o animaciones del sistema activo, XcreenShot sustituye las animaciones de la interfaz (transiciones, pistas animadas, desplazamientos suaves) por cambios instantáneos o fundidos breves. No afecta al contenido del usuario (videos, GIF), ni a los efectos de cursor que se graban, que son contenido.
  - **Transparencia:** con el ajuste de reducir transparencia del sistema activo, los paneles translúcidos pasan a ser opacos.
  - Los textos de la interfaz tienen un tamaño mínimo de 11 pt a 100 % de escala.
- **Criterios de aceptación:**
  - [ ] Con un tema de contraste de Windows activo, todos los controles del editor siguen visibles y con borde.
  - [ ] A 200 % de escala, el texto y los íconos se ven nítidos y ninguna ventana se sale de la pantalla en un monitor de 1920×1080.
  - [ ] Con reducir movimiento activado en el sistema, abrir y cerrar paneles no tiene animaciones de más de 100 ms.
  - [ ] Al mover una ventana de un monitor al 100 % a uno al 200 %, se ve nítida en ambos.
- **Fuentes:** research/04-biblioteca-compartir.md §5.4 (Accessibility); research/02-editor.md §1 (Accessibility)

---

## 10. Registros y diagnóstico

### APP-046 · Registros e información de diagnóstico
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (Snagit envía informes de fallos opcionales en Mac y tiene una herramienta de soporte aparte; XcreenShot lo resuelve en local)
- **Comportamiento:**
  - XcreenShot escribe registros locales en la carpeta de datos de la app (o en la carpeta portable, APP-038):
    - rotan a diario o al llegar a 10 MB, y se conservan los 7 más recientes, con un máximo de 50 MB en total (decisión de diseño);
    - nivel normal: arranque y cierre, versiones, errores, avisos, atajos que no se pudieron activar, permisos, comandos recibidos por CLI y MCP (APP-032), configuración administrada, actualizaciones;
    - **Registro detallado** (Avanzado; por defecto: no) añade información de depuración. Se desactiva solo a las 24 horas (decisión de diseño).
  - **Nunca se escriben en el registro:** contenido de las capturas, texto reconocido, texto de las anotaciones, contraseñas o tokens, ni el contenido de los parámetros sensibles. En el nivel normal, las rutas de archivo dentro de la carpeta del usuario se escriben abreviadas (por ejemplo, `~/…/captura.png`).
  - **Abrir la carpeta de registros** (menú Ayuda y Avanzado) la abre en el explorador de archivos.
  - **Copiar información de diagnóstico** copia al portapapeles un texto que el usuario puede revisar antes de pegarlo:
    - versión, compilación, canal y arquitectura;
    - sistema operativo y versión, idioma, número de monitores con su resolución y escalado;
    - estado de los permisos;
    - atajos globales en conflicto;
    - configuración administrada activa;
    - funciones en prueba activas;
    - las últimas 200 líneas del registro.
    - No incluye nombres de archivos de la biblioteca ni información del usuario.
  - CLI: `xcreenshot app diagnostics [--json]` devuelve la misma información.
  - **Sin envío automático:** XcreenShot nunca sube registros ni informes de fallos. Tras un cierre inesperado (APP-047), el siguiente arranque ofrece **Copiar información del fallo** para que el usuario la adjunte, si quiere, en Enviar comentarios (APP-015).
- **Criterios de aceptación:**
  - [ ] Copiar información de diagnóstico deja en el portapapeles un texto con versión, sistema, permisos y monitores, y sin ninguna ruta de la biblioteca.
  - [ ] Tras una semana de uso normal, la carpeta de registros no supera 50 MB.
  - [ ] Tras usar Extraer texto sobre una captura con texto conocido, ese texto no aparece en ningún archivo de registro (ni con el registro detallado).
  - [ ] El Registro detallado activado hace 24 horas aparece desactivado.
- **Fuentes:** research/04-biblioteca-compartir.md §5.3 (General: crash reports), §5.4 (Telemetry and notifications), §1.3 (Support Tool repoint)

### APP-047 · Recuperación tras un cierre inesperado
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** recuperación de grabaciones no guardadas tras un fallo; recuperación de una captura de pasos en curso; aviso si falla el autoguardado
- **Comportamiento:**
  - Si XcreenShot se cerró de forma inesperada (fallo, corte de luz, apagado forzado), el siguiente arranque muestra **Recuperar trabajo**, con lo que se puede recuperar:
    - documentos con cambios sin guardar (`EDT-` define cada cuánto se guarda una copia de recuperación);
    - grabaciones interrumpidas (`VID-`);
    - grabaciones de pasos en curso (SMT- (Grabador de pasos)).
  - Cada elemento tiene **Recuperar** (lo abre o lo pasa a la biblioteca) y **Descartar**. **Más tarde** cierra la ventana y deja los elementos en Ayuda > Recuperar trabajo.
  - Los elementos que no se recuperan ni descartan se conservan 30 días (decisión de diseño) y después se borran, con aviso previo en la ventana.
  - Si la app falla al arrancar dos veces seguidas, el tercer arranque es en **modo seguro**: sin funciones en prueba y sin los atajos globales registrados. Avisa de ello y ofrece **Copiar información del fallo** (APP-046) y **Arrancar normalmente** (decisión de diseño).
  - Si el autoguardado en la biblioteca falla (disco lleno, carpeta inaccesible), aparece un aviso inmediato (categoría Problemas) y el contenido queda en la carpeta de recuperación. El detalle lo define `LIB-`.
- **Criterios de aceptación:**
  - [ ] Matar el proceso con un documento editado y sin guardar y volver a abrir la app muestra Recuperar trabajo; Recuperar abre el documento con las ediciones hasta el último guardado de recuperación.
  - [ ] Matar el proceso durante una grabación de 2 minutos y volver a abrir permite recuperar un video que se reproduce hasta el momento del fallo (con la pérdida máxima que fije `VID-`).
  - [ ] Tras dos fallos seguidos al arrancar, el tercer arranque entra en modo seguro y lo dice.
- **Fuentes:** research/03-video.md §D (Crash or interruption recovery); research/01-captura.md §8.1; research/05-plataformas-versiones.md §1B (2024.2.5, 2026.1.0)

---

## 11. Requisitos no funcionales

Los valores se miden en el **equipo de referencia**, salvo que se indique otra cosa:
- **Windows:** Windows 11, procesador de 4 núcleos de 2019 o posterior, 16 GB de RAM, SSD, un monitor 1920×1080 al 100 % y otro 3840×2160 al 150 %.
- **macOS:** Mac con Apple Silicon M1 o posterior, 16 GB de RAM, pantalla Retina integrada más un monitor 4K.

Cada requisito tiene una prueba automática o un procedimiento manual repetible; los resultados se guardan con cada versión.

### APP-048 · Del atajo a la cruceta
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** rapidez en la aparición de la cruceta tras iniciar una captura
- **Comportamiento:**
  - Desde que se pulsa el atajo de una captura interactiva (Capturar, Captura universal, región) hasta que la cruceta está visible y responde al ratón: **< 150 ms** (percentil 95), con la app en reposo.
  - Desde el menú del ícono: < 300 ms (CAP- (Menú de captura del ícono)).
  - Primera captura tras arrancar la app con el inicio de sesión: < 500 ms.
- **Criterios de aceptación:**
  - [ ] En 100 repeticiones en el equipo de referencia, el percentil 95 del tiempo del atajo a la cruceta es < 150 ms en ambas plataformas.
  - [ ] Con el editor abierto con un documento de 10.000 × 30.000 px, el tiempo sigue siendo < 200 ms (percentil 95).
- **Fuentes:** research/01-captura.md §3.1; research/05-plataformas-versiones.md §1A (2025.2.0)

### APP-049 · De la captura al editor
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** apertura rápida del editor tras capturar (editor en segundo plano)
- **Comportamiento:**
  - Desde que se confirma la selección hasta que la captura se ve en el editor y se puede anotar:
    - **< 400 ms** para una región de hasta 3840×2160 con el editor cargado en segundo plano (APP-004);
    - < 1,5 s con el editor no cargado.
  - Desde que se confirma la selección hasta que la imagen está en el portapapeles (Abrir en = Ninguno): < 250 ms.
  - El autoguardado en la biblioteca se hace en segundo plano y no retrasa la apertura en el editor.
- **Criterios de aceptación:**
  - [ ] En 50 capturas de región 4K, el percentil 95 de la selección al editor es < 400 ms con el editor cargado.
  - [ ] Una captura con Copiar al portapapeles y Abrir en = Ninguno se puede pegar en otra app < 250 ms después de confirmarla.
- **Fuentes:** research/01-captura.md §4.5, §11.1; research/02-editor.md §11

### APP-050 · Editor con imágenes muy grandes
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** capturas con desplazamiento grandes (Snagit avisa de que son lentas)
- **Comportamiento:**
  - Con un documento de **10.000 × 30.000 px** (captura con desplazamiento) y 200 objetos:
    - abrir: < 3 s;
    - desplazar y hacer zoom: ≥ 50 fotogramas por segundo, sin saltos de más de 100 ms;
    - crear y mover objetos: respuesta visual < 50 ms;
    - deshacer y rehacer: < 200 ms;
    - guardar `.xshot`: < 5 s, sin bloquear la interfaz;
    - exportar a PNG: < 10 s, con progreso visible.
  - Memoria con ese documento abierto: < 2,5 GB por encima del consumo en reposo.
  - Límite duro de tamaño de imagen: 32.000 px por lado o 500 megapíxeles. Por encima, la captura o importación se rechaza con un mensaje claro, que en las capturas con desplazamiento ofrece dividir el resultado (decisión de diseño; el detalle está en CAP- (Unión y procesamiento del desplazamiento)).
- **Criterios de aceptación:**
  - [ ] Con el documento de prueba de 10.000 × 30.000 px y 200 objetos, una medición automática del desplazamiento da ≥ 50 fps de media y ningún fotograma de más de 100 ms.
  - [ ] Guardar ese documento no congela la interfaz: durante el guardado se puede seguir desplazando.
  - [ ] Importar una imagen de 40.000 × 10.000 px muestra el mensaje del límite y no cierra la app.
- **Fuentes:** research/01-captura.md §6.1, §7; research/02-editor.md §1 (Zoom)

### APP-051 · Consumo en reposo y arranque
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** proceso en segundo plano ligero; arranque rápido de la captura
- **Comportamiento:**
  - En reposo (solo el ícono y los atajos, ninguna ventana visible, sin capturas programadas):
    - memoria: < 150 MB con Mantener el editor cargado desactivado; < 400 MB con él activado (APP-004);
    - CPU: < 0,5 % de media, sin despertares periódicos de más de 1 por minuto (para no gastar batería);
    - sin actividad de disco ni de red, salvo la búsqueda programada de actualizaciones (APP-036).
  - Arranque desde el inicio de sesión hasta que el atajo funciona: < 3 s (APP-003).
  - Tras cerrar el editor, la memoria vuelve a menos de 1,5 veces el valor de reposo en < 30 s.
- **Criterios de aceptación:**
  - [ ] Tras 1 hora en reposo, la memoria del conjunto de procesos de XcreenShot está por debajo del límite que corresponde al ajuste.
  - [ ] Un monitor de energía del sistema clasifica a XcreenShot en reposo como de impacto bajo.
  - [ ] Tras abrir y cerrar 20 capturas en el editor, la memoria en reposo no crece más de un 10 % respecto de la inicial.
- **Fuentes:** research/01-captura.md §1.1; research/05-plataformas-versiones.md §1A (2025.2.0)

### APP-052 · Escala de la biblioteca
- **Hito:** M2 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** rendimiento con bibliotecas grandes (más de 5.000 elementos) y en la nube
- **Comportamiento:**
  - Con una biblioteca de **50.000 elementos** (mezcla de imágenes, videos y guías):
    - abrir la vista Biblioteca: < 1 s hasta ver las primeras miniaturas;
    - desplazarse por la cuadrícula: ≥ 50 fps; las miniaturas pendientes aparecen como marcadores y se completan en < 2 s;
    - buscar por texto, etiqueta, aplicación o sitio web: resultados en < 300 ms;
    - aplicar filtros (fecha, tipo, etiqueta): < 300 ms;
    - reconstruir el índice desde los archivos (`library.db`, SPEC §7): < 10 min, en segundo plano y con progreso.
  - Los mismos tiempos valen si la biblioteca está en una carpeta sincronizada con un servicio en la nube, con los archivos descargados en el equipo.
- **Criterios de aceptación:**
  - [ ] Con una biblioteca generada de 50.000 elementos, una búsqueda por etiqueta devuelve resultados en < 300 ms (percentil 95 de 20 búsquedas).
  - [ ] Desplazarse de principio a fin por la cuadrícula de 50.000 elementos no congela la interfaz más de 100 ms en ningún momento.
  - [ ] Borrar `library.db` y abrir la app reconstruye el índice sin perder etiquetas que estén guardadas en los archivos.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Large-library handling), §1.2 (Cloud Library)

### APP-053 · Cruceta con varios monitores 4K
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** límites documentados de la cruceta (2–3 monitores, o dos 4K a 60 Hz); cruceta simplificada
- **Comportamiento:**
  - La cruceta, la lupa y la selección siguen al ratón a la frecuencia del monitor (hasta 144 Hz), con un retraso < 16 ms, en configuraciones de hasta:
    - **4 monitores**, de los cuales hasta **3 son 4K**, con escalados mixtos.
  - Con 5 o más monitores, o más de 3 monitores 4K, sigue funcionando sin bloquearse, aunque puede bajar de 60 fps. En ese caso, XcreenShot propone una vez activar la cruceta simplificada (CAP- (Cruceta simplificada)).
  - El tiempo de APP-048 se cumple también con 3 monitores 4K.
- **Criterios de aceptación:**
  - [ ] Con 3 monitores 4K, mover el ratón en círculos durante la selección mantiene ≥ 60 fps en la cruceta (medido con el contador de fotogramas de diagnóstico).
  - [ ] Con 3 monitores 4K, el percentil 95 del tiempo del atajo a la cruceta es < 150 ms.
  - [ ] Con 5 monitores, la selección funciona y aparece una vez la sugerencia de cruceta simplificada.
- **Mejora sobre Snagit:** más monitores y monitores 4K sin degradación, frente a los 2–3 monitores que documenta Snagit.
- **Fuentes:** research/01-captura.md §3.1, §3.2, §3.5

### APP-054 · Local y privado
- **Hito:** M0 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (Snagit recoge datos de uso con distintos niveles de consentimiento y envía informes de fallos; SPEC §2.3 lo excluye)
- **Comportamiento:**
  - **Sin telemetría:** XcreenShot no recoge ni envía datos de uso, métricas, identificadores de instalación, informes de fallos ni datos de las funciones en prueba. No hay ajuste para activarlo, porque no existe.
  - **Sin cuenta:** todas las funciones locales se usan sin iniciar sesión en nada. Solo los destinos de compartir en línea piden la sesión de su propio servicio (`SHR-`).
  - **Funciona sin conexión:** capturar, grabar, editar, funciones inteligentes, biblioteca, ayuda, notas de la versión y destinos locales funcionan igual sin conexión de red. Los destinos en línea sin conexión fallan con un mensaje claro y ofrecen reintentar más tarde.
  - **Conexiones de red permitidas**, y solo estas:
    1. buscar y descargar actualizaciones (APP-036), si está activado;
    2. enviar a un destino en línea o crear un Enlace para compartir, cuando el usuario lo pide (`SHR-`);
    3. capturar una página web por URL, cuando el usuario lo pide (CAP- (Página web por URL));
    4. abrir enlaces en el navegador cuando el usuario hace clic (ayuda externa, Enviar comentarios).
  - Ninguna otra parte de la app abre conexiones: ni fuentes ni íconos remotos, ni noticias, ni comprobación de licencia, ni descarga de modelos en segundo plano. Los modelos de las funciones inteligentes van incluidos en la app o los proporciona el sistema (`SMT-`).
  - Acerca de (APP-015) lista estas conexiones permitidas.
- **Criterios de aceptación:**
  - [ ] Con la búsqueda de actualizaciones desactivada, un monitor de red no registra ninguna conexión saliente de XcreenShot durante una sesión de 1 hora en la que se captura, graba, edita, se usa Extraer texto y Redacción automática, y se busca en la biblioteca.
  - [ ] Con la red desconectada desde el arranque, todas las pruebas de humo de captura, edición, video, biblioteca y funciones inteligentes pasan.
  - [ ] Un análisis del código y de las dependencias no encuentra ningún componente de analítica ni de envío de fallos.
- **Fuentes:** SPEC §2.3, §3.2; research/04-biblioteca-compartir.md §5.2 (General: Data & Privacy), §5.3 (General), §5.4 (Telemetry and notifications), §7; research/05-plataformas-versiones.md §2

### APP-055 · Portabilidad de datos y migración entre equipos
- **Hito:** M2 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** mover la biblioteca a un equipo nuevo copiando su carpeta y apuntando a ella; fusionar bibliotecas; exportar la biblioteca a formatos estándar
- **Comportamiento:**
  - **Datos abiertos:** todo lo que crea el usuario está en formatos documentados (SPEC §7): `.xshot` (ZIP con JSON y PNG), `.mp4`, `.gif`, `.xtheme`, `.xpreset`, `.xsettings`. El índice `library.db` se puede reconstruir a partir de los archivos (APP-052). Ningún dato del usuario queda solo dentro de una base de datos propietaria ni en un servicio.
  - **Exportar a formatos estándar:** la exportación por lotes de la biblioteca a PNG/JPG (`LIB-`, `EDT-`) permite salir de XcreenShot sin perder las imágenes.
  - **Mover XcreenShot a otro equipo** (Ayuda > Mover XcreenShot a otro equipo…) es un asistente que explica y facilita dos pasos:
    1. **Llevar la biblioteca:** copiar su carpeta (botón **Mostrar la carpeta**) o, si ya está en una carpeta sincronizada, no hacer nada.
    2. **Llevar los ajustes:** **Exportar ajustes** (APP-011) a un `.xsettings`.
    En el equipo nuevo:
    - la bienvenida (APP-005) ofrece **Usar una biblioteca existente** y **Importar ajustes**;
    - si la carpeta elegida ya tiene una biblioteca de XcreenShot, se usa tal cual, con sus etiquetas.
  - La biblioteca y los `.xsettings` funcionan entre Windows y macOS (SPEC §7; APP-011).
  - Apuntar a otra biblioteca cuando ya hay una con contenido ofrece **Cambiar** o **Fusionar**. El detalle lo define `LIB-`.
- **Criterios de aceptación:**
  - [ ] Copiar la carpeta de la biblioteca de un Windows a un Mac, elegirla en la bienvenida e importar el `.xsettings` deja en el Mac las mismas capturas, etiquetas, presets, temas y atajos (convertidos).
  - [ ] Un `.xshot` de la biblioteca se puede abrir con una herramienta ZIP estándar y contiene `document.json` y `base.png` legibles.
  - [ ] Elegir en el equipo nuevo una carpeta con una biblioteca existente no duplica ni renombra ningún archivo.
- **Mejora sobre Snagit:** un asistente único para biblioteca y ajustes, y formatos abiertos y documentados.
- **Fuentes:** research/04-biblioteca-compartir.md §1.3 (Move Library to a new computer, Merge libraries, Export Library on exit), §1.2 (Cloud Library), §4 (Preset export/import, Themes)

---

## Relacionado

| Prefijo / archivo | Relación con este archivo |
|---|---|
| `SPEC.md` | Principios §2 (todo es un comando; local primero), alcance §3.2 (API COM → CLI + MCP; despliegue empresarial → configuración administrada), hitos §4, formatos §7 (se propone añadir `.xsettings` y `managed.json`), nombres §8, decisiones D-06, D-07, D-08 |
| `CAP-` (`01-captura.md`) | Panel de captura y su menú Más; menú de captura del ícono; atajo global Capturar; Repetir última captura; comandos de captura (parámetros y resultados); permisos que faltan al capturar; Ocultar XcreenShot al capturar; cruceta simplificada; varios monitores y DPI; presets y su atajo; acciones de captura y atajos por defecto; perfil de compatibilidad; ajustes de la sección Captura |
| `EDT-` (`02-editor.md`) | Ventana principal (editor), barra de herramientas, atajos del editor, ajustes de la sección Editor, comandos `editor.*` y `document.*`, copia de recuperación de documentos, accesibilidad del lienzo |
| `VID-` (`03-video.md`) | Comandos `record.*`, indicador de grabación en el ícono, entradas de estado del menú rápido, ajustes de la sección Video, recuperación de grabaciones, permisos de cámara, micrófono y audio del sistema, directiva `disableVideoCapture` |
| `LIB-`, `TRY-`, `SHR-` (`04-biblioteca-compartir.md`) | Ubicación de la biblioteca y su bloqueo; índice `library.db`; fusionar bibliotecas; exportación por lotes; ajustes de las secciones Biblioteca y Compartir; destinos desactivados y proveedor del Enlace para compartir por configuración administrada; notificaciones de compartir; comandos `library.*`, `tray.*` y `share.*` |
| `SMT-` (`06-inteligentes.md`) | Grabador de pasos (permiso de Accesibilidad, ventanas elevadas, recuperación), Extraer texto, Redacción automática; comandos `smart.*`; candidatas a Funciones en prueba |
| `PARITY.md` | Usa los IDs APP- de este archivo; anota si la compilación nativa ARM64 (APP-041) se retrasa |

---

## Excluido

| Elemento de la investigación | Motivo |
|---|---|
| Suscripción, licencias, claves de producto, activación sin conexión, estados de prueba o caducidad (research/04 §9) | Fuera del alcance (SPEC §3.2): XcreenShot no tiene licencias ni activación |
| Cuenta TechSmith, inicio de sesión y menú de cuenta con foto, licencia y avisos (research/04 §9; research/02 §1) | Fuera del alcance: sin cuenta obligatoria (SPEC §2.3). Buscar actualizaciones está en el menú Ayuda (APP-015) |
| Mostrar u ocultar la clave de registro del producto (research/04 §9) | No aplica: no hay clave |
| Activación a través de otra suscripción de TechSmith (research/04 §9) | Fuera del alcance (licencias) |
| Programa Experiments con recogida de datos de uso anónimos y botón Send Feedback por experimento (research/04 §7; research/05 §5) | Se conserva el mecanismo de funciones opcionales (APP-034) sin telemetría (SPEC §3.2). Los comentarios van por Ayuda (APP-015) |
| Lista concreta de experimentos de Snagit (research/04 §7; research/05 §5) | Cada función la especifica su archivo dueño (`CAP-`, `EDT-`, `VID-`, `SMT-`); si sale como prueba o definitiva lo decide ese archivo |
| Datos y privacidad (uso personalizado, anónimo o ninguno), aviso de recogida de datos, informes de fallos enviados (research/04 §5.2, §5.3, §5.4) | XcreenShot no recoge datos (APP-054); los fallos se gestionan en local (APP-046, APP-047) |
| Aceptación de la política de privacidad y del EULA en el instalador (research/05 §1A) | No hay recogida de datos. La licencia del repositorio se decide en D-01 |
| TechSmith Assets y sus enlaces, y la directiva para quitarlos (research/04 §4, §9) | Fuera del alcance (SPEC §3.2); se sustituye por paquetes locales (`EDT-`) |
| TechSmith Fuse, Mobile Sharing y puerto local de conexión móvil (research/04 §8) | Fuera en v1 (SPEC §3.2). Coherente con APP-032: ningún puerto de red abierto |
| Learning Center, tours guiados, encuesta de bienvenida, avisos de noticias y nivel de ayuda (research/04 §5.2, §5.4, §9; research/02 §1) | Fuera del alcance (SPEC §3.2). Se sustituye por ayuda local propia (APP-015) y pistas propias (APP-014) |
| Tooltips con video o animación (research/02 §1; research/04 §5.2) | Sustituido por tooltips de texto (APP-014); evita contenido de marketing y respeta reducir movimiento |
| API COM (`Snagit.ImageCapture`, interfaces de captura de imagen y video, registro del servidor COM) (research/01 §1.9; research/04 §8) | Reemplazada por CLI + MCP multiplataforma (APP-027 a APP-033). Las capacidades se cubren en APP-030 y CAP- (Comandos de captura) |
| Salidas de la API COM a impresora, correo y FTP (research/01 §1.9) | Se cubren como destinos de compartir (`SHR-`) y `share.send` (APP-030) |
| AppleScript / Atajos de Apple (research/04 §8) | Snagit no tiene diccionario público. El CLI (APP-028) se puede llamar desde Atajos con "Ejecutar script de shell". Integración nativa con Atajos: candidata futura |
| Herramienta de despliegue empresarial, transformaciones MST, opciones del MSI, claves de preferencias de Mac, herramienta de soporte de Mac (research/04 §9; research/05 §3, §1A) | Reemplazado por `managed.json` (APP-035) y las opciones del instalador (APP-037). La reubicación de la biblioteca de la herramienta de soporte la cubre `LIB-` |
| Excluir componentes de base de datos en la instalación y herramienta de conversión de bibliotecas antiguas para TI (research/05 §1A; research/04 §1.3) | No aplica: sin formatos ni bibliotecas de Snagit (SPEC §2.5) |
| Preferencias de biblioteca "aviso para convertir capturas anteriores" e "Importar copia de seguridad" (research/04 §5.2) | Formatos propietarios de Snagit (SPEC §3.2); no aparecen en la sección Biblioteca (APP-008) |
| Lista de proxies del actualizador por directiva de instalación (research/05 §1A) | XcreenShot usa el proxy del sistema (APP-036). Si hiciera falta un proxy propio, se añadiría como directiva de `managed.json` |
| Edición de la Mac App Store (research/05 §3) | No se distribuye por la App Store en v1 (APP-039) |
| Impresora virtual y su configuración (research/01 §11.1; research/04 §8) | Pendiente D-03; propuesta: importar PDF (`CAP-`) |
| Extensión del Explorador de Windows (research/04 §8; research/05 §3) | No se incluye en v1 (decisión de diseño). La exportación por lotes se hace desde la biblioteca (`LIB-`, `EDT-`) o con el CLI (APP-028) |
| Complementos de Office heredados y extensión de Chrome (research/04 §8) | Retirados por TechSmith (SPEC §3.2) |
| Ida y vuelta con Camtasia, botón Edit in Camtasia y su directiva (research/04 §8) | Fuera del alcance (SPEC §3.2) |
| Idiomas francés, alemán, japonés y portugués (research/04 §5.2; research/05 §4) | D-07 fija inglés y español. El sistema de traducciones permite añadir más idiomas después |
| Directiva para desactivar la cámara activada por defecto (research/03 §R) | Se puede fijar con `defaults`/`locked` sobre el ajuste de cámara de `VID-` (APP-035); no necesita directiva propia |
| Directiva para desactivar la captura de imagen (research/04 §9) | XcreenShot es ante todo una herramienta de captura de imagen; sin una necesidad concreta no se añade (decisión de diseño). Se puede añadir como `policy` si hace falta |
| Directivas para desactivar el Share Manager o el inicio de sesión (research/04 §9) | Los destinos se desactivan con `policy.disabledDestinations` (APP-035); no hay inicio de sesión propio |
| Ajustes de registro de Windows para cambiar el método de captura o el audio (research/03 §D) | No se replican; si hace falta un método alternativo, `VID-` lo ofrece como ajuste de la sección Avanzado (APP-009) |
| Nombres de producto ("Snagit" sin año, "Camtasia Snagit") (research/04 §9) | No aplica (D-09) |

---

## Cobertura

| Sección de la investigación | Cubierta por |
|---|---|
| research/04-biblioteca-compartir.md §4 — Presets (exportar/importar, atajos, reordenar, Open In, incorporados) | `CAP-` (presets); atajos: APP-022; inclusión en `.xsettings`: APP-011 |
| research/04 §4 — Temas, estilos guardados, favoritos, efectos guardados, barra de herramientas | `EDT-`; inclusión en `.xsettings`: APP-011; restablecer sin borrarlos: APP-010 |
| research/04 §4 — Perfiles heredados | Excluido (retirados; SPEC §3.2) |
| research/04 §4 — TechSmith Assets | Excluido |
| research/04 §5.1 — Capture Preferences de Windows (General, Capture, Hotkeys) | APP-003, APP-007, APP-008, APP-012, APP-018, APP-019, APP-020, APP-036; ajustes de captura y video: `CAP-`, `VID-` |
| research/04 §5.2 — Editor Preferences de Windows (General, Editor, Share, Library, Advanced, Experiments) | APP-007, APP-008, APP-009, APP-010, APP-012, APP-013, APP-014, APP-034; ajustes concretos: `EDT-`, `SHR-`, `LIB-`, `TRY-`; Data & Privacy: Excluido / APP-054 |
| research/04 §5.3 — Settings de Mac (General, Capture, Share, Keyboard, Advanced/Editor, Experiments) | APP-002, APP-003, APP-006 a APP-010, APP-012, APP-014, APP-016, APP-018, APP-024, APP-034, APP-036; ajustes concretos: `CAP-`, `VID-`, `EDT-`, `SHR-`, `LIB-` |
| research/04 §5.4 — Ajustes transversales (idioma, actualizaciones, telemetría y notificaciones, restablecer, atajos de la app, accesibilidad, ayuda) | APP-013, APP-036, APP-016, APP-054, APP-010, APP-017, APP-021, APP-043, APP-044, APP-045, APP-015 |
| research/04 §7 — Programa Experiments | APP-034 (mecanismo); experimentos concretos: archivos dueños; telemetría: Excluido |
| research/04 §8 — Fuse, TechSmith Capture | Excluido |
| research/04 §8 — AirDrop, Continuity Camera, capturas del sistema | `CAP-` |
| research/04 §8 — Impresora virtual | Excluido (D-03; `CAP-`) |
| research/04 §8 — Extensión del Explorador, complementos de Office, extensión de navegador | Excluido |
| research/04 §8 — API COM | APP-027, APP-028, APP-029, APP-030, APP-031, APP-032, APP-033; CAP- (Comandos de captura) |
| research/04 §8 — AppleScript / Atajos | APP-028 (vía CLI); integración nativa: Excluido (futuro) |
| research/04 §8 — App de Slack | `SHR-` |
| research/04 §8 — Ida y vuelta con Camtasia; `.snagx` en iOS | Excluido |
| research/04 §9 — Suscripción, inicio de sesión, menú de cuenta, estados de caducidad, activación con Camtasia, nombres de producto | Excluido |
| research/04 §9 — Requisitos de plataforma | APP-040 |
| research/04 §9 — Controles de despliegue para administradores | APP-035 (directivas equivalentes); el resto: Excluido |
| research/01-captura.md §1.1 — App de captura en segundo plano, ícono, menú rápido, ícono rojo y monocromo, iniciar al iniciar sesión | APP-001, APP-002, APP-003, APP-004; entradas de captura del menú: CAP- (Menú de captura del ícono) |
| research/01 §1.9 — API COM (entradas, salidas, selección, desplazamiento, retardo, nombres de archivo, formatos, estados y eventos) | APP-027 a APP-031; parámetros de captura: CAP- (Comandos de captura); formatos: `EDT-`; salidas: `SHR-` |
| research/01 §10 — Atajos globales y de captura (valores por defecto, personalización, conflictos, atajos de destinos) | APP-017 a APP-023; valores por defecto: CAP- (Acciones de captura y atajos por defecto), CAP- (Perfil de compatibilidad), `VID-`, `EDT-` |
| research/01 §11.1 — Preferencias de captura de Windows | APP-003, APP-007, APP-008, APP-012, APP-036; ajustes concretos: `CAP-`, `VID-`, `LIB-`, `SHR-` |
| research/01 §11.2 — Settings de Mac relacionados con la captura | APP-002, APP-003, APP-007, APP-008, APP-010, APP-024, APP-036; ajustes concretos: `CAP-`, `VID-`, `SHR-`, `LIB-` |
| research/01 §11.3 — Sonidos y notificaciones de captura | APP-016; sonido de captura: CAP- (Confirmación de captura) |
| research/01 §14 — Permisos de macOS, Windows elevado, VM y escritorio remoto, controles de administración | APP-024, APP-026, APP-035, APP-040; CAP- (Permisos que faltan al capturar) |
| research/03-video.md §Q — Permisos de macOS (pantalla, cámara, micrófono, reanudar) | APP-024 |
| research/03 §Q — Aviso periódico de macOS 15 | APP-025 |
| research/03 §Q — Comportamiento según la versión de macOS | APP-040; detalle por función: `VID-` |
| research/03 §Q — Requisitos de Windows para video; privacidad de cámara y micrófono; H.265 en Windows 10; programas de seguridad | APP-040, APP-026; H.265: `VID-` |
| research/03 §Q — Contenido protegido | APP-026; CAP- (Permisos que faltan al capturar) |
| research/03 §Q — Entornos virtuales y remotos | APP-026 |
| research/03 §R — Desactivar la captura de video | APP-035 (`policy.disableVideoCapture`) |
| research/03 §R — Cámara activada por defecto | APP-035 (`defaults`/`locked` sobre el ajuste de `VID-`) |
| research/03 §R — Destinos desactivados y Enlace para compartir | APP-035; `SHR-` |
| research/03 §R — Funciones en prueba y controles de IA | APP-034, APP-035; funciones inteligentes: `SMT-` |
| research/03 §R — Canal de actualizaciones | APP-036, APP-035 |
| research/02-editor.md §1 — Pestañas Editor y Biblioteca, Assets, botón Capturar, menú Crear, barra de herramientas, zona de compartir, área de trabajo, panel lateral, zoom, cuadrícula de píxeles, pantalla completa | `EDT-`, `LIB-`, `SHR-`; Assets: Excluido |
| research/02 §1 — Tema de color | APP-012 |
| research/02 §1 — Tooltips mejorados, pistas de introducción | APP-014 |
| research/02 §1 — Menú de ayuda y comentarios, Learning Center y ayuda sin conexión | APP-015; Learning Center: Excluido |
| research/02 §1 — Menú de cuenta | Excluido |
| research/02 §1 — `Esc` cierra el editor; vista previa del estilo junto al puntero; barra de edición rápida; indicador de cambios sin guardar | `EDT-` (aparecen en APP-008) |
| research/02 §1 — Accesibilidad | APP-043, APP-044, APP-045 |
| research/02 §11 — Preferencias del editor | APP-008 (sección Editor y Biblioteca), APP-004 (editor en segundo plano), APP-034 (funciones en prueba); detalle: `EDT-`, `TRY-` |
| research/05-plataformas-versiones.md §3 — Diseño de las preferencias | APP-006 |
| research/05 §3 — Edición de la Mac App Store | Excluido (APP-039) |
| research/05 §3 — Despliegue empresarial | APP-035, APP-037; MSI/MST: Excluido |
| research/05 §3 — Atajos por defecto | APP-021; valores: `CAP-`, `VID-`, `EDT-` |
| research/05 §3 — Matrices de captura, video, editor, grabador de pasos y destinos | `CAP-`, `VID-`, `EDT-`, `SMT-`, `SHR-`; impresora en ARM: APP-041 / Excluido |
| research/05 §4 — Requisitos de Windows (sistema, CPU, ARM64, RAM, disco, componentes, idiomas) | APP-040, APP-041, APP-037, APP-013 |
| research/05 §4 — Requisitos de macOS (sistema, Apple Silicon e Intel, configuraciones no compatibles, idiomas) | APP-040, APP-039, APP-013 |
| research/05 §5 — Pestaña Experiments | APP-034; experimentos concretos: archivos dueños; telemetría: Excluido |
