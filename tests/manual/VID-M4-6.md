# El video en el editor (M4.6)

Reproducir, seleccionar, cortar y guardar videos en el editor ([ADR 0008](../../docs/adr/0008-video-en-el-editor.md)). IDs: VID-052 y VID-056 a VID-070.

## Visto en la instancia de desarrollo (2026-11-13, tarde)

- Pruebas automáticas:
  - Motor (`xs-video`), con un video de prueba de 3 s (rojo, verde y azul, un segundo cada uno, con un tono):
    - el fotograma de un instante tiene su color;
    - cortar el segundo verde junta el rojo y el azul, dura 2 s y el sonido también;
    - la copia sin sonido no tiene pistas;
    - un tramo sin fotogramas propios (pantalla quieta) conserva el de antes;
    - cancelar no deja archivo.
  - Editor (`a_video_in_the_editor`):
    - las herramientas de imagen se niegan explicando la alternativa;
    - cortar, deshacer y rehacer;
    - no se puede cortar todo;
    - el fotograma de 1,2 s del video cortado es azul y queda en la biblioteca;
    - el primer segundo como video nuevo;
    - la copia sin sonido;
    - al guardar, el archivo dura 2 s y no queda nada por deshacer.
  - Línea de tiempo: los mismos casos en Rust y en TypeScript.
- En la ventana (sin tocar el ratón real, por el protocolo de depuración):
  - el video se reproduce y el tiempo avanza;
  - diez veces → desde 0:00 deja el cabezal en 0:01.0;
  - `I` en 1,0 s y `O` en 2,0 s marcan "Selección 00:01.0–00:02.0 (1.0 s)";
  - `Supr` corta: queda "00:01.0 / 00:02.0", la marca del corte y el paso de Deshacer "Cortar";
  - Guardar deja el archivo en 2,03 s con sonido y el reproductor recarga el video ya cortado;
  - **Guardar fotograma** abre una imagen de 320 × 240 en el editor.
- **Abrir en = Editor:** al terminar una grabación de 1 s, el video queda a la vista en el editor.

## Para Carlos (en la próxima caminata)

1. [ ] Graba un video de unos 20 s (Captura de video). Al detenerlo, se abre en el editor, pausado en su primer fotograma, con la barra de reproducción encima de la barra de estado. Las herramientas de imagen aparecen apagadas; pasa el ratón por la Flecha: el aviso dice que uses Guardar fotograma.
2. [ ] **Reproducir** (o `Espacio`): se ve y se oye; la barra muestra el tiempo. Un clic sobre el video pausa. Prueba la velocidad 2× (el tono de la voz no cambia) y Silenciar.
3. [ ] Arrastra el cabezal: el video muestra el fotograma de cada punto. Las flechas avanzan y retroceden de fotograma en fotograma; mantener pulsado el botón ⏭ avanza seguido.
4. [ ] Haz clic en el tiempo, escribe `0:10` y `Enter`: el cabezal va al segundo 10.
5. [ ] Selección: arrastra el tirador azul (inicio) y el amarillo (fin); se ve el fotograma de cada tirador y un rótulo con el tiempo y la duración. Cerca del cabezal, el tirador se pega a él. También con `I` y `O` en el cabezal. `Esc` vuelve a todo el video.
6. [ ] **Cortar** (o `Supr`): el tramo desaparece, queda una marca en la línea de tiempo y el video se reproduce saltando el corte. `Ctrl+Z` lo devuelve; `Ctrl+Y` lo vuelve a quitar.
7. [ ] Con todo el video seleccionado, **Cortar** está apagado.
8. [ ] **Guardar fotograma**: se abre en el editor una imagen igual a lo que se veía, que puedes anotar.
9. [ ] Menú **⋯** de la barra > **Guardar selección como video**: aparece en la bandeja un video nuevo con solo ese tramo; el original sigue igual.
10. [ ] Menú **⋯** > **Extraer texto del fotograma** con texto a la vista: aparece la ventana con el texto; no se añade ninguna imagen a la biblioteca.
11. [ ] Menú **⋯** > **Copia sin sonido** (con un video con sonido): aparece "<nombre> (sin sonido)" en la bandeja; al reproducirlo no suena.
12. [ ] Panel lateral: la información del video (tamaño, duración, fps, códec, sonido, tamaño del archivo…) y **Copiar información**.
13. [ ] Con un corte sin guardar, cierra el video: pregunta si guardar. **Guardar** tarda un poco (vuelve a codificar el video, con su progreso en la barra); después el archivo dura menos.
14. [ ] Pestaña **Video** del panel > **Abrir en** = Nada: al detener una grabación no se abre ninguna ventana y aparece la notificación; al hacer clic en ella, el video se abre en el editor.
15. [ ] En la Biblioteca, doble clic en un video: se abre en el editor (ya no en el reproductor de Windows).
16. [ ] **Copiar todo** con un video y pega en una carpeta del Explorador: aparece el MP4. Con un corte sin guardar, el MP4 pegado ya va cortado.
17. [ ] Arrastra el video desde la bandeja a una carpeta: se copia el MP4 al momento. **Duplicar** (clic derecho en la bandeja) crea una copia en la biblioteca.
18. [ ] Menú **Video** de la barra de menús: Reproducir/Pausar, fotogramas, Ir al inicio y al final, **Ir a…**, Cortar selección, Guardar selección, Guardar fotograma, Extraer texto, Copia sin sonido e Información. Con una imagen a la vista, todo el menú está apagado; con un video, el menú **Imagen** está apagado.
19. [ ] Abre cualquier menú (Archivo, Objeto…): se ve entero, por encima de la barra de herramientas (antes, la barra tapaba sus primeras entradas).

### GIF animado (VID-068 a VID-070)

20. [ ] Con un video abierto, botón **GIF** de la barra (o Video > Crear GIF animado…): el diálogo muestra Selección actual / Video completo, Optimizar para, la estimación ("Unos … · … s · … × … px · … fps") y una vista previa que se mueve.
21. [ ] Cambia Optimizar para a **Archivo pequeño**: la estimación baja y el tamaño pasa a 200 px de lado mayor. Con un tramo de más de 30 s aparece el aviso de duración.
22. [ ] **Personalizado**: 50 %, Difuminado, Repetición 3 veces, Fundido a negro 1 s. **Crear GIF**: la barra muestra "Creando el GIF… %" con **Cancelar**; mientras tanto puedes hacer una captura. Al terminar, el GIF se abre en el editor y se reproduce 3 veces, oscureciéndose al final.
23. [ ] Repite y pulsa **Cancelar** a mitad: no aparece ningún GIF en la biblioteca.
24. [ ] Con el GIF abierto, el botón GIF y el menú Video (salvo Información) están apagados; arrástralo a un chat o a una carpeta.

### Combinar videos (VID-067)

25. [ ] Con un video abierto, botón **+** de la barra (o Video > Combinar videos…): arriba, los videos de la biblioteca; abajo, el abierto como 1. Filtra escribiendo parte de un nombre y cambia el orden.
26. [ ] Haz clic en otro video para añadirlo; reordénalos con ← → (o `Ctrl+←/→` con uno elegido) y quita uno con ✕. Abajo se ven la duración total y el tamaño del resultado.
27. [ ] **Combinar** con un video grande (1920 × 1080) y uno pequeño: el resultado se abre en el editor; el pequeño sale centrado con bandas negras, y los originales no cambian. Prueba también **Cancelar** a mitad: no queda ningún video nuevo.

Todavía no: GIF desde Exportar, exportar a H.264, y Combinar desde la bandeja con varios elegidos.
