# Caminata por XcreenShot (M1 en Windows)

Esta guía es para recorrer la app con calma, sin pasos obligatorios. La idea es que la uses como en tu trabajo y anotes lo que no te guste, lo que no entiendas o lo que falle. Después corrijo lo que encuentres y, con todo ya estable, escribo **un único guion de pruebas** para cerrar M1.

## Antes de empezar

1. Instala la versión nueva con el instalador que te dejé (`XcreenShot_0.2.0_x64-setup.exe`). Se instala encima de la anterior; tus capturas y tus ajustes se conservan.
2. Si Claude Desktop tiene abierto XcreenShot por MCP, no hace falta cerrarlo.
3. Para anotar: una línea por cosa, con una captura si ayuda. Por ejemplo: "Biblioteca: al borrar 3 capturas, el aviso dice 1".

## Qué mirar

### Captura
- **Capturar** (`Ctrl+Shift+PrtScn`) y la captura universal. Prueba arrastrar, hacer clic en una ventana, `F` (pantalla completa) y `Esc`.
- Durante la selección:
  - `H` o `F1` muestran las teclas;
  - `G` muestra guías de tamaños estándar; cerca de 1280×720 la selección se ajusta sola y el borde se pone amarillo.
- Panel de captura: activa **Capturar cursor** y haz una captura con el puntero sobre algo; en el editor el puntero se puede mover o borrar.
- Preferencias > Captura: **Sonido al capturar** (prueba Obturador), **Destello**, **Cruceta simplificada**.
- Con **Abrir en = Nada**: sale un aviso de Windows; un clic en él (o en **Abrir**) abre la captura.
- **Multicaptura**, la cuenta atrás y Repetir última captura (pruebas que tenías pendientes).

### Editor
- Las herramientas, copiar y pegar objetos, recortar, deshacer.
- Deja el puntero sobre una herramienta: el tooltip dice qué hace (Preferencias > General > Tooltips).
- Con el teclado: `F6` cambia de zona y las flechas recorren la barra de herramientas.

### Biblioteca y bandeja reciente
- El conmutador **Editor | Biblioteca** (`Ctrl+Shift+1` / `2`).
- Biblioteca: seleccionar con clic, `Ctrl`, `Shift` y arrastrando un rectángulo; doble clic para abrir; `F2` renombrar; `Supr` borrar y **Deshacer**.
- **Arrastra una captura de la bandeja o de la biblioteca** a un chat de Teams, a un correo de Outlook y a una carpeta. Si tiene marcas, debe llegar con ellas.
- Bandeja: el botón **Recientes**, el punto de cambios sin guardar, el menú del clic derecho y "No encontrado" si mueves un archivo desde el Explorador.

### Preferencias
- General: avisos (notificaciones) por tipo, **Volver a mostrar avisos y pistas** y **Permisos** (dice si Windows deja mostrar notificaciones).
- **Restablecer esta sección** al final de cada sección y **Restablecer todos los ajustes…** en Avanzado.
- Avanzado: carpeta de datos y comando de terminal.

### Ayuda
- `F1` o Ayuda > **Ayuda de XcreenShot**: páginas propias, en tu idioma, sin conexión.
- Ayuda > **Acerca de XcreenShot**: versión, privacidad, licencias y **Comprobar comunicación**.
- Ayuda > **Volver a mostrar la bienvenida** (la bienvenida solo sale sola en un equipo donde XcreenShot nunca se abrió; pruébala en el equipo de un colega si puedes).
- **Enviar comentarios…**: abre un borrador en el navegador; no envía nada.

### Para agentes y scripts
- `xcreenshot events watch` en una ventana de PowerShell, y una captura: van saliendo los eventos.

## Lo que todavía no está (no es un fallo)
- Video, compartir con servicios y las herramientas del editor de M2 (sellos, pasos, efectos, plantillas).
- macOS: se prueba cuando decidas empezar con Mac.
- Combinar capturas arrastrándolas al lienzo (M2).

## Preguntas que me ayudaría que contestes
- **Enviar comentarios**: ¿a dónde quieres que lleve? Hoy abre una incidencia en el repositorio de GitHub; si tus colegas no tienen acceso, dime un correo o una página.
- **Tamaño mínimo del texto**: el spec pide 11 pt (14,7 px) y Windows usa 9 pt (12 px). Hoy la app usa 12 px como mínimo. ¿Lo dejamos así?
