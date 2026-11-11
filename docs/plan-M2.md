# Plan de M2: editor completo y biblioteca (propuesta)

Propuesta de Claude del 2026-11-11, **provisional hasta que Carlos la apruebe**. M2 tiene 127 IDs (85 del editor, 35 de biblioteca y bandeja, 7 de la app); 22 son Must. El spec de M2 ya está escrito: plantillas y exportación en `02-editor.md` y la parte de M2 de `04-biblioteca-compartir.md`.

Cada entrega termina como las de M1: pruebas automáticas, `PARITY.md`, `QUESTIONS.md` con las decisiones provisionales y un instalador. La caminata de M1 y sus arreglos van primero cuando Carlos vuelva; lo de M2 que ya esté hecho entra en la siguiente caminata.

| Entrega | Contenido | IDs Must |
|---|---|---|
| M2.1 · Objetos | Selección múltiple, transformar varios, orden de apilado, agrupar, alinear y distribuir, bloquear, posición y tamaño numéricos, duplicar, aplanar | EDT-079, EDT-086 |
| M2.2 · Herramientas de anotación | Paso, Globo, Línea, más formas, Resaltador, Lápiz, Lupa, Foco, Sellos y Emoji, Simplificar | EDT-035, EDT-028 |
| M2.3 · Selección de píxeles | Selección y sus acciones, relleno del hueco, varita mágica, relleno, borrador | EDT-043, EDT-044 |
| M2.4 · Estilos | Estilos guardados, favoritos, temas y `.xtheme`, degradados, sombra de objetos | EDT-015 |
| M2.5 · Efectos y lienzo | Panel Efectos, bordes, sombra, perspectiva, filtros, ajuste de color; cambiar tamaño, tamaño del lienzo, girar, voltear, esquinas, volver al original | EDT-053, EDT-069 |
| M2.6 · Plantillas, exportación e impresión | Plantillas y `.xtemplate`, nueva imagen, más formatos, opciones de formato, exportación por lotes, imprimir; barra de herramientas configurable, barra de edición rápida, área de trabajo | — |
| M2.7 · Biblioteca | Ordenar y lista, filtros, búsqueda, etiquetas, favoritos, detalles, vista rápida, mover la biblioteca, sincronizada, bibliotecas grandes, comandos `library.*`; bandeja: fijar, reordenar, combinar arrastrando al lienzo; exportar e importar ajustes, hoja de atajos, MCP completo | LIB-004, LIB-005, LIB-007, LIB-009, LIB-023, LIB-025, LIB-028, LIB-029, LIB-034, TRY-014, APP-050, APP-052 |

## Orden

Primero M2.1, porque la selección múltiple es la base de alinear, agrupar y de casi todas las acciones sobre varios objetos; luego M2.2 (Paso y Globo son de uso diario). M2.7 puede ir en paralelo si Carlos lo prefiere, porque toca otra parte de la app.

## Para decidir con Carlos

- Aprobar este orden, o cambiarlo.
- Si quiere una caminata intermedia (por ejemplo, tras M2.2) o una sola al final de M2.
