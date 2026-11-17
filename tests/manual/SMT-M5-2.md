# Redacción automática (M5.2)

Encontrar y tapar los datos sensibles de una captura, en el equipo: correos, teléfonos, tarjetas, documentos de identidad, direcciones, IP, URLs, fechas, rostros y los términos propios. IDs: SMT-010 a SMT-013 y SMT-017.

## Visto en la instancia de desarrollo (2026-11-13, noche)

- Pruebas automáticas (`xs-capture`, `xs-editor`, UI):
  - un formulario con un correo, un teléfono y una tarjeta: los tres tapados, 1 por categoría, la etiqueta visible;
  - volver a analizar no duplica nada; un solo Deshacer quita todo el análisis;
  - apagar Teléfonos quita su tapa y la cuenta sigue en 1;
  - Pixelar cambia todas las tapas en un paso, y Deshacer las devuelve a sólido;
  - el documento no guarda el texto, y Extraer texto sobre la imagen final ya no lo encuentra;
  - un término propio y un patrón cuentan aparte; un patrón no válido se nombra y no se aplica;
  - con `path`, el archivo se redacta aparte en "<nombre> (redacted).png" y el original no cambia; un video se rechaza sin abrirlo;
  - Siguiente y Anterior seleccionan cada tapa y centran la vista; Listo solo recuerda el riesgo si hay tapas de desenfoque o pixelado.
- En la app (formulario de prueba con datos inventados): Imagen > Redacción automática… tapa correo, tarjeta, IP y teléfono, cuenta la fecha aunque esté apagada, el patrón "Pedidos" (`PO-\d{6}`) tapa el número de pedido, Siguiente selecciona la tapa con sus tiradores, Listo con tapas pixeladas muestra el recordatorio, y reabrir el panel en español recupera las 4 tapas sin duplicarlas.
- Límite conocido: el reconocimiento de Windows no lee una línea "4111 1111 1111 1111"; con otro número de tarjeta válido funciona.

## Para Carlos (en la próxima caminata)

1. [ ] Captura una página con un correo, un teléfono y una tarjeta (por ejemplo, un formulario de prueba). En el editor: **Imagen > Redacción automática…** Los datos quedan tapados en negro, las etiquetas ("Correo:") se ven, y el panel dice 1 en cada una de esas categorías.
2. [ ] Pulsa **Siguiente** varias veces: cada tapa se selecciona y la vista se centra en ella.
3. [ ] Desmarca **Números de teléfono**: su tapa desaparece y las demás siguen. Vuelve a marcarla: vuelve.
4. [ ] En **Estilo**, elige **Pixelado**: cambian todas a la vez. `Ctrl+Z` una vez: vuelven a negro.
5. [ ] En **Mis términos**, añade una palabra que salga en la captura (por ejemplo tu nombre) y un patrón con nombre. Cada uno aparece con su número y su tapa. Un patrón roto (por ejemplo `(`) sale en rojo.
6. [ ] Pulsa **Listo** con alguna tapa en Desenfoque o Pixelado: sale el recordatorio de seguridad. Con **No volver a preguntar**, ya no sale; Preferencias > General > **Volver a mostrar avisos y pistas** lo recupera.
7. [ ] Después de Listo, selecciona una tapa con **Mover** y cámbiala a Pixelado en sus propiedades: solo cambia esa.
8. [ ] Vuelve a abrir el panel: las tapas siguen y no se duplican.
9. [ ] Con la herramienta **Desenfocar** elegida, sus propiedades tienen el botón **Redacción automática…**
10. [ ] Una foto con caras (por ejemplo, una videollamada capturada): cada cara de frente queda tapada con una elipse. **Anota si alguna cara de perfil se queda sin tapar** (ver QUESTIONS).
11. [ ] Exporta a PNG y amplía al 800 %: no se ve nada del dato bajo una tapa sólida.
12. [ ] Preferencias > Atajos: **Redacción automática** está en la lista, sin atajo; asígnale uno y pruébalo.
