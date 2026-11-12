# ADR 0005 · Las herramientas de píxeles editan la imagen con parches

**Estado:** propuesta (pendiente de Carlos) · **Fecha:** 2026-11-12 · **IDs:** EDT-043 a EDT-048, EDT-091, EDT-092, EDT-106

## Contexto

Hasta M2.2 la imagen original (`base.png`) no cambia nunca: todo lo que el usuario añade son objetos encima, y lo aplanado (EDT-091) son capas que se dibujan sobre ella. Deshacer guarda el documento (`document.json`) de antes de cada paso y basta.

M2.3 trae herramientas que **cambian píxeles**: Relleno (EDT-047), Borrador (EDT-048), y la Selección con sus acciones (EDT-043, EDT-044): mover, borrar o cortar píxeles, rellenar el hueco (EDT-045), Varita mágica (EDT-046). Además:

- EDT-091 dice que pintar, borrar o rellenar sobre una zona aplanada impide desaplanarla.
- EDT-092 pide al menos 100 pasos de Deshacer, también para estas herramientas.
- Un borrado o un relleno suelen tocar una parte pequeña de la imagen; copiar la imagen entera en cada paso gastaría mucha memoria (una captura 4K son 33 MB sin comprimir).

## Decisión

1. **Cada edición de píxeles es un parche.** Un parche es un rectángulo de la imagen, en píxeles de `base.png` (un recorte no lo mueve), con sus píxeles **ya resueltos**: lo que se ve allí después de la edición. Se guarda como una imagen más de la carpeta `images/` y el documento lleva la lista `patches` (el más antiguo primero).
2. **Orden de dibujo:** color del lienzo → `base.png` → capas aplanadas → parches (cada uno **sustituye** su rectángulo, también con transparencia) → objetos. Así un parche ve lo aplanado de debajo, y una herramienta que trabaja "sobre la imagen" ve lo mismo que el usuario.
3. **Lo aplanado bajo un parche queda fijo.** Si una edición toca una capa aplanada que se podía desaplanar, esa capa pasa a ser permanente con el motivo "pintado" (EDT-091), porque el parche ya contiene sus píxeles.
4. **Deshacer sigue siendo el documento.** Añadir o quitar un parche cambia `document.json`; las imágenes de `images/` no se borran mientras algún paso de Deshacer las use. No hace falta guardar la imagen entera en cada paso: solo la zona tocada.
5. **Al guardar, los parches se funden en `base.png`**, junto con las capas aplanadas permanentes, y el archivo queda sin parches. Lo borrado no viaja en el `.xshot` (lo mismo que con un desenfoque aplanado).
6. **Imágenes flotantes:** editarlas crea una imagen nueva en `images/` y el objeto pasa a apuntar a ella. Deshacer devuelve el nombre anterior.
7. **El editor muestra la imagen resultante:** el núcleo sirve `base.png` con capas y parches ya aplicados (como hoy con lo aplanado), y la interfaz no reproduce las ediciones de píxeles.

## Consecuencias

- El formato `.xshot` gana el campo `patches` (solo en documentos abiertos con cambios sin guardar y en las copias de recuperación; un archivo guardado no lo trae).
- "Volver al original" (M2.5) no puede contar con los píxeles originales de un archivo ya guardado tras borrar: hay que decidirlo cuando se especifique.
- Muchas ediciones grandes seguidas (rellenar fondos enteros) ocupan memoria hasta que salen del historial de Deshacer. Se medirá en M2.3 y, si hace falta, se comprimirán los parches viejos.
- No hay dependencias nuevas.
