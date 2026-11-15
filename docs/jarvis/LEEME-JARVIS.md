# XcreenShot — guía para Jarvis

Jarvis: vas a ejecutar el protocolo de pruebas (OQ) de XcreenShot en este equipo (CRAC_BEAST) para Carlos. Esta guía dice qué es la app, qué leer, cómo manejarla y qué **no** hacer. Léela entera antes de empezar.

Fecha de esta guía: 2026-11-15. El estado al día está siempre en `C:\Projects\XcreenShot\docs\ESTADO.md` (sección "Dónde estamos" y la primera entrada de "Cambios de la última sesión").

---

## 1. Qué es XcreenShot

- App de **captura de pantalla, anotación y grabación de video** para Windows y macOS, desarrollada para Carlos. En Windows está construida casi entera; macOS todavía no se ha probado.
- **Versión instalada en este equipo:** la que diga `xcreenshot --version` (0.2.4 o posterior). Las versiones se publican en https://github.com/cracvision/xcreenshot-releases y la app se actualiza con **Ayuda > Buscar actualizaciones**.
- Partes principales:
  - **captura**: región, ventana, pantalla completa, desplazamiento, página web, PDF, intervalo, programada y multicaptura;
  - **editor** con herramientas de anotación, efectos, plantillas y sellos;
  - **biblioteca** y **bandeja reciente**;
  - **funciones inteligentes**: extraer texto, redacción automática, grabador de pasos, quitar fondo y códigos QR;
  - **video**: grabación, micrófono, cámara, dibujo en vivo, GIF y diapositivas narradas;
  - **compartir**: correo, Office, otra app, Compartir de Windows y red local con otros usuarios;
  - **automatización**: CLI `xcreenshot` y servidor MCP.
- La interfaz está en **español** (también en inglés). Los menús y botones de las guías usan los nombres en español.

### Dónde vive

| Qué | Ruta |
|---|---|
| App instalada | `%LOCALAPPDATA%\XcreenShot\xcreenshot-app.exe` |
| CLI | `%LOCALAPPDATA%\XcreenShot\xcreenshot.exe` (también `xcreenshot` en una terminal nueva) |
| Ajustes | `%APPDATA%\XcreenShot\config` |
| Datos y registros | `%LOCALAPPDATA%\XcreenShot\data` (registros en `data\logs\`) |
| Biblioteca de Carlos (sus capturas reales) | `C:\Users\cracv\OneDrive\Documents\XcreenShot` |
| Proyecto (código y documentación) | `C:\Projects\XcreenShot` |

---

## 2. Qué leer y en qué orden

1. **Este archivo.**
2. **El protocolo que vas a ejecutar:** `C:\Projects\XcreenShot\tests\manual\XcreenShot_OQ_Windows_v1_1.xlsx`.
   - Hoja **Portada**: objetivo, alcance, prerrequisitos, criterios de aceptación y las pruebas que necesitan algo especial.
   - Hoja **Pruebas**: 174 casos con acción y resultado esperado.
   - Hojas **Resumen** (se calcula sola) y **Desviaciones**.
3. **La misma guía en texto:** `tests\manual\E2E-windows.md`. Mismo orden; cada bloque enlaza su guía detallada.
4. **Las guías detalladas** de `tests\manual\*.md`, cuando un caso no esté claro. Cada bloque del OQ dice cuáles.
5. **Cómo manejar la app por comandos:** `C:\Projects\XcreenShot\AGENTS.md`. Ahí están todos los comandos del CLI y del MCP, con ejemplos.
6. **Qué debe hacer cada función:** el spec, en `spec\`.
   - `SPEC.md` es el índice.
   - `01-captura.md`, `02-editor.md`, `03-video.md`, `04-biblioteca-compartir.md`, `05-app.md` y `06-inteligentes.md` tienen los requisitos.
   - Cada caso del OQ cita sus IDs (por ejemplo `CAP-012`): busca el ID en el spec para ver el comportamiento exacto y sus criterios de aceptación.
7. **Qué está hecho y qué no:** `spec\PARITY.md`. Una fila por ID: `Win` = verificado en Windows; `En curso` = construido; `—` = no construido.
8. **Decisiones provisionales:** `spec\QUESTIONS.md`. Si algo se comporta distinto de lo que esperarías pero está anotado ahí como decisión provisional, **no es un fallo**: anótalo como comentario, no como desviación.
9. **Historia y contexto (opcional):** `docs\ESTADO.md` y las decisiones de arquitectura de `docs\adr\`.

No leas `spec\research\`: es material del proceso y no hace falta para probar.

**Si trabajas con el paquete `.zip`** (`XcreenShot_Jarvis_Kit.zip`), los mismos archivos están en carpetas numeradas:

| Carpeta del paquete | Contenido |
|---|---|
| `00-LEEME-JARVIS.md` | Esta guía |
| `01-OQ\` | El Excel del OQ y `E2E-windows.md` |
| `02-AGENTS.md` | Los comandos del CLI y del MCP |
| `03-guias\` | Las guías detalladas (`tests\manual\`) |
| `04-spec\` | El spec, `PARITY.md` y `QUESTIONS.md` |
| `05-estado\` | `ESTADO.md`, las decisiones de arquitectura y el esquema del formato `.xshot` |

Las rutas de las secciones 1, 3, 4 y 6 son las del equipo, no las del paquete. Si el proyecto cambia, se regenera con `powershell -File C:\Projects\XcreenShot\scripts\make-jarvis-kit.ps1`.

---

## 3. Cómo ejecutar el OQ

1. **Trabaja sobre una copia.** Copia el Excel a `%USERPROFILE%\Documents\XcreenShot-OQ\XcreenShot_OQ_Windows_v1_1_ejecucion_<AAAA-MM-DD>.xlsx` y rellena esa. El original del proyecto se queda como plantilla.
2. **Guarda la evidencia** (capturas, videos) en `%USERPROFILE%\Documents\XcreenShot-OQ\evidencia\`, con el ID del caso en el nombre (`1.4.2.png`). No la guardes dentro de `C:\Projects\XcreenShot`: contiene la pantalla de Carlos.
3. Ejecuta los casos **en orden**, una sesión cada vez. En cada fila rellena solo las columnas amarillas:
   - **Resultado obtenido**;
   - **Resultado**: PASS, FAIL o N/A;
   - **Ejecutado por**: `JARVIS`;
   - **Fecha**;
   - **Evidencia**;
   - **Desviación**;
   - **Comentarios**.
4. **PASS** solo si lo obtenido coincide con el resultado esperado. **FAIL** si no. **N/A** si el caso no aplica, con el motivo en Comentarios (por ejemplo: "requiere dos monitores").
5. **Todo FAIL abre una desviación** en la hoja Desviaciones (DEV-001, DEV-002…) con la descripción, la severidad y la evidencia. No intentes arreglar el fallo tú: lo corrige Claude Code (la sesión de desarrollo en `C:\Projects\XcreenShot`) a partir de tu registro.
6. Al terminar cada sesión, guarda el Excel y dile a Carlos cuántos PASS, FAIL y N/A hubo, y las desviaciones abiertas.

### Cómo comprobar resultados sin depender de la vista

El CLI y el MCP leen el estado real de la app. Úsalos para confirmar lo que ves. Los comandos exactos están en `AGENTS.md`; añade siempre `--no-launch` (si la app no está abierta, el comando falla en lugar de abrirla).

| Para saber… | Comando |
|---|---|
| versión y rutas | `xcreenshot --no-launch app about --json` |
| documentos abiertos en el editor | `xcreenshot --no-launch document list --json` |
| lo último de la biblioteca | `xcreenshot --no-launch library list --json` |
| un ajuste | `xcreenshot --no-launch settings get <clave> --json` |
| si está grabando | `xcreenshot --no-launch record status --json` |
| cámaras y micrófonos | `xcreenshot --no-launch record cameras --json` · `record devices --json` |

- El servidor MCP es `xcreenshot mcp`, por stdio. Sus herramientas son los mismos comandos.
- `document_image` devuelve la imagen final de una captura para que la mires.
- Los comandos actúan sobre la app real de Carlos. Para **comprobar**, úsalos sin miedo. Para **cambiar** cosas, solo cuando el caso lo pida.

---

## 4. Reglas de seguridad (obligatorias)

### 4.1 Ratón y teclado

**Dibujo en vivo** (bloque 3.10) y **Multicaptura** (bloque 1.9) se quedan con los clics del ratón, y Dibujo en vivo también con el teclado, mientras están activos. Es su comportamiento normal. El 2026-11-15, Dibujo en vivo dejó a Carlos sin ratón ni teclado dos veces y tuvo que apagar el equipo con el botón. Desde la versión 0.2.4 hay protecciones, pero:

- **`Esc` siempre los apaga** y devuelve el ratón y el teclado al instante. La barra de Dibujo en vivo tiene además un botón **✕** con **Esc**.
- Ejecuta esos dos bloques **solo cuando Carlos no esté usando el equipo**, y avísale antes.
- Ejecútalos con un **tiempo límite**, así ratón y teclado vuelven solos aunque algo falle:
  1. Cierra XcreenShot: `xcreenshot --no-launch app quit --json`.
  2. Ábrelo desde una terminal con el límite: `$env:XS_INPUT_RELEASE_SECS = "30"; Start-Process "$env:LOCALAPPDATA\XcreenShot\xcreenshot-app.exe"`.
  3. Al terminar el bloque, ciérralo y ábrelo de nuevo de forma normal.
- Si dudas, ejecútalos dentro de **Windows Sandbox**, un Windows desechable donde nada sale de su ventana. Ver la sección 6.

### 4.2 Atajos de teclado que chocan contigo

- Casi todos los atajos globales de XcreenShot son `Ctrl+Alt+Mayús+<letra>`. **Tus propios atajos (Jarvis) usan combinaciones con `Ctrl+Alt`** y pueden quedarse con ellos.
  - Si un atajo de XcreenShot no responde, usa la misma función desde el **menú del ícono** (junto al reloj), desde el panel de captura o con el CLI.
  - Anótalo en Comentarios; no es un fallo de XcreenShot.
- Atajos propios del perfil de Carlos:
  - **Capturar**: `Ctrl+Shift+PrtScn` (temporal, mientras otra app de captura ocupe `PrtScn`);
  - **Dibujo en vivo**: `Ctrl+Shift+F11`.
  - Las guías dicen `Ctrl+Alt+Mayús+D` para Dibujo en vivo: ese es el de fábrica, no el de Carlos.
- `Ctrl+P` no es un atajo de XcreenShot: imprime normal.

### 4.3 Pide permiso a Carlos antes de

- **Desinstalar o reinstalar XcreenShot** (bloque 3.9).
- **Cambiar ajustes de Windows o de seguridad**: suspender el equipo (bloque 1.2), privacidad, firewall, características de Windows. También cualquier paso que pida permisos de administrador, como `managed.json` en `C:\ProgramData`.
- **Mandar algo fuera del equipo.** Compartir > Correo solo deja un borrador: **nunca lo envíes**. Tampoco la IA en la nube, que usa la clave de Carlos y es de pago.
- **Borrar** capturas de su biblioteca real. Para pruebas que borran, usa capturas que hayas hecho tú en la prueba.
- **Publicar** cualquier cosa.

### 4.4 Otras reglas

- **Privacidad:** las capturas muestran la pantalla de Carlos. No las subas a ningún sitio ni las adjuntes fuera de su equipo.
- **No uses, abras ni analices la otra app de captura** que hay instalada, ni sus archivos, para comparar. El proyecto se construye solo a partir de su propio spec (regla de "sala limpia").
- **Las rutas de prueba** (exportar la biblioteca, exportar por lotes) van a una carpeta tuya bajo `Documents\XcreenShot-OQ\`, nunca dentro de la biblioteca de Carlos.

---

## 5. Lo que necesita a Carlos o un aparato

Ver la hoja **Portada > 8** del Excel. En resumen:

- **Webcam:** la GoPro de Carlos, con la app GoPro Webcam. Aparece como **GoPro Webcam**. Hay que elegirla en la barra de grabación (flecha de **Cámara**). Tarda unos segundos en pasar de su pantalla gris de espera al video en vivo. Funciona desde la versión **0.2.5**.
- Probar la cámara **tapada o desconectada** (VID-038) es una acción física: pídesela a Carlos.
- **Otro PC** con XcreenShot en la misma red y un colega (red local, SHR-042 a SHR-044).
- **Clave de la API de Anthropic** de Carlos (IA en la app, SMT-039 a SMT-043, tanda A).
- **Word, PowerPoint, Excel y Outlook** (Compartir en Office y Correo), una impresora o Microsoft Print to PDF, un teléfono (leer un QR), un segundo monitor, auriculares con cable.
- **Micrófono:** antes de los casos de sonido, comprueba en Configuración > Sistema > Sonido > Entrada que la barra se mueve al hablar. El micrófono de la bocina de Carlos entregaba silencio.

---

## 6. Windows Sandbox (pruebas sin riesgo)

Windows Sandbox está activado en este equipo: es un Windows limpio y desechable dentro de una ventana. Nada de lo que pasa dentro toca el Windows de Carlos y, al cerrarlo, se borra todo.

- Script: `C:\Projects\XcreenShot\scripts\dev\sandbox.ps1 -Inside <prueba.ps1> -Files <instalador.exe>`. Instala XcreenShot dentro, corre la prueba, guarda un registro y se apaga solo. Pruebas de ejemplo en `scripts\dev\sandbox\` (`fresh.ps1`, `input.ps1`, `update.ps1`).
- **Solo un Sandbox a la vez.** No lo cierres a la fuerza mientras arranca: la prueba no correría.
- El Sandbox no tiene cámara ni micrófono.
- Las capturas de pantalla hechas con la API clásica de Windows (GDI) no muestran las ventanas de XcreenShot. Para ver lo que muestra la app, pregúntale a la app (CLI o MCP).

---

## 7. Si algo va mal

- **La app no responde:** `xcreenshot --no-launch app about --json`. Si falla, revisa el registro en `%LOCALAPPDATA%\XcreenShot\data\logs\` (un archivo por día).
- **El ratón o el teclado no responden:** pulsa `Esc`. Si no vuelven, espera el tiempo límite que pusiste (sección 4.1). Como último recurso, `Ctrl+Alt+Supr` siempre funciona.
- **Información para un informe:** **Ayuda > Copiar información de diagnóstico** (no incluye rutas de la biblioteca).
- **Todo FAIL** se registra en Desviaciones con evidencia. Claude Code lo corrige y publica una versión nueva. Entonces repites el caso con la versión nueva y anotas la verificación en la desviación.
