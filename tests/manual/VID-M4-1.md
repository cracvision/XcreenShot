# Grabar video: primera parte (M4.1)

Motor de grabación, comandos `record.*`, Captura de video con la barra de grabación. IDs: VID-002, VID-003, VID-011 a VID-015, VID-020, VID-024 a VID-026, VID-028, VID-029, VID-044 a VID-047 y VID-075.

## Visto en la instancia de desarrollo (2026-11-13, madrugada)

- Motor (pruebas `cargo test -p xs-video -- --ignored`, que graban la pantalla de verdad): 2 s con el codificador por hardware y con el de software, leídos de vuelta con Media Foundation (1280 × 720, su duración). Una ventana de colores conocidos (rojo 200, 40, 40 y azul 30, 160, 220) sale con ±2 niveles a 1280 × 720 y a 900 × 500, con los dos codificadores.
- Con la pantalla quieta, la duplicación del escritorio no entrega imágenes (esa noche el monitor estaba en reposo): por eso la primera imagen se toma con GDI. Sin ella, el video salía negro.
- `record.start` de una región de 1500 × 900 al 150 %: video de 1000 × 600 (Reducir pantallas de alta densidad). Pantalla completa del monitor de 5120 × 2160: 3412 × 1440. `busy` con una grabación en curso, `not_found` sin ninguna. Una pausa de 1,5 s queda fuera (2,35 s grabados). `limit: 2` se detiene sola a los 2 s.
- Puntero (VID-044): con el ratón sobre la ventana de prueba, el primer fotograma tiene 160 píxeles del cursor junto a su punta (prueba `cursor_is_drawn`; el ratón se movió un momento y volvió a su sitio). El interruptor **Puntero** de la barra lo apaga y lo enciende durante la grabación.
- Captura de video (`record.video`): arrastrar un área de 900 × 525 en la selección deja la barra *Lista* con "600 × 350" y **Grabar**; **Grabar** empieza; Pausar y Reanudar cambian la barra y el borde (discontinuo en pausa); **Detener** cierra la barra y el borde y guarda el MP4 en la biblioteca (carpeta del mes, "Video AAAA-MM-DD HH.MM.SS.mp4"); durante una grabación, Captura de video la detiene.

- Efectos (VID-045 a VID-047), prueba `effects_are_drawn` en memoria (sin tocar la pantalla): sobre un fondo gris de 400 × 300, el resaltado rojo al 50 % da (191, 64, 64) en su centro, fuera sigue gris (128), la etiqueta "Ctrl + C" es oscura y la onda azul está. `record.effects`: un efecto del puntero con el puntero apagado lo enciende, un color activa su efecto, "red" se rechaza, y los atajos de grabación (`Ctrl+Mayús+F9`, `Ctrl+Mayús+F10`, `Ctrl+Alt+Mayús+V`) quedan fuera de las teclas mostradas.

## Para Carlos (en la próxima caminata)

1. [ ] Menú del ícono > **Captura de video** (o `Ctrl+Alt+Mayús+V`): arrastra un área. Aparece un borde azul alrededor y, debajo, la barra con `00:00:00`, el tamaño del video y **Grabar**.
2. [ ] Pulsa **Grabar**: hay una cuenta atrás de 3 s (en Preferencias > Video puedes elegir Ninguna, 5 o 10; `Esc` la cancela y vuelves a *Lista*). Después el borde se vuelve rojo y el tiempo avanza. Mueve una ventana dentro del área.
3. [ ] **Pausar** (o `Ctrl+Mayús+F9`): el tiempo se para y el borde queda discontinuo. **Reanudar**.
4. [ ] **Detener** (o `Ctrl+Mayús+F10`): aparece el aviso "Video guardado". En la biblioteca aparece el video con la miniatura de su primer fotograma; ábrelo (se abre con el reproductor de Windows): se ve lo que hiciste, sin el borde ni la barra, y sin el tiempo de la pausa.
5. [ ] Empieza otra y, mientras graba, abre el menú del ícono: arriba aparece **Detener grabación**.
6. [ ] La **×** de la barra durante una grabación pregunta antes de cancelar; **Cancelar** no deja ningún archivo.
7. [ ] Graba moviendo el ratón: el puntero se ve en el video, con su forma (flecha, mano sobre un enlace, I sobre un texto). Pulsa **Puntero** en la barra a mitad: desde ahí ya no sale.
8. [ ] Con algo sonando (un video en el navegador), en la barra *Lista* activa **Sonido** y graba 10 s: el MP4 tiene el sonido, a la vez que la imagen. Sin altavoces ni auriculares conectados, el aviso de "Video guardado" dice que no se grabó el audio del sistema.
9. [ ] Panel de captura > pestaña **Video**: Selección = Pantalla completa y **Capturar**: la barra aparece sin pedir un área (el monitor donde está el ratón). Con Temporizador = 1 min, la grabación se detiene sola al minuto.
10. [ ] Pestaña **Video** > Selección = **Ventana**, **Capturar** y elige una ventana. Graba y mueve la ventana o cámbiale el tamaño: el video la sigue. Ciérrala: la grabación se detiene y se guarda sola.
11. [ ] En *Lista*, **Imagen** (o `I`) toma una captura del área en lugar de grabar. Grabando, **↺** (Reiniciar) vuelve a *Lista* con la misma área; pasados 10 s pregunta antes.
12. [ ] Haz dos grabaciones seguidas de áreas distintas: la barra y el borde salen bien colocados y del tamaño correcto las dos veces.
13. [ ] Pestaña **Video** > Pantalla completa, Temporizador 5 min > Presets **+ > Desde los ajustes actuales**: aparece "Video: Pantalla completa" con una "V". Ejecútalo desde la lista o el menú del ícono: la barra queda lista para grabar.
14. [ ] Preferencias > Video: **Calidad de video** en Muy alta (60 fps) y graba algo que se mueva rápido.
15. [ ] Efectos del cursor (VID-045, VID-046): pestaña **Video** > activa **Resaltar cursor** y **Animación de clic**. Graba haciendo clics normales, un clic derecho y un arrastre: en el video hay un círculo amarillo bajo el puntero, una onda azul en cada clic, una naranja en el clic derecho y un punto fijo mientras arrastras. El puntero se ve encima del círculo.
16. [ ] Durante esa grabación pulsa **Clics** en la barra: desde ahí ya no hay ondas. Apaga **Puntero**: desaparecen el puntero y el círculo; enciéndelo y vuelven los dos.
17. [ ] Preferencias > Video: cambia el **Color del resaltado** (selector de color), el tamaño a 80 y la forma a **Anillo**, y graba: el anillo sale con ese color y tamaño.
18. [ ] Teclas pulsadas (VID-047): activa **Mostrar teclas pulsadas** y graba copiando y pegando (`Ctrl+C`, `Ctrl+V`) y escribiendo una palabra: salen "Ctrl + C" y "Ctrl + V" abajo en el centro, y la palabra no. `Ctrl+Mayús+F9` (Pausar) no sale.
19. [ ] Con **Qué teclas mostrar = Todas las teclas**, escribe "hola" en el Bloc de notas: salen las letras. Después escribe en un campo de contraseña (por ejemplo, el de una página de inicio de sesión): no sale ninguna tecla.

Todavía no hay micrófono, cámara, "Solo la ventana" ni reproductor en el editor: llegan en las siguientes partes de M4.
