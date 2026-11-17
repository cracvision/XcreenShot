# Grabador de pasos (M5.5a)

Convertir los clics en una guía paso a paso: una imagen y una frase por clic. IDs: SMT-022 a SMT-028. Diseño: [ADR 0009](../../docs/adr/0009-guias-de-pasos.md).

## Visto en la instancia de desarrollo (2026-11-14)

- Pruebas automáticas:
  - el clasificador: clic, clic derecho, arrastre, un temblor de 3 px es un clic, el doble clic es un paso y dos clics lentos son dos pasos;
  - las frases en español e inglés (botón, menú, elemento de menú, casilla según su estado, texto reconocido, "Clic aquí", "En **App**, …", nombres de más de 60 caracteres recortados);
  - el encuadre: el diálogo con margen, el ícono de la barra de tareas da la pantalla completa, la ventana y la pantalla;
  - la guía: encabezado, pasos numerados 1 y 2, un texto largo completo encima de su imagen, anillos naranja y violeta, flecha y pie "Creado por";
  - el encuadre de cada zona cae exacto en monitores de 1080p, 4K y 5K.
- Sobre el escritorio real, sin clics de verdad (`a_recording_on_this_desktop`, a mano): los clics durante el aviso no cuentan; un clic, un doble clic, un clic derecho y un arrastre dan 4 pasos del tipo correcto; Deshacer quita el quinto; Terminar crea la guía "Pasos en Claude" con 4 pasos, imágenes encuadradas, anillos, flecha y textos leídos con el reconocimiento ("Doble clic en «Proyecto o carpeta»"), y borra la carpeta de la grabación.

## Para Carlos (en la próxima caminata)

1. [ ] Menú del ícono > **Grabar pasos**. En el centro aparece "Grabando pasos: cada clic crea un paso" durante 2 s y, abajo a la izquierda, la barra con "0 pasos".
2. [ ] Abre el Bloc de notas y haz clic en **Archivo** y luego en **Guardar como…**. El contador sube a 1 y a 2 y destella. Pulsa Cancelar en el diálogo de Windows.
3. [ ] Pulsa **Pausar**, haz 3 clics en otra app y pulsa **Reanudar**: el contador no cambió. Haz un clic más: sube.
4. [ ] Haz un doble clic en un archivo del Explorador: cuenta como **un** paso. Un clic derecho y un arrastre: dos pasos más.
5. [ ] **Deshacer último paso**: el contador baja en uno.
6. [ ] Arrastra la barra a otro sitio y haz un clic justo debajo: la barra no sale en la imagen del paso.
7. [ ] **Terminar**: se abre la guía "Pasos en Bloc de notas" (o la app que más usaste) con un paso por clic, numerados, con su frase ("En **Bloc de notas**, abrir el menú **Archivo**", "Elegir **Guardar como…** en el menú") y la imagen recortada al diálogo o al menú, con el anillo del clic. Al final, "Creado por <tu nombre> · fecha".
8. [ ] Otra grabación con 3 pasos y `Esc`: la barra pregunta "¿Descartar 3 pasos?"; **Descartar** no deja nada en la biblioteca. `Esc` no le llega a la app que estabas usando.
9. [ ] Panel de captura > Imagen > Selección = **Grabador de pasos**: aparecen Área y Título de la guía; Intervalo y Programada están desactivados. Con Área = **Una ventana**, el primer clic elige la ventana y los clics fuera de ella no suman.
10. [ ] Con **Capturar cursor** activado, cada paso tiene el puntero como objeto, que se puede borrar.
11. [ ] Preferencias > Captura > **Anotar el uso del teclado**: con él activado, `Ctrl+S` crea "Pulsar **Ctrl+S**" y escribir en un campo añade "y escribir en el campo «…»", **sin lo que escribiste** (pruébalo con una contraseña de prueba).
12. [ ] Mientras graba pasos, intenta empezar una grabación de video: no deja y lo dice.
13. [ ] El menú del ícono muestra **Pausar grabación de pasos** y **Terminar grabación de pasos** mientras graba.
