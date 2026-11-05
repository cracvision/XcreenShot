# Aspecto nuevo: Snap con los colores del logo · APP-012, EDT-001, EDT-002

Qué cambió (2026-11-05):
- **Colores y letra.** La paleta del logo de Carlos: azul eléctrico `#0064FB`, el degradado azul-violeta-magenta en el botón principal, el amarillo de la estrella, el rojo REC y el azul marino del texto. Los títulos van en Outfit, la fuente gratuita más parecida a la del logo, empaquetada en la app.
- **Logo en el editor vacío.** Aparece siempre que no hay ninguna captura a la vista, en versión clara u oscura. El punto de REC parpadea; con reducir movimiento se queda encendido.
- **Ícono nuevo.** La X del logo dentro del marco azul-violeta, sobre una baldosa azul marino. En la bandeja, el marco y la X sin baldosa.
- **Animaciones.**
  - Los menús entran con un pequeño rebote.
  - En las pestañas y las herramientas, el resaltado se desliza hasta la opción elegida.
  - Los botones suben al pasar el puntero y se hunden al pulsarlos; el principal tiene un brillo que lo cruza.
  - Las miniaturas de la tira se levantan al pasar el puntero y entran con un salto.
  - Los avisos suben y los segundos de la cuenta atrás aparecen de golpe.
- **Menús dibujados por la app.** En Windows, la barra de menús del editor y los menús del clic derecho ya no son los nativos: se animan como los demás.

Pruebas automáticas:
- `app/src/components/Menu.test.tsx`: teclado de los menús (flechas, Inicio, Fin, Escape), la animación de salida y el menú contextual en el puntero.
- `app/src/editor/MenuBar.test.tsx`:
  - `F10` y `Alt` llevan a la barra, y `Alt`+letra abre un menú;
  - las flechas recorren la barra y entran en los menús;
  - un submenú se abre con la flecha derecha y se cierra con la izquierda;
  - una entrada desactivada se ve pero no hace nada;
  - las teclas del editor no se quedan con las flechas de los menús.
- `app/src/editor/menus.test.ts`: las entradas del clic derecho sobre un objeto, sobre la imagen y sobre la barra de herramientas.
- `crates/xs-core/src/menus.rs`: los menús que la ventana dibuja tienen las mismas entradas y comandos que los nativos.
- Contraste comprobado con números en los dos temas: todo el texto llega a 4,5:1 y los bordes de los controles a 3:1. El texto blanco sobre el degradado, a 4,9:1 como mínimo.

Probado en una instancia aislada de desarrollo, 2026-11-05, Windows 11:
- [x] Editor, Preferencias, panel de captura y diálogos, en claro y en oscuro.
- [x] El logo con el punto de REC parpadeando: fotogramas alternos encendido y apagado.
- [x] `F10`, flecha derecha y flecha abajo abren el menú Edición; Ver > Barra de herramientas ▸ abre su submenú, con la marca de "Mostrar etiquetas".
- [x] El clic derecho sobre el lienzo muestra Pegar, Pegar en el mismo sitio y Copiar todo, con sus atajos.

## Tanda · Carlos

Con el instalador nuevo:
1. [ ] El ícono nuevo se ve en la barra de tareas, en el menú Inicio y en la bandeja, y se distingue en barra clara y oscura.
2. [ ] Sin capturas abiertas, el editor muestra el logo y el punto de REC parpadea más o menos una vez por segundo.
3. [ ] En **Preferencias > General > Tema**, cambia entre Claro y Oscuro: el logo cambia de versión y todo se lee bien en los dos.
4. [ ] Pulsa `Alt` y suéltalo: se subraya una letra en Archivo, Edición, Ver, IA y Ayuda. Con las flechas abre Ver > Barra de herramientas ▸ y desactiva "Mostrar etiquetas": la barra queda solo con íconos. Vuelve a activarla con el ratón.
5. [ ] `Alt`+`A` abre Archivo (en inglés, `Alt`+`F`). `Escape` lo cierra.
6. [ ] Abre una captura y haz clic derecho sobre ella: el menú sale junto al puntero, con un pequeño rebote.
7. [ ] Pasa el puntero por las herramientas y haz clic en varias: el resaltado azul se desliza de una a otra. Haz lo mismo con las pestañas Estilos y Propiedades.
8. [ ] Abre dos o tres capturas: entran en la tira con un salto, y la que estás viendo lleva las esquinas amarillas.
9. [ ] En Windows, activa Configuración > Accesibilidad > Efectos visuales > **Efectos de animación: desactivado**. Las animaciones quedan en fundidos cortos y el punto de REC deja de parpadear. Vuelve a activarlo.
10. [ ] Las teclas de siempre siguen funcionando: herramientas (`V`, `A`, `T`…), `Ctrl+Z`, mover un objeto con las flechas.
