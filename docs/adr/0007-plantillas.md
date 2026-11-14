# ADR 0007 · Plantillas: un documento normal con secciones, zonas y objetos con nombre

**Estado:** propuesta (pendiente de Carlos) · **Fecha:** 2026-11-12 · **IDs:** EDT-095 a EDT-101

## Contexto

Las plantillas (EDT-095 a EDT-101) colocan varias imágenes en un diseño con título, números, leyendas y pie, que se puede reordenar por secciones y cambiar con ajustes (tema, numerar, invertir el orden…). El resultado tiene que ser un `.xshot` normal (EDT-095), exportarse y copiarse como cualquier documento, y deshacerse paso a paso. Una plantilla propia es un `.xshot` con zonas marcadas (SPEC §7, `.xtemplate`).

## Decisión

1. **Una plantilla es un documento normal** con un campo opcional `template` en `document.json`: qué plantilla es (`id`), sus **ajustes** (EDT-097) y sus **secciones** apiladas (`[{id, kind, height}]`, EDT-099). Sin ese campo, el documento no es una plantilla.
2. **Zonas de imagen**: un tipo de objeto nuevo, `zone` (`rect`, `image` opcional, `fit` = encajar o llenar, `zoom` y `offset` del encuadre). Recorta su imagen a su rectángulo. Una zona vacía no se dibuja al exportar, copiar ni imprimir, ni su leyenda ni su número (EDT-098); el editor la muestra con su botón **Capturar**.
3. **Todo lo demás son objetos de siempre** (texto, paso, forma, línea), editables directamente. Los que pone la plantilla llevan un **nombre de lugar** (`slot`): `title`, `footer`, `zone:<sección>:<n>`, `caption:<sección>:<n>`, `number:<sección>:<n>`, etc.
4. **Rehacer el diseño**: cada cambio de ajustes o de secciones vuelve a generar la disposición en el núcleo y **conserva lo del usuario**: el texto de cada lugar, la imagen y el encuadre de cada zona, y las anotaciones sin lugar, que se mueven con la sección donde están. Los números se calculan por el orden de las zonas, así que se renumeran solos.
5. El fondo de la plantilla es el **color del lienzo** (EDT-074); la imagen base es un píxel transparente y el lienzo la sobrepasa (como al ampliar el lienzo, EDT-070).
6. Las plantillas incorporadas (EDT-096) se generan en Rust a partir de su familia y del número de imágenes, con los colores y la fuente del tema activo. Las propias (`.xtemplate`, EDT-101) son un `.xshot` con `template` y zonas, en la carpeta de plantillas del usuario.
7. Cada operación es un comando (`document.from_template`, `document.template`, `editor.templates`…) y un solo paso de Deshacer.

## Consecuencias

- Exportar, copiar, imprimir, efectos y Deshacer funcionan sin código propio para plantillas, salvo omitir las zonas vacías al dibujar la imagen final.
- Una app anterior que abra una plantilla ve los objetos y las zonas como objetos desconocidos (los conserva, EDT-106) y pierde la posibilidad de rehacer el diseño.
- No hay dependencias nuevas.
