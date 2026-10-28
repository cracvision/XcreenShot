# Editor, parte 4c (M1, entrega 4c) · EDT-002, EDT-003, EDT-012, EDT-013, EDT-115, EDT-116, EDT-117, EDT-118

Pruebas automáticas:
- `crates/xs-core/src/shortcuts.rs`: ningún atajo por defecto se repite donde los dos funcionarían, en los dos perfiles y las dos plataformas (EDT-116); un atajo del editor puede repetir uno de la app (`Ctrl+W`) pero no uno global ni otro del editor; una tecla sola vale en el editor (EDT-115); valores de macOS (`Cmd`, `Control` en Compatibilidad).
- `crates/xs-core`: todos los comandos de los menús están registrados y sus textos traducidos (APP-027); cada fila de atajos tiene nombre en inglés y en español.
- `app/src/editor/announce.test.ts`: un texto "Paso 1" se anuncia como "Texto: Paso 1, en 320, 140, 200 por 80 píxeles" (EDT-013).

Pruebas con la app en marcha (instancia aislada; menús leídos y pulsados desde Windows, teclas y clics simulados en la ventana), 2026-10-28, Windows 11, monitor al 150 %:
- [x] La barra de menús tiene Archivo, Edición, Ver y Ayuda; cada entrada muestra su atajo a la derecha.
- [x] Sin selección, Cortar, Copiar, Eliminar y Deseleccionar están en gris; Deshacer, en gris sin nada que deshacer. Panel lateral y Mostrar etiquetas, marcados.
- [x] Ver > Acercar lleva el zoom de 100 % a 150 %; Ver > Panel lateral lo oculta y quita la marca.
- [x] `Ctrl+Alt+P`, `Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`, `Ctrl`+`0` del teclado numérico y `U` hacen lo suyo.
- [x] Con Herramienta Flecha en `L`: `L` activa Flecha y el tooltip dice "Flecha (L)". Con Guardar en otra combinación, el menú Archivo la muestra; al restablecer vuelve `Ctrl+S`. Deshacer no acepta la combinación del atajo global de prueba.
- [x] Clic derecho en una zona vacía: Pegar, Pegar en el mismo sitio, Copiar todo. Sobre una flecha: la selecciona y ofrece Cortar, Copiar, Eliminar.
- [x] `editor.toolbarLabels` desactivado: la barra queda con íconos y el menú quita la marca de Mostrar etiquetas.
- [x] Con "Esc cierra el editor": el primer `Esc` deselecciona y el segundo cierra la ventana; el documento sigue abierto.
- [x] `F6` recorre lienzo, panel lateral, tira y barra de estado; `Shift+F6` vuelve.
- [x] El lector de pantalla recibe "Texto: Paso 1, en 320, 140, 53 por 22 píxeles", "Flecha, en 100, 400, 200 por 20 píxeles" y "Herramienta: Flecha".
- [x] Deslizador de opacidad: `Shift`+↓ baja 10 y ↓ baja 1.

No se probó con el teclado real: `Alt` o `F10` para entrar en la barra de menús y recorrerla con las flechas. Va en la tanda 1.

## Tanda 1 · Menús — Carlos

Instalador: `target\release\bundle\nsis\XcreenShot_0.2.0_x64-setup.exe`, compilación de la entrega 4c o posterior (aparece junto a la versión con `xcreenshot app ping`).

1. [ ] La ventana tiene la barra de menús **Archivo**, **Edición**, **Ver** y **Ayuda**. Cada entrada muestra su atajo a la derecha.
2. [ ] Pulsa `Alt` (o `F10`) y recorre los menús con las flechas; `Enter` ejecuta la entrada elegida y `Esc` sale.
3. [ ] Sin nada seleccionado, **Edición** muestra Cortar, Copiar, Eliminar y Deseleccionar en gris. Selecciona una flecha: se activan.
4. [ ] **Ver > Acercar**, **Alejar** y **Ajustar a la ventana** cambian el zoom. **Ver > Panel lateral** oculta el panel y lo vuelve a mostrar; su marca cambia.
5. [ ] Clic derecho sobre una flecha: Cortar, Copiar y Eliminar. Clic derecho en una zona sin objetos: Pegar, Pegar en el mismo sitio y Copiar todo.
6. [ ] Clic derecho en la barra de herramientas > **Solo íconos**: desaparecen los nombres. **Ver > Barra de herramientas > Mostrar etiquetas** los devuelve.
7. [ ] **Ayuda > Recuperar trabajo…** avisa de que no hay nada que recuperar (o lo ofrece, si lo hay).

## Tanda 2 · Atajos — Carlos

1. [ ] **Preferencias > Atajos**, **Mostrar: Del editor**: salen los atajos del editor (Abrir, Guardar, Deshacer, Cortar…, y una fila por herramienta).
2. [ ] Cambia **Herramienta Flecha** a `L`. En el editor, `L` activa Flecha y su tooltip dice "Flecha (L)". Restablécelo.
3. [ ] Cambia **Guardar** a otra combinación: **Archivo** muestra la nueva junto a Guardar. Restablécelo.
4. [ ] Intenta asignar a **Deshacer** tu atajo global `Ctrl+P`: avisa de que ya lo usa otro comando.
5. [ ] En el editor, `Ctrl+W` cierra el documento, no la ventana. En Preferencias, `Ctrl+W` cierra Preferencias.
6. [ ] `Ctrl+Alt+Shift+E` abre Exportar (antes era `Ctrl+Alt+Shift+S`, que choca con tu atajo de Capturar imagen).

## Tanda 3 · Preferencias del editor y accesibilidad — Carlos

1. [ ] **Preferencias > Editor** muestra siete ajustes: Reducir para ajustar al abrir, Recordar el zoom y la posición, Esc cierra el editor, Etiquetas de la barra, Seleccionar la anotación al terminar de dibujarla, Ampliar el lienzo automáticamente y Eliminar el contenido recortado.
2. [ ] Desactiva **Reducir para ajustar al abrir** y haz una captura de pantalla completa: se abre al 100 %, sin ajustarse a la ventana. Vuelve a activarlo.
3. [ ] Activa **Esc cierra el editor**. Con una flecha seleccionada, el primer `Esc` la deselecciona y el segundo cierra la ventana; al reabrir el editor desde el ícono, el documento sigue ahí. Desactívalo.
4. [ ] Desactiva **Seleccionar la anotación al terminar de dibujarla**: una flecha nueva queda sin seleccionar. Vuelve a activarlo.
5. [ ] Haz clic en la imagen y pulsa `F6` varias veces: el foco pasa por el panel lateral, la tira de documentos, la barra de estado y la barra de herramientas. `Shift+F6` vuelve atrás.
6. [ ] Selecciona una flecha, haz clic en el deslizador de Opacidad y pulsa ↓: baja 1 %. `Shift`+↓: baja 10 %.
7. [ ] (Opcional) Con el Narrador de Windows activo (`Ctrl+Win+Enter`), haz clic en la imagen y pulsa `Tab`: anuncia cada objeto con su tipo, su texto y su posición.
