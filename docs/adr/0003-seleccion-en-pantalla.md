# ADR 0003 · Selección en pantalla: imagen congelada nativa y capa web transparente

**Estado:** aceptada · **Fecha:** 2026-10-13 · **IDs:** CAP-009, CAP-013, CAP-015, APP-048, APP-053

## Contexto

CAP-013 y APP-053 exigen que la cruceta aparezca en **≤ 300 ms con hasta tres monitores 4K** y siga el puntero a ≥ 60 fps, sobre una imagen congelada de todas las pantallas.

Primera versión medida en Windows (un monitor 5120×2160): congelar ~80 ms + **pasar la imagen (44 MB) a la ventana web ~260 ms** + dibujarla ~12 ms ≈ **350 ms**. Con tres monitores 4K el paso de la imagen superaría los 700 ms.

## Decisión

1. **Congelar** en `xs-capture` (`interactive::freeze`): una imagen RGBA por monitor, la lista de ventanas para la detección y la posición del puntero.
2. **Ventanas de selección precreadas**, ocultas desde el arranque: una por monitor (`overlay-<n>`), sin bordes, siempre encima y excluidas de cualquier captura o pantalla compartida.
3. **Windows: la imagen congelada la pinta una ventana nativa** (`freeze_win.rs`, GDI `SetDIBitsToDevice`) colocada justo debajo de la ventana de selección, que es **transparente**. La conversión RGBA→BGRA se hace en paralelo, fuera del hilo de la interfaz y en código optimizado.
4. **La lupa** pide al shell solo los píxeles alrededor del puntero (`xs_frame_patch`, unos KB por movimiento), así que el color y la rejilla son exactos sin transferir la imagen entera.
5. **macOS (pendiente)**: de momento la ventana de selección recibe la imagen por el protocolo `xsframe` (lo bastante rápido para pruebas). El equivalente nativo será una `NSWindow` con la imagen como contenido de su capa, debajo del `WKWebView` transparente.
6. **La imagen final sale de los mismos fotogramas congelados** (`crop_frames`). Así, lo que desaparece mientras el usuario elige (tooltips, menús) sale en la captura. La captura de ventana es la excepción: se toma en vivo para obtener la ventana entera aunque esté tapada (CAP-027).

## Resultado medido

Windows 11, un monitor 5120×2160 al 150 %, compilación de desarrollo: congelar ~80 ms + mostrar ~25 ms ≈ **105 ms** del comando a la cruceta visible.

## Consecuencias

- El rendimiento ya no depende del tamaño de los monitores, salvo el propio congelado.
- Dos ventanas por monitor durante la selección: la nativa no recibe entrada ni foco.
- Las capturas de pantalla de la propia selección (para pruebas o soporte) no la muestran, porque está excluida de la captura a propósito.
