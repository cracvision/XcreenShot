# Barra en una ventana estrecha (arreglo tras la caminata) · EDT-001, EDT-003, LIB-002

Lo encontró Carlos en la caminata del 2026-11-12: con tantas herramientas, al estrechar la ventana los íconos se montaban unos sobre otros.

Pruebas automáticas:
- `app/src/components/fit.test.ts`: la barra sube de nivel mientras su contenido es más ancho que ella, baja cuando el nivel de abajo cabe (o lo prueba una vez) y nunca se queda subiendo y bajando.

Visto en la instancia de desarrollo (2026-11-12), simulando anchos de 1920, 1600, 1366, 1100 y 900 px, estrechando y agrandando: en ninguno hay botones montados ni cortados en el borde.
- Editor a 1366–1920 px: Capturar y Editor/Biblioteca con su nombre; las herramientas solo con íconos. A 900 px: Recortar y Selección pasan a **Más** (en Píxeles e Imagen); Abrir, Guardar, el idioma y Preferencias, al menú **⋯**.
- Biblioteca a 1366 px: el buscador algo más estrecho y todo lo demás visible. A 900 px: Crear, Etiquetas, Detalles, el idioma, Preferencias, Abrir carpeta y Abrir, en su menú **⋯**.

## Para Carlos (en la próxima caminata)

1. [ ] Achica la ventana del editor poco a poco hasta su mínimo: nada se monta; primero desaparecen los nombres de las herramientas y después los textos; al final, lo que no cabe está en **Más** y en **⋯**.
2. [ ] Agrándala otra vez: todo vuelve a su sitio.
3. [ ] Lo mismo en la vista Biblioteca.
