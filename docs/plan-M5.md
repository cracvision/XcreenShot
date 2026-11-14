# Plan de M5: funciones inteligentes (propuesta)

Propuesta de Claude del 2026-11-13 (noche), **provisional hasta que Carlos la apruebe**. M5 tiene 38 IDs (`SMT-001` a `SMT-038`, en `spec/06-inteligentes.md`); la IA en la nube (`SMT-039` a `SMT-043`) ya está hecha. Como en M2 a M4, cada entrega termina con pruebas automáticas, `PARITY.md`, `QUESTIONS.md` con las decisiones provisionales y un instalador probado en su forma empaquetada. Todo es local: el reconocimiento de texto es el del sistema (SMT-001).

| Entrega | Contenido | Estado |
|---|---|---|
| M5.1 · Texto en el editor | Extraer texto en el editor (SMT-007) y de un fotograma de video (SMT-008), activar y desactivar las funciones inteligentes (SMT-002) | SMT-007 y SMT-008 hechos en Windows (2026-11-13, noche); falta SMT-002 |
| M5.2 · Redacción automática | Analizar la imagen, categorías de datos sensibles, palabras y patrones propios, estilo, añadir y quitar con un clic, todas las coincidencias, varias capturas a la vez, rendimiento y privacidad (SMT-010 a SMT-017) | SMT-010 a SMT-013 y SMT-017 hechos en Windows (2026-11-13, noche); faltan el clic para añadir y quitar (SMT-014), Redactar todas las coincidencias (SMT-015) y varias capturas a la vez (SMT-016) |
| M5.3 · Retoques automáticos | Aplicar solo al texto (SMT-021), Simplificar automático (SMT-019), Quitar fondo (SMT-020), Separar elementos (SMT-018) | — |
| M5.4 · Editar texto de la imagen | Reconocer, borrar el original y poner un texto editable igual (SMT-009) | — |
| M5.5 · Grabador de pasos ([ADR 0009](adr/0009-guias-de-pasos.md), propuesta) | En cuatro partes. **a)** El grabador: iniciarlo, su barra, qué crea un paso, su texto y su imagen, límites y recuperación, y una guía básica al terminar (SMT-022 a SMT-028). **b)** Editar la guía: panel de pasos, negritas, candado, escala, capturar en una sección vacía, grabar más pasos (SMT-029). **c)** Exportar a Word, PowerPoint, Markdown, HTML, PDF, imágenes y portapapeles (SMT-030, SMT-032 a SMT-034). **d)** Guías en la biblioteca (SMT-035) | En curso (2026-11-14): partes a y b hechas en Windows (falta unir y dividir pasos) |
| M5.6 · Códigos QR | Crear un QR (SMT-036, SMT-037) y leerlos (SMT-038) | — |

## Para decidir con Carlos

- Aprobar este orden, o cambiarlo. La redacción automática (M5.2) va primero porque es la más útil para compartir capturas sin datos personales.
- **QR:** crear uno se puede hacer con código propio; **leerlos** no tiene ayuda del sistema en Windows. Se propondrá en `QUESTIONS.md` un lector en Rust puro (por ejemplo el crate `rqrr`) cuando llegue M5.6.
- **Quitar fondo y Separar elementos (M5.3):** sin modelos de IA locales, se propondrá un método clásico (bordes y colores parecidos), bueno para capturas de pantalla y peor para fotos.
- **Word y PowerPoint del grabador de pasos (SMT-030):** propuesto en el [ADR 0009](adr/0009-guias-de-pasos.md): se escriben con el crate `zip` que ya usa la app (son ZIP con XML), sin dependencias nuevas; el PDF con texto buscable usa las fuentes estándar de PDF.
