# Editar la guía (M5.5b)

Una guía de pasos se edita como una plantilla, con un panel de pasos propio. IDs: SMT-029 (y SMT-027, la memoria de las guías largas). Diseño: [ADR 0009](../../docs/adr/0009-guias-de-pasos.md).

## Visto en la instancia de desarrollo (2026-11-14)

- Pruebas automáticas (`xs-editor`, `xs-image`):
  - mover el paso 4 delante del paso 2 numera del 1 al 4 y su flecha sigue en el mismo sitio de su imagen;
  - un texto largo empuja su imagen hacia abajo y la flecha va con ella;
  - duplicar un paso copia su imagen, su texto y su flecha; añadir un texto y una imagen (la vacía no sale en la imagen final);
  - las imágenes al 50 %: la mitad de anchas y la flecha a la mitad, en el mismo sitio;
  - borrar dos pasos a la vez; Grabar más pasos aquí pone los nuevos detrás del elegido y un Deshacer los quita;
  - 60 imágenes de 2000×2000 no pasan de 512 MB descodificados.
- En la app: una guía de prueba muestra la pestaña **Pasos** con cada paso en miniatura y numerado, y Diseño con el tamaño de las imágenes, el autor y el tema; al 50 % las imágenes se ven a la mitad con sus anillos en su sitio.

## Para Carlos (en la próxima caminata)

1. [ ] Abre una guía grabada. En el panel lateral, pestaña **Pasos**: cada paso con su miniatura y su texto.
2. [ ] Haz clic en un paso: la vista se centra en él. Con los botones, súbelo y bájalo: los números se recalculan solos y sus anotaciones van con él.
3. [ ] **Duplicar** un paso y **Eliminar** otro. Con Ctrl+clic elige dos y pulsa `Supr`: se van los dos. `Ctrl+Z` los devuelve.
4. [ ] **+ > Texto** añade una sección de texto sin número; **+ > Imagen**, una imagen vacía con su botón **Capturar**; captura algo y queda dentro.
5. [ ] Doble clic en el texto de un paso, escribe un párrafo largo: al terminar, la imagen baja para dejarle sitio. `**así**` sale en negrita.
6. [ ] **Tamaño de las imágenes** al 50 %: todas las imágenes a la mitad, con sus anillos y flechas en su sitio.
7. [ ] **+ > Grabar más pasos aquí**: graba dos clics y pulsa Terminar; aparecen detrás de ese paso, numerados con los demás.
8. [ ] Abre una guía de muchos pasos (por ejemplo, 50): se abre sin que la memoria de XcreenShot se dispare (Administrador de tareas).
