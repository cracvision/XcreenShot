# ADR 0006 · Cambiar tamaño, girar y voltear sustituyen los píxeles originales con una imagen

**Estado:** propuesta (pendiente de Carlos) · **Fecha:** 2026-11-12 · **IDs:** EDT-069, EDT-072, EDT-073, EDT-076, EDT-092

## Contexto

Las herramientas de píxeles (ADR 0005) cambian zonas de la imagen con parches. Pero **Cambiar tamaño de la imagen** (EDT-069), **Girar** (EDT-072) y **Voltear** (EDT-073) cambian la imagen entera y, en los dos primeros casos, sus dimensiones. Un parche no puede hacerlo, y Deshacer solo guarda el documento (`document.json`), no `base.png`.

**Volver al original** (EDT-076) también tiene que poder deshacerse (decisión del spec).

## Decisión

1. El documento gana el campo opcional **`baseImage`**: el nombre de una imagen de `images/` que sustituye a `base.png` como píxeles originales. Sin él, los originales son `base.png`, como hasta ahora.
2. Cambiar tamaño, girar o voltear calculan la imagen nueva (los originales con los parches ya aplicados), la guardan en `images/` y ponen su nombre en `baseImage`. En el mismo paso se ajustan el recorte, los objetos y las capas aplanadas, y los parches se vacían (ya están en la imagen nueva).
3. **Deshacer** devuelve el `baseImage` anterior (o ninguno): es parte del documento, como cualquier otro cambio.
4. **Al guardar**, la imagen de `baseImage` (con lo aplanado permanente y los parches) se escribe como `base.png` y el campo desaparece del archivo.
5. **Volver al original** pone un documento nuevo sobre `base.png` del archivo (sin objetos, efectos, recortes ni `baseImage`), con su resolución de origen; los metadatos se conservan. Es un paso de Deshacer más.

## Consecuencias

- Un `.xshot` guardado nunca lleva `baseImage`; solo aparece en documentos abiertos con cambios sin guardar y en las copias de recuperación.
- Tras guardar, "el original" es el `base.png` guardado: si se cambió de tamaño y se guardó, Volver al original devuelve ese tamaño. El diálogo lo avisa igual que con el contenido recortado (EDT-050).
- Cada cambio de tamaño o giro guarda una imagen entera en memoria mientras siga en el historial de Deshacer.
- No hay dependencias nuevas.
