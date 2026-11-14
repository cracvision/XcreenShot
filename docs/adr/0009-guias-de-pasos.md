# ADR 0009 · Grabador de pasos y guías

**Estado:** propuesta (pendiente de Carlos) · **Fecha:** 2026-11-14 · **IDs:** SMT-022 a SMT-035

## Contexto

El Grabador de pasos (SMT-022 a SMT-027) convierte los clics del usuario en una guía paso a paso: una imagen y una frase por clic. La guía (SMT-028, SMT-029) se edita en el editor y sale a Word, PowerPoint, Markdown, HTML, PDF, imágenes y portapapeles (SMT-030 a SMT-034), y vive en la biblioteca (SMT-035). Hay que decidir cómo se detectan los clics sin molestar a la app documentada, de dónde sale el texto de cada paso y qué es una guía dentro de un `.xshot`.

## Decisión

1. **Una guía es una plantilla** (ADR 0007) con `template.id = "guide"`: encabezado (título, fecha y descripción), secciones de **paso** (número, texto encima e imagen), de **texto** y de **imagen**, y pie ("Creado por"). Así se aprovechan el apilado de secciones, los números que se recalculan solos, los temas, Deshacer, exportar y copiar. Diferencias con las demás plantillas: no tiene límite de imágenes, el texto de cada paso crece en altura lo que haga falta, y no aparece en el selector de plantillas.
2. **Imagen de cada paso:** el monitor entero, en `images/step-NNN.png`, dentro de una zona de imagen cuyo **encuadre** (zoom y desplazamiento de la zona) es el de SMT-026. Recortar cambia el encuadre y nunca pierde lo de alrededor. El puntero, el anillo del clic y la flecha de los arrastres son objetos normales de su sección.
3. **Texto con negritas:** el texto de un paso marca los nombres en negrita con `**…**`. El motor de texto (`cosmic-text`) los dibuja en negrita. Mientras se escribe, se ven los asteriscos (provisional). Las exportaciones convierten esa marca a la negrita de cada formato.
4. **Detección de clics (Windows):** un gancho de ratón de bajo nivel **que solo escucha**: nunca retiene ni cambia un clic y vuelve al instante. La única tecla que se queda es la de cancelar (`Esc` o `Mayús+Esc`, SMT-023). Los eventos pasan a un hilo de trabajo que hace todo lo lento: la imagen, la información de accesibilidad, el texto y escribir a disco. Los clics sobre ventanas de XcreenShot se descartan por el proceso dueño de la ventana. Con "Anotar el uso del teclado" se escucha también el teclado, pero solo se guarda si hubo un atajo (su nombre) o si se escribió algo (sin el contenido).
5. **Momento de la imagen:** se toma en el hilo de trabajo justo al recibir la pulsación, sin esperar a soltar. Casi siempre la app aún no ha redibujado. Una captura dentro del propio gancho la haría exacta, pero retrasaría el clic y Windows desactiva los ganchos lentos. Si en las pruebas aparecen imágenes "después" del clic, se pasará a un búfer con el último fotograma de cada monitor (duplicación del escritorio).
6. **Texto del paso:** UI Automation del sistema (`ElementFromPoint`: tipo y nombre del elemento, y su contenedor para el encuadre). Si no da nada útil, el reconocimiento de texto (SMT-003) alrededor del clic. Si tampoco, la frase genérica. Las frases salen de plantillas en `locales/{en,es}.json`, sin modelos generativos.
7. **Recuperación y memoria:** cada paso se escribe al momento en una carpeta de la grabación dentro de la carpeta de recuperación (imagen PNG y una línea de `steps.jsonl`). En memoria solo quedan los datos de cada paso, sin imágenes. Al terminar se arma el `.xshot` desde esa carpeta. Tras un cierre inesperado, Recuperar trabajo (APP-047) la ofrece.
8. **Imágenes de una guía larga:** una guía de 300 pasos no puede descodificar todas sus imágenes al abrirse (más de 2 GB en 1080p). Las imágenes de los pasos se descodificarán al dibujarse y se soltarán después, con una caché de tamaño limitado. Se hará en la entrega de edición de la guía, medido con 300 pasos.
9. **Exportaciones sin dependencias nuevas:**
   - `.docx` y `.pptx` son ZIP con XML, escritos con el crate `zip`, que ya se usa.
   - Markdown y HTML son texto.
   - El PDF con texto buscable usa las fuentes estándar de PDF (Helvetica, codificación WinAnsi), que cubren el español y el inglés sin incrustar fuentes. Un carácter fuera de ese juego sale como "?" en el texto del PDF y se anotará como límite.
10. Cada acción es un comando: `smart.steps_*` y `smart.guide_export`.

## Consecuencias

- La guía funciona como una plantilla en el editor desde el primer día. Lo propio (panel de pasos, candado, escala, unir y dividir) se añade encima.
- Un `.xshot` de guía abierto por una versión anterior se ve como una plantilla (los conserva, EDT-106).
- En macOS harán falta el permiso de Accesibilidad, un `CGEventTap` de solo escucha y la API de accesibilidad para el texto; el diseño no cambia.
- Riesgo anotado: un clic sobre una ventana elevada (Administrador de tareas) puede no llegar al gancho sin permisos de administrador. Entonces se avisa una vez y se ofrece reiniciar como administrador (SMT-027).
