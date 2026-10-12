# Preguntas y decisiones abiertas

Formato: ID afectado · pregunta · propuesta · estado. Carlos decide; el equipo de construcción anota aquí sus dudas.

## Decisiones de SPEC.md §9 (pendientes de Carlos)

| ID | Pregunta | Propuesta | Estado |
|---|---|---|---|
| D-01 | Licencia del repo | Privado; MIT o Apache-2.0 si se abre | Abierta |
| D-02 | Proveedor del Enlace para compartir | Carpeta sincronizada de OneDrive o Google Drive primero | Abierta |
| D-03 | Impresora virtual | No; importar PDF (CAP-041) | Abierta |
| D-04 | Codificación de video | Codificadores nativos, sin FFmpeg | Abierta |
| D-05 | Destinos con OAuth prioritarios | Los que usen Carlos y sus colegas | Abierta |
| D-06 | Firma de código | Authenticode + Apple Developer Program | Abierta |
| D-07 | Idiomas | Inglés y español | Abierta |
| D-08 | Atajos por defecto | Perfil "Coexistencia" mientras Snagit siga instalado | Abierta |

## Dudas que dejaron los redactores del spec

| ID | Pregunta | Propuesta | Estado |
|---|---|---|---|
| CAP-065 | Los atajos de coexistencia usan `Ctrl+Alt+Shift` (Windows) y `⌃⌥⇧` (Mac). `Ctrl+Alt` equivale a `AltGr` en algunos teclados, y `⌃⌥` choca con VoiceOver | Probar en los teclados de Carlos antes de M1 | Abierta |
| CAP-037 | La captura de página web por URL carga la página sin las cookies del navegador, así que las páginas con inicio de sesión salen como las ve un visitante | Aceptar en v1 | Abierta |
| CAP-041 | En Windows, importar PDF exige imprimir primero a PDF | Aceptar (D-03) | Abierta |
| APP-021 | Nombre del perfil de atajos: "Compatibilidad con Snagit" usa la marca en la UI | Llamarlo "Compatibilidad" | Abierta |
| APP-040 | Versión mínima de macOS | macOS 14+, compilación universal | Abierta |
| SPEC §7 | Faltan en la tabla de formatos `.xsettings` (exportar preferencias), `managed.json` (configuración administrada) y la estructura del documento `guide` | Añadirlos cuando se cierre el spec de M1 | Abierta |
| VID-036 | ¿La cámara se muestra en espejo por defecto? | Sí; Carlos puede preferir que no | Abierta |
| VID-025 | Calidad de video por defecto | Alta (30 fps) | Abierta |
| VID-075 | Posibles choques entre atajos de video y del editor | Revisar con EDT- al cerrar los atajos | Abierta |
| VID-062 / SMT- | Extraer texto de un fotograma depende del OCR, que llega en M3/M5 | Mover VID-062 a M5 | Abierta |
| SMT-010 | Redacción automática: URLs y fechas desactivadas por defecto; documentos de identidad de EE. UU., España y Brasil | Aceptar | Abierta |
| 05-app.md | Referencia otras funciones por nombre, no por ID | Pasada de enlaces antes de M1 | Abierta |
| 02 / 04 | Faltan tablas de cierre en `02-editor.md` y el marco de compartir y destinos en `04-biblioteca-compartir.md` | Completar antes de M2 | Abierta |
