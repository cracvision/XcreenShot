# Plan de M4: video y GIF (propuesta)

Propuesta de Claude del 2026-11-13, **provisional hasta que Carlos la apruebe**. M4 tiene 77 IDs (`VID-001` a `VID-077`, en `spec/03-video.md`). Cada entrega termina como las de M2 y M3: pruebas automáticas, `PARITY.md`, `QUESTIONS.md` con las decisiones provisionales y un instalador probado en su forma empaquetada.

| Entrega | Contenido | Estado |
|---|---|---|
| M4.1 · Motor y barra | Motor de grabación (pantalla de un monitor o región → MP4 H.264, frecuencia variable, tamaño del video, hardware o software), comandos `record.*`, Captura de video desde el menú del ícono y su atajo, barra de grabación mínima, borde del área, Preferencias > Video (VID-002, VID-003, VID-011 a VID-015, VID-020, VID-024 a VID-026, VID-028, VID-029, VID-075) | Hecha en Windows (2026-11-13, madrugada) |
| M4.2 · Iniciar y terminar | Pestaña Video del panel (VID-001), resumen Listo para grabar (VID-006), empezar sin esperar y cuenta atrás (VID-007, VID-008), presets de video (VID-009), grabar una ventana y una pantalla completa (VID-004, VID-005), posición de la barra (VID-016), XcreenShot fuera del video (VID-017), Abrir en, compartir y la biblioteca con videos (VID-052 a VID-055) | Casi hecha en Windows (2026-11-13): faltan empezar sin esperar (VID-007), Abrir en (VID-052, necesita el reproductor del editor), compartir (VID-053) e importar videos (VID-055) |
| M4.3 · Audio | Micrófono, audio del sistema, pistas, dispositivos y reducción de ruido (VID-039 a VID-043, VID-019), con su permiso (VID-077) | — |
| M4.4 · Cursor y teclas | Capturar y resaltar el cursor, animación de clic, teclas pulsadas (VID-044 a VID-047) | Hecha en Windows (2026-11-13, tarde); falta el menú Efectos con vista previa |
| M4.5 · Cámara | Activar, imagen en imagen, forma, pantalla completa, espejo, borde y fondo (VID-031 a VID-038) | — |
| M4.6 · Editor de video | Reproductor, selección, cortar, guardar selección y fotograma, extraer texto de un fotograma, copia sin sonido, información, combinar (VID-056 a VID-067) | En curso en Windows (2026-11-13, tarde, ADR 0008): hechos el reproductor, la selección, cortar y guardar, guardar selección y fotograma, extraer texto, copia sin sonido, información y Abrir en (VID-052). Faltan el menú Video, exportar y copiar videos (VID-066) y combinar (VID-067) |
| M4.7 · GIF y diapositivas | GIF animado (VID-068 a VID-070) y diapositivas narradas (VID-071 a VID-074) | GIF hecho en Windows (2026-11-13, noche) con el escritor propio, sin dependencias nuevas; faltan las diapositivas |
| M4.8 · Robustez | Dibujo en vivo (VID-048 a VID-051), escritorio limpio (VID-018), mantener despierto (VID-021), recuperar grabaciones (VID-022), grabaciones largas (VID-023), H.265 (VID-027), sincronía y rendimiento medidos (VID-030), convertir en imagen (VID-010) | — |

## Para decidir con Carlos

- Aprobar este orden, o cambiarlo.
- **Micrófono y cámara:** probarlos enciende sus dispositivos; esas entregas necesitan que Carlos esté delante (o su permiso para hacerlo sin él).
- **GIF:** resuelto sin dependencias nuevas: se amplió el escritor de GIF propio de la exportación de imágenes.
