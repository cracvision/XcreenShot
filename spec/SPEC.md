# XcreenShot — Especificación funcional

**Versión del spec:** 0.1 (borrador) · **Fecha:** 2026-10-12
**Referencia de paridad:** Snagit para Windows 2026.4.0 (6 oct 2026) y Snagit para Mac 2026.4.1 (9 oct 2026), según documentación pública.

---

## 0. Qué es este documento y cómo se usa

Este es el spec funcional de un **clean room**. Lo escribe el *equipo de especificación* a partir de fuentes públicas, todas registradas en [`sources.md`](sources.md). El *equipo de construcción* (Claude Code) trabaja **únicamente** a partir de estos archivos:

| Archivo | Contenido | Prefijo de IDs |
|---|---|---|
| `SPEC.md` | Este documento: objetivo, principios, alcance, hitos, convenciones, formatos, glosario, decisiones pendientes | — |
| `01-captura.md` | Captura de imagen, modos de selección, presets, hotkeys de captura | `CAP-` |
| `02-editor.md` | Editor de imágenes: herramientas, estilos, efectos, lienzo, plantillas, archivos | `EDT-` |
| `03-video.md` | Grabación de video, cámara, audio, dibujo en vivo, edición de video, GIF | `VID-` |
| `04-biblioteca-compartir.md` | Biblioteca, bandeja reciente, compartir y destinos | `LIB-`, `TRY-`, `SHR-` |
| `05-app.md` | Aplicación: arranque, ícono de bandeja/menú, preferencias, atajos, permisos, CLI/MCP, actualizaciones, accesibilidad, distribución | `APP-` |
| `06-inteligentes.md` | Funciones inteligentes en el dispositivo: grabador de pasos, OCR, redacción automática, separar elementos, quitar fondo, etc. | `SMT-` |
| `PARITY.md` | Lista de verificación de paridad (se genera a partir de los IDs) | — |
| `QUESTIONS.md` | Preguntas abiertas del equipo de construcción | — |
| `research/` | Inventarios de investigación (evidencia del clean room). **No hace falta leerlos para construir.** | — |

El spec describe **qué hace la app y cómo se comporta para el usuario**. No describe cómo lo implementa Snagit, y no hace falta saberlo.

Si algo es ambiguo, el equipo de construcción **no** consulta Snagit. Anota la duda en `QUESTIONS.md` y pregunta a Carlos.

---

## 1. Objetivo

Construir **XcreenShot**, una herramienta de captura de pantalla, anotación y grabación de video para **Windows y macOS**, con **paridad funcional** con Snagit, para uso de Carlos y sus colegas. Una vez alcanzada la paridad, XcreenShot se extenderá con funciones propias.

### Cuándo se considera alcanzada la paridad
1. Todas las funciones con prioridad **Must** de `PARITY.md` cumplen sus criterios de aceptación en Windows 11 y en macOS (Apple Silicon).
2. Carlos trabaja **dos semanas** con XcreenShot sin abrir Snagit.

---

## 2. Principios

1. **Una app, un comportamiento.** El mismo código y el mismo comportamiento en Windows y macOS. Cuando Snagit ofrece algo en una sola plataforma, XcreenShot lo ofrece en ambas siempre que el sistema operativo lo permita. Si no lo permite, la función se marca con su plataforma y el motivo.
2. **Todo es un comando.** Cada acción de la app es un comando registrado, con ID, parámetros y resultado. La UI, los atajos de teclado, el CLI y el servidor MCP llaman a los mismos comandos. Esto hace la app extensible y operable por agentes.
3. **Local primero y privado.** Sin telemetría, sin cuenta obligatoria y sin enviar nada a la nube salvo que el usuario elija un destino de compartir. Todo el procesamiento inteligente (OCR, detección de rostros, etc.) ocurre en el dispositivo.
4. **Identidad propia.** Nombres, íconos, colores, textos de ayuda y diseño visual propios. La UI no usa marcas ni nombres distintivos de TechSmith (ver §8). Los nombres genéricos de herramientas (Flecha, Texto, Recortar, Desenfocar) sí se pueden usar.
5. **Formatos propios y abiertos.** Documentos editables en ZIP con JSON y PNG (ver §7). **No hay compatibilidad con `.snagx`, `.snagtheme`, `.snagpresets` ni `.snag`**: son formatos propietarios y leerlos exigiría ingeniería inversa.
6. **Mejorar donde Snagit tiene límites documentados.** Ejemplos: posición y tamaño numéricos de objetos; una biblioteca que no depende del indexador del sistema operativo; funciones que en Snagit son solo de una plataforma.

---

## 3. Alcance

### 3.1 Dentro del alcance
Todas las funciones de Snagit para Windows y Mac vigentes en octubre de 2026 que el usuario puede ver y usar, salvo las excluidas en §3.2.

### 3.2 Fuera del alcance y reemplazos

| Función de Snagit | Decisión para XcreenShot |
|---|---|
| Screencast y Share Link hacia servidores de TechSmith | Reemplazado por Share Link con proveedor configurable (`SHR-`; ver D-02) |
| Camtasia Editor, Camtasia Online, Camtasia Rev, grabación multipista hacia Camtasia | Fuera. Candidato futuro: exportar pistas separadas (pantalla, cámara, audio, cursor) |
| TechSmith Assets (tienda de stamps, temas, plantillas, fotos; generador de imágenes con IA) | Fuera. Se reemplaza por paquetes locales importables de stamps, temas y plantillas |
| Cuenta TechSmith, licencias, suscripción, activación | Fuera |
| TechSmith Fuse (app móvil) | Fuera en v1. Candidato futuro |
| Programa Experiments y recopilación de uso | Fuera. Puede haber *feature flags* internos, sin telemetría |
| COM Server API (Windows) | Reemplazado por CLI + servidor MCP multiplataforma (`APP-`) |
| Herramienta de despliegue empresarial | Reemplazada por un archivo de configuración administrada (`APP-`) |
| Snagit Printer (impresora virtual) | Pendiente (D-03). Propuesta: importar PDF multipágina como captura |
| Importar `.snagx`, `.snag`, `.snagproj`, `.snagtheme`, `.snagpresets`, `.snagarchive` | Fuera (formatos propietarios) |
| Funciones que TechSmith ya retiró (TWAIN, Translate Text/XLIFF, banderas y perfiles legados, Knowmia, Panopto, X/Twitter, extensión de Chrome) | Fuera |
| Funciones generativas de IA | Fuera (Snagit tampoco las tiene dentro de la app) |
| Menús y fondos de "marketing" de TechSmith (Learning Center, tours, encuestas) | Fuera. Hay ayuda propia y sencilla (`APP-`) |

---

## 4. Hitos

Cada hito termina con una versión instalable en **ambas plataformas** que se puede usar a diario.

| Hito | Nombre | Objetivo | Criterio de salida |
|---|---|---|---|
| **M0** | Fundaciones | Esqueleto de la app: Tauri 2 + React, registro de comandos, CLI, ícono en la bandeja/barra de menú, atajos globales, permisos de macOS, preferencias, i18n EN/ES, CI que compila Windows y macOS | La app arranca en ambas plataformas, registra un atajo global y ejecuta un comando de prueba desde la UI, el CLI y el atajo |
| **M1** | Captura y edición básicas | Captura de región, ventana y pantalla completa con la captura universal; lupa; multimonitor y DPI; portapapeles; editor con Flecha, Texto, Forma, Desenfocar, Recortar, deshacer/rehacer; guardar PNG/JPG y `.xshot`; biblioteca con autoguardado; bandeja reciente básica | Carlos puede reemplazar Snagit para capturas rápidas con anotaciones simples |
| **M2** | Editor completo y biblioteca | Todas las herramientas, estilos guardados, temas, favoritos, efectos, operaciones de lienzo, plantillas, formatos de exportación, exportación por lotes, barra de herramientas configurable, impresión; biblioteca con etiquetas, búsqueda y filtros | Paridad del editor de imágenes |
| **M3** | Captura avanzada | Desplazamiento (manual y automático), menús, mano alzada, múltiples áreas, región fija, intervalo y programada, presets con atajos, extraer texto (OCR), detalles de captura, página web por URL, cámara como imagen, importar capturas del sistema | Paridad de captura de imagen |
| **M4** | Video y GIF | Grabación con barra de control, micrófono, audio del sistema, cámara (PiP y pantalla completa), efectos de cursor, dibujo en vivo, temporizador, calidad y FPS, recorte, guardar fragmento, guardar fotograma, combinar videos, GIF, diapositivas narradas, reducción de ruido, fondo de cámara, recuperación tras fallo | Paridad de video |
| **M5** | Funciones inteligentes | Grabador de pasos con sus salidas, redacción automática, editar texto de la imagen, separar elementos, simplificar automático, quitar fondo, aplicar solo al texto, QR, medición de píxeles | Paridad de funciones inteligentes |
| **M6** | Compartir y distribución | Destinos de compartir, Share Link, historial, firma de código, instaladores, actualizador automático | **Gate de paridad** (§1) |

Las funciones pueden adelantarse a un hito anterior si resultan sencillas, pero ningún hito se cierra con funciones **Must** pendientes.

---

## 5. Convenciones del spec

### 5.1 IDs
- Formato `PREFIJO-NNN` (por ejemplo, `CAP-001`), numerados por archivo.
- Un ID **nunca se reutiliza**. Si una función se elimina, se marca **Retirado** y se explica por qué.
- Los commits, las pruebas y `PARITY.md` hacen referencia a los IDs.

### 5.2 Plantilla de cada función

```markdown
### CAP-001 · Nombre de la función
- **Hito:** M1 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** <nombre de la función en Snagit> (solo para seguimiento; no usar en la UI)
- **Comportamiento:**
  - Qué ve y qué puede hacer el usuario, en viñetas.
  - Opciones, valores por defecto y casos límite.
- **Criterios de aceptación:**
  - [ ] Resultado observable y verificable.
  - [ ] Otro resultado observable.
- **Notas de plataforma:** (opcional) diferencias inevitables entre Windows y macOS.
- **Mejora sobre Snagit:** (opcional) dónde XcreenShot va más allá.
- **Fuentes:** research/01-captura.md §6.1
```

### 5.3 Prioridades
- **Must:** necesaria para el gate de paridad. Carlos la usa o la usaría a diario.
- **Should:** parte de la paridad, pero puede llegar después del gate.
- **Could:** detalle menor, caso marginal o comodidad.

### 5.4 Criterios de aceptación
- Describen resultados que se pueden **observar o probar**, nunca detalles de implementación.
- Cuando aplica, incluyen un caso límite.
- Si la función difiere por plataforma, el criterio indica en cuál aplica.

---

## 6. Glosario (modelo de dominio)

| Término | Significado |
|---|---|
| **Captura** | Resultado de capturar: una imagen, un video o una guía de pasos, con sus metadatos de origen |
| **Documento** | Imagen editable: píxeles base más objetos vectoriales y efectos. Se guarda como `.xshot` |
| **Objeto** | Anotación vectorial editable sobre el lienzo (flecha, texto, globo, forma, sello, paso, etc.) o imagen flotante |
| **Aplanar** | Fusionar un objeto con los píxeles; deja de ser editable |
| **Lienzo** | Área de la imagen del documento, con color de fondo propio (incluido transparente) |
| **Biblioteca** | Carpeta del usuario con todas las capturas, más un índice local con etiquetas y metadatos |
| **Bandeja reciente** | Tira de miniaturas en el editor con las capturas y archivos abiertos recientemente |
| **Preset** | Conjunto guardado de ajustes de captura, con nombre y atajo opcional |
| **Tema** | Conjunto con nombre de colores, una fuente y sombra sí/no, del que se generan los estilos de cada herramienta |
| **Estilo guardado** | Combinación de propiedades de una herramienta, lista para aplicar con un clic |
| **Favoritos** | Herramienta que reúne estilos marcados con estrella de distintas herramientas |
| **Destino** | Lugar al que se envía una captura: archivo, portapapeles, aplicación, servicio en la nube, etc. |
| **Comando** | Acción registrada de la app, invocable desde la UI, atajos, CLI y MCP |
| **Metadatos de captura** | Fecha y hora, aplicación de origen y versión, URL (si es un navegador), sistema operativo, usuario |

---

## 7. Formatos de archivo propios

| Formato | Contenido | Uso |
|---|---|---|
| `.xshot` | ZIP con `manifest.json` (versión del formato), `document.json` (lienzo, objetos, efectos, historial de recorte, tipo `image` o `guide`), `base.png` (píxeles originales sin recortar), `images/*.png` (imágenes flotantes y sellos incrustados), `meta.json` (metadatos de captura) y `thumb.png` | Documento editable de imagen o de guía de pasos |
| `.mp4` | Video H.264 (por defecto) o H.265 | Grabaciones; metadatos en el índice de la biblioteca |
| `.gif` | GIF animado | Exportación de video |
| `.xtheme` | JSON | Exportar e importar temas |
| `.xpreset` | JSON (uno o varios presets) | Exportar e importar presets de captura |
| `.xstamps` | ZIP de imágenes más `pack.json` | Paquetes de sellos importables |
| `.xtemplate` | `.xshot` con zonas de imagen marcadas | Plantillas propias |
| `library.db` | SQLite dentro de la carpeta de la biblioteca | Índice, etiquetas, favoritos, metadatos. Se puede reconstruir a partir de los archivos |

Reglas:
- Los formatos llevan número de versión y la app migra versiones antiguas.
- Un `.xshot` creado en Windows se abre igual en macOS y viceversa, **sin pérdida de propiedades** (mejora sobre Snagit).

---

## 8. Nombres en la interfaz (identidad propia)

La UI está en inglés y español (D-07). Esta tabla fija los nombres propios de XcreenShot para conceptos que en Snagit tienen nombre distintivo. Los términos genéricos de edición (Arrow/Flecha, Text/Texto, Crop/Recortar, Blur/Desenfocar, Magic Wand/Varita mágica, Library/Biblioteca, etc.) se pueden usar libremente.

| En Snagit | UI en inglés | UI en español |
|---|---|---|
| All-in-One capture | Universal Capture | Captura universal |
| Capture Window | Capture Panel | Panel de captura |
| Capture Widget | Edge Launcher | Lanzador de borde |
| Recent Captures Tray | Recent Tray | Bandeja reciente |
| Quick Styles | Saved Styles | Estilos guardados |
| Quick Effects | Saved Effects | Efectos guardados |
| Step Capture | Step Recorder | Grabador de pasos |
| Smart Redact | Auto Redact | Redacción automática |
| Smart Move | Lift Objects | Separar elementos |
| Grab Text | Extract Text | Extraer texto |
| Screen Draw | Live Draw | Dibujo en vivo |
| Video from Images / Add Narration | Narrated Slides | Diapositivas narradas |
| Capture Info | Capture Details | Detalles de captura |
| Cut Out | Remove Strip | Quitar franja |
| Image from Template | New from Template | Nuevo desde plantilla |
| Share Link | Share Link | Enlace para compartir |
| Screencast | — (no existe; ver D-02) | — |

---

## 9. Decisiones pendientes (Carlos)

| ID | Decisión | Propuesta del equipo de especificación |
|---|---|---|
| D-01 | Licencia del repositorio | Privado por ahora. Si se abre: MIT o Apache-2.0 |
| D-02 | Proveedor de Share Link | Configurable por plugin. Primero: carpeta de OneDrive o Google Drive sincronizada (enlace del propio servicio). Después, opcional: servidor propio en el Mac Mini |
| D-03 | Impresora virtual (Snagit Printer) | No hacer impresora virtual. Importar PDF multipágina como captura (`CAP-`) |
| D-04 | Codificación de video | Codificadores nativos del sistema (Media Foundation en Windows; AVFoundation/VideoToolbox en macOS). Evitar empaquetar FFmpeg para no complicar licencias |
| D-05 | Destinos que requieren OAuth (Slack, Teams, Google Drive, Dropbox, Box, OneDrive, YouTube) | Requieren registrar apps de desarrollador en cada servicio. Priorizar los que Carlos y sus colegas usan; el resto, después del gate |
| D-06 | Firma de código | Certificado Authenticode para Windows y Apple Developer Program para macOS, necesarios para compartir la app con colegas sin bloqueos |
| D-07 | Idiomas de la UI | Inglés y español desde M0 |
| D-08 | Atajos por defecto | Mientras Snagit y XcreenShot convivan, usar atajos que no choquen con los de Snagit (por ejemplo, `Ctrl+Shift+PrtScn` en Windows). Tras el gate, ofrecer el perfil de atajos equivalente al de Snagit para mantener la memoria muscular |
| D-09 | Nombre definitivo | XcreenShot |
