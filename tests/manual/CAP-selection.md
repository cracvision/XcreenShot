# Selección en pantalla (M1, entrega 2) · CAP-009, CAP-011, CAP-012, CAP-013, CAP-015, CAP-016, CAP-017, CAP-018, CAP-021, CAP-025, CAP-027, APP-048

Pruebas automáticas: geometría en `app/src/overlay/geometry.test.ts` (tamaños exactos, proporciones, tiradores, barra, lupa) y recorte de los fotogramas en `crates/xs-capture` (`crop_has_the_exact_selected_size`, cruce de monitores).

Pruebas con entrada simulada (SendInput) contra la app en marcha: `xcreenshot capture region|universal|window --target interactive` y un arrastre o clic del ratón.

## Rendimiento (CAP-013, APP-048)
- [x] Del comando a la cruceta visible: ~105 ms en un monitor 5120×2160 (congelar ~80 ms + mostrar ~25 ms; ADR 0003).
- [ ] Con 3 monitores 4K: ≤ 300 ms en el percentil 90 de 10 intentos.
- [ ] La cruceta sigue al puntero sin retraso visible.

## Región (CAP-025) y universal (CAP-009)
- [x] Arrastrar de (1000, 800) a (1301, 1001) produce exactamente 301×201 px en esa posición.
- [x] Universal: arrastrar 640×480 → barra de captura → `I` produce 640×480 px.
- [x] `Esc` cancela: el CLI recibe `cancelled` y no queda ninguna captura.
- [x] Clic sin arrastrar sobre una ventana (sin mover el ratón antes) selecciona su rectángulo.
- [ ] Clic sin arrastrar sobre el escritorio sin ventanas no captura nada.
- [ ] Con un tooltip abierto al pulsar el atajo, el tooltip sale en la captura aunque desaparezca durante la selección.

## Ventana (CAP-027)
- [x] `capture window --target interactive` y clic sobre una ventana: la captura completa de esa ventana (2551×2088, Claude).

## Visual (Carlos)
- [ ] La cruceta (líneas finas a todo lo ancho y alto) se ve sobre fondo blanco y sobre fondo negro.
- [ ] Fuera de la selección la pantalla se oscurece; la ventana bajo el puntero se resalta con borde discontinuo.
- [ ] El rótulo junto al puntero muestra la posición y, al arrastrar, el tamaño en píxeles.
- [ ] La lupa muestra la zona ampliada con rejilla, posición y color; `M` la oculta/muestra, `+`/`-` cambian el aumento y `C` copia el color.
- [ ] En Universal, la barra de captura aparece bajo la selección, con X, Y, Ancho y Alto editables; escribir Ancho = 1280 y Alto = 720 + `Enter` produce 1280×720.
- [ ] Tiradores: cambiar el tamaño por las esquinas y mover la selección arrastrando dentro.
- [ ] `Shift` al arrastrar da un cuadrado; `Alt` 16:9; `Alt+Shift` 4:3.
- [ ] Flechas mueven la selección 1 px (`Shift` 10 px); `Alt` + flechas cambian el tamaño.
- [ ] `F` o el botón **Pantalla completa** seleccionan el monitor entero.
- [ ] **Rehacer** (o `Retroceso`) vuelve a la cruceta sin cerrar la captura.

## Pendiente en esta entrega
- Subáreas (contenido del navegador, paneles) en la detección (CAP-016): hoy solo ventanas.
- Teclado en la fase de apuntado (mover el puntero con flechas, `Shift+Enter`), guías `G`, ayuda `H`, proporción en la barra.
- Varios monitores y escalas mixtas: probado solo con un monitor.

## Resultados
- Windows 11 (26200), modo desarrollo, 2026-10-13: lo marcado arriba. Fallos encontrados y corregidos: desfase de 1 px por redondeo al 150 %; clic sobre una ventana sin mover antes el ratón no la detectaba; tiempo de aparición de ~370 ms (rehecho con ventana nativa, ADR 0003).
