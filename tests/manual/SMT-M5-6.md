# Códigos QR (M5.6)

IDs: SMT-036, SMT-038 (SMT-037 espera a M6).

## Visto en la instancia de desarrollo (2026-11-14)

- Pruebas automáticas:
  - `xs-image` `qr`: un código de `example.com` (con `https://` añadido) se lee de vuelta; claro sobre oscuro también; a 400 px sigue sin bordes grises; los límites de tamaño y el aviso de contraste.
  - `xs-capture` `create_insert_and_read`: insertado en una captura y exportado a PNG, se lee como `https://example.com`; cambiarlo actualiza el código; agrandado a 600 px, solo negro y blanco; gris claro sobre blanco avisa; demasiado largo para la corrección elegida, error con el máximo.
  - `xs-capture` `three_codes_and_files`: tres códigos en una imagen (enlace, texto y Wi-Fi con su red y contraseña), una parte de la imagen, y archivos PNG y SVG.
  - UI: el enlace se abre solo tras el sí; la dirección con su dominio aparte; el aviso de demasiado largo; doble clic abre el panel con el contenido y el estilo del código.
- Durante la captura y en Extraer texto (2026-11-14):
  - `xs-capture` `codes_on_the_frozen_screens`: los códigos de dos monitores congelados (el segundo a la izquierda) en el escritorio virtual, con su tipo.
  - `a_4k_screen` (*ignored*): buscar en una pantalla 4K tarda **127 ms**, en segundo plano.
  - UI `codesIn`: cuenta un código cuando su centro está en la selección, también en un monitor a la izquierda.
- En la instancia aislada: el panel con la vista previa al escribir `example.com`; Insertar pone el objeto QR en el lienzo; Leer códigos QR lo lista con Copiar, Abrir enlace y Redactar el código; Abrir enlace pide confirmación con el dominio en negrita y Cancelar no abre nada.

## Para Carlos (en la próxima caminata)

1. [ ] Menú **Imagen > Insertar código QR…**, escribe `example.com` > **Insertar**. Exporta a PNG y léelo con la cámara del teléfono: abre `https://example.com`.
2. [ ] Agranda el código al 400 %: los bordes de los módulos siguen nítidos.
3. [ ] Pon el color del código en gris claro: aparece "Puede que algunos lectores no lo lean".
4. [ ] Doble clic en el código: cambia el enlace y el código se actualiza.
5. [ ] Captura una pantalla con un código QR (por ejemplo, una web con uno) y ábrela en el editor: aparece "Código QR detectado"; púlsala y se ven su contenido, **Copiar** y **Abrir enlace**.
6. [ ] **Abrir enlace** pide confirmación y muestra el dominio; con Cancelar no se abre el navegador.
7. [ ] Desactiva Preferencias > Captura > **Detectar códigos QR al capturar**: ya no aparece la pastilla, y clic derecho > **Leer códigos QR** sigue funcionando.
8. [ ] Con el ajuste activado, empieza una captura universal sobre una página con un código QR y selecciona la zona del código.
   - La barra de captura muestra **QR**.
   - Al pulsarlo, o con la tecla `Q`, se abre el panel con la dirección; **Copiar** la copia.
   - `Esc` cierra el panel y otro `Esc` cancela la captura sin dejar nada en la biblioteca.
9. [ ] En ese panel, **Abrir enlace** pide confirmación antes de abrir el navegador.
10. [ ] **Extraer texto** de una zona con texto y un código QR: al final de la ventana aparecen los **Códigos QR** con **Copiar** y **Abrir enlace**.
11. [ ] En la barra de herramientas, **Más > Imagen > Insertar código QR…** abre el mismo panel que el menú Imagen.
