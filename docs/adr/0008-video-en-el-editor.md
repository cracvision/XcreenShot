# ADR 0008 · Videos en el editor: un documento más, con cortes y un reproductor del sistema

**Estado:** propuesta (pendiente de Carlos) · **Fecha:** 2026-11-13 · **IDs:** VID-052, VID-055 a VID-066

## Contexto

Los videos grabados (M4.1) van a la biblioteca como MP4, pero hasta ahora se abrían con el reproductor de Windows. El spec pide que se abran en el editor (VID-052, VID-056) con una barra de reproducción, una selección en la línea de tiempo (VID-058), cortes que se deshacen (VID-059), guardar un tramo como otro video (VID-060), guardar un fotograma como imagen (VID-061), extraer texto de un fotograma (VID-062), una copia sin sonido (VID-063), información (VID-064), herramientas de imagen desactivadas (VID-065) y guardar, exportar, copiar y arrastrar (VID-066). La bandeja, la tira, el aviso de cambios sin guardar, Deshacer y Guardar ya existen para las imágenes.

## Decisión

1. **Un video abierto es un documento del editor**, con el formato nuevo `video` (`.mp4`, `.m4v`, `.mov`). Así entra en la tira y en la bandeja, se fija, se etiqueta y se cierra como una imagen. Su documento interno es de tipo `video`, del tamaño del video, sin objetos; su imagen base es el primer fotograma (para las miniaturas).
2. **Los cortes son parte del documento**: una lista de tramos quitados, en segundos del archivo (`cuts`). Cortar es un paso de Deshacer como cualquier otro, así que Deshacer y Rehacer no necesitan código propio. Los cortes no se escriben en ningún `.xshot`: viven mientras el video está abierto.
3. **El reproductor es el del sistema**: la etiqueta `<video>` de WebView2 (Edge), que ya decodifica H.264, H.265 (si el sistema tiene el códec) y AAC con aceleración por hardware. El archivo le llega por el protocolo propio `xsdoc` (`/<documentId>/video`), con peticiones por rangos para buscar; solo sirve el archivo de un documento abierto, nunca una ruta cualquiera. Mientras hay cortes, el reproductor salta los tramos quitados, y la línea de tiempo muestra el video ya cortado.
4. **La línea de tiempo** (qué queda, pasar de un tiempo del video cortado a uno del archivo y al revés, convertir una selección en tramos del archivo) está en Rust (`xs_video::timeline`) y su copia en TypeScript (`editor/videoTimeline.ts`), con pruebas de los mismos casos en los dos lados, como la geometría de los objetos.
5. **Crear un video nuevo** (guardar con cortes, guardar un tramo, copia sin sonido) lo hace `xs_video` con Media Foundation: lee el archivo, decodifica, quita los tramos y vuelve a codificar en H.264 y AAC con el codificador del sistema (por hardware si lo hay). Sin FFmpeg. Un fotograma en un instante sale del mismo lector.
6. **Cada acción es un comando** (`video.cut`, `video.save_selection`, `video.save_frame`, `video.extract_text`, `video.silent_copy`, `video.info` con documento), con su ID del spec. Reproducir, pausar, ir a un fotograma y la selección son estado de la ventana: el menú Video y los atajos los envían al editor con `editor.action`, como las demás acciones de vista.
7. **Guardar** un video con cortes escribe el video cortado en un archivo temporal y después sustituye el de la biblioteca; Deshacer ya no vuelve más atrás de ese guardado (provisional). Al cerrar con cortes sin guardar se aplica la misma regla que a una imagen.

## Consecuencias

- La bandeja, la tira, Cerrar, el aviso de cambios y Deshacer funcionan para videos sin código nuevo; las operaciones de imagen se desactivan mirando el formato del documento.
- Guardar con cortes vuelve a codificar todo el video: la calidad baja muy poco (se codifica con más tasa de bits que la grabación) y tarda más que los 5 s para 10 min que pide VID-059. Hacer los cortes sin volver a codificar (cortar en fotogramas clave y codificar solo los trozos de los bordes) queda como mejora posterior, anotada en `spec/QUESTIONS.md`.
- No hay dependencias nuevas.
