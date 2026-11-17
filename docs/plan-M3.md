# Plan de M3: captura avanzada (propuesta)

Propuesta de Claude del 2026-11-12, **provisional hasta que Carlos la apruebe**. M3 tiene 38 IDs en `PARITY.md` (33 de captura, 5 de reconocimiento de texto); 13 son Must. El spec de M3 ya está escrito en `01-captura.md` y `06-inteligentes.md`.

Cada entrega termina como las de M2: pruebas automáticas, `PARITY.md`, `QUESTIONS.md` con las decisiones provisionales y un instalador probado en su forma empaquetada.

| Entrega | Contenido | IDs Must |
|---|---|---|
| M3.1 · Presets | Lista de presets en el panel de captura y en el menú del ícono, crear desde los ajustes actuales o desde cero, editar, renombrar, duplicar, eliminar con Deshacer, reordenar, atajo global por preset, `.xpreset`, presets incorporados, comandos `preset.*` (CAP-058 a CAP-064) | CAP-058, CAP-059, CAP-060, CAP-062 |
| M3.2 · Extraer texto | Reconocimiento de texto del sistema (en Windows, `Windows.Media.Ocr`), sin conexión; idiomas; Extraer texto como modo de captura y su ventana de resultados (SMT-001, SMT-003 a SMT-006) | SMT-001, SMT-003, SMT-004, SMT-005, SMT-006 |
| M3.3 · Desplazamiento | Captura con desplazamiento manual y automático, y la unión de las partes (CAP-034 a CAP-036); el preset incorporado Desplazamiento llega aquí | CAP-034, CAP-035, CAP-036 |
| M3.4 · Más modos de selección | Región fija y posición, selector de ventanas, ventanas como objetos, menú, mano alzada, múltiples áreas (CAP-026, CAP-029 a CAP-033) | — |
| M3.5 · Tiempo y destino | Intervalo y programada (con el indicador En marcha de los presets), efectos al capturar, compartir al capturar, nombre, formato y carpeta del destino Archivo, anotación rápida (CAP-049 a CAP-053, CAP-057) | — |
| M3.6 · Otras fuentes | Portapapeles, archivo, PDF multipágina, capturas del sistema, página web por URL, cámara web (CAP-037 a CAP-042); pantallas HDR (CAP-022) y Lanzador de borde (CAP-004) | — |
| Solo macOS | AirDrop y carpeta de recepción (CAP-043), iPhone o iPad (CAP-044): cuando empiecen las pruebas en Mac | — |

## Orden

Primero M3.1, porque los presets guardan los ajustes de todos los modos que vienen después (cada modo nuevo se añade a lo que un preset puede guardar). Luego Extraer texto (todo Must y aislado del resto) y el desplazamiento, que es lo más difícil de M3. M3.4 a M3.6 son Should o Could.

## Para decidir con Carlos

- Aprobar este orden, o cambiarlo.
- **Dependencias nuevas** (se proponen en `spec/QUESTIONS.md` cuando llegue su entrega):
  - Extraer texto en Windows usa el reconocimiento de texto de Windows (está en el stack: "OCR del sistema operativo"). Para llamarlo desde Rust hace falta el crate `windows` de Microsoft con sus partes de WinRT; hoy la app usa `windows-sys`, que no las tiene.
  - PDF multipágina (CAP-041): en Windows, `Windows.Data.Pdf` (del sistema, con el mismo crate); en macOS, PDFKit. Sin librerías de terceros.
  - Página web por URL (CAP-037): con el navegador integrado (WebView2), en una ventana oculta.
- Lo de video y compartir que mencionan algunos IDs (presets de video, destinos) llega con M4 y M6.
