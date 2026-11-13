# Ideas de Carlos (funciones propias, sin aprobar)

Ideas que Carlos quiere conservar y que todavía **no son spec**:
- No se construyen hasta que Carlos responda sus preguntas y elija el hito.
- Cuando una se aprueba, se escribe su bloque con ID en el archivo del spec que corresponda (`CAP-`, `SMT-`…), y aquí se anota qué ID recibió.

Formato de cada idea:
- qué propone Carlos;
- con qué funciones del spec se relaciona;
- qué choca o qué riesgos tiene;
- las preguntas pendientes, con una propuesta para cada una.

---

## IDEA-001 · Modo multicaptura (2026-11-01)

> **Aprobada (2026-11-01):** es **CAP-067** en `01-captura.md`. Carlos respondió: clic captura la ventana resaltada y arrastrar un rectángulo; el clic llega a la app; `Esc` termina; la X y Limpiar no borran de la biblioteca. Lo que sigue queda como registro.

**Lo que propone Carlos:**
- Un modo en el que **cada clic izquierdo hace una captura** y el **clic derecho sale** del modo.
- Mientras dura, un **panel** muestra las miniaturas de las capturas de esa sesión, y solo de esa sesión. Tiene un botón **Limpiar**, y cada miniatura tiene una **X** para sacarla del panel.
- En el panel se eligen **una o varias** capturas para **insertarlas en un documento**.

**Relación con el spec:**
- **CAP-033 Múltiples áreas** (M3) también elige varias zonas seguidas, con una X para quitar cada una, pero produce **una sola imagen compuesta**. Esta idea produce **capturas separadas** y deja elegir después cuáles usar.
- **Grabador de pasos** (SMT-022 a SMT-024, M5) también captura en cada clic, con el clic llegando a la app de debajo, pero produce una guía con texto.
- Insertar capturas en un documento necesita **objetos de imagen** en el editor, que llegan en M2; CAP-033 también los usa.
- La **Bandeja reciente** (`TRY-`) ya muestra todas las capturas. El panel sería como una bandeja reducida a la sesión.

**Preguntas para Carlos:**

1. **¿Qué captura cada clic?**
   - a) Lo resaltado bajo el puntero (la ventana o la subárea, como en la captura universal, CAP-016); arrastrar captura una región.
   - b) Siempre la pantalla completa.
   - c) Una región fija que se elige al empezar.
   - *Propuesta:* a.
2. **¿El clic llega a la app de debajo?**
   - Si llega, se puede ir navegando por una app y capturando cada pantalla, sin congelarla.
   - Si no llega (pantalla congelada, como al capturar hoy), todos los clics capturan la misma imagen congelada y, para cambiar lo que se ve, hay que salir del modo.
   - *Propuesta:* que llegue. Es la que permite documentar un proceso paso a paso.
3. **¿Se guardan en la biblioteca?**
   - *Propuesta:* cada captura se guarda como una captura normal. La X y Limpiar solo la sacan del panel; no la borran.
4. **¿Dónde se insertan?**
   - En el documento abierto en el editor, como objetos de imagen que se pueden mover.
   - En un documento nuevo que las junta (una debajo de otra, como la composición "Apiladas" de CAP-033).
   - *Propuesta:* en el documento abierto; si no hay ninguno, en uno nuevo con las capturas apiladas.
5. **Teclado y accesibilidad:**
   - *Propuesta:* `Esc` también sale del modo.

**Riesgos:**
- Si el clic llega a la app de debajo, salir con clic derecho impide usar los menús contextuales de esa app durante la sesión.
  - *Propuesta:* salir con `Esc` o con el botón **Terminar** del panel.
  - Si Carlos prefiere el clic derecho, que sea un ajuste.
- En macOS, detectar los clics en otras apps necesita el permiso de Accesibilidad, igual que el Grabador de pasos.

**Hito propuesto:** M3, junto a CAP-033, con la que comparte casi todo. Necesita los objetos de imagen de M2.

---

## IDEA-002 · Entender imágenes con IA en la nube (2026-11-01)

> **Aprobada (2026-11-01):** es **SMT-039 a SMT-043** en `06-inteligentes.md` (sección 8). Carlos respondió: acepta cambiar el principio "todo local" con las salvaguardas; las dos vías (A, la app con una clave, y B, el servidor MCP); cada colega pone su propia clave. Las preguntas 3 y 5 quedaron con la propuesta: nombre a petición, nunca automático (c por defecto y a como complemento), y modelo configurable (Opus 5.5 para preguntas, Haiku 5.5 para nombres). Lo que sigue queda como registro.

**Lo que propone Carlos:**
- Conectar la app a la API de Claude (modelo Opus 5.5, `claude-opus-5-5`) para que reconozca lo que hay en una imagen y cree contexto a partir de ella.
  - Ejemplo: el usuario sube la foto de una pareja en la playa, da los detalles relevantes y pide un poema basado en esa imagen.
- **Nombrar cada archivo** según el contenido visual de la imagen.

**Choca con el spec actual. Para hacerla, Carlos tiene que cambiar un principio:**
- **SPEC §2.3, "Local primero y privado":** no se envía nada a la nube salvo a un destino de compartir elegido por el usuario, y todo el procesamiento inteligente ocurre en el equipo.
- **SPEC §3.2:** las funciones generativas de IA están fuera del alcance.
- **APP-054:** hay una lista cerrada de conexiones de red permitidas. Esta sería una nueva, y la prueba `scripts/check-no-telemetry.mjs` tendría que admitirla.
- **CLAUDE.md:** "El procesamiento inteligente es siempre local."

**Cómo cambiarlo sin perder la privacidad (propuesta):**
- La función viene **desactivada**. Se activa en Preferencias con una clave de API, que se guarda en el llavero del sistema (nunca en archivos ni en registros).
- **Nada se envía sin una acción del usuario** sobre esa imagen. Antes del primer envío, un aviso explica qué se envía y a quién.
- Acerca de (APP-015) y APP-054 la añaden a las conexiones permitidas, "solo si el usuario la activa".
- Todas las funciones locales siguen funcionando sin ella y sin conexión.

**Riesgo principal: el nombre automático.**
- Nombrar cada archivo al capturar significa enviar **todas** las capturas a la nube.
- Las capturas suelen tener correos, datos de clientes, contraseñas a la vista o datos regulados (GxP).
- Opciones:
  - a) **A petición:** un botón "Sugerir nombre", sobre una captura o sobre varias elegidas. Nunca automático.
  - b) **Automático**, solo con un ajuste explícito, y con excepciones (por ejemplo, nunca para ciertas apps).
  - c) **Local, sin IA en la nube:** los marcadores del nombre de archivo (CAP-053: app, título de la ventana, fecha) y el OCR del sistema (`SMT-`) ya dan nombres útiles sin red.
- *Propuesta:* c por defecto, y a como complemento.

**Otra vía, sin meter la IA en la app:**
- XcreenShot ya tiene un servidor MCP. Claude Desktop o Claude Code, conectados a él, podrían:
  - leer una captura;
  - describirla o escribir el poema;
  - renombrarla con los comandos de la app.
- La app seguiría siendo local, y la IA la pondría el cliente que cada usuario elija, con su propia cuenta.
- Faltan dos comandos: uno que entregue la imagen al cliente y el de renombrar (TRY-006, entrega 5).

**Usos con más valor en una herramienta de capturas** (el mecanismo es el mismo que el del poema: imagen más petición, y sale texto):
- texto alternativo para accesibilidad;
- la descripción de una captura para un correo o un ticket;
- el texto de cada paso del Grabador de pasos (SMT-025);
- el resumen de una captura con mucho texto.

**Preguntas para Carlos:**
1. ¿Aceptas cambiar el principio "todo local" para esta función, con las salvaguardas de arriba?
2. ¿Qué vía?
   - **A:** la app llama a la API con una clave.
   - **B:** el servidor MCP, con Claude Desktop o Claude Code.
   - Las dos.
3. **Nombre automático:** a, b o c.
4. **¿Quién paga el uso de la API?** Con la vía A, cada colega necesita su propia clave, o se usa una clave de la empresa. Cada imagen enviada tiene un costo.
5. **¿Qué modelo?**
   - Opus 5.5 es el más capaz.
   - Para sugerir nombres basta uno más pequeño y barato, como Haiku 5.5.
   - *Propuesta:* que el modelo sea configurable.

**Hito propuesto:** después del gate de paridad (M6), como primera función propia. La vía B puede llegar antes, porque casi no toca la app.

---

## IDEA-003 · Reabrir las capturas al volver a abrir XcreenShot (2026-11-01)

> **Aprobada (2026-11-01):** es **EDT-120** en `02-editor.md`. Surgió en la prueba de la vía MCP: al salir, el editor se vaciaba. Claude Code preguntó si las capturas abiertas debían reaparecer, y Carlos contestó "sí me interesa". Los detalles quedaron con la propuesta, marcados como provisionales en `QUESTIONS.md`.

**Lo que propone Carlos:**
- Al volver a abrir XcreenShot, que reaparezcan en el editor las capturas que estaban abiertas.

---

## IDEA-004 · Botón "Añadir a Claude Desktop" (propuesta de Claude Code, 2026-11-01)

> **Sin aprobar.** Pendiente del visto bueno de Carlos.

**Lo que se propone:**
- En Preferencias > IA, un botón que añada XcreenShot a la configuración de Claude Desktop:
  - guarda antes una copia de seguridad;
  - añade solo la entrada `xcreenshot`, sin tocar los demás servidores;
  - comprueba que el archivo sigue siendo válido.
- Sustituiría a **Copiar la configuración**, que copia un bloque completo y rompe el archivo si ya hay otros servidores. Eso le pasó a Carlos en la tanda B de SMT-043.

**Relación con el spec:** SMT-043 y APP-029.

**Riesgo:**
- La app escribiría en un archivo de otra aplicación.
- Por eso, solo cuando el usuario pulse el botón y con una copia de seguridad.

**Hito propuesto:** con las correcciones de SMT-043, en M1.

---

## IDEA-005 · Compartir capturas y videos con otros usuarios por la red (2026-11-13)

> **Sin aprobar.** Carlos la dejó anotada para no olvidarla; no se construye hasta que conteste las preguntas y elija el hito.

**Lo que propone Carlos:**
- Desde la biblioteca, buscar a otro usuario de XcreenShot y enviarle una captura o un video.
- El otro usuario lo recibe cuando está en línea.

**Relación con el spec:** compartir (`SHR-`, que se completa antes de M6), la biblioteca (`LIB-`) y la bandeja (`TRY-`): lo recibido llegaría a la biblioteca de quien lo recibe, como CAP-043 (carpeta de recepción).

**Qué choca o qué riesgos tiene:**
- **Privacidad (CLAUDE.md):** hoy la única red es compartir cuando el usuario lo elige, las actualizaciones y la IA en la nube. Esta idea es compartir, pero "buscar usuarios" y "estar en línea" necesitan algo que diga quién existe y quién está conectado.
- **Fuera de la misma red** hace falta un servidor (o un servicio de terceros) que guarde los envíos hasta que el otro se conecte: alguien tiene que mantenerlo, pagarlo y protegerlo, y por él pasarían las capturas.
- **Seguridad:** las capturas pueden tener datos sensibles; habría que cifrar de extremo a extremo y que nadie pueda recibir sin aceptar.

**Preguntas pendientes, con una propuesta para cada una:**
1. ¿Solo dentro de la misma red (oficina, VPN) o también por internet? *Propuesta:* empezar por la misma red, sin servidor: los equipos se descubren solos (mDNS) y se envía directo de uno a otro.
2. ¿Qué pasa si el otro no está en línea? *Propuesta:* en la misma red, el envío espera en el equipo de quien envía y sale cuando el otro aparece; por internet, haría falta el servidor.
3. ¿Cómo se identifica a un usuario? *Propuesta:* nombre visible y equipo; la primera vez, los dos aceptan (como emparejar un dispositivo).
4. ¿Hay que aceptar cada envío? *Propuesta:* sí, con un aviso; de un contacto de confianza, se puede aceptar siempre.
5. ¿Hito? *Propuesta:* con compartir, en M6 o después.
