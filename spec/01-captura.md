# 01 · Captura de imagen

**Archivo:** `01-captura.md` · **Prefijo de IDs:** `CAP-` · **Versión:** 0.1 (borrador) · **Fecha:** 2026-10-12

Este archivo especifica cómo se inicia una captura de imagen en XcreenShot, cómo elige el usuario qué capturar (una región, una ventana, la pantalla, un menú, un área con desplazamiento, o fuentes que no son la pantalla, como una página web, la cámara, el portapapeles, un archivo o un PDF), qué opciones se aplican en el momento de capturar, a dónde va el resultado y cómo se guardan esos ajustes en presets con atajos propios. La grabación de video (`VID-`), el editor (`EDT-`), la biblioteca y la bandeja reciente (`LIB-`, `TRY-`), los destinos de compartir (`SHR-`), la app en sí (`APP-`) y las funciones inteligentes (`SMT-`) se especifican en sus propios archivos; aquí solo se citan por su prefijo. Salvo que una función diga otra cosa, todo lo descrito vale igual para Windows y macOS (SPEC §2.1).

**Convenciones de este archivo**
- **Teclas.** `Ctrl`/`Cmd` significa `Ctrl` en Windows y `Command` (⌘) en macOS; `Alt`/`Option` igual. `Enter` es Intro en Windows y Retorno en macOS. En macOS, `⌃⌥⇧` abrevia `Control+Option+Shift`. Las teclas de una sola letra que funcionan durante la selección son las mismas en inglés y en español.
- **(decisión de diseño)** marca un comportamiento que la documentación pública de Snagit no describe, o que XcreenShot define a propósito de otra forma. Lo fija este spec.
- **"Por defecto"** significa el valor en una instalación nueva.
- **Fase de apuntado / fase de ajuste.** Durante una selección en pantalla, la *fase de apuntado* va desde que aparece la cruceta hasta que hay una selección; la *fase de ajuste* empieza cuando la selección está hecha y aparece la barra de captura.

---

## Índice

1. **Iniciar una captura** — CAP-001 Panel de captura · CAP-002 Pestaña Imagen y lista Selección · CAP-003 Restablecer ajustes de captura · CAP-004 Lanzador de borde · CAP-005 Menú de captura del ícono · CAP-006 Atajo global Capturar · CAP-007 Repetir última captura · CAP-008 Comandos de captura
2. **Captura universal** — CAP-009 Flujo de captura universal · CAP-010 Opciones de la pestaña Universal · CAP-011 Ajustar la selección · CAP-012 Rehacer la selección y cambiar de tipo
3. **Selección en pantalla** — CAP-013 Pantalla congelada y cruceta · CAP-014 Cruceta simplificada · CAP-015 Lupa · CAP-016 Detección automática · CAP-017 Teclado durante la selección · CAP-018 Bloqueo de proporción · CAP-019 Guías de tamaño estándar · CAP-020 Ayuda en pantalla · CAP-021 Varios monitores y DPI mixto · CAP-022 Pantallas HDR · CAP-023 Ocultar XcreenShot al capturar · CAP-024 Permisos que faltan
4. **Modos de captura de pantalla** — CAP-025 Región · CAP-026 Región fija y posición · CAP-027 Ventana · CAP-028 Pantalla completa · CAP-029 Ventanas como objetos · CAP-030 Selector de ventanas · CAP-031 Menú · CAP-032 Mano alzada · CAP-033 Múltiples áreas · CAP-034 Desplazamiento manual · CAP-035 Desplazamiento automático · CAP-036 Unión y procesamiento del desplazamiento
5. **Otras fuentes e importación** — CAP-037 Página web por URL · CAP-038 Cámara web como imagen · CAP-039 Portapapeles como fuente · CAP-040 Archivo como fuente · CAP-041 Importar PDF multipágina · CAP-042 Capturas del sistema · CAP-043 AirDrop y carpeta de recepción · CAP-044 iPhone o iPad
6. **Opciones al capturar y salida** — CAP-045 Abrir en · CAP-046 Copiar al portapapeles · CAP-047 Capturar cursor · CAP-048 Retardo · CAP-049 Intervalo · CAP-050 Programada · CAP-051 Efectos al capturar · CAP-052 Compartir al capturar · CAP-053 Nombre, formato y carpeta del destino Archivo · CAP-054 Ruta de la captura · CAP-055 Confirmación de captura · CAP-056 Metadatos de captura
7. **Anotación rápida** — CAP-057 Anotación rápida de imagen
8. **Presets** — CAP-058 Lista de presets · CAP-059 Crear un preset · CAP-060 Editar, renombrar, duplicar y eliminar · CAP-061 Reordenar · CAP-062 Atajo por preset · CAP-063 Exportar e importar `.xpreset` · CAP-064 Presets incorporados
9. **Atajos de captura** — CAP-065 Acciones de captura y atajos por defecto · CAP-066 Perfil de compatibilidad
10. [Relacionado](#relacionado) · [Excluido](#excluido) · [Cobertura](#cobertura)

---

## 1. Iniciar una captura

### CAP-001 · Panel de captura
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Capture Window (pestañas All-in-One / Image / Video, botón Capture, campo de atajo, engranaje, flecha de expandir)
- **Comportamiento:**
  - Ventana compacta desde la que se configuran y se lanzan las capturas. Se abre desde el ícono de bandeja o de la barra de menús (`APP-`), desde el botón **Capturar** del editor (`EDT-`), desde el Lanzador de borde (CAP-004) o con su propio atajo (`APP-`).
  - Tiene tres pestañas: **Universal** (CAP-010), **Imagen** (CAP-002) y **Video** (sus ajustes los define `VID-`). Debajo de las pestañas está la sección **Presets** (CAP-058).
  - Un botón grande **Capturar** inicia una captura con los ajustes de la pestaña activa.
  - Junto al botón, el campo **Atajo** muestra el atajo global de la acción "Capturar" (CAP-006). Al hacer clic en el campo y pulsar una combinación, el atajo se reasigna; la detección de conflictos es de `APP-`.
  - Un control muestra u oculta las opciones de la pestaña (vista compacta o ampliada). El panel recuerda el estado.
  - Un menú **Más** (⋯) reúne: Preferencias de captura (`APP-`), Importar presets (CAP-063), Restaurar presets incorporados (CAP-064), Importar desde iPhone o iPad (solo macOS, CAP-044), Mostrar Lanzador de borde (CAP-004), Restablecer ajustes de captura (CAP-003) y Ayuda (`APP-`).
  - Entre sesiones recuerda la pestaña activa, la posición de la ventana y los ajustes de cada pestaña.
  - Se oculta mientras dura la captura (CAP-023) y vuelve a su estado anterior al terminar.
  - Es una ventana normal del sistema, con la barra de título nativa: se puede mover, minimizar y abrir el menú de ventana del sistema.
  - Todos los controles se pueden usar con el teclado (Tab, flechas, Enter, Espacio) y tienen etiqueta accesible para lectores de pantalla (`APP-`).
- **Criterios de aceptación:**
  - [ ] Con la pestaña Imagen activa y Selección = Ventana, pulsar **Capturar** inicia una captura de ventana.
  - [ ] Al cerrar y volver a abrir la app se conservan la pestaña activa y los ajustes de cada pestaña.
  - [ ] Solo con el teclado se puede cambiar de pestaña, cambiar la Selección y lanzar una captura.
  - [ ] Con "Ocultar XcreenShot al capturar" activado, el panel no aparece en ninguna captura.
- **Fuentes:** research/01-captura.md §1.1, §1.2; research/05-plataformas-versiones.md §1A (2025.4.0)

### CAP-002 · Pestaña Imagen y lista Selección
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Image tab, Selection dropdown, Selection Properties
- **Comportamiento:**
  - Controles, en este orden: **Selección**; **Propiedades** (engranaje, visible solo si el modo tiene opciones propias); **Efectos** (CAP-051); **Compartir** (CAP-052); **Abrir en** (CAP-045); **Copiar al portapapeles** (CAP-046); **Capturar cursor** (CAP-047); **Tiempo**, con los valores Ninguno, Retardo, Intervalo y Programada (CAP-048 a CAP-050).
  - Lista **Selección**:
    - Primer nivel: Región (CAP-025), Ventana (CAP-027), Pantalla completa (CAP-028), Selector de ventanas (CAP-030), Desplazamiento (CAP-034 y CAP-035), Menú (CAP-031), Extraer texto (`SMT-`) y Grabador de pasos (`SMT-`).
    - Submenú **Más**: Mano alzada (CAP-032), Múltiples áreas (CAP-033), Página web (CAP-037), Cámara web (CAP-038), Portapapeles (CAP-039), Archivo (CAP-040) y PDF (CAP-041).
    - La región fija no es un modo aparte: es una propiedad de Región (CAP-026).
  - Un modo solo aparece en la lista cuando ya está implementado. No se muestran entradas desactivadas de hitos futuros (decisión de diseño).
  - Las opciones que no se aplican al modo elegido se desactivan, y su tooltip explica el motivo. Las propiedades de un modo nunca aparecen en otro.
  - Qué opciones admite cada modo (decisión de diseño):

    | Modo | Cursor | Retardo | Intervalo | Programada | Efectos | Compartir, Abrir en, Copiar |
    |---|---|---|---|---|---|---|
    | Región y región fija | ✓ | ✓ | ✓ (área elegida al iniciar) | ✓ (área elegida al programar) | ✓ | ✓ |
    | Ventana | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
    | Pantalla completa | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
    | Selector de ventanas | ✓ | ✓ | — | — | ✓ | ✓ |
    | Menú | ✓ | ✓ | ✓ (el menú abierto en ese momento) | ✓ | ✓ | ✓ |
    | Mano alzada, Múltiples áreas | ✓ | ✓ | — | — | ✓ | ✓ |
    | Desplazamiento | — | ✓ | — | — | ✓ | ✓ |
    | Página web | — | — | ✓ | ✓ | ✓ | ✓ |
    | Cámara web | — | ✓ | — | — | ✓ | ✓ |
    | Portapapeles | — | — | ✓ | ✓ | ✓ | ✓ |
    | Archivo | — | — | ✓ (vuelve a leer el archivo) | ✓ | ✓ | ✓ |
    | PDF | — | — | — | — | ✓ | ✓ |
- **Criterios de aceptación:**
  - [ ] Con Selección = Página web, Capturar cursor y Retardo aparecen desactivados y su tooltip explica por qué.
  - [ ] Con una región fija configurada, al pasar de Región a Ventana desaparecen las propiedades de región fija; al volver a Región reaparecen con los mismos valores.
  - [ ] Cada combinación marcada ✓ en la tabla produce una captura que respeta la opción.
- **Fuentes:** research/01-captura.md §4.1, §4.2, §4.8, §16 (punto 3)

### CAP-003 · Restablecer ajustes de captura
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** reset to defaults del Capture Window
- **Comportamiento:**
  - La acción **Restablecer ajustes de captura** (menú Más del panel) devuelve las pestañas Universal, Imagen y Video a sus valores por defecto. Los de Video los define `VID-`.
  - Antes de aplicar pide confirmación.
  - No toca los presets, los atajos ni las preferencias de fuera del panel.
  - Valores por defecto (decisión de diseño):
    - **Universal:** Selección = Región; Compartir = ninguno; Abrir en = Editor; Copiar al portapapeles = sí; Capturar cursor = no; Retardo de 5 s = no.
    - **Imagen:** Selección = Región, sin región fija; Efectos = ninguno; Compartir = ninguno; Abrir en = Editor; Copiar al portapapeles = sí; Capturar cursor = no; Tiempo = Ninguno, con el valor de Retardo en 5 s.
- **Criterios de aceptación:**
  - [ ] Después de cambiar varios ajustes y restablecer, todos los campos coinciden con los valores por defecto, incluidos Selección y el número de segundos de Retardo.
  - [ ] Los presets y sus atajos siguen igual tras restablecer.
  - [ ] Si se cancela la confirmación, no cambia ningún ajuste.
- **Fuentes:** research/01-captura.md §1.2

### CAP-004 · Lanzador de borde
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Capture Widget (antes pestaña OneClick)
- **Comportamiento:**
  - Pestaña delgada pegada al borde superior, izquierdo o derecho de cualquier monitor. Al pasar el puntero por encima se despliega y muestra el botón **Capturar** y el botón **Presets**.
  - Se arrastra a lo largo de los bordes, también a otro monitor. Recuerda su posición; si ese monitor ya no existe, vuelve al borde superior del monitor principal.
  - El botón **Capturar** del lanzador usa siempre estos ajustes fijos, sin importar lo que diga el panel: captura universal; Abrir en = Editor; Capturar cursor = sí; sin retardo, sin efectos y sin destinos de compartir.
  - **Presets** abre la lista de presets (CAP-058); un clic en un nombre lo ejecuta.
  - Al crear un preset nuevo, el lanzador muestra durante unos segundos un globo con su nombre. Los nombres largos se recortan con "…" y el nombre completo aparece en el tooltip.
  - Preferencia **Interfaz de captura** con las opciones Panel de captura / Lanzador de borde / Ambos (por defecto: Panel de captura). También se activa desde el menú Más del panel (CAP-001).
  - Mientras una app ocupa la pantalla completa (presentación, video), el lanzador se oculta en ese monitor (decisión de diseño).
  - Nunca aparece en las capturas (CAP-023).
- **Criterios de aceptación:**
  - [ ] Con el lanzador activado, al pasar el puntero se despliega, y **Capturar** inicia una captura universal que se abre en el editor con el cursor como objeto, aunque el panel tenga otros ajustes.
  - [ ] Si se arrastra el lanzador al borde izquierdo del segundo monitor y se reinicia la app, aparece en ese mismo sitio.
  - [ ] Si se desconecta el monitor donde estaba, el lanzador aparece en el monitor principal.
  - [ ] Un clic en un preset de la lista del lanzador lo ejecuta con sus propios ajustes.
- **Mejora sobre Snagit:** Snagit lo documenta solo en Windows; XcreenShot lo ofrece en ambas plataformas.
- **Fuentes:** research/01-captura.md §1.3, §11.1, §11.2, §16 (punto 6); research/05-plataformas-versiones.md §3 (Captura)

### CAP-005 · Menú de captura del ícono de bandeja o barra de menús
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** menú rápido del ícono de la barra de menús (Mac) y menú del ícono de bandeja (Windows)
- **Comportamiento:**
  - El ícono, su clic principal y el ciclo de vida de la app los define `APP-`. Este ID define las entradas de captura del menú del ícono (clic derecho en Windows; menú del ícono en macOS).
  - Entradas, en este orden: Captura universal; Captura de imagen; Captura de video (`VID-`); Anotación rápida (CAP-057); Repetir última captura (CAP-007); submenú **Presets** con la lista de CAP-058; Abrir panel de captura (CAP-001).
  - Cada entrada muestra su atajo, si lo tiene (CAP-065).
  - Mientras hay una captura por intervalo o programada activa (CAP-049, CAP-050), aparece arriba del todo la entrada **Detener captura: <nombre>**. Hay una entrada por cada captura activa.
  - Durante un desplazamiento manual aparece arriba del todo **Terminar desplazamiento** (CAP-034), por si la vista previa queda tapada.
  - En macOS, soltar sobre el ícono de la barra de menús una URL inicia una captura de página web (CAP-037), y soltar un PDF o una imagen la importa (CAP-041, CAP-040).
  - Las entradas que dependen de funciones de M3 (presets, Detener captura, soltar URL o archivos) aparecen cuando esas funciones existen.
- **Criterios de aceptación:**
  - [ ] Desde el menú del ícono, **Captura universal** muestra la cruceta en menos de 300 ms.
  - [ ] (Desde M3) Con un intervalo activo, el menú muestra "Detener captura" y al elegirlo el intervalo se detiene (ninguna captura nueva después).
  - [ ] (Desde M3) Los presets del submenú aparecen en el mismo orden que en el panel.
  - [ ] (macOS, desde M3) Arrastrar la URL de la barra de direcciones de un navegador sobre el ícono inicia la captura de esa página.
- **Notas de plataforma:** En Windows, los íconos del área de notificación no aceptan que se suelte nada encima. Por eso, en Windows, la URL, el PDF o la imagen se sueltan sobre el Panel de captura o sobre el Lanzador de borde.
- **Fuentes:** research/01-captura.md §1.1, §1.7; research/05-plataformas-versiones.md §1B (2024.3.0)

### CAP-006 · Atajo global Capturar
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Global Capture hotkey (Print Screen / Control+Shift+C)
- **Comportamiento:**
  - La acción **Capturar** inicia una captura con los ajustes de la pestaña activa del panel (Universal, Imagen o Video), aunque el panel esté cerrado.
  - El valor por defecto depende del perfil de atajos (CAP-065, CAP-066). Se cambia en el campo Atajo del panel o en Preferencias > Atajos (`APP-`).
  - Mientras ya hay una selección en pantalla, volver a pulsar el atajo no hace nada (decisión de diseño).
  - Si el atajo no se puede registrar porque lo usa el sistema u otra app, XcreenShot avisa al arrancar y marca el campo Atajo en rojo con el motivo. Ofrece elegir otro o intentar quedarse con él mientras XcreenShot esté abierto; ese mecanismo lo define `APP-`.
- **Criterios de aceptación:**
  - [ ] Con la pestaña Imagen activa y Selección = Pantalla completa, el atajo captura la pantalla sin abrir el panel.
  - [ ] Al cambiar el atajo en el panel, la combinación anterior deja de funcionar al instante y la nueva empieza a funcionar, sin reiniciar.
  - [ ] Si se asigna una combinación que ya usa otra app, aparece el aviso y el campo queda marcado en rojo.
- **Notas de plataforma:** En Windows 11, el sistema puede reservar `PrtScn` para su propia herramienta de recorte. El perfil de compatibilidad (CAP-066) usa esa tecla. Si el sistema la tiene reservada, XcreenShot lo detecta y explica qué ajuste del sistema hay que cambiar.
- **Fuentes:** research/01-captura.md §1.4, §10

### CAP-007 · Repetir última captura
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Repeat Last Capture
- **Comportamiento:**
  - **Repetir última captura** vuelve a lanzar la última captura con el mismo modo y los mismos ajustes (efectos, destinos, abrir en, portapapeles, cursor, tiempo). En los modos interactivos, el usuario vuelve a seleccionar. Con una región fija con posición, captura al instante.
  - **Repetir última área** (decisión de diseño) captura al instante, sin cruceta, el mismo rectángulo de pantalla de la última captura de región, región fija o ventana. Si la última captura fue de ventana y la ventana sigue abierta, captura esa ventana en su posición actual.
  - La última captura se recuerda entre sesiones. Si nunca se hizo ninguna, ambas acciones abren el panel de captura.
  - Si la última captura venía de un preset, se repite ese preset.
- **Criterios de aceptación:**
  - [ ] Tras una captura de menú con retardo de 3 s, **Repetir última captura** inicia otra cuenta atrás de 3 s en modo Menú.
  - [ ] Tras una captura de región, **Repetir última área** produce, sin interacción, una imagen del mismo tamaño en píxeles y de la misma zona de la pantalla.
  - [ ] Sin capturas previas, ambas acciones abren el panel de captura y no dan error.
- **Mejora sobre Snagit:** la acción "Repetir última área" es propia de XcreenShot y sirve para capturas de antes y después.
- **Fuentes:** research/01-captura.md §1.5, §10

### CAP-008 · Comandos de captura (CLI y MCP)
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** COM Server API (entradas, selección de ventana, región, desplazamiento automático, retardo, salidas, estados y eventos)
- **Comportamiento:**
  - Cada modo de captura de este archivo es un comando registrado (SPEC §2.2). La UI, los atajos, el CLI y el servidor MCP llaman a los mismos comandos. Los nombres de los comandos los fija el registro de `APP-`; este ID fija sus parámetros y resultados.
  - Parámetros comunes:
    - **modo**: cualquiera de la lista de CAP-002;
    - **preset**: nombre (si se indica, sus ajustes son la base y los demás parámetros los sobrescriben);
    - **cursor**, **efectos**, **abrir en**, **copiar al portapapeles**, **destinos**;
    - **archivo**: ruta o patrón de nombre de CAP-053;
    - **retardo** en segundos.
  - Selección de ventana:
    - `interactiva`;
    - `activa` (la ventana con el foco);
    - `en punto x,y`;
    - `por título o proceso` (la primera que coincida).
  - Selección de región:
    - `interactiva`;
    - `rectángulo` con monitor, x, y, ancho y alto, en las mismas unidades que CAP-026.
  - Desplazamiento automático: dirección (`vertical`, `horizontal` o `ambas`) e inicio (`actual` o `principio`), como en CAP-035.
  - Los modos no interactivos (pantalla completa, ventana activa, región por rectángulo, página web, portapapeles, archivo, PDF) se ejecutan sin mostrar ninguna superposición. Los interactivos muestran la selección normal y el comando espera a que termine.
  - Resultado: **estado** (`completada`, `cancelada`, `fallida` u `ocupada`), identificador de la captura en la biblioteca, rutas de los archivos escritos, tamaño en píxeles y metadatos (CAP-056).
  - Si ya hay otra captura en curso, el comando responde `ocupada` de inmediato.
  - Eventos que se pueden observar: captura completada, archivo escrito y error.
  - Los parámetros de los modos avanzados (desplazamiento, intervalo, etc.) llegan con su hito (M3).
- **Criterios de aceptación:**
  - [ ] Desde el CLI, una captura de pantalla completa con salida a un archivo termina sin superposición, escribe el archivo y devuelve `completada` con la ruta.
  - [ ] Una captura de `ventana activa` desde el CLI produce la imagen de la ventana que tenía el foco.
  - [ ] Si se lanza un comando de captura mientras hay una selección en pantalla, devuelve `ocupada` y no interrumpe la selección.
  - [ ] Si el usuario pulsa `Esc` en una captura interactiva lanzada desde MCP, el resultado es `cancelada`.
- **Mejora sobre Snagit:** reemplaza la API COM, que es solo de Windows, por comandos multiplataforma que también pueden usar los agentes.
- **Fuentes:** research/01-captura.md §1.9

---

## 2. Captura universal

### CAP-009 · Flujo de captura universal
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** All-in-One capture
- **Comportamiento:**
  - Se inicia desde la pestaña Universal (botón Capturar o atajo **Capturar**), con el atajo propio **Captura universal** (CAP-065), desde el Lanzador de borde o desde el menú del ícono.
  - Paso 1, apuntado: la pantalla se congela (CAP-013) y aparecen la cruceta (CAP-013) y la lupa (CAP-015). El usuario puede:
    - arrastrar una región;
    - pasar el puntero por una ventana o subárea detectada (CAP-016) y hacer clic;
    - pulsar el botón **Pantalla completa** de un monitor (CAP-021) o la tecla `F`;
    - mantener `Ctrl`/`Cmd` mientras arrastra para añadir varias áreas (CAP-033);
    - hacer clic en una flecha de desplazamiento automático (CAP-035).
  - Paso 2, ajuste: aparece la **barra de captura** junto a la selección, con los botones **Imagen**, **Video** y **Desplazamiento**, y las teclas `I`, `V` y `S` en ambas plataformas.
    - **Imagen** termina la captura y sigue la ruta de CAP-054.
    - **Video** pasa la misma área a la grabación (`VID-`). La pantalla deja de estar congelada.
    - **Desplazamiento** inicia el desplazamiento manual sobre esa área (CAP-034).
  - En la fase de ajuste se puede corregir la selección (CAP-011) o rehacerla (CAP-012).
  - La imagen tiene exactamente el tamaño en píxeles seleccionado, aunque sea impar. Si el video necesita tamaños pares, lo resuelve `VID-`.
- **Criterios de aceptación:**
  - [ ] Atajo → arrastrar 301×201 px → `I` produce una imagen de 301×201 px que se abre en el editor.
  - [ ] Atajo → clic en una ventana detectada → **Imagen** produce la imagen de esa ventana sin pasos extra.
  - [ ] En la barra de captura, `V` inicia el flujo de video de `VID-` con la misma área seleccionada.
  - [ ] `Esc` en cualquier paso cancela: no se crea ninguna captura ni entrada en la biblioteca.
- **Fuentes:** research/01-captura.md §2.1, §10; research/05-plataformas-versiones.md §1A (2024.1.2)

### CAP-010 · Opciones de la pestaña Universal
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** All-in-One basic settings (panel expandible)
- **Comportamiento:**
  - **Selección**: Región (por defecto; cruceta con detección), Ventana (solo ventanas y subáreas, sin arrastre) o Pantalla completa (salta el apuntado y muestra directamente la barra de captura con la pantalla elegida según CAP-021).
  - **Compartir**: solo muestra los destinos que aceptan tanto imagen como video (`SHR-`).
  - **Abrir en** (CAP-045), **Copiar al portapapeles** (CAP-046) y **Capturar cursor** (CAP-047).
  - **Retardo de 5 s**: casilla con una cuenta atrás fija de 5 segundos (CAP-048), pensada para abrir menús o estados que desaparecen.
  - Los ajustes se guardan al momento y persisten entre sesiones.
- **Criterios de aceptación:**
  - [ ] Con Selección = Pantalla completa en Universal, el atajo muestra la barra de captura con toda la pantalla seleccionada, sin arrastrar.
  - [ ] Con Retardo de 5 s activado, la cruceta aparece 5 s (±0,5 s) después de pulsar Capturar.
  - [ ] La lista Compartir de Universal no incluye destinos que solo aceptan imágenes.
- **Mejora sobre Snagit:** en Snagit para Mac, Universal no ofrece la selección Ventana; XcreenShot la ofrece en ambas plataformas.
- **Fuentes:** research/01-captura.md §2.2

### CAP-011 · Ajustar la selección y barra de captura
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** tiradores, mover la selección, ancho/alto escritos, barra arrastrable del All-in-One
- **Comportamiento:**
  - La selección muestra 8 tiradores para cambiar el tamaño. Arrastrar dentro de ella la mueve.
  - La barra de captura muestra campos editables **X**, **Y**, **Ancho** y **Alto** en píxeles. Al escribir un valor y pulsar `Enter`, la selección cambia y la esquina superior izquierda queda fija.
  - Los valores fuera de la pantalla se ajustan al límite, y el campo parpadea en rojo para avisarlo.
  - La barra de captura se puede arrastrar a cualquier lugar, también fuera de la selección. Recuerda su posición durante la sesión. Nunca forma parte de la imagen.
  - Por defecto, la barra se coloca bajo la selección. Si no cabe, va encima; si tampoco, dentro, en la esquina inferior derecha.
  - La selección puede ocupar varios monitores (CAP-021). Tamaño mínimo: 1×1 px.
  - Con la selección hecha, las teclas de CAP-017 siguen funcionando para moverla y cambiar su tamaño.
- **Criterios de aceptación:**
  - [ ] Si se escribe Ancho = 1280 y Alto = 720, la imagen resultante mide exactamente 1280×720 px.
  - [ ] Si la barra se arrastra encima de la selección, la imagen no la incluye.
  - [ ] Si se escribe un ancho mayor que el escritorio, el valor se limita al máximo posible y el campo avisa.
- **Mejora sobre Snagit:** también se pueden escribir X e Y, no solo el ancho y el alto.
- **Fuentes:** research/01-captura.md §2.1, §3.4

### CAP-012 · Rehacer la selección y cambiar de tipo
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** redo de la selección en All-in-One; cambiar una captura de video iniciada por error a imagen
- **Comportamiento:**
  - El botón **Rehacer** de la barra de captura (o `Retroceso`) descarta la selección y vuelve a la fase de apuntado, sin salir de la captura.
  - Tras rehacer, la barra de captura resalta el último tipo elegido (Imagen, Video o Desplazamiento), para que `Enter` lo repita.
  - Si el usuario eligió Video por error, la pantalla de preparación de la grabación (`VID-`) ofrece **Capturar imagen**: toma una imagen de la misma área y cancela la grabación. Si la grabación ya había empezado, la detiene y la descarta, y la imagen muestra el contenido actual del área.
- **Criterios de aceptación:**
  - [ ] **Rehacer** devuelve la cruceta sin cerrar la captura; una nueva selección + **Imagen** produce una sola captura.
  - [ ] Desde la preparación de video, **Capturar imagen** crea una imagen del área y no deja ningún video en la biblioteca.
- **Notas de plataforma:** la parte de video depende de `VID-` (M4); hasta entonces solo existe **Rehacer**.
- **Fuentes:** research/01-captura.md §2.1


---

## 3. Selección en pantalla

### CAP-013 · Pantalla congelada y cruceta
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** crosshairs a pantalla completa
- **Comportamiento:**
  - Al iniciar una selección interactiva, XcreenShot toma una imagen fija de todos los monitores y la muestra encima, para que nada cambie mientras el usuario elige (tooltips, menús, animaciones). La imagen final sale de esa imagen fija, no de lo que haya en pantalla después (decisión de diseño).
  - Excepción: en Video y Desplazamiento la pantalla deja de estar congelada al elegir el tipo.
  - Sobre la imagen congelada se ven unas líneas finas horizontal y vertical, de borde a borde y en todos los monitores, que se cruzan en el puntero. Un rótulo junto al puntero muestra su posición (x, y) y, mientras se arrastra, el tamaño (ancho × alto) en píxeles físicos.
  - El color de la cruceta y del borde de selección es el color de acento de XcreenShot (`APP-`). Tiene contraste suficiente sobre fondos claros y oscuros: línea de color con un contorno fino del color opuesto.
  - Fuera de la selección, la pantalla se oscurece un 40 %.
  - Rendimiento: la cruceta aparece en ≤ 300 ms desde el atajo con hasta 3 monitores 4K, y sigue el puntero sin retraso visible (≥ 60 fps).
- **Criterios de aceptación:**
  - [ ] Si se abre un tooltip y se lanza la captura con el atajo, el tooltip sale en la imagen aunque desaparezca durante la selección.
  - [ ] El rótulo de tamaño coincide con el tamaño en píxeles de la imagen final.
  - [ ] Con 3 monitores 4K, la cruceta aparece en ≤ 300 ms (medido en 10 intentos, percentil 90).
  - [ ] La cruceta se ve tanto sobre una página blanca como sobre una terminal negra.
- **Fuentes:** research/01-captura.md §2.1, §3.1, §3.2

### CAP-014 · Cruceta simplificada
- **Hito:** M1 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Simplified crosshairs
- **Comportamiento:**
  - Preferencia **Cruceta simplificada**, desactivada por defecto. Al activarla, la cruceta dibuja solo líneas de 1 px sin contorno, sin oscurecer el fondo y sin rótulo de coordenadas (la lupa sigue disponible).
  - Sirve para equipos lentos, máquinas virtuales, escritorio remoto o muchos monitores de alta resolución.
  - Si XcreenShot detecta que la cruceta va a menos de 30 fps durante 2 s seguidos, ofrece activarla una sola vez (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con la preferencia activada, la selección no oscurece el fondo y la cruceta es una línea de 1 px.
  - [ ] Las capturas hechas con la cruceta simplificada tienen el mismo contenido y tamaño que con la normal.
- **Mejora sobre Snagit:** disponible también en macOS; Snagit la documenta solo en Windows.
- **Fuentes:** research/01-captura.md §3.2

### CAP-015 · Lupa
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Magnifier
- **Comportamiento:**
  - Durante la selección, una ventana pequeña junto al puntero muestra ampliada la zona de alrededor, con una rejilla de píxeles y el píxel central marcado.
  - Debajo muestra la posición (x, y) y el color del píxel central en hexadecimal (decisión de diseño). `C` copia ese color al portapapeles sin salir de la captura (decisión de diseño).
  - La tecla `M` la oculta o la muestra durante la captura. La preferencia **Mostrar lupa** fija el estado inicial (por defecto: activada).
  - La lupa se aparta del borde de la pantalla para no taparse, y nunca sale en la captura.
  - Aumento por defecto ×4. Las teclas `+` y `-` lo cambian entre ×2, ×4, ×8 y ×16, y la app recuerda el último valor (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Al pulsar `M` durante la selección, la lupa desaparece; al pulsarla otra vez, vuelve.
  - [ ] Con **Mostrar lupa** desactivada, la captura empieza sin lupa y `M` la muestra.
  - [ ] En una pantalla Retina o al 200 %, la rejilla de la lupa coincide con los píxeles físicos.
  - [ ] Con el puntero en la esquina inferior derecha, la lupa se ve entera dentro de la pantalla.
- **Mejora sobre Snagit:** muestra el color del píxel y permite copiarlo.
- **Fuentes:** research/01-captura.md §3.1, §3.4

### CAP-016 · Detección automática de ventanas y subáreas
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** detección automática / ajuste a ventanas y objetos
- **Comportamiento:**
  - Al pasar el puntero en la fase de apuntado, se resalta con un borde discontinuo el elemento que hay debajo: una ventana, o una subárea (el contenido de un navegador, un panel, una barra de herramientas o un control). Un clic selecciona ese rectángulo.
  - Si hay elementos anidados, se resalta el más pequeño que contiene el puntero. La rueda del ratón (o `Tab` / `Shift+Tab`) sube o baja un nivel (decisión de diseño): control → panel → ventana → monitor.
  - Para ignorar la detección, basta con arrastrar.
  - La detección nunca elige el propio puntero, las capas transparentes que no se pueden pulsar (superposiciones de juegos, controladores gráficos, programas de compartir pantalla) ni las ventanas de XcreenShot.
  - Si una superposición del sistema o de otra app impide la detección, la captura sigue funcionando por arrastre. La preferencia **Detección automática** (por defecto: activada) permite desactivarla.
- **Criterios de aceptación:**
  - [ ] Al pasar por el contenido de un navegador se resalta solo el área de la página, sin pestañas ni barra de direcciones; un clic la captura con ese tamaño exacto.
  - [ ] Con el puntero sobre un botón, la rueda sube del botón a su panel y de ahí a la ventana completa.
  - [ ] Con una app de videollamada compartiendo pantalla, la detección resalta las ventanas reales y no el marco de la app.
  - [ ] Con **Detección automática** desactivada, no se resalta nada y el arrastre funciona igual.
- **Mejora sobre Snagit:** se puede elegir el nivel de anidamiento y desactivar la detección.
- **Fuentes:** research/01-captura.md §3.3

### CAP-017 · Teclado durante la selección
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** atajos durante la captura (Esc, F1/H, M, F, flechas, Enter, G, etc.)
- **Comportamiento:** las mismas teclas en ambas plataformas (decisión de diseño; en Snagit difieren y algunas no están claras en Mac).

  | Tecla | Fase de apuntado | Fase de ajuste |
  |---|---|---|
  | `Esc` | Cancela la captura | Cancela la captura |
  | Flechas | Mueven el puntero 1 px | Mueven la selección 1 px |
  | `Shift` + flechas | Mueven el puntero 10 px | Mueven la selección 10 px |
  | `Alt`/`Option` + flechas | — | Agrandan o reducen la selección 1 px por el borde derecho o inferior |
  | `Enter` | Selecciona el elemento resaltado por la detección (CAP-016); si no hay ninguno, hace lo mismo que `Shift+Enter` | Termina como **Imagen** (o con el último tipo, CAP-012) |
  | `Shift+Enter` | Fija una esquina en la posición del puntero e inicia una selección por teclado: las flechas mueven la esquina opuesta y `Enter` la termina | — |
  | `Espacio` (mantener) | — | Mientras se arrastra, mueve la selección en vez de cambiar su tamaño |
  | `F` | Pantalla completa del monitor bajo el puntero (CAP-021) | — |
  | `A` | Todos los monitores (CAP-021) | — |
  | `M` | Muestra u oculta la lupa | Igual |
  | `+` / `-` | Cambian el aumento de la lupa (CAP-015) | Igual |
  | `C` | Copia el color del píxel bajo el puntero (CAP-015) | — |
  | Rueda o `Tab` / `Shift+Tab` | Cambian el nivel de detección (CAP-016) | — |
  | `G` | Muestra u oculta las guías de tamaño (CAP-019) | Igual |
  | `H` o `F1` | Muestra u oculta la ayuda (CAP-020) | Igual |
  | `I` / `V` / `S` | — | Imagen / Video / Desplazamiento |
  | `R` / `D` / `X` | Flecha de desplazamiento automático horizontal / vertical / ambas (CAP-035) | — |
  | `Retroceso` | — | Rehacer (CAP-012) |
  | `Shift` (mantener) | Bloquea un cuadrado 1:1 (CAP-018) | Igual |
  | `Alt`/`Option` (mantener) | Bloquea 16:9; con `Shift`, 4:3 (CAP-018) | Igual |
  | `Ctrl`/`Cmd` (mantener) al arrastrar | Añade otra área (CAP-033) | — |

  - El perfil de compatibilidad (CAP-066) cambia en Windows los modificadores de proporción y de múltiples áreas, las letras del desplazamiento automático y la tecla de la ayuda, para que coincidan con Snagit.
  - El teclado nunca dispara acciones de la app que está debajo mientras dura la selección.
- **Criterios de aceptación:**
  - [ ] Pulsar `→` 5 veces en la fase de ajuste mueve la selección exactamente 5 px a la derecha; `Shift+→` la mueve 10 px.
  - [ ] Hacer toda una captura de región solo con el teclado (mover el puntero, `Enter`, flechas, `Enter`) produce una imagen.
  - [ ] En macOS, `Enter` termina la captura y nunca la convierte en pantalla completa.
  - [ ] `Esc` funciona en ambas fases y en todos los modos interactivos.
- **Mejora sobre Snagit:** teclas iguales en las dos plataformas, y cambio de tamaño con teclado.
- **Fuentes:** research/01-captura.md §3.4, §10, §16 (punto 5)

### CAP-018 · Bloqueo de proporción
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** bloqueo cuadrado (Shift), 16:9 (Ctrl), 4:3 (Ctrl+Shift)
- **Comportamiento:**
  - Mientras se arrastra o se cambia el tamaño:
    - `Shift` mantenido bloquea la proporción 1:1;
    - `Alt`/`Option` mantenido bloquea 16:9;
    - `Alt+Shift` / `Option+Shift` mantenidos bloquean 4:3.
  - Estos modificadores son iguales en ambas plataformas (decisión de diseño). `Ctrl`/`Cmd` queda libre para añadir áreas (CAP-033). En Windows, el perfil de compatibilidad (CAP-066) usa en su lugar `Ctrl` = 16:9 y `Ctrl+Shift` = 4:3, como Snagit; con ese perfil, las áreas múltiples solo se hacen en el modo Múltiples áreas.
  - La barra de captura tiene además un selector de **Proporción** (Libre, 1:1, 4:3, 16:9, 9:16, 3:2 y Personalizada a:b). Esa proporción queda fija hasta que se cambie, y se recuerda entre capturas (decisión de diseño).
  - Al soltar la tecla, la proporción deja de estar bloqueada y se mantiene el tamaño actual.
  - El rótulo de tamaño muestra la proporción activa.
- **Criterios de aceptación:**
  - [ ] Arrastrar con `Shift` produce una imagen cuadrada (ancho = alto ±0 px).
  - [ ] Con Proporción = 16:9 en la barra, cualquier arrastre da una relación 16:9 (redondeada a 1 px).
  - [ ] `Alt`/`Option` bloquea 16:9 y `Ctrl`/`Cmd` sigue sirviendo para añadir áreas, sin conflicto entre ambos.
  - [ ] (Windows, perfil de compatibilidad) `Ctrl` mantenido bloquea 16:9.
- **Mejora sobre Snagit:** el 4:3 y las proporciones personalizadas existen en ambas plataformas y valen para imagen, no solo para video.
- **Fuentes:** research/01-captura.md §3.4; research/05-plataformas-versiones.md §3 (Video: bloqueo 4:3)

### CAP-019 · Guías de tamaño estándar
- **Hito:** M1 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** guías de tamaño estándar (G)
- **Comportamiento:**
  - `G` muestra u oculta, durante la selección, contornos tenues de tamaños estándar: 3840×2160, 2560×1440, 1920×1080, 1280×720, 1080×1080, 640×480 y 320×240.
  - Las guías se anclan a la esquina donde empezó el arrastre.
  - Mientras las guías están visibles, si la selección queda a 8 px o menos de un tamaño estándar, se ajusta a él y el borde se resalta. Con las guías ocultas no hay ajuste.
  - La lista de tamaños se puede editar en Preferencias (decisión de diseño).
  - Sirven igual para imagen y para video.
- **Criterios de aceptación:**
  - [ ] Con las guías activas, al arrastrar cerca de 1280×720 la selección se ajusta a 1280×720 exactos y el borde se resalta.
  - [ ] Con las guías ocultas, se puede seleccionar 1275×718 sin que se ajuste.
- **Fuentes:** research/01-captura.md §3.4

### CAP-020 · Ayuda en pantalla durante la captura
- **Hito:** M1 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** ayuda en pantalla (F1 en Windows, H en Mac)
- **Comportamiento:**
  - `H` o `F1` muestra u oculta un panel semitransparente con las teclas que sirven en ese momento. Su contenido cambia según la fase y el modo.
  - Las 3 primeras veces que se usa la cruceta aparece una pista breve, "H: ayuda", que se puede desactivar (decisión de diseño).
  - El panel nunca sale en la captura.
- **Criterios de aceptación:**
  - [ ] Al pulsar `H` durante una selección de región aparece el panel con las teclas de CAP-017; al pulsar `H` otra vez desaparece.
  - [ ] Con el panel visible y la captura terminada, la imagen no lo incluye.
- **Fuentes:** research/01-captura.md §3.4

### CAP-021 · Varios monitores y DPI mixto
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** soporte multimonitor y DPI mixto; botón Fullscreen por monitor
- **Comportamiento:**
  - La selección cubre todos los monitores a la vez, y una región puede cruzar de un monitor a otro.
  - En la parte superior central de cada monitor aparece un botón **Pantalla completa** que captura ese monitor. Cerca del primero, otro botón **Todos los monitores** captura el escritorio completo. Teclas: `F` para el monitor bajo el puntero y `A` para todos.
  - Con escalados distintos (por ejemplo 100 % y 200 %), la imagen se genera en píxeles físicos. Si una región cruza monitores con escalados distintos, se usa el escalado mayor y la parte del monitor de menor densidad se amplía (decisión de diseño). El usuario lo ve en el rótulo de tamaño.
  - Cada captura guarda su escala (1×, 2×, etc.) en los metadatos. El editor la usa para mostrarla a su tamaño real (`EDT-`).
  - En el escritorio completo, las zonas que no cubre ningún monitor (monitores de tamaños distintos) quedan transparentes.
  - Si se conecta o desconecta un monitor durante la selección, la captura se cancela con un aviso.
- **Criterios de aceptación:**
  - [ ] Con dos monitores, el botón Pantalla completa del secundario produce una imagen del tamaño nativo de ese monitor.
  - [ ] **Todos los monitores** produce una imagen que cubre el rectángulo envolvente de todos, con transparencia en los huecos.
  - [ ] Con un monitor al 100 % y otro al 200 %, una región de la misma zona lógica mide el doble en píxeles en el monitor al 200 %.
  - [ ] Una región que cruza dos monitores produce una sola imagen continua, sin desplazamientos en la unión.
- **Mejora sobre Snagit:** la misma elección (un monitor o todos) en Windows y en macOS.
- **Fuentes:** research/01-captura.md §3.5; research/05-plataformas-versiones.md §3 (Captura: pantalla completa multimonitor)

### CAP-022 · Pantallas HDR
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** HDR image capture, "Support HDR color settings", experimento HDR Capture Fidelity
- **Comportamiento:**
  - Si un monitor está en modo HDR, la imagen capturada se convierte a SDR (sRGB) para que se vea como en pantalla, sin colores lavados ni blancos grises.
  - Preferencia **Corrección de color HDR**: Automática (por defecto) / Desactivada.
  - Opción avanzada **Fidelidad HDR alta**: ajusta los tonos a los de la pantalla, aunque tarde más.
  - Aplica a todas las capturas de imagen, incluidos el desplazamiento y la captura de video. Para video, ver `VID-`.
  - Los documentos se guardan siempre en SDR de 8 bits por canal (decisión de diseño). No se guarda HDR.
- **Criterios de aceptación:**
  - [ ] En un monitor HDR, una página blanca capturada da blanco puro (255,255,255 ±3) en la imagen.
  - [ ] Con la corrección desactivada, la imagen se toma sin conversión y es visiblemente distinta.
  - [ ] Una captura de desplazamiento en un monitor HDR tiene el mismo tono que una de región de la misma zona.
- **Notas de plataforma:** la conversión depende de las funciones de color de cada sistema. En macOS, el sistema puede entregar ya la imagen en SDR; en ese caso la opción no cambia el resultado y se indica así.
- **Mejora sobre Snagit:** también en macOS y también en capturas de desplazamiento, que Snagit excluye.
- **Fuentes:** research/01-captura.md §3.6, §11.1; research/05-plataformas-versiones.md §1A (2024.2.2, 2024.3.2, 2025.4.0), §2, §5

### CAP-023 · Ocultar XcreenShot al capturar
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** "Hide Snagit during capture and remove Snagit controls from completed captures"
- **Comportamiento:**
  - Preferencia **Ocultar XcreenShot al capturar** (por defecto: activada). Antes de tomar la imagen se ocultan:
    - el panel de captura;
    - el editor;
    - el Lanzador de borde;
    - las notificaciones de XcreenShot.
  - Al terminar, todo vuelve a su estado anterior.
  - Siempre, también con la preferencia desactivada, quedan fuera de cualquier captura (imagen, desplazamiento, intervalo, video):
    - la cruceta y la lupa;
    - la barra de captura;
    - la ayuda en pantalla;
    - la vista previa y los controles del desplazamiento;
    - las cuentas atrás.
  - Si el editor está abierto, un botón **Incluir el editor** en la barra de captura (decisión de diseño) lo vuelve a mostrar, para capturar el propio XcreenShot.
  - Ocultar y restaurar tarda ≤ 150 ms en total.
- **Criterios de aceptación:**
  - [ ] Con el editor abierto y maximizado, una captura de pantalla completa muestra lo que había detrás del editor.
  - [ ] Una captura de desplazamiento manual no incluye la vista previa ni el botón Listo, aunque estuvieran sobre el área.
  - [ ] Con la preferencia desactivada y el panel abierto, el panel aparece en la captura, pero la cruceta y la lupa no.
- **Notas de plataforma:** en Windows, excluir ventanas propias de la imagen depende de una función del sistema que no existe en versiones antiguas de Windows 10. En ellas XcreenShot oculta sus ventanas antes de cada fotograma (`APP-` fija las versiones mínimas).
- **Fuentes:** research/01-captura.md §6.1, §11.1, §11.2; research/05-plataformas-versiones.md §1A (2024.1.0, 2026.2.0), §3 (Captura)

### CAP-024 · Permisos que faltan al capturar
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas (con detalle en macOS)
- **Paridad con Snagit:** "Fix Capture Permissions"; captura que solo muestra el escritorio sin permiso
- **Comportamiento:**
  - Antes de capturar, XcreenShot comprueba que tiene los permisos necesarios: en macOS, **Grabación de pantalla** y, para el desplazamiento automático, **Accesibilidad**.
  - Si falta un permiso, no hace una captura incompleta (por ejemplo, solo el fondo de escritorio). Muestra un aviso que explica qué falta y un botón que abre el ajuste del sistema. La gestión de permisos la define `APP-`.
  - Cuando se concede el permiso, la captura pendiente se reanuda sola sin repetir los pasos. Si el sistema exige reiniciar la app, XcreenShot lo dice y vuelve a la misma captura al arrancar.
  - Si al capturar una ventana protegida o con más privilegios solo se obtienen píxeles negros, la captura termina igual y se avisa al usuario.
- **Criterios de aceptación:**
  - [ ] (macOS) Si se retira el permiso de Grabación de pantalla, el siguiente intento de captura muestra el aviso y no crea ninguna imagen del fondo.
  - [ ] (macOS) Tras conceder el permiso desde el aviso, la captura sigue sin volver a pulsar el atajo.
  - [ ] (Windows) La captura de una ventana con más privilegios que XcreenShot muestra su contenido o, si el sistema lo impide, avisa.
- **Fuentes:** research/01-captura.md §6.2, §14; research/05-plataformas-versiones.md §1B (2024.1.0)

---

## 4. Modos de captura de pantalla

### CAP-025 · Región
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Region (Image tab > Selection = Region)
- **Comportamiento:**
  - En la pestaña Imagen, con Selección = Región, la captura muestra la cruceta (CAP-013), la lupa (CAP-015) y la detección automática (CAP-016).
  - El usuario arrastra un rectángulo, o hace clic en un elemento resaltado para usar su rectángulo.
  - La región captura los píxeles visibles en ese rectángulo, con todo lo que haya encima: ventanas superpuestas, menús o tooltips. Si se quiere solo la ventana, aunque esté tapada, se usa Ventana (CAP-027).
  - Al soltar el ratón, la captura termina al momento y sigue la ruta de CAP-054.
  - Propiedad **Confirmar antes de capturar** (por defecto: desactivada) (decisión de diseño): al activarla, después de soltar aparece la fase de ajuste. Ahí están los tiradores, los campos X, Y, Ancho y Alto (CAP-011) y un único botón **Capturar** (`Enter`).
  - Una selección hecha con el teclado (`Shift+Enter`, CAP-017) pasa siempre por la fase de ajuste.
  - Durante la selección valen los bloqueos de proporción (CAP-018), las guías de tamaño (CAP-019) y la región fija (CAP-026).
  - Un clic sin arrastrar, sin ningún elemento resaltado, no captura nada: la cruceta sigue esperando.
- **Criterios de aceptación:**
  - [ ] Arrastrar 640×480 px y soltar produce, sin más pasos, una imagen de 640×480 px.
  - [ ] Si una ventana tapa parte de otra, una región sobre ambas incluye las dos tal como se veían.
  - [ ] Con **Confirmar antes de capturar** activada, al soltar aparecen los tiradores y la imagen solo se crea al pulsar `Enter` o **Capturar**.
  - [ ] Caso límite: un clic sin arrastrar y sin elemento resaltado no crea ninguna captura.
- **Fuentes:** research/01-captura.md §5.1, §3.4

### CAP-026 · Región fija y posición
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Selection Properties > Fixed region, Set position, Select Region
- **Comportamiento:**
  - En las **Propiedades** de Región está la casilla **Región fija**, con los campos **Ancho** y **Alto**.
  - La casilla opcional **Posición fija** añade los campos **Monitor**, **X** e **Y**. El botón **Elegir en pantalla** permite arrastrar un área y rellena los cinco campos con ella.
  - Unidades (decisión de diseño):
    - píxeles físicos;
    - X e Y se miden desde la esquina superior izquierda del monitor elegido;
    - el monitor se identifica por su nombre y, si hay dos iguales, por su posición.
    - Son las mismas unidades que usan los comandos (CAP-008).
  - **Sin posición fija:** un rectángulo del tamaño indicado sigue al puntero. Un clic lo coloca y captura. Las flechas lo mueven 1 px (`Shift` + flechas, 10 px) y `Enter` captura. El rectángulo nunca sale del monitor.
  - **Con posición fija:** la captura se hace al momento, sin cruceta ni superposición, después del retardo si lo hay. Es la base de los presets de "misma zona" y del intervalo (CAP-049).
  - Si el monitor guardado no existe o el rectángulo no cabe en él, no se hace una captura parcial: aparece un aviso con **Elegir de nuevo** y **Usar el monitor principal**.
  - Límites: mínimo 1×1 px; máximo, el tamaño del monitor elegido. Si se escribe un valor mayor, se limita a ese máximo y el campo avisa. Nunca se cierra la app.
  - El panel de captura se oculta mientras se usa **Elegir en pantalla**.
- **Criterios de aceptación:**
  - [ ] Con 1080×1080 sin posición, un rectángulo de 1080×1080 sigue al puntero y un clic produce una imagen de 1080×1080 px.
  - [ ] Con posición fija (monitor 2, X = 100, Y = 50, 800×600), el atajo produce sin interacción una imagen idéntica a una región dibujada a mano en esas coordenadas.
  - [ ] Si se desconecta el monitor 2, la misma captura muestra el aviso y no crea ninguna imagen.
  - [ ] Caso límite: si se escribe Ancho = 100000, el valor se limita al ancho del monitor y no hay error.
- **Fuentes:** research/01-captura.md §4.2; research/05-plataformas-versiones.md §3 (Captura: región fija)

### CAP-027 · Ventana
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Window (Selection = Window en Windows; Window y Exposé Window Capture en Mac)
- **Comportamiento:**
  - Con Selección = Ventana, la cruceta solo resalta ventanas y cuadros de diálogo; no hay arrastre libre. La rueda o `Tab` (CAP-016) bajan a sus subáreas cuando se quiere capturar solo una parte de la aplicación.
  - Un clic captura la ventana resaltada. La imagen es la de la propia ventana, **entera aunque otra ventana la tape o una parte quede fuera de la pantalla** (decisión de diseño). Si el sistema no permite obtener la ventana por separado (por ejemplo, contenido protegido), se usan los píxeles visibles de su rectángulo y se avisa.
  - Propiedades:
    - **Esquinas redondeadas transparentes** (por defecto: activada): las esquinas redondeadas que dibuja el sistema quedan transparentes, no con trozos del fondo ni cuadradas.
    - **Incluir sombra** (por defecto: desactivada): añade alrededor la sombra que el sistema dibuja para la ventana, sobre un margen transparente.
  - La acción **Ventana activa** (CAP-065, CAP-008) captura sin cruceta la ventana que tiene el foco.
  - Para video, la grabación de una ventana que sigue a la ventana cuando se mueve la define `VID-`.
- **Criterios de aceptación:**
  - [ ] Con una ventana tapada a medias por otra, la captura de Ventana muestra la ventana completa sin la que la tapaba.
  - [ ] En Windows 11 y en macOS, las cuatro esquinas de la imagen son transparentes fuera de la curva de la ventana.
  - [ ] Con **Incluir sombra**, la imagen es mayor que la ventana y el margen tiene sombra sobre transparencia.
  - [ ] **Ventana activa** captura la ventana con el foco sin mostrar la cruceta.
- **Mejora sobre Snagit:** esquinas redondeadas correctas también en Windows 11, donde Snagit las deja cuadradas; ventanas tapadas o parcialmente fuera de la pantalla salen completas.
- **Fuentes:** research/01-captura.md §5.2, §1.9; research/05-plataformas-versiones.md §1B (2026.0.0), §3 (Captura: ventana)

### CAP-028 · Pantalla completa
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Fullscreen (tipo de selección, botón Fullscreen y tecla F)
- **Comportamiento:**
  - Con Selección = Pantalla completa, la captura se hace sin cruceta, después del retardo si lo hay.
  - Propiedad **Monitor**:
    - **Bajo el puntero** (por defecto);
    - **Principal**;
    - **Todos los monitores**;
    - un monitor concreto, por nombre.
  - Durante cualquier selección interactiva también están los botones **Pantalla completa** y **Todos los monitores** y las teclas `F` y `A` (CAP-021).
  - En la pestaña Universal, Pantalla completa no captura directamente: muestra la barra de captura con el monitor seleccionado (CAP-010).
  - Propiedad **Ventanas como objetos** (CAP-029).
- **Criterios de aceptación:**
  - [ ] Con Monitor = Bajo el puntero y el puntero en el monitor 2, el atajo produce una imagen del tamaño nativo del monitor 2.
  - [ ] Con Monitor = Todos los monitores, la imagen cubre todos los monitores (CAP-021).
  - [ ] Si el monitor concreto guardado ya no existe, se usa el principal y se avisa.
- **Fuentes:** research/01-captura.md §5.3, §3.5

### CAP-029 · Ventanas como objetos
- **Hito:** M3 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** experimento de Mac "pantalla completa con cada ventana como capa editable"
- **Comportamiento:**
  - Propiedad de Pantalla completa (por defecto: desactivada). Al activarla, el documento resultante tiene como base el escritorio y cada ventana visible como un objeto de imagen aparte (`EDT-`), en su posición y en el mismo orden de apilado.
  - Cada objeto de ventana se captura entero, aunque otra ventana lo tape (como CAP-027).
  - La base es el escritorio sin ventanas (fondo e íconos) cuando el sistema lo permite. Si no lo permite, es la pantalla tal como se veía, y al mover una ventana su hueco muestra lo que había en ese sitio (decisión de diseño).
  - Al copiar al portapapeles o enviar a un destino sin pasar por el editor, todo se aplana en una sola imagen igual a una pantalla completa normal.
  - No incluye las ventanas minimizadas ni las de otros escritorios virtuales.
- **Criterios de aceptación:**
  - [ ] Con tres ventanas visibles, el documento en el editor tiene tres objetos de imagen, cada uno con el nombre de su aplicación.
  - [ ] Si en el editor se elimina el objeto de la ventana de delante, se ve completa la ventana que tenía detrás.
  - [ ] La copia al portapapeles es idéntica a una captura de pantalla completa sin la opción.
- **Mejora sobre Snagit:** función estable en ambas plataformas; en Snagit es un experimento solo de Mac.
- **Fuentes:** research/01-captura.md §5.3, §11.2; research/05-plataformas-versiones.md §5

### CAP-030 · Selector de ventanas
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Mission Control capture (Mac)
- **Comportamiento:**
  - Con Selección = Selector de ventanas, o con su atajo (CAP-065), aparece sobre el monitor bajo el puntero una vista que muestra separadas todas las ventanas abiertas del escritorio actual, también las tapadas y las minimizadas. Cada miniatura lleva el nombre de la aplicación y el título.
  - Al pasar el puntero se resalta una miniatura; un clic captura esa ventana **a tamaño real**, no la miniatura. Se aplican las propiedades de CAP-027 (esquinas, sombra).
  - Teclado: las flechas mueven el resaltado, `Enter` captura y `Esc` cancela. Al escribir, la lista se filtra por aplicación o título (decisión de diseño).
  - Ventanas minimizadas (decisión de diseño): al elegirlas, se restauran un momento, se capturan y vuelven a minimizarse.
  - Capturar cursor: solo se incluye si el puntero estaba sobre esa ventana al abrir el selector.
  - El selector nunca aparece en la imagen.
- **Criterios de aceptación:**
  - [ ] Con una ventana totalmente tapada por otra, el selector la muestra y un clic produce su imagen completa a tamaño real.
  - [ ] Escribir parte del título deja solo las ventanas que coinciden; `Enter` captura la resaltada.
  - [ ] Una ventana minimizada se captura completa y queda minimizada otra vez al terminar.
  - [ ] `Esc` cierra el selector sin crear ninguna captura.
- **Notas de plataforma:** en ninguna de las dos plataformas se pueden capturar las ventanas de otros escritorios virtuales (Spaces en macOS, escritorios en Windows) sin cambiar a ellos; el selector no las muestra.
- **Mejora sobre Snagit:** existe también en Windows, con filtro por texto y control con teclado.
- **Fuentes:** research/01-captura.md §5.4, §10; research/05-plataformas-versiones.md §3 (Captura: ventana)

### CAP-031 · Menú
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Menu capture, Menu Properties ("Capture opened sub-menus", "Capture menu bar"), atajo de menú de Mac
- **Comportamiento:**
  - Dos formas de iniciarla:
    - **Con el menú ya abierto:** el usuario abre el menú (contextual, desplegable o de la barra de menús) y pulsa el atajo **Captura de menú** (CAP-065). La captura se hace sin cuenta atrás.
    - **Desde el panel o un preset:** al pulsar Capturar empieza una cuenta atrás (el valor de Retardo; si Tiempo = Ninguno, 5 s) y el usuario abre el menú mientras corre.
  - Al terminar la cuenta atrás, la pantalla se congela (CAP-013). XcreenShot detecta los menús abiertos y los resalta. `Enter` o un clic confirman; arrastrar hace una región normal en su lugar.
  - Propiedades:
    - **Incluir submenús abiertos** (por defecto: activada): junta todos los niveles abiertos en sus posiciones reales.
    - **Incluir la barra de menús** (por defecto: desactivada): añade el elemento de la barra de menús, o el elemento padre, del que cuelga el menú.
  - El resultado conserva la forma del menú: el espacio entre niveles y fuera de las esquinas redondeadas queda transparente.
  - Si no se detecta ningún menú (por ejemplo, menús que dibuja la propia app), se avisa y la captura sigue como una región.
- **Criterios de aceptación:**
  - [ ] Con un menú y un submenú abiertos y **Incluir submenús**, la imagen contiene los dos niveles con transparencia entre ellos y nada más.
  - [ ] Con **Incluir la barra de menús**, la imagen incluye también el título del menú en la barra.
  - [ ] Desde el panel con Retardo = 3 s, hay 3 s para abrir el menú; después se resalta y `Enter` lo captura.
  - [ ] Cada nivel de menú se captura una sola vez, sin repetirse.
- **Notas de plataforma:** si en alguna app el sistema cierra el menú al pulsar el atajo global, hay que usar la cuenta atrás. XcreenShot lo indica en la ayuda en pantalla (CAP-020).
- **Fuentes:** research/01-captura.md §5.5, §4.8, §10; research/05-plataformas-versiones.md §3 (Captura: menú)

### CAP-032 · Mano alzada
- **Hito:** M3 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Freehand (Advanced > Freehand, Windows)
- **Comportamiento:**
  - Con Selección = Mano alzada, el puntero se convierte en unas tijeras.
  - Propiedad **Forma**:
    - **Libre** (por defecto): se arrastra cualquier contorno y, al soltar, se cierra con una línea recta hasta el punto de partida;
    - **Polígono**: cada clic añade un vértice, y doble clic o `Enter` cierra la forma;
    - **Elipse**: se arrastra; `Shift` la hace círculo.
  - La imagen mide lo que el rectángulo que rodea la forma. Los píxeles fuera de la forma son transparentes (decisión de diseño).
  - En el portapapeles y en los archivos que admiten transparencia, la forma se mantiene; en los que no la admiten, el exterior es blanco (CAP-046).
  - `Esc` cancela y `Retroceso` quita el último vértice del polígono.
- **Criterios de aceptación:**
  - [ ] Un contorno libre en forma de estrella produce una imagen con la estrella y transparencia fuera de ella.
  - [ ] Pegar el resultado en una app que acepta PNG mantiene la forma, no un rectángulo.
  - [ ] Un polígono de 5 clics y `Enter` produce un pentágono.
- **Mejora sobre Snagit:** disponible en macOS y con formas de polígono y elipse.
- **Fuentes:** research/01-captura.md §5.6, §4.6; research/05-plataformas-versiones.md §3 (Captura: mano alzada)

### CAP-033 · Múltiples áreas
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Multiple Area (Advanced > Multiple Area en Windows; Command-drag en All-in-One de Mac)
- **Comportamiento:**
  - Dos formas de iniciarla:
    - Selección = Múltiples áreas;
    - en Universal o en Región, mantener `Ctrl`/`Cmd` mientras se arrastra (CAP-017).
  - Cada arrastre añade una región y cada clic en un elemento resaltado añade ese elemento. Las áreas quedan marcadas con un número.
  - Cada área tiene una **X** para quitarla. `Retroceso` quita la última.
  - Para terminar:
    - el botón **Terminar** de la parte superior del monitor;
    - `Enter`, si hay al menos un área;
    - en el modo de arrastre con `Ctrl`/`Cmd`, soltar la tecla con dos o más áreas.
    - Con el perfil de compatibilidad en Windows (CAP-066), también el clic derecho.
  - Al terminar desde Universal, la barra de captura solo ofrece **Imagen**: Video y Desplazamiento necesitan un área única.
  - Propiedad **Composición** (decisión de diseño):
    - **Posición original** (por defecto): el lienzo es el rectángulo que rodea todas las áreas, cada una en su sitio real, y el resto es transparente. Si dos áreas se solapan, se unen.
    - **Apiladas**: una debajo de otra en el orden en que se eligieron, alineadas a la izquierda, con una separación de 10 px (editable).
    - **En fila**: una al lado de otra, alineadas arriba, con la misma separación.
  - En el editor, cada área llega como un objeto de imagen aparte sobre un lienzo transparente, para poder moverla (`EDT-`). Al portapapeles o a un archivo va aplanada.
- **Criterios de aceptación:**
  - [ ] Dos áreas de 200×100 px en esquinas opuestas, con Posición original, producen una imagen del tamaño del rectángulo que las rodea, con transparencia entre ellas.
  - [ ] Las mismas áreas en Apiladas producen una imagen de 200×210 px.
  - [ ] En el editor, cada área se puede mover por separado.
  - [ ] Si se quita un área con su **X** antes de terminar, no aparece en el resultado.
- **Mejora sobre Snagit:** el usuario elige cómo se componen las áreas; Snagit no documenta cómo las combina.
- **Fuentes:** research/01-captura.md §5.7, §3.4, §16 (punto 4)

### CAP-034 · Desplazamiento manual
- **Hito:** M3 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Scrolling Capture manual (antes Panoramic)
- **Comportamiento:**
  - Se inicia de dos formas:
    - el botón **Desplazamiento** (`S`) de la barra de captura universal;
    - Selección = Desplazamiento con la propiedad **Tipo = Manual**, y luego seleccionar el área.
  - Preferencia **Pedir Iniciar antes de desplazar** (por defecto: desactivada). Si está activada, aparece el botón **Iniciar**; si no, la captura empieza al momento.
  - El usuario desplaza el contenido como quiera: barras de desplazamiento, rueda, arrastre o teclado. Puede ir en vertical, en horizontal o en ambas direcciones.
  - Una **vista previa** junto al área muestra en vivo la imagen unida y su tamaño en píxeles.
  - Botones **Listo** (`Enter`) y **Cancelar** (`Esc`).
  - La vista previa y los botones se colocan fuera del área: en el lado con más espacio o, si no caben, en otro monitor. Nunca salen en el resultado (CAP-023).
  - Si la vista previa queda tapada, el menú del ícono ofrece **Terminar desplazamiento** (CAP-005).
  - El área mínima es de 50×50 px (decisión de diseño). Con un área menor, el botón **Desplazamiento** está desactivado y su tooltip lo explica.
  - Al llegar al tamaño máximo (CAP-036), la captura se detiene sola, conserva lo capturado y avisa.
  - Al pulsar Listo empieza el procesamiento (CAP-036).
- **Criterios de aceptación:**
  - [ ] Al desplazar una página de 5 pantallas de alto con la rueda y pulsar **Listo**, sale una imagen con todo el contenido recorrido, sin huecos ni repeticiones.
  - [ ] Un desplazamiento en diagonal (primero hacia abajo y luego a la derecha) produce una imagen con las dos dimensiones ampliadas.
  - [ ] La vista previa y los botones no aparecen en el resultado aunque estuvieran encima del área.
  - [ ] Caso límite: un área de 30×30 px no permite iniciar el desplazamiento y no causa error.
- **Fuentes:** research/01-captura.md §6 (historia de nombres), §6.1, §11.1, §11.2

### CAP-035 · Desplazamiento automático
- **Hito:** M3 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Automatic Scrolling Capture (flechas de desplazamiento; teclas H/V/B en Windows, R/D/X en Mac)
- **Comportamiento:**
  - Durante una selección en Universal, en Región o en Desplazamiento con **Tipo = Automático**, al pasar por un área que se puede desplazar aparecen tres flechas animadas:
    - **horizontal**, en el borde derecho;
    - **vertical**, en el borde inferior;
    - **ambas**, en la esquina inferior derecha.
  - También se eligen con `R`, `D` y `X` (CAP-017).
  - Con Tipo = Automático, un clic en el área sin tocar ninguna flecha usa la propiedad **Dirección** (por defecto: Vertical).
  - Al elegir una flecha, XcreenShot desplaza el contenido por sí mismo, captura cada tramo y los une (CAP-036).
  - Propiedades:
    - **Empezar desde**: **El principio** (por defecto; arriba, a la izquierda o arriba a la izquierda según la dirección) o **La posición actual**.
    - **Espera entre pasos**: de 0 a 5 s; por defecto 0,3 s. Da tiempo a que cargue el contenido que aparece al desplazar.
    - **Volver a la posición inicial al terminar** (por defecto: activada).
  - Mientras dura, una barra de progreso muestra los píxeles capturados, con **Detener aquí** (`Enter`, conserva lo capturado) y **Cancelar** (`Esc`, no crea captura).
  - Si el usuario hace clic o mueve la rueda dentro del área, el proceso se detiene y conserva lo capturado (decisión de diseño).
  - Si un área no se deja desplazar desde fuera, no aparecen flechas. La ayuda en pantalla sugiere el desplazamiento manual (CAP-034), la página web por URL (CAP-037) o, para documentos, importar el PDF (CAP-041). Casos típicos: varios paneles anidados, algunos visores de PDF o vistas especiales de hojas de cálculo.
  - Si al terminar el contenido no se ha movido nada, se avisa y se ofrece capturar solo el área visible.
  - En macOS hace falta el permiso de Accesibilidad (CAP-024).
- **Criterios de aceptación:**
  - [ ] En un navegador con una página larga, pulsar `D` sobre el área de contenido produce la página completa, de arriba abajo, aunque al empezar estuviera desplazada a la mitad (Empezar desde = El principio).
  - [ ] En una lista de archivos con columnas anchas, `R` produce la imagen completa en horizontal.
  - [ ] **Detener aquí** a mitad de camino produce una imagen con lo recorrido hasta ese momento.
  - [ ] Al terminar, la página vuelve a la posición en que estaba antes de la captura.
- **Mejora sobre Snagit:** mismas teclas en ambas plataformas; espera configurable para contenido que se carga al desplazar; vuelta a la posición inicial.
- **Fuentes:** research/01-captura.md §6.2, §1.9, §3.4, §14; research/05-plataformas-versiones.md §3 (Captura: desplazamiento automático)

### CAP-036 · Unión y procesamiento del desplazamiento
- **Hito:** M3 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** procesamiento del Scrolling Capture (barra de progreso, cancelación, manejo de cabeceras y pies fijos)
- **Comportamiento:**
  - Las capturas de desplazamiento, manual o automático, se unen buscando la parte común entre tramos seguidos.
  - **Detectar elementos fijos** (por defecto: activada): los elementos que no se mueven al desplazar aparecen **una sola vez**:
    - las cabeceras, en su posición de la primera vista;
    - los pies, en la de la última;
    - las barras laterales fijas, a lo largo de todo el resultado;
    - los botones flotantes, en su posición de la última vista.
  - El procesamiento muestra una barra de progreso con mensajes de estado (por ejemplo, "Uniendo 12 de 40") y un botón **Cancelar**. Cancelar no crea ninguna captura.
  - El procesamiento se hace en segundo plano: el usuario puede seguir trabajando e incluso empezar otra captura.
  - Si alguna unión es dudosa (contenido animado, efectos de paralaje), el resultado se crea igual y una notificación indica cuántas uniones pueden tener cortes (decisión de diseño).
  - Tamaño máximo (decisión de diseño): 60 000 px en el lado más largo o 250 megapíxeles.
  - El resultado lleva en sus metadatos el tipo "desplazamiento" (CAP-056), para que la biblioteca lo pueda filtrar (`LIB-`). La exportación a PDF paginado de las capturas largas la define `EDT-`.
  - La corrección HDR también se aplica (CAP-022).
- **Criterios de aceptación:**
  - [ ] En una página de 8000 px con una cabecera fija de 80 px, la cabecera aparece una sola vez, arriba, y el contenido no se repite.
  - [ ] Si se cancela el procesamiento, no queda ninguna captura ni archivo temporal visible.
  - [ ] Mientras se procesa una captura larga, se puede iniciar y terminar una captura de región normal.
  - [ ] Al llegar a 60 000 px, la captura se detiene, se guarda lo capturado y aparece un aviso.
- **Fuentes:** research/01-captura.md §6.1, §6.2, §3.6, §12.1

---

## 5. Otras fuentes e importación

### CAP-037 · Página web por URL
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Webpage (Mac): Selection > Webpage y arrastrar la URL al ícono de la barra de menús o del Dock
- **Comportamiento:**
  - Captura una página web entera, de arriba abajo, a partir de su dirección, sin que el usuario tenga que desplazarse.
  - Formas de iniciarla:
    - Selección = Página web y **Capturar**: aparece un cuadro con el campo **URL**. Si el portapapeles contiene una dirección web, el campo llega relleno con ella (decisión de diseño). `Enter` captura y `Esc` cancela.
    - Arrastrar una dirección (el texto de la barra de direcciones, el ícono del sitio o un enlace) y soltarla sobre el panel de captura o el Lanzador de borde; en macOS, también sobre el ícono de la barra de menús o del Dock (CAP-005). La captura empieza sin pasar por el cuadro.
    - El comando `capture.webpage` con la URL como parámetro (CAP-008).
  - XcreenShot carga la página por su cuenta, sin abrirla en el navegador del usuario ni mostrar ninguna ventana, y entrega la imagen de toda su longitud, incluido lo que no cabría en pantalla.
  - Propiedades (engranaje):
    - **Ancho de la vista**: 375, 768, 1280 (por defecto), 1440 o 1920 px, o un valor propio entre 320 y 3840. Las páginas adaptables se dibujan como en una ventana de ese ancho.
    - **Escala**: 1× o 2×. Por defecto, la del monitor principal (decisión de diseño).
    - **Esperar tras cargar**: de 0 a 10 s; por defecto 1 s, para animaciones y contenido que llega tarde.
    - **Cargar el contenido diferido** (por defecto: activada): antes de capturar, recorre la página para que aparezcan las imágenes y los bloques que solo se cargan al desplazar.
  - Solo acepta direcciones `http` y `https`. Si se escribe sin esquema (`example.com`), se completa con `https://`.
  - La sesión que usa XcreenShot para cargar la página es privada y se vacía tras cada captura: no usa las cookies ni las sesiones iniciadas del navegador del usuario, ni guarda nada (decisión de diseño). Una página que exige iniciar sesión sale como la vería un visitante; para esas páginas, la ayuda recomienda el desplazamiento automático en el navegador del usuario (CAP-035).
  - El resultado puede diferir de lo que el usuario ve en su navegador (fuentes instaladas, extensiones, sesión iniciada); la ayuda lo explica.
  - Mientras carga, una barra de progreso muestra la dirección y el botón **Cancelar**. Límite de espera: 30 s.
    - Si la página no se puede abrir (sin conexión, dirección inexistente, tiempo agotado), no se crea captura y el aviso dice el motivo.
    - Si el servidor responde con una página de error (por ejemplo, 404), se captura igual y la notificación indica el código (decisión de diseño).
  - Una página más alta que el tamaño máximo de CAP-036 se corta en ese límite y se avisa.
  - Con Intervalo o Programada (CAP-049, CAP-050), la URL se pide una sola vez al iniciar y la página se vuelve a cargar en cada vuelta. Con Descartar si no hay cambios, solo se guarda una captura cuando la página cambió (decisión de diseño).
  - Los metadatos (CAP-056) guardan la URL final (tras las redirecciones) y el título de la página.
  - Es una de las pocas funciones que se conectan a la red, y solo cuando el usuario la pide (APP-054).
- **Criterios de aceptación:**
  - [ ] Con una página de prueba de 9000 px de alto, Ancho de la vista = 1280 y Escala = 1×, la imagen mide 1280 px de ancho y contiene la página entera, de la cabecera al pie.
  - [ ] Soltar sobre el panel de captura una URL arrastrada desde la barra de direcciones de un navegador inicia la captura sin más pasos, en Windows y en macOS.
  - [ ] Con Ancho de la vista = 375, una página adaptable se captura con su diseño de móvil.
  - [ ] Sin conexión de red, la captura falla con un aviso que lo explica y no se crea ninguna entrada en la biblioteca.
  - [ ] Con Cargar el contenido diferido activada, una página cuyas imágenes se cargan al desplazar aparece con todas ellas.
- **Mejora sobre Snagit:** existe también en Windows; se eligen el ancho y la escala; no depende del navegador que use el usuario.
- **Fuentes:** research/01-captura.md §1.7, §6.2 (casos no compatibles); research/05-plataformas-versiones.md §3 (Captura: página web completa arrastrando la URL)

### CAP-038 · Cámara web como imagen
- **Hito:** M3 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** Webcam (Mac): Selection = Webcam, selector Camera y botón Capture
- **Comportamiento:**
  - Con Selección = Cámara web, **Capturar** abre la ventana **Foto con la cámara**, con:
    - la imagen en vivo;
    - el selector **Cámara** (integradas, USB y las que ofrezca el sistema, como un teléfono usado como cámara);
    - el selector **Resolución** (por defecto, la mayor que ofrece la cámara);
    - el botón **Tomar foto** (`Espacio` o `Enter`).
  - Si hay Retardo (CAP-048), al pulsar Tomar foto aparece la cuenta atrás sobre la imagen en vivo y la foto se toma al llegar a cero.
  - La vista en vivo se muestra como un espejo, que es lo natural para encuadrarse. La foto se guarda sin invertir, como la verían otros. La casilla **Invertir la foto** la guarda como en el espejo (decisión de diseño).
  - Casilla **Seguir abierta** (por defecto: desactivada): la ventana no se cierra tras cada foto, para tomar varias seguidas. Cada foto es una captura aparte.
  - La foto sigue la ruta normal (CAP-054): efectos al capturar, portapapeles, biblioteca, editor y destinos. Los efectos de cámara del video (fondo, borde, forma) no se aplican; son de `VID-`.
  - XcreenShot recuerda la última cámara y la última resolución. Si la cámara recordada no está conectada, usa la predeterminada del sistema y lo indica.
  - Si no hay cámara, si otra app la está usando o si falta el permiso (APP-024 en macOS, APP-026 en Windows), la ventana lo explica con un enlace para resolverlo y no se crea captura.
  - `Esc` cierra la ventana sin capturar. Esta ventana nunca sale en otra captura (CAP-023).
- **Criterios de aceptación:**
  - [ ] Con una cámara de 1920×1080, Tomar foto produce una imagen de 1920×1080 px que se abre en el editor.
  - [ ] Con dos cámaras conectadas, el selector muestra las dos y la foto sale de la elegida.
  - [ ] Una foto de un texto impreso muestra el texto legible, sin invertir; con Invertir la foto, sale como en el espejo.
  - [ ] (macOS) Con el permiso de cámara denegado, la ventana muestra el aviso con Reparar permisos y no se crea ninguna captura.
- **Mejora sobre Snagit:** también en Windows; varias fotos seguidas sin volver a abrir la ventana.
- **Fuentes:** research/01-captura.md §5.8, §16 (punto 10); research/05-plataformas-versiones.md §3 (Captura: foto fija con la cámara)

### CAP-039 · Portapapeles como fuente
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Clipboard (Selection = Clipboard, Windows)
- **Comportamiento:**
  - Con Selección = Portapapeles, **Capturar** convierte lo que hay en el portapapeles en una captura, sin selección en pantalla. Sirve para que una imagen copiada en otra app pase por la ruta de captura: efectos, destinos, nombre de archivo y biblioteca.
  - Contenido que acepta, por orden de preferencia:
    1. **Imagen**, en cualquier formato que ofrezca el sistema. Conserva la transparencia si la trae.
    2. **Archivos de imagen** copiados en el Explorador o en el Finder: se tratan como en CAP-040, con una captura por archivo.
    3. **Texto** (decisión de diseño): se crea un documento con un objeto Texto editable (`EDT-`) que contiene **todo** el texto, con el estilo de texto del tema activo, ajuste de línea a 1000 px de ancho, margen de 20 px y lienzo blanco. Si el texto tiene formato (HTML o RTF), se usa su versión de texto plano.
  - Si el portapapeles está vacío o contiene otra cosa, un aviso lo dice ("El portapapeles no contiene una imagen ni texto") y no se crea captura.
  - Un texto que no cabe en el tamaño máximo (60 000 px de alto, como en CAP-036) se corta y se avisa.
  - Con Copiar al portapapeles activado, el resultado sustituye al contenido del portapapeles (CAP-046).
  - Metadatos (CAP-056): origen "Portapapeles" y, si el sistema lo indica, la aplicación que copió el contenido.
  - Con Intervalo (CAP-049), cada vuelta lee el portapapeles y solo crea una captura si el contenido cambió desde la vuelta anterior. Así sirve para recoger todo lo que se va copiando (decisión de diseño).
  - Se diferencia de **Nuevo desde el portapapeles** del editor (`EDT-`), que abre la imagen sin pasar por la ruta de captura.
- **Criterios de aceptación:**
  - [ ] Tras copiar una imagen en un navegador, Capturar con Selección = Portapapeles abre en el editor una imagen del mismo tamaño en píxeles.
  - [ ] Un PNG con transparencia copiado al portapapeles conserva la transparencia en la captura.
  - [ ] Tras copiar un texto de tres párrafos, la captura contiene un objeto Texto editable con los tres párrafos completos, incluida la última línea.
  - [ ] Con el portapapeles vacío aparece el aviso y no se crea ninguna entrada en la biblioteca.
  - [ ] Con Intervalo de 5 s durante 1 minuto en el que se copian 3 imágenes distintas, se crean exactamente 3 capturas.
- **Mejora sobre Snagit:** también en macOS; comportamiento definido para el texto y para el intervalo.
- **Fuentes:** research/01-captura.md §5.9, §4.1, §4.8, §16 (punto 3)

### CAP-040 · Archivo como fuente
- **Hito:** M3 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** File (Selection = File, Windows); entrada "archivo gráfico" de la API COM
- **Comportamiento:**
  - Con Selección = Archivo, **Capturar** abre el selector de archivos del sistema, que admite elegir varios. Cada archivo se convierte en una captura, en el orden de la lista, y sigue la ruta normal (CAP-054). Sirve, por ejemplo, para aplicar los mismos efectos y destinos a imágenes que ya existen.
  - Propiedad **Archivo fijo** (decisión de diseño): una ruta guardada, que se usa sin preguntar.
  - Con Intervalo o Programada (CAP-049, CAP-050), cada vuelta vuelve a leer un único archivo y solo crea una captura si cambió. Es el Archivo fijo o, si no lo hay, el que se elige una sola vez al iniciar (decisión de diseño).
  - Formatos: PNG, JPG, GIF (primer fotograma), BMP, TIFF (primera página), WebP, HEIC/HEIF (si el sistema lo admite), ICO (el tamaño mayor) y SVG (convertido a píxeles a su tamaño propio, con un máximo de 8192 px de lado).
  - Un PDF elegido aquí abre Importar PDF (CAP-041). Un `.xshot` se abre en el editor como documento (`EDT-`), sin crear una captura nueva.
  - Los archivos no compatibles o dañados se omiten; al terminar, un aviso los lista con el motivo. El resto se importa.
  - El archivo original nunca se modifica ni se mueve.
  - Soltar imágenes sobre el panel de captura o el Lanzador de borde (y, en macOS, sobre el ícono de la barra de menús) hace lo mismo (CAP-005).
  - Metadatos (CAP-056): origen "Archivo", ruta y fecha de modificación del original y, si la imagen la trae, su fecha de toma.
- **Criterios de aceptación:**
  - [ ] Elegir un JPG de 4000×3000 px con el efecto Borde activo crea una captura de 4000×3000 px más el borde, y el JPG original queda idéntico (misma suma de comprobación).
  - [ ] Elegir 3 imágenes crea 3 capturas en la biblioteca, en el orden elegido.
  - [ ] Si uno de los archivos está dañado, las demás capturas se crean y el aviso nombra el archivo dañado.
  - [ ] Con Archivo fijo e Intervalo, sobrescribir el archivo con otra imagen produce una captura nueva en la vuelta siguiente; si el archivo no cambia, no se crea ninguna.
- **Mejora sobre Snagit:** también en macOS; varios archivos a la vez.
- **Fuentes:** research/01-captura.md §5.10, §1.9 (entradas), §4.1, §16 (punto 3)

### CAP-041 · Importar PDF multipágina
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Snagit Printer / Print to Snagit (impresora virtual de Windows), sustituida según SPEC D-03
- **Comportamiento:**
  - Convierte las páginas de un PDF en capturas. Sustituye a la impresora virtual: cualquier programa que imprime puede guardar un PDF con la función del sistema, y ese PDF se importa aquí.
  - Formas de iniciarlo:
    - Selección = PDF y **Capturar**, que abre el selector de archivos;
    - soltar un PDF sobre el panel de captura, el Lanzador de borde o, en macOS, el ícono de la barra de menús o del Dock;
    - **Abrir con > XcreenShot** desde el sistema. XcreenShot se declara capaz de abrir PDF, pero nunca se pone como app predeterminada para ellos (decisión de diseño);
    - el comando `capture.pdf`;
    - la carpeta de recepción (CAP-043).
  - En macOS, el menú **PDF** del diálogo de impresión del sistema ofrece **Enviar a XcreenShot**, que lleva el documento a este mismo cuadro sin guardar antes un archivo (decisión de diseño).
  - Cuadro **Importar PDF**:
    - miniaturas de todas las páginas, todas marcadas; se pueden desmarcar o escribir un rango (`1-3, 7`);
    - **Resolución**: 72, 96, 150 (por defecto), 200 o 300 ppp;
    - **Resultado**: **Una captura por página** (por defecto) o **Todas las páginas en una imagen**, una debajo de otra, con 20 px de separación sobre fondo gris claro (decisión de diseño);
    - **Fondo de las páginas**: blanco (por defecto) o transparente, para las páginas que no tienen fondo propio.
  - Las páginas se ven como al imprimirlas: texto, imágenes, comentarios y campos de formulario rellenados.
  - Si el PDF tiene contraseña, se pide. Si es incorrecta, se puede repetir o cancelar.
  - Límites (decisión de diseño): 200 páginas por importación; lado mayor de cada imagen, 60 000 px, como en CAP-036. En el modo de una sola imagen, si se pasa del límite, el resultado se reparte en varias imágenes seguidas y se avisa.
  - La importación muestra una barra de progreso con **Cancelar** y se hace en segundo plano. Cancelar no deja ninguna captura a medias.
  - Cada captura guarda en sus metadatos (CAP-056) el nombre del PDF, el número de página y el texto de la página, si lo tiene, para que la búsqueda de la biblioteca lo encuentre (`LIB-`). La imagen no conserva el texto seleccionable; para eso está Extraer texto (`SMT-`).
  - Con Abrir en = Editor, se abre la primera página y las demás quedan en la bandeja reciente (`TRY-`) (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Un PDF de 5 páginas A4 importado a 150 ppp crea 5 capturas de 1240×1754 px, en el orden de las páginas.
  - [ ] Con Todas las páginas en una imagen, el mismo PDF crea una sola imagen con las 5 páginas una debajo de otra.
  - [ ] Con el rango `2-3`, solo se crean las capturas de las páginas 2 y 3.
  - [ ] Un PDF con contraseña la pide; con la correcta se importa y, si se cancela, no se crea nada.
  - [ ] (macOS) Desde el diálogo de impresión de cualquier app, PDF > Enviar a XcreenShot abre el cuadro Importar PDF con el documento.
- **Notas de plataforma:** en Windows no se puede aparecer en el diálogo de impresión sin instalar una impresora virtual, que D-03 descarta. Allí se imprime con la impresora PDF del sistema ("Microsoft Print to PDF") y se importa el archivo, o se guarda en la carpeta de recepción (CAP-043), que lo importa sola.
- **Mejora sobre Snagit:** funciona en ambas plataformas y también en Windows en ARM, donde la impresora virtual de Snagit no funciona; se eligen las páginas y la resolución.
- **Fuentes:** research/01-captura.md §1.8, §6.2 (PDF en visores); research/05-plataformas-versiones.md §3 (Captura: impresora virtual), §4 (ARM64); SPEC D-03

### CAP-042 · Capturas del sistema
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** "Open screenshots copied from system tools" (Windows); "Open system screenshots and videos in Snagit" con "Keep original file" (Mac); importación posterior de las capturas del sistema hechas durante una grabación (Mac)
- **Comportamiento:**
  - Preferencia **Abrir en XcreenShot las capturas del sistema** (Preferencias > Captura; por defecto: desactivada). Con ella activada, cada captura nueva que hace la herramienta de capturas del propio sistema llega a XcreenShot como una captura más: entra en la biblioteca y en la bandeja reciente y sigue la opción **Al recibirla**: **Abrir en el editor** (por defecto) o **Solo guardar en la biblioteca**, con una notificación.
  - Qué se detecta:
    - **Windows:** las capturas de la Herramienta Recortes, incluidas las de `Win+Shift+S` y las de `PrtScn` cuando el sistema asigna esa tecla a Recortes, tanto si solo van al portapapeles como si además se guardan en la carpeta de capturas del sistema; y sus grabaciones de pantalla.
    - **macOS:** las capturas y grabaciones de la herramienta del sistema (`Cmd+Shift+3`, `4` y `5`) que se guardan como archivo, en la ubicación que el usuario tenga elegida en el sistema. Las que van solo al portapapeles no se detectan (decisión de diseño).
  - Las imágenes copiadas desde otras apps nunca se importan: solo las de la herramienta del sistema.
  - Una misma captura del sistema crea **una sola** captura en XcreenShot, aunque aparezca a la vez en el portapapeles y como archivo.
  - Subopción **Conservar el archivo original** (por defecto: activada). Si se desactiva, el archivo se traslada a la biblioteca y desaparece de donde estaba (por ejemplo, del Escritorio). No aplica a las capturas que solo estaban en el portapapeles.
  - Las grabaciones del sistema entran en la biblioteca como video (`VID-`, `LIB-`), sin abrirse en el editor de imágenes.
  - Si llega una captura del sistema mientras XcreenShot captura o graba, se pone en cola y se importa al terminar, sin interrumpir nada.
  - Solo se importan las capturas hechas con la opción activada; las anteriores, no.
  - Metadatos (CAP-056): origen "Herramienta del sistema" y la fecha del archivo.
  - XcreenShot no cambia ni intercepta los atajos del sistema.
- **Criterios de aceptación:**
  - [ ] (Windows) Con la opción activada, una captura con `Win+Shift+S` se abre una sola vez en el editor de XcreenShot.
  - [ ] (macOS) Con la opción activada, `Cmd+Shift+4` abre la captura en el editor. Con Conservar el archivo original, el archivo sigue en el Escritorio; sin ella, desaparece del Escritorio y está en la biblioteca.
  - [ ] Una captura del sistema hecha durante una grabación de XcreenShot aparece en la biblioteca al detener la grabación, y la grabación no se interrumpe.
  - [ ] Con la opción desactivada, las capturas del sistema no llegan a XcreenShot.
- **Notas de plataforma:** si en Windows la Herramienta Recortes no guarda archivos automáticamente, solo se detecta la copia del portapapeles y Conservar el archivo original no aplica.
- **Mejora sobre Snagit:** el mismo comportamiento en ambas plataformas, conservar el original también en Windows e importación de las grabaciones del sistema.
- **Fuentes:** research/01-captura.md §1.6, §11.1, §11.2; research/05-plataformas-versiones.md §3 (Captura: abrir capturas del sistema), §1B (2025.1.0)

### CAP-043 · AirDrop y carpeta de recepción
- **Hito:** M3 · **Prioridad:** Could
- **Plataformas:** Ambas (AirDrop, solo macOS: es una función de los sistemas de Apple que no existe en Windows)
- **Paridad con Snagit:** "Open AirDrop images and videos in Snagit" (Mac)
- **Comportamiento:**
  - **AirDrop (macOS).** Preferencia **Abrir en XcreenShot lo recibido por AirDrop** (Preferencias > Captura; por defecto: desactivada). Las imágenes y los videos que llegan al Mac por AirDrop se abren en XcreenShot: las imágenes, como captura en el editor; los videos, en la biblioteca (`VID-`, `LIB-`). El archivo recibido se queda donde lo deja el sistema. Los demás tipos de archivo no se tocan.
  - **Carpeta de recepción (ambas plataformas; decisión de diseño).** Preferencia **Carpeta de recepción** (Preferencias > Captura, junto a la de AirDrop; por defecto: ninguna). XcreenShot vigila esa carpeta y convierte en captura cada archivo nuevo que aparece en ella:
    - las imágenes, como en CAP-040;
    - los PDF, como en CAP-041 pero sin preguntar: todas las páginas, una captura por página, a 150 ppp. Si un PDF tiene contraseña, una notificación la pide;
    - los videos, a la biblioteca (`VID-`, `LIB-`).
  - Un archivo se procesa una sola vez, cuando ha terminado de escribirse (su tamaño no cambia durante 2 s). Después se mueve a la subcarpeta **Importados** de la misma carpeta; la opción **Dejar los archivos donde están** (por defecto: desactivada) lo evita. Las subcarpetas no se vigilan.
  - Por defecto, lo que llega a la carpeta solo se guarda en la biblioteca y una notificación lo indica (decisión de diseño); la opción **Abrir en el editor** lo cambia.
  - Sirve para todo lo que deja archivos en una carpeta: en Windows, el uso compartido con dispositivos cercanos (eligiendo esa carpeta en el sistema), las apps que bajan las fotos del móvil o una carpeta sincronizada; en ambas plataformas, la impresión a PDF (CAP-041).
- **Criterios de aceptación:**
  - [ ] (macOS) Con la opción de AirDrop activada, una imagen enviada por AirDrop desde un iPhone se abre en el editor de XcreenShot.
  - [ ] Con una carpeta de recepción elegida, copiar en ella 2 PNG y un PDF de 3 páginas crea 5 capturas, y los 3 archivos pasan a Importados.
  - [ ] Un archivo grande que tarda en copiarse se importa una sola vez y completo.
  - [ ] Con Carpeta de recepción = ninguna, no se importa ningún archivo.
- **Notas de plataforma:** en Windows, el equivalente práctico de AirDrop es la carpeta de recepción.
- **Mejora sobre Snagit:** carpeta de recepción en ambas plataformas, también para PDF.
- **Fuentes:** research/01-captura.md §1.6, §11.2; research/05-plataformas-versiones.md §3 (Captura: AirDrop), §1B (2024.3.0)

### CAP-044 · iPhone o iPad
- **Hito:** M3 · **Prioridad:** Could
- **Plataformas:** macOS (la Cámara de Continuidad es una función del sistema entre un Mac y un iPhone o iPad; Windows no la tiene)
- **Paridad con Snagit:** Import from iPhone or iPad (Continuity Camera, Mac)
- **Comportamiento:**
  - Entrada **Importar desde iPhone o iPad ▸** en el menú Más del panel de captura (CAP-001) y en el menú Archivo del editor (`EDT-`). Lista los dispositivos cercanos que ofrece el sistema y, para cada uno:
    - **Tomar foto**: el iPhone o iPad abre la cámara y, al aceptar la foto, esta llega a XcreenShot;
    - **Escanear documentos**: el dispositivo escanea una o varias páginas, y cada página es una captura.
  - Lo recibido sigue la ruta normal (CAP-054), con los efectos y los destinos de la pestaña Imagen.
  - Si no se cumplen los requisitos del sistema (misma cuenta de Apple, Wi-Fi y Bluetooth activados), la entrada aparece desactivada y su tooltip explica por qué.
  - Metadatos (CAP-056): origen "iPhone o iPad" y el nombre del dispositivo.
  - Comando `capture.from_device` (solo macOS; en Windows responde `not_supported_on_platform`, APP-027).
- **Criterios de aceptación:**
  - [ ] Con un iPhone compatible cerca, Tomar foto deja la foto en el editor de XcreenShot.
  - [ ] Un escaneo de 3 páginas crea 3 capturas en la biblioteca.
  - [ ] Sin dispositivos disponibles, la entrada aparece desactivada y su tooltip explica el motivo.
- **Notas de plataforma:** en Windows la entrada no aparece. Alternativas: usar el teléfono como cámara, si el sistema lo permite, y tomar la foto con CAP-038; o enviar las fotos a la carpeta de recepción (CAP-043).
- **Fuentes:** research/01-captura.md §5.13; research/05-plataformas-versiones.md §3 (Captura: Continuity Camera), §1B (2025.2.0)

---

## 6. Opciones al capturar y salida

### CAP-045 · Abrir en
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Open In (Snagit Editor / None), antes Preview in Editor
- **Comportamiento:**
  - Control **Abrir en** de las pestañas Universal e Imagen. Valores:
    - **Editor** (por defecto): la captura se abre en el editor (`EDT-`), que pasa al frente.
    - **Anotación rápida**: la captura se anota en el sitio y no abre el editor (CAP-057). Este valor aparece cuando existe esa función (M3) y, mientras sea función en prueba, solo si está activada (APP-034).
    - **Ninguno**: no se abre ninguna ventana. La captura va solo a la biblioteca, al portapapeles y a los destinos, según sus ajustes (CAP-054).
  - Con **Ninguno**, la confirmación de CAP-055 avisa de que la captura está hecha y permite abrirla con un clic.
  - Si con **Ninguno** la captura no fuera a ningún sitio (sin portapapeles, sin destinos y con el autoguardado de la biblioteca desactivado, `LIB-`), se guarda igualmente en la biblioteca y una pista lo explica la primera vez (decisión de diseño). Una captura nunca se pierde sin que el usuario lo haya decidido.
  - El editor se abre en el estado en que estaba (posición, tamaño y monitor). Si estaba minimizado, se restaura. El tiempo de apertura lo fija APP-049; la opción de dejar el editor cargado en segundo plano es de APP-004.
  - Con Intervalo o Programada (CAP-049, CAP-050), el editor no pasa al frente con cada captura, para no interrumpir. Las capturas llegan a la biblioteca y a la bandeja reciente (decisión de diseño).
  - Varias capturas seguidas, con Abrir en = Ninguno, quedan en la biblioteca y en la bandeja en el mismo orden en que se hicieron.
- **Criterios de aceptación:**
  - [ ] Con Abrir en = Editor, una captura de región aparece en el editor, en primer plano.
  - [ ] Con Abrir en = Ninguno y Copiar al portapapeles activado, no se abre ninguna ventana, la imagen está en el portapapeles y la captura está en la biblioteca.
  - [ ] Con Abrir en = Ninguno, sin portapapeles ni destinos y con el autoguardado desactivado, la captura aparece igualmente en la biblioteca.
  - [ ] Cinco capturas rápidas con Abrir en = Ninguno aparecen en la bandeja reciente en el orden en que se hicieron.
- **Fuentes:** research/01-captura.md §4.5, §2.2, §12.1; research/05-plataformas-versiones.md §1A (2026.1.0), §1B (2025.3.0)

### CAP-046 · Copiar al portapapeles
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Copy to Clipboard
- **Comportamiento:**
  - Casilla **Copiar al portapapeles** de las pestañas Universal e Imagen (por defecto: activada, CAP-003). Al terminar la captura, la imagen pasa al portapapeles del sistema antes de abrir el editor o enviar a los destinos.
  - Lo que se copia es el resultado final: con los efectos al capturar (CAP-051) y con el cursor (CAP-047) y las áreas múltiples (CAP-033) ya aplanados.
  - Se copia en dos variantes a la vez, para que cada app pegue la que entiende:
    - PNG con transparencia (formas de mano alzada, esquinas de ventana, menús);
    - mapa de bits estándar del sistema, sin transparencia: las zonas transparentes se rellenan de blanco.
  - Tamaño en píxeles físicos, el mismo que el de la captura. En pantallas de alta densidad, el cursor se copia con su tamaño proporcional.
  - Junto con Abrir en = Ninguno es el flujo "capturar y pegar": la notificación "Captura copiada" la define CAP-055 (APP-016).
  - Si el portapapeles está bloqueado por otra app, XcreenShot lo reintenta durante 1 s. Si no lo consigue, avisa con una notificación de la categoría Problemas. La captura sigue en la biblioteca y los demás pasos de CAP-054 continúan.
  - Con Intervalo, cada captura sustituye en el portapapeles a la anterior.
- **Criterios de aceptación:**
  - [ ] Con la casilla activada y Abrir en = Ninguno, pegar en otra app una captura de 800×600 px da una imagen de 800×600 px.
  - [ ] Una captura de mano alzada pegada en una app que acepta PNG conserva la forma, y en una app sin transparencia sale sobre blanco.
  - [ ] Con el efecto Borde activo, la imagen pegada incluye el borde.
  - [ ] (macOS) En una pantalla Retina, el cursor pegado tiene el mismo tamaño relativo que en pantalla.
- **Fuentes:** research/01-captura.md §4.6, §2.2, §4.3

### CAP-047 · Capturar cursor
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Capture Cursor (el cursor queda como objeto vectorial editable)
- **Comportamiento:**
  - Casilla **Capturar cursor** (por defecto: desactivada; activada en el Lanzador de borde, CAP-004).
  - Activada, el puntero se añade al documento como un **objeto Cursor** aparte (`EDT-`), no mezclado con los píxeles. Tiene la forma real que tenía en ese momento (flecha, mano, barra de texto, cursores propios de la app), su tamaño en pantalla y su posición.
  - La imagen base nunca contiene el puntero. Al quitar o mover el objeto Cursor, debajo se ve el contenido real, sin restos.
  - El objeto Cursor tiene fondo transparente: solo contiene el dibujo del puntero, sin píxeles del fondo.
  - Momento de la posición (decisión de diseño):
    - en las capturas con selección en pantalla, la del momento en que empezó la captura (al pulsar el atajo o al terminar el retardo), no la de la cruceta;
    - en las capturas sin selección (pantalla completa, posición fija, ventana activa), la del momento de tomar la imagen.
  - Si el puntero estaba fuera del área capturada, no se añade objeto.
  - Desactivada, el puntero no aparece nunca, tampoco el puntero ampliado de las opciones de accesibilidad del sistema.
  - Al copiar al portapapeles o enviar a un destino sin pasar por el editor, el cursor se aplana en la imagen (CAP-046).
  - El cambio de la imagen del cursor por otra (sellos de cursor) es del editor (`EDT-`).
  - No se aplica al desplazamiento, ni a las capturas sin pantalla (página web, cámara, portapapeles, archivo, PDF) (CAP-002). En el Selector de ventanas, ver CAP-030.
- **Criterios de aceptación:**
  - [ ] Con la casilla activada, la captura abierta en el editor tiene un objeto Cursor que se puede mover, cambiar de tamaño y eliminar. Al eliminarlo, no queda rastro del puntero en la imagen.
  - [ ] Con el puntero ampliado al máximo en la accesibilidad del sistema y la casilla desactivada, la captura no contiene el puntero.
  - [ ] Si el puntero está sobre un campo de texto, el objeto tiene la forma de barra de texto.
  - [ ] En una pantalla al 200 %, el objeto Cursor tiene el mismo tamaño relativo que en pantalla y está en la posición correcta.
- **Fuentes:** research/01-captura.md §4.7, §2.2, §1.3, §1.9 (IncludeCursor)

### CAP-048 · Retardo
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Time Delay > Delay; "5 Second Delay" del All-in-One; cuenta atrás cancelable
- **Comportamiento:**
  - **Tiempo = Retardo** en la pestaña Imagen, con un campo de segundos (1 a 60; por defecto 5). En la pestaña Universal es la casilla fija de 5 s (CAP-010).
  - Al pulsar Capturar empieza una cuenta atrás. Al terminar:
    - en los modos interactivos, la pantalla se congela y aparece la cruceta (CAP-013);
    - en los demás (pantalla completa, posición fija, ventana activa), la captura se hace en ese momento.
  - La cuenta atrás es un contador pequeño en la esquina inferior derecha del monitor bajo el puntero. Tiene un botón **Cancelar**, y `Esc` también cancela.
  - La cuenta atrás **no toma el foco del teclado ni del ratón** (decisión de diseño): mientras corre, el usuario puede abrir menús, desplegar listas o dejar el puntero sobre un elemento para que aparezca su tooltip, y nada se cierra al terminar.
  - El contador nunca sale en la captura (CAP-023).
  - Propiedad **Mostrar la cuenta atrás** (por defecto: activada). Si se desactiva, no hay contador visible; para cancelar se usa `Esc` o el menú del ícono.
  - El retardo se guarda con la captura para Repetir última captura (CAP-007) y en los presets (CAP-059). Los comandos lo reciben como parámetro (CAP-008).
  - En la captura de menú, si Tiempo = Ninguno, se usa una cuenta atrás de 5 s (CAP-031).
- **Criterios de aceptación:**
  - [ ] Con Retardo = 3 s, la cruceta aparece 3 s (±0,3 s) después de pulsar Capturar.
  - [ ] Si se abre un menú durante la cuenta atrás, sigue abierto cuando termina y sale en la captura.
  - [ ] `Esc` durante la cuenta atrás cancela: no hay cruceta ni ninguna captura nueva.
  - [ ] El contador no aparece en una captura de pantalla completa con retardo.
- **Fuentes:** research/01-captura.md §4.8, §2.2, §1.9 (Delay)

### CAP-049 · Intervalo
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Time Delay > Interval (captura a intervalos), con descarte de capturas idénticas
- **Comportamiento:**
  - **Tiempo = Intervalo**, con estos campos:
    - **Cada**: de 3 s a 24 h; por defecto 60 s. Un valor menor de 3 s se sube a 3 s y el campo lo avisa.
    - **Detener** (decisión de diseño): **Manualmente** (por defecto), **Tras N capturas**, **Tras un tiempo** o **A una hora**.
    - **Descartar si no hay cambios** (por defecto: activada): no guarda una captura igual a la última guardada.
    - **Sensibilidad** (decisión de diseño): **Exacta** (por defecto; cualquier píxel distinto cuenta como cambio) o **Ignorar cambios pequeños** (menos del 0,5 % de los píxeles; por ejemplo, el reloj de la barra de tareas o un cursor parpadeante).
  - El botón **Capturar** pasa a llamarse **Iniciar**. Al pulsarlo:
    - en Región, el usuario elige el área una sola vez y se reutiliza en todas las vueltas;
    - en Ventana, elige la ventana una vez y se captura cada vez en su posición actual;
    - en Pantalla completa, Menú, Página web (CAP-037), Portapapeles (CAP-039) y Archivo (CAP-040), no hay selección.
  - La primera captura se hace al iniciar y las siguientes, cada intervalo.
  - Mientras está activo:
    - el botón pasa a **Detener**;
    - el panel muestra cuántas capturas lleva y la hora de la siguiente;
    - el ícono muestra el reloj (APP-002);
    - el menú del ícono ofrece **Detener captura: <nombre>** (CAP-005), donde el nombre es el del preset o "Intervalo".
  - Las capturas no abren el editor (CAP-045). Siguen el resto de la ruta: biblioteca, portapapeles, destinos y efectos.
  - Cada serie lleva un identificador común y un número de orden en los metadatos (CAP-056), para que la biblioteca las agrupe (`LIB-`) (decisión de diseño).
  - Casos límite (decisión de diseño):
    - si en el momento de una vuelta hay otra captura en curso, o la sesión está bloqueada, esa vuelta se omite;
    - si el equipo se suspende, no se recuperan las vueltas perdidas: la serie sigue en la siguiente vuelta tras reanudar;
    - si la ventana elegida se cierra, o el área deja de existir porque se desconectó su monitor, la serie se detiene y una notificación lo explica.
  - Puede haber varias series activas a la vez (por ejemplo, de presets distintos), cada una con su entrada en el menú del ícono. Las series no sobreviven al cierre de la app; al salir, XcreenShot avisa (APP-004).
  - Un preset con Intervalo funciona como interruptor: su atajo inicia y detiene la serie (CAP-062).
- **Criterios de aceptación:**
  - [ ] Con Cada = 5 s durante 60 s sobre un contenido que cambia, se crean 12 capturas (±1), numeradas en orden.
  - [ ] Con Descartar si no hay cambios y la pantalla quieta, en 60 s se crea una sola captura.
  - [ ] **Detener captura** en el menú del ícono detiene la serie al momento, y no se crea ninguna captura después.
  - [ ] Si se escribe Cada = 1 s, el campo queda en 3 s y lo avisa.
  - [ ] Si se cierra la ventana elegida, la serie se detiene y aparece la notificación.
- **Mejora sobre Snagit:** condiciones de parada, sensibilidad ajustable y agrupación de la serie en la biblioteca.
- **Fuentes:** research/01-captura.md §4.8, §1.1; research/05-plataformas-versiones.md §1A (2026.3.1), §1B (2026.1.0), §3 (Captura: intervalo y programada)

### CAP-050 · Programada
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Time Delay > Schedule
- **Comportamiento:**
  - **Tiempo = Programada**, con estos campos:
    - **Fecha** y **Hora**, en la hora local, con precisión de minutos;
    - **Repetir** (decisión de diseño): **Una vez** (por defecto), **Cada día**, **Días laborables** o **Cada semana**.
  - El botón **Capturar** pasa a **Programar**. Al pulsarlo:
    - en Región se elige el área y en Ventana, la ventana (CAP-002);
    - en los demás modos no hay selección.
  - Una vez programada, el botón pasa a **Cancelar programación**. El ícono muestra el reloj y el menú del ícono ofrece **Detener captura: <nombre> (próxima: <fecha y hora>)** (CAP-005).
  - La captura se hace exactamente al empezar el minuto elegido (segundo 0), aunque falte menos de un minuto para esa hora.
  - Si la hora elegida ya pasó, se rechaza con un aviso.
  - Las capturas programadas **sí** se conservan al cerrar y volver a abrir la app (decisión de diseño). Al arrancar, XcreenShot recupera las pendientes.
  - Casos límite (decisión de diseño):
    - si a esa hora la app está cerrada o el equipo dormido, XcreenShot no lo despierta. Si la app vuelve a estar activa en los 5 minutos siguientes, la captura se hace en ese momento y sus metadatos indican el retraso; si no, se marca como omitida y una notificación lo dice;
    - si a esa hora la ventana elegida no existe, la captura se omite y se avisa;
    - si la sesión está bloqueada, la captura se omite y se avisa.
  - Las capturas siguen la misma ruta que las de intervalo: no abren el editor (CAP-045).
  - Un preset con Programada funciona como interruptor: su atajo programa o cancela (CAP-062).
- **Criterios de aceptación:**
  - [ ] Programada para dentro de 40 s a las 10:15, la captura se hace a las 10:15:00 (±1 s).
  - [ ] Una captura programada sigue pendiente tras salir y volver a abrir XcreenShot, y se hace a su hora.
  - [ ] Con Repetir = Cada día, después de la primera captura, el menú del ícono muestra como próxima la misma hora del día siguiente.
  - [ ] Si el equipo está dormido a la hora programada y se despierta 10 minutos después, no se crea captura y aparece la notificación de captura omitida.
- **Mejora sobre Snagit:** repetición diaria o semanal, y conservación de las capturas programadas entre arranques.
- **Fuentes:** research/01-captura.md §4.8; research/05-plataformas-versiones.md §1B (2026.1.0), §3 (Captura: intervalo y programada)

### CAP-051 · Efectos al capturar
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Effects del Image tab (Capture Info, Border, Corner Rounding, Scale Image; escalar y girar como efectos de preset)
- **Comportamiento:**
  - El control **Efectos** de la pestaña Imagen muestra la lista de efectos que se aplican solos a cada captura, en orden. Acciones:
    - **Añadir efecto**;
    - el engranaje de cada efecto, para sus opciones;
    - **Quitar** cada efecto;
    - arrastrar para cambiar el orden;
    - **Ninguno**, que vacía la lista.
  - Efectos disponibles: todos los efectos de imagen del editor que no necesitan que el usuario haga nada sobre la imagen (decisión de diseño). Por ejemplo:
    - Borde;
    - Sombra;
    - Bordes decorativos;
    - Perspectiva;
    - Esquinas redondeadas (con su radio);
    - Escalar imagen (porcentaje o lado mayor en píxeles);
    - Girar (90°, 180° o 270°);
    - filtros de color;
    - Marca de agua;
    - Detalles de captura;
    - los **Efectos guardados**.
  - La definición y las opciones de cada efecto son de `EDT-`; aquí solo se elige cuáles se aplican al capturar.
  - Los efectos llegan al documento como efectos **editables**, no aplanados: en el editor se pueden cambiar o quitar (decisión de diseño).
  - Al portapapeles, a los archivos y a los destinos van ya aplicados (CAP-046, CAP-052).
  - Se aplican sobre la imagen completa, con los objetos de la captura (cursor y áreas): Escalar imagen, por ejemplo, escala también el cursor.
  - **Detalles de captura** puede tener la opción **Pedir texto al capturar**. Con ella, al terminar cada captura aparece un cuadro breve para escribir un texto que el efecto muestra junto a los metadatos (CAP-056). `Esc` deja el texto vacío y no cancela la captura.
  - Los modos que no producen una imagen (Extraer texto, `SMT-`) desactivan el control.
  - La pestaña Universal no tiene efectos (CAP-010).
- **Criterios de aceptación:**
  - [ ] Con Borde (4 px, negro) y Sombra, la captura se abre en el editor con los dos efectos aplicados, y se puede quitar cualquiera de ellos.
  - [ ] Con Escalar imagen al 50 % sobre una captura de 3840×2160 px, la imagen del portapapeles mide 1920×1080 px.
  - [ ] Con Detalles de captura y Pedir texto al capturar, aparece el cuadro de texto, y lo escrito aparece en la imagen.
  - [ ] **Ninguno** vacía la lista, y la siguiente captura sale sin efectos.
- **Mejora sobre Snagit:** cualquier efecto no interactivo, en el orden que elija el usuario y editable después.
- **Fuentes:** research/01-captura.md §4.3, §12.2, §16 (punto 9); research/05-plataformas-versiones.md §1B (2025.2.1), §3 (Editor: esquinas redondeadas al capturar)

### CAP-052 · Compartir al capturar
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Share del Image tab y del All-in-One (uno o varios destinos; envío al pulsar Finish en el editor o directo si no hay vista previa)
- **Comportamiento:**
  - El control **Compartir** de las pestañas Universal e Imagen permite elegir **ninguno, uno o varios destinos**, incluidas las instancias con nombre. Por defecto: ninguno. Los destinos, sus opciones y su forma de enviar los define `SHR-`.
  - La pestaña Imagen solo lista destinos que aceptan imágenes, y la Universal, solo los que aceptan imagen y video (CAP-010).
  - El destino Portapapeles no aparece en esta lista, porque ya existe la casilla Copiar al portapapeles (CAP-046) (decisión de diseño).
  - Con **Abrir en = Editor**:
    - no se envía nada al momento. El editor muestra una franja "Pendiente de enviar a: <destinos>" con **Enviar** (`Ctrl+Enter`/`Cmd+Enter`) y **No enviar** (decisión de diseño);
    - si se cierra el documento sin enviar, no se envía nada y la captura queda en la biblioteca, desde donde se puede compartir después (`SHR-`).
  - Con **Abrir en = Ninguno** o en Intervalo y Programada: se envía al momento, en segundo plano, a cada destino en el orden de la lista.
  - Cada destino se envía por separado: si uno falla, los demás siguen, y la notificación de `SHR-` dice cuál falló y ofrece **Reintentar**.
  - Los destinos que necesitan al usuario (por ejemplo, un correo que se abre para escribir o un archivo con Preguntar nombre) muestran su ventana igualmente.
  - Destino **Archivo**: sus opciones están en CAP-053.
  - Hasta que `SHR-` entregue el resto de destinos (M6), solo aparecen los que ya existen.
  - Las capturas lanzadas desde CLI o MCP siguen las reglas de destinos en línea de APP-032.
- **Criterios de aceptación:**
  - [ ] Con Archivo y otro destino local elegidos y Abrir en = Ninguno, una captura escribe el archivo y llega al otro destino sin abrir ninguna ventana.
  - [ ] Con Abrir en = Editor, no se envía nada hasta pulsar **Enviar**; con **No enviar**, no se envía nada.
  - [ ] Si un destino falla (por ejemplo, una carpeta sin permiso de escritura), el otro destino recibe la captura y la notificación nombra el que falló.
  - [ ] La lista Compartir de la pestaña Imagen no muestra destinos que solo aceptan video.
- **Fuentes:** research/01-captura.md §4.4, §2.2, §4.5

### CAP-053 · Nombre, formato y carpeta del destino Archivo
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** opciones del destino File (nombre, formato y carpeta); nombres automáticos con prefijo y contador de la API COM; "Save As Preferences" (Windows) y "Default sharing format" (Mac)
- **Comportamiento:**
  - El engranaje del destino **Archivo** abre sus opciones. Los presets las guardan (CAP-059) y el CLI las acepta (`--out` con patrón, CAP-008).
  - **Carpeta** (por defecto: la carpeta Imágenes del usuario, en una subcarpeta `XcreenShot`; decisión de diseño).
  - **Prefijo**: texto libre (por defecto `XcreenShot`).
  - **Nombre**: un patrón con texto y marcadores entre llaves (decisión de diseño).
    - Por defecto: `{prefix}_{date}_{time}`, que da por ejemplo `XcreenShot_2026-10-12_14-05-33.png`.
    - Los marcadores se escriben siempre en inglés, como los demás identificadores de máquina (APP-013), para que el patrón sea el mismo en cualquier idioma. Un botón **Insertar** los lista con su explicación en el idioma de la interfaz.
    - Debajo del campo, una vista previa muestra el nombre que saldría con la captura actual.

    | Marcador | Resultado | Ejemplo |
    |---|---|---|
    | `{prefix}` | El texto del campo Prefijo | `XcreenShot` |
    | `{date}` | Fecha local, AAAA-MM-DD | `2026-10-12` |
    | `{date:compact}` | Fecha local, AAAAMMDD | `20261012` |
    | `{time}` | Hora local, HH-MM-SS (24 h) | `14-05-33` |
    | `{time:compact}` | Hora local, HHMMSS | `140533` |
    | `{app}` | Nombre de la aplicación de origen (CAP-056) | `Excel` |
    | `{title}` | Título de la ventana o de la página web | `Informe Q3` |
    | `{site}` | Dominio de la página, si viene de un navegador | `example.com` |
    | `{mode}` | Modo de captura, en inglés | `region` |
    | `{width}`, `{height}` | Tamaño en píxeles | `1920`, `1080` |
    | `{user}` | Usuario del sistema | `carlos` |
    | `{counter}` o `{counter:N}` | Contador con N dígitos, de 1 a 9 (por defecto 4) | `0007` |

  - **Contador:**
    - empieza en 1 y avanza con cada archivo guardado;
    - hay un contador por cada destino Archivo y por cada formato;
    - el campo **Siguiente número** permite cambiarlo o reiniciarlo;
    - si el nombre resultante ya existe, se salta al siguiente número libre.
  - **Limpieza del nombre**, igual en Windows y macOS, para que un patrón dé el mismo nombre en ambos:
    - los caracteres que no valen en alguna de las dos plataformas (`\ / : * ? " < > |` y los de control) que vengan de un marcador se sustituyen por `-`;
    - se quitan los espacios y puntos del final;
    - un nombre reservado de Windows (por ejemplo `CON`) recibe un `_` al final;
    - el nombre se limita a 200 caracteres;
    - un marcador vacío (por ejemplo, `{site}` fuera de un navegador) se elimina, y los separadores que quedan repetidos se reducen a uno.
  - Una `/` escrita en el patrón crea subcarpetas: `{date}/{prefix}_{time}` guarda cada día en su carpeta.
  - **Formato**: PNG (por defecto), JPG o cualquier formato de exportación de imagen de `EDT-`, incluido `.xshot`. Opciones:
    - JPG: **Calidad**, de 1 a 100 (por defecto 90). Las zonas transparentes quedan blancas.
    - **Usar el formato por defecto al compartir**: sigue la preferencia de `SHR-` en lugar de fijar uno.
  - **Si el archivo ya existe**: **Añadir un número** (por defecto; `nombre (2).png`), **Sobrescribir** o **Preguntar**.
  - **Preguntar nombre y carpeta en cada captura** (por defecto: desactivada): abre el diálogo de guardar del sistema, con el nombre del patrón, la carpeta y el formato ya puestos.
  - **Incluir los metadatos de captura en el archivo** (por defecto: desactivada; decisión de diseño por privacidad): escribe los metadatos de CAP-056 en los campos estándar del formato, si los tiene.
  - Si la carpeta no está disponible (disco desconectado, unidad de red sin acceso), la captura queda en la biblioteca y una notificación ofrece **Elegir otra carpeta** y **Reintentar**.
  - Al escribir el archivo se publica el evento `file.written` (APP-031).
- **Criterios de aceptación:**
  - [ ] Con Prefijo = `Informe` y patrón `{prefix}_{date}_{counter:3}`, dos capturas del 12-10-2026 se guardan como `Informe_2026-10-12_001.png` e `Informe_2026-10-12_002.png`.
  - [ ] Con `{title}` y una ventana titulada `a/b: c?`, el nombre contiene `a-b- c-` y el archivo se crea sin error, en Windows y en macOS.
  - [ ] El patrón `{date}/{time}` crea una subcarpeta con la fecha y guarda dentro el archivo.
  - [ ] Con la carpeta en un disco desconectado, la captura aparece en la biblioteca y la notificación ofrece Elegir otra carpeta.
  - [ ] Con Formato = JPG, una captura de mano alzada se guarda con fondo blanco fuera de la forma.
- **Mejora sobre Snagit:** patrón de nombre documentado, con marcadores de origen, contador y subcarpetas, igual en ambas plataformas.
- **Fuentes:** research/01-captura.md §4.4, §1.9 (nombres de archivo), §2.2, §11.1 (nombres con el origen), §16 (punto 1)

### CAP-054 · Ruta de la captura
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** lo que pasa después de capturar (editor, portapapeles, destinos, autoguardado en la biblioteca, bandeja reciente)
- **Comportamiento:**
  - Cuando una captura termina, XcreenShot hace siempre estos pasos, en este orden (decisión de diseño):
    1. **Crea el documento**: píxeles base, objetos (cursor, áreas, ventanas), efectos al capturar (CAP-051) y metadatos (CAP-056).
    2. **Lo guarda en la biblioteca** (`LIB-`) antes que nada, para que no se pierda. Si el autoguardado está desactivado, queda en la bandeja reciente como no guardado (`TRY-`).
    3. **Copia al portapapeles**, si está activado (CAP-046).
    4. **Abre** según Abrir en (CAP-045).
    5. **Envía** a los destinos, al momento o al pulsar Enviar (CAP-052).
    6. **Confirma** la captura (CAP-055) y publica los eventos `capture.succeeded` y `file.written` (APP-031).
  - La captura aparece en la bandeja reciente (`TRY-`) en cuanto existe.
  - Si un paso falla (portapapeles bloqueado, carpeta no disponible, destino caído), los siguientes siguen y la captura queda siempre en la biblioteca o en la bandeja.
  - Las capturas seguidas se procesan en orden; una no adelanta a la anterior.
  - Una captura cancelada (`Esc`, Cancelar) no deja ninguna entrada en la biblioteca ni archivos temporales visibles.
  - Los tiempos de cada paso los fijan los requisitos no funcionales (APP-048, APP-049).
- **Criterios de aceptación:**
  - [ ] Con Abrir en = Ninguno, Copiar al portapapeles y el destino Archivo, una captura queda en la biblioteca, en el portapapeles y en la carpeta, y aparece en la bandeja reciente.
  - [ ] Cinco capturas hechas en menos de 5 s aparecen en la biblioteca en el orden en que se hicieron.
  - [ ] Con la carpeta del destino Archivo no disponible, la captura está igualmente en la biblioteca y en el portapapeles.
  - [ ] Tras cancelar con `Esc`, la biblioteca y la carpeta temporal de la app no tienen ningún archivo nuevo.
- **Fuentes:** research/01-captura.md §12.1, §4.4, §4.5, §4.6

### CAP-055 · Confirmación de captura
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** notificación "Capture Copied" (Mac); avisos de éxito o fallo cuando no se abre ninguna ventana
- **Comportamiento:**
  - Ajuste **Confirmación de captura** en Preferencias > Captura (APP-008), con tres partes (decisión de diseño):
    - **Sonido al capturar**: Ninguno (por defecto), Obturador o Clic suave. Suena una vez en el momento de tomar la imagen, con el volumen del sistema. No suena si el sistema está en silencio o en modo No molestar o Concentración.
    - **Destello** (por defecto: activado): el área capturada se ilumina un instante (≤ 200 ms) en el momento de tomar la imagen. Con la opción del sistema de reducir el movimiento, se sustituye por un borde que aparece y desaparece sin animación (APP-045). El destello nunca sale en ninguna captura.
    - **Notificación** (por defecto: activada; categoría Capturas de APP-016): solo cuando no se abre ninguna ventana (Abrir en = Ninguno). El texto dice qué pasó: "Captura copiada", "Captura guardada en <carpeta>" o "Captura guardada en la biblioteca". Acciones: **Abrir** (en el editor) y, si hay archivo, **Mostrar en la carpeta**. Un clic en la notificación abre la captura en el editor.
  - En Intervalo y Programada no hay sonido, destello ni notificación por cada captura. Solo se avisa al empezar y al terminar la serie, y cuando hay un problema (decisión de diseño).
  - Los errores (permiso que falta, portapapeles bloqueado, carpeta no disponible) siempre se notifican, en la categoría Problemas (APP-016).
- **Criterios de aceptación:**
  - [ ] Con Sonido = Obturador, cada captura suena una sola vez, y con el sistema en silencio no suena.
  - [ ] Con Abrir en = Ninguno y Copiar al portapapeles, aparece "Captura copiada", y un clic en ella abre la captura en el editor.
  - [ ] El destello no aparece en una captura hecha justo después de otra.
  - [ ] Con Abrir en = Editor no aparece ninguna notificación de captura.
- **Fuentes:** research/01-captura.md §11.3, §16 (punto 2), §7 (aviso sin ventana)

### CAP-056 · Metadatos de captura
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** información recogida al capturar (fecha y hora, aplicación y versión, URL del navegador, sistema operativo, usuario), usada por el efecto Capture Info y por la biblioteca
- **Comportamiento:**
  - Cada captura guarda, en el momento de capturar, sus metadatos en `meta.json` dentro del `.xshot` (SPEC §7) y en el índice de la biblioteca (`LIB-`):

    | Campo | Contenido |
    |---|---|
    | Fecha y hora | Hora local con zona horaria (ISO 8601) |
    | Modo y origen | Modo de captura (región, ventana, desplazamiento…) y origen: pantalla, página web, cámara, portapapeles, archivo, PDF, herramienta del sistema, AirDrop, carpeta de recepción, iPhone o iPad |
    | Aplicación | Nombre visible, versión e identificador (ruta del ejecutable en Windows; identificador del paquete en macOS) |
    | Ventana | Título de la ventana |
    | URL | Dirección y título de la página, si la aplicación es un navegador (desde M3) |
    | Sistema | Nombre, versión y compilación del sistema operativo |
    | Usuario y equipo | Nombre de la cuenta del sistema y nombre del equipo |
    | Pantalla | Monitor, escala (1×, 2×…), tamaño en píxeles y si estaba en HDR (CAP-021, CAP-022) |
    | XcreenShot | Versión de la app |
    | Datos propios del modo | Por ejemplo: serie y número de orden (CAP-049), retraso de una programada (CAP-050), archivo y página (CAP-040, CAP-041), dispositivo (CAP-044) |

  - Qué aplicación se registra (decisión de diseño):
    - en Región, la dueña de la ventana que ocupa la mayor parte del área;
    - en Ventana, la de la ventana;
    - en Pantalla completa, la que estaba en primer plano;
    - en Múltiples áreas, la del área más grande.
  - **URL del navegador.** Se obtiene de la pestaña activa en los navegadores habituales (Chrome, Edge, Firefox, Safari, Brave y los basados en Chromium). El objetivo es que funcione sin que el usuario cambie ajustes del navegador (decisión de diseño). Si un navegador exige activar alguna opción, la ayuda de Detalles de captura lo explica con texto propio. Si no se puede obtener, el campo queda vacío. En macOS puede necesitar el permiso de Accesibilidad (APP-024).
  - Reunir los metadatos no retrasa la captura. Si un dato tarda (por ejemplo, la URL), se completa después en el documento y en el índice (decisión de diseño).
  - Privacidad. En Preferencias > Captura > **Metadatos de captura** (APP-008) hay una casilla para cada uno de estos datos: URL, título de la ventana, usuario y equipo. Todas están activadas por defecto. Desmarcar una deja de guardarla en las capturas nuevas.
  - Los metadatos nunca salen del equipo por sí solos: no se escriben en los archivos exportados salvo con la opción de CAP-053, ni se muestran en la imagen salvo con el efecto Detalles de captura (`EDT-`).
  - Usos:
    - el efecto Detalles de captura (`EDT-`);
    - agrupar y filtrar por aplicación o sitio web en la biblioteca (`LIB-`);
    - los marcadores del nombre de archivo (CAP-053);
    - el resultado de los comandos (CAP-008).
- **Criterios de aceptación:**
  - [ ] (Desde M3) Una captura de región sobre una página abierta en Chrome guarda la URL de la pestaña activa y el título de la página.
  - [ ] Una captura de ventana del Bloc de notas (Windows) o de TextEdit (macOS) guarda el nombre de la aplicación y su versión.
  - [ ] El `meta.json` de un `.xshot` contiene la fecha con zona horaria, el sistema operativo, la escala de la pantalla y la versión de XcreenShot.
  - [ ] Con la casilla URL desmarcada, una captura de un navegador no guarda URL.
  - [ ] Los campos de `meta.json` tienen los mismos nombres en Windows y en macOS.
- **Notas de plataforma:** la manera de leer la URL del navegador es distinta en cada sistema y en algunos navegadores puede no funcionar; en ese caso el campo queda vacío sin error.
- **Mejora sobre Snagit:** el usuario elige qué datos personales se guardan; los metadatos son legibles en un archivo abierto.
- **Fuentes:** research/01-captura.md §12.2, §12.1; research/05-plataformas-versiones.md §1B (2024.0.0: usuario del sistema)

---

## 7. Anotación rápida

### CAP-057 · Anotación rápida de imagen
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** experimento Quick Image Annotation (Windows): capturar, anotar en el sitio y copiar al portapapeles sin abrir el editor
- **Comportamiento:**
  - Flujo para marcar algo en la pantalla y pegarlo enseguida, sin pasar por el editor. Se inicia de estas formas:
    - con el atajo **Anotación rápida** (CAP-065);
    - desde el menú del ícono (CAP-005);
    - con Abrir en = Anotación rápida (CAP-045);
    - con un preset que la use (CAP-059).
  - **Paso 1, selección.** Es igual que en la captura universal (CAP-009): arrastrar, clic en un elemento detectado o `F` para la pantalla completa. La pantalla sigue congelada.
  - **Paso 2, anotación en el sitio.** Al soltar, el área seleccionada queda iluminada sobre el fondo oscurecido y aparece junto a ella una barra compacta:
    - herramientas: **Flecha**, **Forma** (rectángulo o elipse), **Texto**, **Resaltar**, **Desenfocar** y **Paso** (número que avanza solo). Se eligen con un clic o con su número del `1` al `6`;
    - un selector con los colores del tema activo y otro de grosor;
    - **Deshacer** y **Rehacer** (`Ctrl+Z`/`Cmd+Z`; `Ctrl+Shift+Z`/`Cmd+Shift+Z`);
    - **Copiar**, **Abrir en el editor** y **Cancelar**.
  - Las anotaciones son los mismos objetos del editor, con los estilos del tema activo (`EDT-`). Mientras dura el paso 2 se pueden seleccionar, mover y borrar (`Supr`). Solo se dibuja dentro del área.
  - Antes de la primera anotación, el área todavía se puede ajustar con sus tiradores (CAP-011). Después queda fija.
  - **Terminar:**
    - **Copiar** (`Enter` o `Ctrl+C`/`Cmd+C`) copia al portapapeles la imagen con las anotaciones aplanadas (CAP-046), cierra la barra y guarda la captura en la biblioteca **con las anotaciones editables** (decisión de diseño). La confirmación de CAP-055 avisa de que la captura se copió.
    - **Abrir en el editor** (`Ctrl+E`/`Cmd+E`) pasa la captura al editor con las anotaciones como objetos editables.
    - **Cancelar** (`Esc`). Si hay una herramienta con un objeto a medio dibujar, el primer `Esc` solo anula ese objeto. Si hay anotaciones, pide confirmación antes de descartar. Cancelar no crea ninguna captura.
  - Ajustes que usa:
    - siempre, Capturar cursor de la pestaña Imagen (el cursor queda como objeto, CAP-047);
    - Efectos y Compartir, solo si se lanza desde un preset que los tenga; se aplican al terminar, sobre la imagen ya anotada.
  - La barra nunca sale en la imagen (CAP-023), se coloca como la barra de captura (CAP-011) y se puede manejar solo con el teclado.
  - Preferencias > Captura > **Anotación rápida** (APP-008) (decisión de diseño):
    - **Herramienta inicial**: Flecha (por defecto) o la última usada;
    - **Guardar en la biblioteca al copiar** (por defecto: activada). Desactivada, Copiar solo deja la imagen en el portapapeles y la captura queda en la bandeja reciente como no guardada (`TRY-`).
  - Se publica primero como función en prueba (APP-034), con la clave `experimental.quickAnnotate`. Mientras lo sea, el atajo, la entrada del menú del ícono, el valor de Abrir en y sus preferencias solo aparecen con ella activada.
- **Criterios de aceptación:**
  - [ ] Atajo → arrastrar un área → dibujar una flecha → `Enter`: el portapapeles contiene la imagen con la flecha, no se abre ninguna ventana del editor y la biblioteca tiene la captura con la flecha como objeto editable.
  - [ ] **Abrir en el editor** después de dibujar un texto y un paso abre el documento con esos dos objetos editables.
  - [ ] `Ctrl+Z`/`Cmd+Z` quita la última anotación y `Ctrl+Shift+Z`/`Cmd+Shift+Z` la recupera.
  - [ ] `Esc` con anotaciones pide confirmación; al confirmar, no queda ninguna captura en la biblioteca.
  - [ ] La barra aparece ≤ 200 ms después de soltar la selección.
- **Mejora sobre Snagit:** también en macOS; la captura queda en la biblioteca con las anotaciones editables.
- **Fuentes:** research/01-captura.md §12.3, §11.1 (Experiments); research/05-plataformas-versiones.md §5 (Quick Image Annotation), §1A (2026.1.0)

---

## 8. Presets

### CAP-058 · Lista de presets
- **Hito:** M3 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** sección Presets del Capture Window (ejecutar con el botón que aparece al pasar el puntero o con el atajo del preset; icono Properties); View Presets del Capture Widget
- **Comportamiento:**
  - Un **preset** es un conjunto de ajustes de captura con nombre (SPEC §6). La lista aparece:
    - en la sección **Presets** del panel de captura (CAP-001), bajo las pestañas;
    - en el submenú **Presets** del menú del ícono (CAP-005);
    - en el botón **Presets** del Lanzador de borde (CAP-004).
  - Las tres listas tienen siempre los mismos presets en el mismo orden (CAP-061).
  - Cada fila del panel muestra:
    - un ícono del tipo (Universal, Imagen, Video, Grabador de pasos);
    - el nombre;
    - un resumen breve (por ejemplo, "Región 1080×1080 · Portapapeles");
    - el atajo, si lo tiene;
    - un indicador **En marcha** en los presets de intervalo o programada activos.
  - **Ejecutar un preset:** botón **Capturar** de la fila (aparece al pasar el puntero o con la fila seleccionada), doble clic en la fila, `Enter` con la fila seleccionada, su atajo (CAP-062), el menú del ícono, el Lanzador de borde o `preset.run` (CAP-008).
  - Ejecutar un preset usa **solo** sus ajustes y no cambia los de las pestañas del panel (decisión de diseño).
  - Si un preset está En marcha, ejecutarlo lo detiene (CAP-049, CAP-050).
  - El menú **⋯** de cada fila (y el clic secundario) ofrece:
    - Editar, Cambiar nombre, Duplicar y Eliminar (CAP-060);
    - Actualizar con los ajustes actuales (CAP-060);
    - Asignar atajo (CAP-062);
    - Exportar (CAP-063).
  - Teclado:
    - flechas para moverse por la lista;
    - `Enter` para ejecutar;
    - `F2` para cambiar el nombre;
    - `Supr` para eliminar;
    - `Alt`/`Option` + flechas para mover el preset (CAP-061).
  - Con más de 8 presets aparece un campo **Filtrar** sobre la lista (decisión de diseño). No hay límite de presets.
  - Un preset que usa algo que ya no existe (un destino borrado, un monitor desconectado, un modo de otra plataforma) muestra un aviso en su fila. Al ejecutarlo, se omite lo que falta y se avisa, salvo que falte el área o el modo; en ese caso no captura y ofrece **Editar**.
- **Criterios de aceptación:**
  - [ ] Ejecutar un preset de Región fija 1080×1080 con Abrir en = Ninguno no cambia la Selección ni el Abrir en de la pestaña Imagen.
  - [ ] Los presets aparecen en el mismo orden en el panel, en el menú del ícono y en el Lanzador de borde.
  - [ ] Con un preset de intervalo En marcha, ejecutarlo de nuevo detiene la serie y el indicador desaparece.
  - [ ] Si se elimina un destino que usa un preset, la fila muestra el aviso y, al ejecutarlo, la captura se hace sin ese destino.
- **Fuentes:** research/01-captura.md §9.1, §1.2, §1.3, §4.8; research/05-plataformas-versiones.md §1B (2025.4.0)

### CAP-059 · Crear un preset
- **Hito:** M3 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Add Preset > "New preset from current settings" y "New Preset"
- **Comportamiento:**
  - El botón **Añadir preset** (＋) de la sección Presets ofrece:
    - **Desde los ajustes actuales**: copia todos los ajustes de la pestaña activa del panel (Universal, Imagen o Video), con las propiedades del modo;
    - **Nuevo preset**: abre el editor de presets con los valores por defecto de CAP-003, para configurarlo desde cero. Primero se elige el tipo: Universal, Imagen o Video;
    - **Grabador de pasos**: crea un preset con los ajustes del grabador (`SMT-`).
  - Tras crearlo, el nombre queda editable en la propia fila, con un nombre propuesto que describe sus ajustes (por ejemplo, "Región 1080×1080"). `Enter` lo confirma y `Esc` deja el propuesto.
  - Reglas del nombre:
    - de 1 a 64 caracteres;
    - único sin distinguir mayúsculas; si se repite, se añade " (2)";
    - se permite cualquier carácter salvo saltos de línea.
  - Después de confirmar el nombre, el campo de atajo de la fila queda resaltado durante unos segundos, por si el usuario quiere asignar uno (CAP-062). No es obligatorio.
  - **Qué guarda un preset**: todo lo que se puede ajustar en su pestaña.
    - Tipo (Universal, Imagen o Video).
    - Selección y sus propiedades: región fija y posición (CAP-026), monitor, propiedades de ventana y de menú, tipo y propiedades del desplazamiento, URL y opciones de página web (CAP-037), cámara (CAP-038), archivo fijo (CAP-040), opciones de PDF (CAP-041) y composición de múltiples áreas (CAP-033).
    - Efectos (CAP-051).
    - Compartir: destinos e instancias con nombre (CAP-052) y opciones del destino Archivo (CAP-053).
    - Abrir en, Copiar al portapapeles y Capturar cursor (CAP-045 a CAP-047).
    - Tiempo: retardo, intervalo o programación (CAP-048 a CAP-050).
    - En los presets de video, los ajustes de cámara, micrófono, audio y demás que define `VID-`.
  - Un preset **no** guarda preferencias generales, como la lupa, la cruceta o Ocultar XcreenShot al capturar.
  - Los presets se guardan al momento y se conservan al actualizar la app. Se incluyen en Exportar ajustes (APP-011).
  - Al crear un preset, el Lanzador de borde muestra su nombre unos segundos (CAP-004).
  - Comando: `preset.create` con los mismos ajustes que `capture.<modo>` (CAP-008) más el nombre (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con la pestaña Imagen en Región fija 800×600, efecto Borde y destino Archivo, **Desde los ajustes actuales** crea un preset que, al ejecutarlo, produce una imagen de 800×600 con borde y escribe el archivo.
  - [ ] **Nuevo preset** de tipo Imagen abre el editor de presets con los valores por defecto, no con los del panel.
  - [ ] Crear dos presets con el nombre "Ventana" deja "Ventana" y "Ventana (2)".
  - [ ] Tras reiniciar la app, los presets y sus ajustes siguen iguales.
- **Fuentes:** research/01-captura.md §9.1, §1.3, §4.2, §4.8, §6.1 (preset de desplazamiento)

### CAP-060 · Editar, renombrar, duplicar y eliminar
- **Hito:** M3 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** icono Properties del preset (editar, exportar, etc.)
- **Comportamiento:**
  - **Editar** abre la ventana **Editar preset**, con los mismos controles que la pestaña de su tipo, el nombre y el atajo. Los cambios se aplican con **Guardar**; **Cancelar** los descarta. El tipo (Universal, Imagen o Video) no se puede cambiar; para eso se crea otro preset.
  - Si se edita un preset que está En marcha, al guardar se avisa de que los cambios se aplican la próxima vez que se inicie.
  - **Actualizar con los ajustes actuales** (decisión de diseño) sustituye los ajustes del preset por los de la pestaña activa del panel, si es del mismo tipo. Conserva el nombre, el atajo y la posición, y pide confirmación.
  - **Cambiar nombre** (`F2` o doble clic lento en el nombre) edita el nombre en la fila, con las reglas de CAP-059. El atajo se conserva (APP-022).
  - **Duplicar** crea una copia justo debajo, llamada "<nombre> (copia)", **sin atajo**, para evitar el conflicto.
  - **Eliminar** (`Supr`) pide confirmación, salvo que se haya marcado "No volver a preguntar" (APP-010):
    - si el preset está En marcha, primero lo detiene;
    - su atajo queda libre (APP-022);
    - durante 10 s, una franja ofrece **Deshacer**, que lo recupera con su nombre, atajo y posición (decisión de diseño).
  - Se pueden seleccionar varios presets (`Ctrl`/`Cmd` o `Shift` + clic) para eliminarlos o exportarlos juntos.
  - Comandos: `preset.update`, `preset.rename`, `preset.duplicate` y `preset.delete` (este último exige `--yes` en el CLI, como los demás comandos que borran, APP-030).
- **Criterios de aceptación:**
  - [ ] Editar un preset, cambiar Retardo a 3 s y **Cancelar** deja el preset como estaba; con **Guardar**, la siguiente ejecución espera 3 s.
  - [ ] Cambiar el nombre de un preset con atajo mantiene el atajo funcionando.
  - [ ] **Duplicar** crea "<nombre> (copia)" sin atajo, con los mismos ajustes.
  - [ ] Tras eliminar un preset, **Deshacer** lo recupera en la misma posición y con su atajo funcionando.
- **Fuentes:** research/01-captura.md §9.1

### CAP-061 · Reordenar
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** arrastrar por el asa de movimiento (Mac)
- **Comportamiento:**
  - Cada fila tiene un asa (⋮⋮) para arrastrarla a otra posición. Una línea indica dónde va a caer.
  - Con el teclado, `Alt`/`Option` + `↑`/`↓` sube o baja el preset seleccionado una posición.
  - El orden es el mismo en el panel, el menú del ícono y el Lanzador de borde (CAP-058), y en la sección Presets de Preferencias > Atajos (APP-018). Se guarda al momento.
  - Los presets nuevos se añaden al final; los duplicados, justo debajo del original; los importados, al final, en el orden del archivo (CAP-063).
  - Con el campo Filtrar en uso, el arrastre se desactiva y su tooltip explica por qué (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Arrastrar el tercer preset al primer lugar cambia el orden en el panel y en el menú del ícono sin reiniciar.
  - [ ] `Alt`/`Option` + `↑` con el segundo preset seleccionado lo pasa al primer lugar.
  - [ ] El orden se conserva tras reiniciar la app.
- **Mejora sobre Snagit:** reordenar también con el teclado y en ambas plataformas.
- **Fuentes:** research/01-captura.md §9.1; research/05-plataformas-versiones.md §1B (2025.4.0)

### CAP-062 · Atajo por preset
- **Hito:** M3 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** "Add Hotkey" (Windows) / "+ Shortcut" (Mac) de cada preset; el campo de atajo sigue visible en los presets de intervalo
- **Comportamiento:**
  - Cada preset puede tener un **atajo global**. Se asigna en el campo de atajo de su fila, en la ventana Editar preset o en Preferencias > Atajos, grupo Presets. Son el mismo atajo (APP-018, APP-022).
  - Reglas de validación y conflictos: las de APP-017 y APP-019. Un atajo de preset no puede repetir otro atajo de XcreenShot.
  - Pulsar el atajo ejecuta el preset desde cualquier app, sin abrir el panel (CAP-058).
  - En los presets de **Intervalo** o **Programada**, el atajo funciona como interruptor: inicia o programa la primera vez, y detiene o cancela la siguiente. Una notificación breve confirma cada cambio (decisión de diseño).
  - Si se pulsa durante una selección en pantalla o una grabación, no hace nada (como CAP-006).
  - Los presets incorporados no tienen atajo por defecto en ningún perfil (CAP-064, CAP-066). Cambiar de perfil de atajos no toca los atajos de los presets (APP-021).
  - Al eliminar el preset, el atajo se libera; al cambiarle el nombre, se conserva; al duplicarlo, la copia no lo hereda (CAP-060).
- **Criterios de aceptación:**
  - [ ] Asignar `Ctrl+Alt+Shift+1` (Windows) o `⌃⌥⇧1` (macOS) a un preset hace que esa combinación lo ejecute con el panel cerrado.
  - [ ] En un preset de intervalo, la primera pulsación del atajo inicia la serie y la segunda la detiene.
  - [ ] Asignar a un preset el atajo de Capturar ofrece Reasignar o Cancelar (APP-018).
  - [ ] El atajo asignado en la fila del preset aparece también en Preferencias > Atajos.
- **Fuentes:** research/01-captura.md §9.1, §4.8, §10

### CAP-063 · Exportar e importar `.xpreset`
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** File > Import Presets (selección múltiple) y Properties > Export (Windows), con archivos `.snagpresets`
- **Comportamiento:**
  - **Exportar** (menú ⋯ de un preset o de varios seleccionados) guarda un archivo `.xpreset` (SPEC §7). Es un JSON legible que contiene:
    - la versión del formato, la versión de la app y la plataforma de origen;
    - la lista de presets, con su nombre, tipo, ajustes, orden y atajo. El atajo se guarda por posición física de tecla (APP-017).
    - Los destinos se guardan por tipo y nombre de instancia, **nunca** con contraseñas, tokens ni sesiones iniciadas. Las rutas (carpeta del destino Archivo, archivo fijo) se guardan tal cual.
  - **Importar** desde:
    - el menú Más del panel > **Importar presets…**, que admite varios archivos (CAP-001);
    - soltar `.xpreset` sobre el panel;
    - doble clic en el archivo (APP-001);
    - `preset.import` (APP-030);
    - la configuración administrada (`policy.presets`, APP-035).
  - Antes de importar, un resumen lista los presets del archivo, con una casilla para cada uno. Después:
    - si ya existe un preset con el mismo nombre: **Reemplazar**, **Conservar ambos** (el importado recibe " (2)") u **Omitir**;
    - si su atajo ya lo usa otro comando, se importa sin atajo y se avisa (APP-022);
    - si usa un destino que no existe en este equipo, se importa igual y la fila muestra el aviso de CAP-058;
    - si viene de la otra plataforma, los atajos se convierten como en APP-011, y los ajustes que no existen en esta plataforma (por ejemplo, iPhone o iPad, CAP-044) se conservan en el preset sin aplicarse;
    - las rutas que no existen en este equipo se marcan para revisarlas.
  - Un archivo dañado, de una versión de formato más nueva que la app o que no es un `.xpreset` no importa nada y lo explica. Un archivo de una versión anterior se migra (SPEC §7).
  - Los archivos de presets de Snagit no se importan (SPEC §2.5).
- **Criterios de aceptación:**
  - [ ] Exportar 3 presets e importarlos en una instalación nueva crea los 3 con los mismos ajustes, orden y atajos.
  - [ ] Un `.xpreset` exportado con un destino con sesión iniciada no contiene ningún token (se comprueba buscando el valor en el archivo).
  - [ ] Un preset de Windows con atajo `Ctrl+Alt+Shift+I`, importado en macOS, queda con `⌃⌥⇧I` y funciona.
  - [ ] Importar un preset con el mismo nombre que uno existente ofrece Reemplazar, Conservar ambos u Omitir, y respeta la elección.
  - [ ] Un archivo `.xpreset` dañado muestra el error y no crea ningún preset.
- **Mejora sobre Snagit:** formato abierto, importación con resumen y conversión de atajos entre plataformas.
- **Fuentes:** research/01-captura.md §9.1, §1.2; SPEC §7

### CAP-064 · Presets incorporados
- **Hito:** M3 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** presets que vienen con la app ("Scrolling Capture" en Windows; "Step Capture")
- **Comportamiento:**
  - Una instalación nueva incluye estos presets, sin atajo (decisión de diseño en la selección):

    | Preset | Ajustes |
    |---|---|
    | **Desplazamiento** | Imagen; Selección = Desplazamiento, Tipo = Automático, Dirección = Vertical, Empezar desde = El principio; Abrir en = Editor |
    | **Grabador de pasos** | Grabador de pasos con sus ajustes por defecto (`SMT-`) |
    | **Región al portapapeles** | Imagen; Selección = Región; Abrir en = Ninguno; Copiar al portapapeles = sí; Capturar cursor = no |
    | **Extraer texto** | Imagen; Selección = Extraer texto (`SMT-`) |

  - Cada uno aparece cuando existe su función: Grabador de pasos y Extraer texto, con `SMT-` (M5).
  - Se comportan como cualquier otro preset: se pueden ejecutar, editar, renombrar, reordenar, asignarles atajo, exportar y eliminar.
  - Menú Más del panel > **Restaurar presets incorporados** vuelve a crear los que se eliminaron. Si alguno se modificó, pregunta si se deja como está o se restablece.
  - Al actualizar la app, no se cambian los presets incorporados que ya existían. Si una versión nueva añade alguno, se crea al final de la lista.
  - Sus nombres se traducen al idioma de la interfaz mientras el usuario no los cambie (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] En una instalación nueva con M3, la lista incluye Desplazamiento y Región al portapapeles, sin atajo.
  - [ ] Ejecutar **Región al portapapeles** deja la imagen en el portapapeles sin abrir el editor.
  - [ ] Tras eliminar Desplazamiento, **Restaurar presets incorporados** lo vuelve a crear con sus ajustes originales.
  - [ ] Con la interfaz en inglés, los presets incorporados aparecen con su nombre en inglés.
- **Fuentes:** research/01-captura.md §9.1, §6.1 (preset de desplazamiento), §8.1

---

## 9. Atajos de captura

### CAP-065 · Acciones de captura y atajos por defecto
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** atajos globales de captura (Global Capture; en Mac, All-in-One, Image, Mission Control, Exposé Window, Menu y Repeat Last Capture)
- **Comportamiento:**
  - Esta ficha lista las acciones de captura que pueden tener un **atajo global** y fija su valor por defecto en cada perfil (D-08). El sistema de atajos (ámbitos, validación, grabación, conflictos, tomar atajos ocupados, perfiles y hoja de atajos) es de `APP-` (APP-017 a APP-023). Las teclas que funcionan durante una selección son contextuales y están en CAP-017.
  - Todas las acciones aparecen en Preferencias > Atajos, grupo **Captura** (APP-018); los presets, en el grupo **Presets** (CAP-062). Cada una llega con su función y su hito: hasta entonces no aparece.
  - **Perfil Coexistencia** (por defecto, APP-021; decisión de diseño). Usa una familia de combinaciones que Snagit y el sistema no usan por defecto:
    - Windows: `Ctrl+Alt+Shift` + letra. La excepción es **Capturar**, que usa `Ctrl+Shift+PrtScn` (D-08).
    - macOS: `⌃⌥⇧` (`Control+Option+Shift`) + letra.
    - Las letras son las que usa Snagit en Mac para la misma acción, siempre que existan, para facilitar el paso al perfil de compatibilidad.
  - **Perfil Compatibilidad con Snagit:** sus valores y diferencias están en CAP-066. Las acciones para las que Snagit no tiene atajo por defecto en esa plataforma conservan el valor de Coexistencia (decisión de diseño).
  - Valores por defecto:

    | Acción | Qué hace | Windows · Coexistencia | Windows · Compatibilidad | macOS · Coexistencia | macOS · Compatibilidad |
    |---|---|---|---|---|---|
    | **Capturar** | Captura con los ajustes de la pestaña activa del panel (CAP-006) | `Ctrl+Shift+PrtScn` | `PrtScn` | `⌃⌥⇧C` | `⌃⇧C` |
    | **Captura universal** | Captura universal con los ajustes de la pestaña Universal (CAP-009) | `Ctrl+Alt+Shift+O` | = Coexistencia | `⌃⌥⇧O` | `⌃⇧O` |
    | **Captura de imagen** | Captura con los ajustes de la pestaña Imagen, sea cual sea la pestaña activa | `Ctrl+Alt+Shift+S` | = Coexistencia | `⌃⌥⇧S` | `⌃⇧S` |
    | **Captura de ventana** | Selección = Ventana, con las demás opciones de la pestaña Imagen (CAP-027) | `Ctrl+Alt+Shift+W` | = Coexistencia | `⌃⌥⇧W` | `⌃⇧W` |
    | **Ventana activa** | Captura sin cruceta la ventana con el foco (CAP-027) | `Ctrl+Alt+Shift+A` | = Coexistencia | `⌃⌥⇧A` | = Coexistencia |
    | **Selector de ventanas** | Abre el selector de ventanas (CAP-030) | `Ctrl+Alt+Shift+M` | = Coexistencia | `⌃⌥⇧M` | `⌃⇧M` |
    | **Captura de menú** | Captura el menú abierto, sin cuenta atrás (CAP-031) | `Ctrl+Alt+Shift+E` | = Coexistencia | `⌃⌥⇧E` | `⌃⇧E` |
    | **Repetir última captura** | CAP-007 | `Ctrl+Alt+Shift+R` | = Coexistencia | `⌃⌥⇧R` | `⌃⇧R` |
    | **Anotación rápida** | CAP-057 | `Ctrl+Alt+Shift+Q` | = Coexistencia | `⌃⌥⇧Q` | = Coexistencia |
    | **Extraer texto** | Captura en modo Extraer texto (`SMT-`) | `Ctrl+Alt+Shift+T` | = Coexistencia | `⌃⌥⇧T` | = Coexistencia |
    | **Mostrar u ocultar el panel de captura** | CAP-001 | `Ctrl+Alt+Shift+P` | = Coexistencia | `⌃⌥⇧P` | = Coexistencia |

  - Acciones **sin atajo por defecto** en ningún perfil, que se pueden asignar:
    - Región, Pantalla completa (monitor bajo el puntero), Todos los monitores y Desplazamiento;
    - Repetir última área (CAP-007);
    - Página web, Cámara web, Portapapeles, Archivo e Importar PDF (CAP-037 a CAP-041);
    - Importar desde iPhone o iPad (solo macOS, CAP-044);
    - Grabador de pasos (`SMT-`);
    - Detener todas las capturas por intervalo o programadas (CAP-049, CAP-050);
    - cada preset (CAP-062).
  - Estas acciones usan siempre los ajustes de la pestaña Imagen para lo que no es el modo (efectos, destinos, abrir en, portapapeles, cursor y tiempo). Para combinaciones fijas se usan presets.
  - **Reservas para video.** En las dos familias de Coexistencia, las letras `V`, `D` y `F` y la barra espaciadora quedan reservadas para las acciones de grabación (captura de video, dibujo en vivo, cambio de cámara, iniciar/pausar). Sus valores, en ambos perfiles, los fija `VID-`.
  - Si se pulsa un atajo de captura mientras ya hay una selección en pantalla o una grabación, no hace nada (CAP-006).
  - Las combinaciones de Coexistencia no coinciden con ningún atajo por defecto de Snagit en esa plataforma, ni con los atajos de captura del sistema:
    - Windows: `PrtScn`, `Alt+PrtScn`, `Win+PrtScn`, `Win+Shift+S` y `Win+Alt+PrtScn`;
    - macOS: `Cmd+Shift+3`, `4`, `5` y `6`.
- **Criterios de aceptación:**
  - [ ] En una instalación nueva (Coexistencia), cada acción de la tabla responde a su combinación en la plataforma correspondiente, con el panel cerrado.
  - [ ] En una instalación nueva, ninguna combinación de la tabla coincide con un atajo por defecto de Snagit ni con un atajo de captura del sistema (se comprueba con la lista de esta ficha y la de `VID-`).
  - [ ] **Captura de imagen** usa la Selección de la pestaña Imagen aunque la pestaña activa del panel sea Universal.
  - [ ] Una acción sin atajo por defecto, como Repetir última área, se puede asignar en Preferencias > Atajos y funciona al momento.
  - [ ] La hoja de atajos (APP-023) muestra todas las acciones de la tabla con su valor actual.
- **Notas de plataforma:**
  - Si el teclado no tiene tecla `PrtScn` o solo la envía junto con `Fn` (frecuente en portátiles), la bienvenida (APP-005) propone usar `Ctrl+Alt+Shift+C` para Capturar (decisión de diseño).
  - En Windows, `Ctrl+Alt` equivale a `AltGr` en algunas distribuciones de teclado. Si en la del usuario alguna combinación de la familia escribe un carácter, el usuario la cambia en Preferencias > Atajos; la ayuda de Atajos lo explica.
  - En macOS, con VoiceOver activo, algunas combinaciones que empiezan por `Control+Option` son órdenes de VoiceOver. La hoja de atajos lo advierte.
- **Mejora sobre Snagit:** atajos por defecto para todas las acciones principales también en Windows, donde Snagit solo trae el de captura global.
- **Fuentes:** research/01-captura.md §10, §1.4, §1.5, §5.2, §5.4, §5.5, §2.1; research/05-plataformas-versiones.md §3 (Default hotkeys); SPEC D-08

### CAP-066 · Perfil de compatibilidad
- **Hito:** M1 · **Prioridad:** Should
- **Plataformas:** Ambas (con diferencias de teclas durante la selección solo en Windows)
- **Paridad con Snagit:** atajos por defecto de Snagit en Windows y Mac, y teclas durante la captura en Windows (bloqueos con `Ctrl`, `Ctrl+Shift`; clic derecho para terminar las áreas múltiples; `H`, `V` y `B` para el desplazamiento automático; `F1` para la ayuda)
- **Comportamiento:**
  - Con el perfil **Compatibilidad con Snagit** (APP-021), los atajos globales de captura toman los valores de las columnas "Compatibilidad" de CAP-065:
    - Windows: Capturar = `PrtScn`; las demás acciones conservan su valor de Coexistencia, porque Snagit no les da atajo en Windows.
    - macOS: Capturar = `⌃⇧C`; Captura universal = `⌃⇧O`; Captura de imagen = `⌃⇧S`; Captura de ventana = `⌃⇧W`; Selector de ventanas = `⌃⇧M`; Captura de menú = `⌃⇧E`; Repetir última captura = `⌃⇧R`.
    - Los valores de video (`VID-`), del editor (`EDT-`) y de la app (APP-021, por ejemplo `Ctrl+Shift+X` = Salir en Windows) los fija cada dueño.
  - **Teclas durante la selección, solo en Windows** (cambian CAP-017, CAP-018 y CAP-033):
    - `Ctrl` mantenido bloquea 16:9 y `Ctrl+Shift`, 4:3. `Alt` deja de bloquear proporciones.
    - Mantener `Ctrl` al arrastrar ya no añade áreas: las áreas múltiples solo se hacen con Selección = Múltiples áreas, que también termina con el clic derecho.
    - En la fase de apuntado, `H`, `V` y `B` eligen la flecha de desplazamiento automático horizontal, vertical o ambas (CAP-035). `R`, `D` y `X` siguen funcionando.
    - La ayuda en pantalla (CAP-020) solo se abre con `F1`, porque `H` pasa a ser la flecha horizontal.
  - **En macOS** las teclas durante la selección no cambian: las de CAP-017 ya coinciden con las de Snagit en Mac (`H` para la ayuda; `R`, `D` y `X` para el desplazamiento; `Cmd` para añadir áreas).
  - La ayuda en pantalla (CAP-020) y la hoja de atajos (APP-023) muestran siempre las teclas del perfil activo.
  - **`PrtScn` en Windows 11.** Si el sistema la tiene asignada a la Herramienta Recortes, al activar el perfil se aplica APP-019 (aviso con el ajuste del sistema) y APP-020 (tomarla mientras XcreenShot esté abierto).
  - Cambiar de perfil sigue las reglas de APP-021: conserva los atajos personalizados, lista los cambios y avisa si Snagit está en marcha.
- **Criterios de aceptación:**
  - [ ] (Windows) Con el perfil de compatibilidad, `PrtScn` inicia la captura con los ajustes de la pestaña activa y `Ctrl+Shift+PrtScn` ya no hace nada.
  - [ ] (macOS) Con el perfil de compatibilidad, `⌃⇧O` inicia la captura universal y `⌃⇧E` captura el menú abierto.
  - [ ] (Windows) Con el perfil de compatibilidad, mantener `Ctrl` mientras se arrastra produce una selección 16:9, y `H` en la fase de apuntado inicia el desplazamiento automático horizontal.
  - [ ] (Windows) Con el perfil de compatibilidad, en Múltiples áreas, el clic derecho termina la captura con las áreas elegidas.
  - [ ] Al volver al perfil Coexistencia, las teclas de la selección vuelven a ser las de CAP-017 sin reiniciar.
- **Fuentes:** research/01-captura.md §10, §3.4, §5.7, §6.2 (teclas), §1.4; research/05-plataformas-versiones.md §3 (Default hotkeys; Video: bloqueo 4:3); SPEC D-08

---

## Relacionado

| Prefijo / archivo | Relación con este archivo |
|---|---|
| `SPEC.md` | Principios §2 (una app y un comportamiento; todo es un comando; local primero; formatos propios), alcance §3.2 (impresora virtual → CAP-041; formatos de Snagit excluidos), hitos §4, glosario §6 (Preset, Metadatos de captura), formatos §7 (`.xshot` con `meta.json`, `.xpreset`), nombres de la interfaz §8 (Captura universal, Panel de captura, Lanzador de borde, Extraer texto, Grabador de pasos, Detalles de captura), decisiones D-03 y D-08 |
| `SMT-` (`06-inteligentes.md`) | **Dueño del modo Extraer texto** (entrada de la lista Selección, CAP-002; preset incorporado, CAP-064; atajo, CAP-065) y **del Grabador de pasos** (entrada de la lista Selección, CAP-002; pausa desde el menú del ícono; preset incorporado, CAP-064). También la lectura de códigos QR durante la captura, la redacción automática sobre las capturas y el reconocimiento de texto que usa la búsqueda de las páginas de PDF (CAP-041) |
| `VID-` (`03-video.md`) | Pestaña Video del panel de captura (CAP-001); botón Video de la barra de captura universal (CAP-009); cambiar una grabación iniciada por error a imagen (CAP-012); grabación de una ventana que la sigue al moverse (CAP-027); HDR en video (CAP-022); ajustes de video de los presets (CAP-059); atajos de grabación y letras reservadas (CAP-065, CAP-066); grabaciones del sistema y de AirDrop importadas a la biblioteca (CAP-042, CAP-043); efectos de cámara, que no se aplican a la foto (CAP-038) |
| `EDT-` (`02-editor.md`) | Apertura del editor (CAP-045); objetos Cursor, ventanas y áreas (CAP-047, CAP-029, CAP-033); definición y opciones de los efectos, incluido Detalles de captura (CAP-051, CAP-056); formatos de exportación (CAP-053); Nuevo desde el portapapeles (CAP-039); objeto Texto para el texto del portapapeles (CAP-039); herramientas y estilos de la anotación rápida (CAP-057); exportación de capturas largas a PDF paginado (CAP-036); escala de la captura para el tamaño real (CAP-021) |
| `LIB-`, `TRY-`, `SHR-` (`04-biblioteca-compartir.md`) | Autoguardado e índice de la biblioteca (CAP-054, CAP-056); agrupación de series por intervalo (CAP-049); filtro de capturas de desplazamiento (CAP-036); bandeja reciente (CAP-045, CAP-054, CAP-057); destinos de compartir y su envío (CAP-052); formato por defecto al compartir y "no incluir el origen en los nombres" (CAP-053) |
| `APP-` (`05-app.md`) | Proceso y menú rápido del ícono (APP-001, APP-002 ↔ CAP-005); secciones de Preferencias (APP-008); notificaciones (APP-016 ↔ CAP-055); **sistema de atajos**: ámbitos, sección Atajos, conflictos, tomar atajos, perfiles, atajos de presets y hoja de atajos (APP-017 a APP-023 ↔ CAP-006, CAP-062, CAP-065, CAP-066); **permisos** (APP-024 a APP-026 ↔ CAP-024, CAP-035, CAP-038, CAP-056); registro de comandos, CLI, MCP, catálogo y eventos (APP-027 a APP-031 ↔ CAP-008); seguridad de la automatización (APP-032 ↔ CAP-052); funciones en prueba (APP-034 ↔ CAP-057); configuración administrada (`policy.presets`, APP-035 ↔ CAP-063); conexiones de red permitidas (APP-054 ↔ CAP-037); requisitos no funcionales (APP-048, APP-049, APP-053) |
| `05-app.md` (referencias por nombre) | `05-app.md` cita estas fichas por nombre. Equivalencias: Panel de captura = CAP-001; Pestaña Imagen y lista Selección = CAP-002; Lanzador de borde = CAP-004; Menú de captura del ícono = CAP-005; Comandos de captura = CAP-008; Pantalla congelada y cruceta = CAP-013; Cruceta simplificada = CAP-014; Lupa = CAP-015; Teclado durante la selección = CAP-017; Varios monitores y DPI mixto = CAP-021; Pantallas HDR = CAP-022; Ocultar XcreenShot al capturar = CAP-023; Permisos que faltan al capturar = CAP-024; Desplazamiento manual = CAP-034; Desplazamiento automático = CAP-035; Unión y procesamiento del desplazamiento = CAP-036; Página web por URL = CAP-037; Capturas del sistema = CAP-042; AirDrop = CAP-043; Intervalo = CAP-049; Programada = CAP-050; Confirmación de captura = CAP-055; Metadatos de captura = CAP-056; Anotación rápida de imagen = CAP-057; Lista de presets = CAP-058; Atajo por preset = CAP-062; Exportar e importar `.xpreset` = CAP-063; Acciones de captura y atajos por defecto = CAP-065; Perfil de compatibilidad = CAP-066 |

---

## Excluido

| Elemento de la investigación | Motivo |
|---|---|
| Escáner o cámara TWAIN como modo de captura (research/01 §1.9, §4.1, §5.12) | Retirado por TechSmith (SPEC §3.2). Las fotos y los escaneos llegan por la cámara (CAP-038), iPhone o iPad (CAP-044) o la carpeta de recepción (CAP-043) |
| Entradas retiradas de la API COM: desplazamiento personalizado, captura de enlaces, SWF, pantalla DOS, archivo de programa, fondo de escritorio, DirectX y las formas triángulo y rectángulo redondeado (research/01 §1.9) | Retiradas por TechSmith (SPEC §3.2). Las formas elipse y polígono se recuperan como opciones de Mano alzada (CAP-032) |
| Ventana ampliada (la ventana se agranda más allá de la pantalla antes de capturar) (research/01 §1.9) | Cambiaría el tamaño de la ventana de otra app, y en macOS no se puede hacer de forma fiable. Sus usos se cubren con Ventana entera aunque salga de la pantalla (CAP-027), el desplazamiento (CAP-034, CAP-035) y la página web por URL (CAP-037) (decisión de diseño) |
| Impresora virtual y su configuración (Printer Capture Settings) (research/01 §1.8, §11.1) | D-03: no hay impresora virtual. Se sustituye por Importar PDF multipágina (CAP-041) y la carpeta de recepción (CAP-043) |
| Importar presets `.snagpresets` (research/01 §9.1) | Formato propietario (SPEC §2.5). Los presets de XcreenShot usan `.xpreset` (CAP-063) |
| Formatos de salida `.snag`, `.snagx` y otros heredados de la API COM (WFX, RAS, WPG, MHT, etc.) (research/01 §1.9) | `.snag` y `.snagx` son propietarios (SPEC §2.5). Los formatos de exportación los fija `EDT-` |
| Preset incorporado "abrir la grabación en Camtasia Editor" y Abrir en = Camtasia Editor (research/01 §9.1; research/05 §1A 2026.1.0, §1B 2025.3.0) | Fuera del alcance (SPEC §3.2). Abrir en solo ofrece Editor, Anotación rápida y Ninguno (CAP-045) |
| Destinos Screencast, Camtasia Online, Camtasia Editor y X/Twitter en Compartir (research/01 §4.4) | Fuera del alcance o retirados (SPEC §3.2). El Enlace para compartir con proveedor configurable lo define `SHR-` |
| Versión de la Mac App Store sin flechas de desplazamiento automático (research/01 §6.2) | XcreenShot no se distribuye por la App Store en v1 (APP-039) |
| Ajuste del navegador "permitir JavaScript desde Apple Events" para el desplazamiento automático en Chrome (research/01 §6.2) | Es un requisito del método que usa Snagit, no una función. Los permisos que necesita XcreenShot son los de CAP-024 y APP-024. Si algún navegador necesita un ajuste propio, la ayuda de CAP-035 lo explica |
| Interruptor de registro de Windows para forzar el dibujo por software de la cruceta (research/01 §3.2) | No se usan ajustes ocultos del registro. Se cubre con la Cruceta simplificada y su oferta automática (CAP-014) |
| Aviso antiguo de que la captura web solo funciona con HTTP (research/01 §1.7) | Obsoleto. CAP-037 admite `http` y `https` |
| Importación automática de la app Grab de macOS (research/01 §1.6) | Grab ya no existe en las versiones de macOS compatibles. La herramienta actual del sistema la cubre CAP-042 |
| Nombre comercial con el que TechSmith agrupa sus funciones inteligentes (research/01 §13) | Es un nombre comercial, no una función. Las funciones que agrupa están en `SMT-` y `EDT-` |
| Cambios visuales (renovación de la interfaz de captura, colores e íconos, íconos de los presets, tema de los tooltips) y nombres antiguos de funciones (research/01 §1.2, §1.3, §9.1, §15) | Identidad visual propia (SPEC §2.4). No añaden comportamiento |
| Correcciones de errores de versiones concretas (cierres inesperados, duplicados, posiciones del cursor, menús oscuros, etc.) (research/01 §1–§12, §15) | No son funciones. El comportamiento correcto que implican está en los criterios de aceptación de cada ficha (por ejemplo, CAP-031: cada nivel de menú una sola vez; CAP-046: forma de la mano alzada en el portapapeles; CAP-047: posición del cursor) |
| Lista de compatibilidad de apps concretas con el desplazamiento automático (research/01 §6.2) | No es una función. CAP-035 define el comportamiento general y qué hacer cuando un área no se deja desplazar; la lista de apps probadas la mantiene el equipo de construcción en sus pruebas |

---

## Cobertura

| Sección de la investigación | Cubierta por |
|---|---|
| research/01-captura.md §1.1 — App de captura en segundo plano, ícono, menú del ícono, Detener captura | APP-001, APP-002, APP-003; entradas de captura del menú: CAP-005; Detener captura: CAP-005, CAP-049, CAP-050; pausar el grabador de pasos: `SMT-` |
| §1.2 — Panel de captura (pestañas, botón, atajo, engranaje, menús, restablecer, recordar estado, accesibilidad) | CAP-001, CAP-003; Importar presets: CAP-063; Preferencias: APP-006; buscar actualizaciones: APP-036; navegación con teclado: APP-043; cambios visuales: Excluido |
| §1.3 — Lanzador de borde | CAP-004; presets del lanzador: CAP-058; Interfaz de captura: APP-007 |
| §1.4 — Atajo global de captura y conflictos de `PrtScn` | CAP-006, CAP-065, CAP-066; conflictos y tomar el atajo: APP-019, APP-020; teclados con `Fn` o remapeo: APP-019 (Notas de plataforma) |
| §1.5 — Repetir última captura | CAP-007; atajo: CAP-065; conflicto: APP-019 |
| §1.6 — Abrir en XcreenShot las capturas del sistema, AirDrop, Grab, capturas durante una grabación | CAP-042, CAP-043; Grab: Excluido |
| §1.7 — Página web arrastrando la URL | CAP-037, CAP-005; aviso antiguo de solo HTTP: Excluido |
| §1.8 — Impresora virtual | CAP-041 (D-03); la impresora en sí: Excluido |
| §1.9 — API COM (entradas, salidas, selección de ventana y región, desplazamiento, retardo, propiedades, nombres de archivo, formatos, estados y eventos) | CAP-008; APP-027 a APP-031; Objeto: CAP-016, CAP-025; Archivo: CAP-040; Portapapeles: CAP-039; nombres automáticos: CAP-053; formatos: `EDT-`; salidas a impresora, correo y FTP: `SHR-`; Ventana ampliada y entradas retiradas: Excluido |
| §2.1 — Flujo de captura universal | CAP-009, CAP-011, CAP-012; flechas de desplazamiento: CAP-035 |
| §2.2 — Opciones de la pestaña Universal | CAP-010; Compartir y Archivo: CAP-052, CAP-053; Abrir en, portapapeles y cursor: CAP-045 a CAP-047; retardo de 5 s: CAP-048; tamaños impares: CAP-009 |
| §3.1 — Cruceta y lupa | CAP-013, CAP-015; rapidez: APP-048 |
| §3.2 — Cruceta simplificada y límites de monitores | CAP-014; varios monitores 4K: APP-053, CAP-013; interruptor de registro: Excluido |
| §3.3 — Detección automática de ventanas y subáreas | CAP-016 |
| §3.4 — Teclado durante la selección y guías de tamaño | CAP-017, CAP-018, CAP-019, CAP-020, CAP-011; diferencias de Windows: CAP-066 |
| §3.5 — Varios monitores y DPI | CAP-021, CAP-028 |
| §3.6 — Captura en pantallas HDR | CAP-022 |
| §4.1 — Lista Selección (actual y heredada) | CAP-002; Extraer texto y Grabador de pasos: `SMT-`; Ventana con desplazamiento y Panorámica: CAP-034, CAP-035; Objeto: CAP-016; Portapapeles y Archivo: CAP-039, CAP-040; Cámara web y Página web: CAP-038, CAP-037; Selector de ventanas: CAP-030; TWAIN: Excluido |
| §4.2 — Región fija y posición | CAP-026 |
| §4.3 — Efectos al capturar | CAP-051; efecto Detalles de captura: `EDT-` |
| §4.4 — Compartir y destino Archivo | CAP-052, CAP-053; lista de destinos: `SHR-`; Screencast, Camtasia y X: Excluido |
| §4.5 — Vista previa en el editor (Abrir en) | CAP-045; editor en segundo plano: APP-004 |
| §4.6 — Copiar al portapapeles | CAP-046 |
| §4.7 — Capturar cursor | CAP-047 |
| §4.8 — Retardo, Intervalo y Programada | CAP-048, CAP-049, CAP-050; presets con intervalo: CAP-062 |
| §5.1 — Región | CAP-025 |
| §5.2 — Ventana | CAP-027; ventana que sigue al moverse (video): `VID-` |
| §5.3 — Pantalla completa y ventanas como capas | CAP-028, CAP-029 |
| §5.4 — Elegir una ventana entre todas las ventanas extendidas | CAP-030 |
| §5.5 — Menú | CAP-031 |
| §5.6 — Mano alzada | CAP-032 |
| §5.7 — Múltiples áreas | CAP-033 |
| §5.8 — Foto con la cámara web | CAP-038 |
| §5.9 — Portapapeles como fuente | CAP-039 |
| §5.10 — Archivo como fuente | CAP-040 |
| §5.11 — Captura de objeto (heredada) | CAP-016, CAP-025 (clic en un elemento detectado); no hay modo aparte (decisión de diseño) |
| §5.12 — Escáner o cámara TWAIN | Excluido |
| §5.13 — Importar desde iPhone o iPad (Continuity Camera) | CAP-044 |
| §6 — Historia de nombres del desplazamiento | CAP-034 (manual), CAP-035 (automático) |
| §6.1 — Desplazamiento manual | CAP-034, CAP-036; ocultar la vista previa: CAP-023; preset incorporado: CAP-064; HDR: CAP-022; PDF paginado: `EDT-` |
| §6.2 — Desplazamiento automático (flechas, teclas, permisos, casos no compatibles, apps) | CAP-035, CAP-036; permisos: CAP-024, APP-024; ventanas elevadas y programas de seguridad: APP-026; alternativas para PDF y páginas web: CAP-041, CAP-037; teclas `H`/`V`/`B`: CAP-066; App Store, ajuste de Chrome y lista de apps: Excluido |
| §7 — Extraer texto al capturar | `SMT-` (Extraer texto); entrada en Selección: CAP-002; atajo: CAP-065; preset: CAP-064 |
| §8.1 — Grabador de pasos | `SMT-` (Grabador de pasos); entrada en Selección: CAP-002; preset: CAP-059, CAP-064 |
| §9.1 — Presets (ejecutar, crear, editar, reordenar, contenido, importar y exportar, incorporados) | CAP-058, CAP-059, CAP-060, CAP-061, CAP-062, CAP-063, CAP-064; ajustes de video en presets: `VID-`; `.snagpresets` y preset de Camtasia: Excluido |
| §10 — Atajos globales y durante la captura | CAP-065, CAP-066, CAP-017; sistema de atajos: APP-017 a APP-023; atajos de video y de dibujo en vivo: `VID-`; atajos del editor: `EDT-`; mostrar u ocultar y salir: APP-004, APP-021 |
| §11.1 — Preferencias de captura de Windows | Ocultar al capturar: CAP-023; capturas del sistema: CAP-042; lupa: CAP-015; cruceta simplificada: CAP-014; botón Iniciar: CAP-034; HDR: CAP-022; Interfaz de captura: CAP-004; General: APP-003, APP-007, APP-012, APP-036; video: `VID-`; biblioteca: `LIB-`; origen en los nombres: CAP-053, `SHR-`; editor en segundo plano: APP-004; funciones en prueba: CAP-022, CAP-057, `SMT-`, APP-034; impresora: Excluido |
| §11.2 — Ajustes de Mac relacionados con la captura | Ocultar al capturar: CAP-023; capturas del sistema: CAP-042; AirDrop: CAP-043; botón Iniciar: CAP-034; Reparar permisos: CAP-024, APP-024; teclado: APP-018; formato por defecto y escalado al compartir: CAP-053, `SHR-`; copia editable y ubicación de la biblioteca: `LIB-`; ícono monocromo: APP-002; funciones en prueba: CAP-029, `VID-`, `SMT-`, APP-034 |
| §11.3 — Sonidos y notificaciones de captura | CAP-055; mecanismo de notificaciones: APP-016 |
| §12.1 — Qué pasa después de capturar | CAP-054; marca de alta densidad y de desplazamiento: CAP-056, CAP-021, CAP-036; bandeja reciente: `TRY-`; aviso de ubicación de la biblioteca: APP-005, `LIB-` |
| §12.2 — Metadatos de captura y efecto Detalles de captura | CAP-056; pedir texto al capturar: CAP-051; efecto: `EDT-`; agrupar por aplicación o sitio: `LIB-` |
| §12.3 — Anotación rápida de imagen | CAP-057 |
| §12.4 — Lectura de códigos QR durante la captura | `SMT-` |
| §13 — Funciones inteligentes cerca de la captura | `SMT-` (grabador de pasos, redacción automática); política de funciones en el dispositivo: SPEC §2.3, APP-054; nombre comercial: Excluido |
| §14 — Permisos, requisitos y controles de administración | CAP-024, CAP-035; APP-024, APP-025, APP-026, APP-035, APP-040 |
| §15 — Cronología de versiones | Informativa: cada cambio está cubierto en la fila de su sección; cambios visuales y correcciones: Excluido |
| §16 — Preguntas abiertas | 1: CAP-053 · 2: CAP-055 · 3: CAP-002, CAP-039, CAP-040 · 4: CAP-033 · 5: CAP-017, CAP-018 · 6: CAP-004 · 7 y 8: `SMT-` · 9: CAP-051 · 10: CAP-038 (todas resueltas como decisión de diseño) |
| §17 — Lista de URL consultadas | Informativa (sin funciones) |
| research/05-plataformas-versiones.md §3 — Matriz de captura (desplazamiento, menú, intervalo y programada, ventana, mano alzada, impresora, lanzador, extensión del Explorador, página web, cámara, AirDrop, Continuity Camera, capturas del sistema, múltiples áreas, repetir, multimonitor, HDR, ocultar controles, región fija) | CAP-034, CAP-035, CAP-031, CAP-049, CAP-050, CAP-027, CAP-030, CAP-032, CAP-041, CAP-004, CAP-037, CAP-038, CAP-042, CAP-043, CAP-044, CAP-033, CAP-007, CAP-021, CAP-028, CAP-022, CAP-023, CAP-026; extensión del Explorador: Excluido en `05-app.md` |
| research/05 §3 — Atajos por defecto | CAP-065, CAP-066; video: `VID-`; editor: `EDT-` |
| research/05 §5 — Funciones en prueba relacionadas con la captura (Anotación rápida, Fidelidad HDR, ventanas como capas, QR) | CAP-057, CAP-022, CAP-029; QR: `SMT-`; mecanismo: APP-034 |
| research/05 §1A y §1B — Cambios de versión que afectan a la captura (Abrir en, intervalo mínimo de 3 s, intervalo y programada en Mac, ventana en Mac, esquinas redondeadas al capturar, usuario en los metadatos, Continuity Camera, AirDrop, reordenar presets, reanudar tras conceder permisos) | CAP-045, CAP-049, CAP-050, CAP-027, CAP-051, CAP-056, CAP-044, CAP-043, CAP-061, CAP-024 |
