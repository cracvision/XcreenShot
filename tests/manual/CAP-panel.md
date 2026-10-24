# Panel de captura y retardo (M1, entrega 3) · CAP-001, CAP-002, CAP-003, CAP-005, CAP-006, CAP-007, CAP-010, CAP-045, CAP-048, CAP-065

Pruebas automáticas:
- `crates/xs-capture/src/panel.rs`: valores por defecto de CAP-003, cambios parciales y valores no válidos, qué hace Capturar con cada pestaña (CAP-006), opciones de la pestaña Imagen para las acciones de un solo modo (CAP-065).
- `crates/xs-capture/src/lib.rs`: `panel_settings_configure_and_reset` (CAP-001, CAP-003) y `repeat_last_without_history_opens_the_panel` (CAP-007).
- `crates/xs-core`: perfil de compatibilidad (Capturar pasa a `PrtScn` en Windows) y nombres propios de los atajos (`shortcut_rows_with_their_own_name_are_translated`).

Carlos marcó como PASS (2026-10-18, instalador 0.2.0 `9c46348`) las casillas [x] de las secciones de abajo.

Pruebas con la app en marcha (CLI y entrada simulada), 2026-10-18, Windows 11, un monitor 5120×2160 al 150 %:
- [x] `capture panel --show true` abre el panel; su alto sigue al contenido (ampliado y compacto, sin barra de desplazamiento).
- [x] Un cambio desde el CLI (`capture configure`) se ve al momento en el panel abierto.
- [x] Clic en **Capturar** con Imagen = Pantalla completa: captura 5120×2160; el panel no aparece en la imagen y vuelve a mostrarse al terminar.
- [x] Universal con Selección = Pantalla completa: la selección empieza con el monitor entero y `Enter` captura 5120×2160.
- [x] `--delay 3`: la captura llega 3,0 s después; el contador se muestra abajo a la derecha del área de trabajo y no aparece en la captura.
- [x] `Esc` durante la cuenta atrás cancela al instante (`cancelled`, sin captura).
- [x] `capture repeat-last` repite la última captura con el mismo modo y ajustes.

## Panel (CAP-001, CAP-002, CAP-010) — Carlos
- [x] Clic izquierdo en el ícono de la bandeja abre el panel (Preferencias > General > Clic en el ícono = Abrir el panel de captura).
- [x] Pestañas **Universal** e **Imagen**; con el teclado: `Tab` hasta las pestañas, flechas para cambiar, `Tab` a las opciones, `Enter` en **Capturar**.
- [x] Imagen, Selección = Ventana → **Capturar** inicia una captura de ventana (clic sobre una ventana).
- [x] La flecha ▴/▾ alterna entre vista compacta y ampliada; el panel lo recuerda.
- [x] Cerrar XcreenShot (Salir) y volver a abrirlo: se conservan la pestaña activa, los ajustes de cada pestaña y la posición del panel.
- [x] **Capturar cursor** aparece desactivado y su tooltip explica que llega con el editor.
- [x] El campo **Atajo** muestra `Ctrl+Shift+PrtScn`; al cambiarlo, la combinación nueva funciona al instante y la anterior deja de funcionar.
- [ ] Volver a grabar `Ctrl+Shift+PrtScn` en el campo Atajo (y en Preferencias > Atajos). Fallaba: Windows solo avisa de `PrtScn` al soltarla; corregido, falta confirmarlo.
- [ ] Al cambiar de pestaña con las flechas, el anillo de foco queda dentro de la pestaña (antes asomaba un pedacito turquesa en la vecina).

## Más (⋯) y restablecer (CAP-003)
- [ ] **Preferencias de captura…** abre Preferencias en la sección Captura.
- [x] El menú ⋯ se ve entero en la vista ampliada y en la compacta, y se cierra al pulsar ⋯ otra vez (`f7906cf`).
- [x] **Restablecer ajustes de captura…** pide confirmación; **Cancelar** no cambia nada; **Restablecer** deja todo como al principio (Región, Editor, Copiar sí, Capturar cursor no, Retardo de 5 s no, Tiempo Ninguno con 5 s guardados).

## Retardo (CAP-048)
- [x] Imagen, Tiempo = Retardo, 3 s: la cruceta aparece 3 s después de **Capturar**.
- [x] Durante la cuenta atrás se puede abrir un menú (por ejemplo el menú Inicio o un menú de una app) y sigue abierto cuando termina; sale en la captura.
- [x] El botón **Cancelar** del contador cancela sin robar el foco a lo que estabas haciendo.
- [ ] Universal con **Retardo de 5 s**: la cruceta aparece a los 5 s.

## Menú del ícono (CAP-005) y atajos (CAP-006, CAP-007, CAP-065)
- [x] Clic derecho en el ícono: arriba aparecen Captura universal, Captura de imagen, Repetir última captura y Abrir panel de captura, cada una con su atajo.
- [ ] **Captura universal** desde el menú muestra la cruceta enseguida.
- [x] `Ctrl+Shift+PrtScn` con el panel cerrado captura con la pestaña activa.
- [x] `Ctrl+Alt+Shift+P` muestra el panel; pulsado otra vez con el panel delante, lo oculta.
- [x] `Ctrl+Alt+Shift+W` (Captura de ventana) y `Ctrl+Alt+Shift+A` (Ventana activa) usan las opciones de la pestaña Imagen (por ejemplo, Abrir en = Nada no abre la ventana principal).
- [x] `Ctrl+Alt+Shift+R` repite la última captura (mismo tipo y ajustes; en región se vuelve a elegir el área).
- [ ] **Repetir última área** (sin atajo por defecto; se asigna en Preferencias > Atajos): tras una captura de región, captura al instante el mismo rectángulo, sin cruceta. Tras una de ventana, captura esa ventana donde esté ahora.
