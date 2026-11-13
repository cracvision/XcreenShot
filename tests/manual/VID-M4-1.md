# Grabar video: primera parte (M4.1)

Motor de grabación, comandos `record.*`, Captura de video con la barra de grabación. IDs: VID-002, VID-003, VID-011 a VID-015, VID-020, VID-024 a VID-026, VID-028, VID-029 y VID-075.

## Visto en la instancia de desarrollo (2026-11-13, madrugada)

- Motor (pruebas `cargo test -p xs-video -- --ignored`, que graban la pantalla de verdad): 2 s con el codificador por hardware y con el de software, leídos de vuelta con Media Foundation (1280 × 720, su duración). Una ventana de colores conocidos (rojo 200, 40, 40 y azul 30, 160, 220) sale con ±2 niveles a 1280 × 720 y a 900 × 500, con los dos codificadores.
- Con la pantalla quieta, la duplicación del escritorio no entrega imágenes (esa noche el monitor estaba en reposo): por eso la primera imagen se toma con GDI. Sin ella, el video salía negro.
- `record.start` de una región de 1500 × 900 al 150 %: video de 1000 × 600 (Reducir pantallas de alta densidad). Pantalla completa del monitor de 5120 × 2160: 3412 × 1440. `busy` con una grabación en curso, `not_found` sin ninguna. Una pausa de 1,5 s queda fuera (2,35 s grabados). `limit: 2` se detiene sola a los 2 s.
- Captura de video (`record.video`): arrastrar un área de 900 × 525 en la selección deja la barra *Lista* con "600 × 350" y **Grabar**; **Grabar** empieza; Pausar y Reanudar cambian la barra y el borde (discontinuo en pausa); **Detener** cierra la barra y el borde y guarda el MP4 en la biblioteca (carpeta del mes, "Video AAAA-MM-DD HH.MM.SS.mp4"); durante una grabación, Captura de video la detiene.

## Para Carlos (en la próxima caminata)

1. [ ] Menú del ícono > **Captura de video** (o `Ctrl+Alt+Mayús+V`): arrastra un área. Aparece un borde azul alrededor y, debajo, la barra con `00:00:00`, el tamaño del video y **Grabar**.
2. [ ] Pulsa **Grabar**: el borde se vuelve rojo y el tiempo avanza. Mueve una ventana dentro del área.
3. [ ] **Pausar** (o `Ctrl+Mayús+F9`): el tiempo se para y el borde queda discontinuo. **Reanudar**.
4. [ ] **Detener** (o `Ctrl+Mayús+F10`): aparece el aviso "Video guardado". Abre la carpeta del mes de tu biblioteca y reproduce el MP4 con el reproductor de Windows: se ve lo que hiciste, sin el borde ni la barra, y sin el tiempo de la pausa.
5. [ ] Empieza otra y, mientras graba, abre el menú del ícono: arriba aparece **Detener grabación**.
6. [ ] La **×** de la barra durante una grabación pregunta antes de cancelar; **Cancelar** no deja ningún archivo.
7. [ ] Preferencias > Video: **Calidad de video** en Muy alta (60 fps) y graba algo que se mueva rápido.

Todavía no hay sonido, cámara, cursor en el video, pestaña Video en el panel ni reproductor en el editor: llegan en las siguientes partes de M4.
