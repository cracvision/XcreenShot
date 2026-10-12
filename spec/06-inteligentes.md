# 06 · Funciones inteligentes

**Archivo:** `06-inteligentes.md` · **Prefijo de IDs:** `SMT-` · **Versión:** 0.1 (borrador) · **Fecha:** 2026-10-12

Este archivo especifica las funciones de XcreenShot que analizan el contenido de una imagen o de la pantalla para ahorrar trabajo al usuario: el reconocimiento de texto y sus usos (Extraer texto, Editar texto), la Redacción automática de datos sensibles, la edición inteligente (Separar elementos, Simplificar automático, Quitar fondo, Aplicar solo al texto), el Grabador de pasos con la guía que produce y sus salidas, y los códigos QR. Todas se ejecutan en el propio equipo, funcionan sin conexión y no envían nada a ningún servidor (SPEC §2.3). La captura (`CAP-`), el editor (`EDT-`), el video (`VID-`), la biblioteca, la bandeja reciente y compartir (`LIB-`, `TRY-`, `SHR-`) y la app (`APP-`) se definen en sus archivos; aquí se citan por ID cuando ya existe, o por prefijo y nombre, por ejemplo "EDT- (Desenfocar)". Salvo que una función diga otra cosa, todo vale igual en Windows y macOS (SPEC §2.1).

**Convenciones de este archivo**
- **Teclas.** `Ctrl`/`Cmd` significa `Ctrl` en Windows y `Command` (⌘) en macOS; `Alt`/`Option` igual.
- **(decisión de diseño)** marca un comportamiento que la documentación pública de Snagit no describe, o que XcreenShot define a propósito de otra forma. Lo fija este spec.
- **(sugerencia, no obligatoria)** marca una pista técnica para el equipo de construcción. Puede elegir otra solución si cumple los criterios de aceptación.
- **"Por defecto"** es el valor en una instalación nueva, sin configuración administrada.
- **Equipo de referencia** y forma de medir los tiempos: los de `05-app.md` §11.
- **Texto reconocido:** el resultado del motor de SMT-003 (bloques, líneas y palabras, con su posición y su confianza).
- **Juegos de prueba.** El repositorio incluye imágenes de prueba con su resultado esperado (texto, datos sensibles, rostros, sujetos), que usan los criterios de aceptación (decisión de diseño).

---

## Índice

1. **Garantías comunes y reconocimiento de texto** — SMT-001 Procesamiento local y sin conexión · SMT-002 Activar y desactivar las funciones inteligentes · SMT-003 Motor de reconocimiento de texto · SMT-004 Idiomas del reconocimiento
2. **Extraer y editar texto** — SMT-005 Extraer texto como modo de captura · SMT-006 Ventana de resultados de Extraer texto · SMT-007 Extraer texto en el editor · SMT-008 Extraer texto de un fotograma de video · SMT-009 Editar texto de la imagen
3. **Redacción automática** — SMT-010 Inicio y análisis de la Redacción automática · SMT-011 Categorías de datos sensibles · SMT-012 Palabras y patrones personalizados · SMT-013 Estilo de las redacciones · SMT-014 Añadir y quitar redacciones con un clic · SMT-015 Redactar todas las coincidencias · SMT-016 Redactar varias capturas a la vez · SMT-017 Rendimiento y privacidad de la redacción
4. **Edición inteligente de imágenes** — SMT-018 Separar elementos · SMT-019 Simplificar automático · SMT-020 Quitar fondo · SMT-021 Aplicar solo al texto
5. **Grabador de pasos** — SMT-022 Iniciar el Grabador de pasos · SMT-023 Barra del grabador · SMT-024 Qué crea un paso · SMT-025 Texto automático de cada paso · SMT-026 Imagen de cada paso · SMT-027 Límites, rendimiento, ventanas elevadas y recuperación
6. **Guía de pasos y sus salidas** — SMT-028 Estructura y formato de la guía · SMT-029 Editar la guía · SMT-030 Guía a Word y PowerPoint · SMT-031 Guía a Pages y Keynote (macOS) · SMT-032 Guía a Markdown y HTML · SMT-033 Guía a PDF e impresión · SMT-034 Imágenes de la guía por separado y portapapeles · SMT-035 Guías en la biblioteca
7. **Códigos QR** — SMT-036 Crear un código QR · SMT-037 Código QR de un enlace compartido · SMT-038 Leer códigos QR
8. [Relacionado](#relacionado) · [Excluido](#excluido) · [Cobertura](#cobertura)

---

## 1. Garantías comunes y reconocimiento de texto

### SMT-001 · Procesamiento local y sin conexión
- **Hito:** M3 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** declaración de TechSmith de que sus modelos de IA se ejecutan en el dispositivo, no son generativos, no se usan para entrenar y funcionan sin internet
- **Comportamiento:**
  - Todas las funciones de este archivo analizan las imágenes **en el equipo**. Ninguna envía píxeles, texto reconocido ni resultados a un servidor, ni siquiera de forma anónima.
  - Funcionan igual sin conexión de red. Los modelos y datos que necesitan van incluidos en la app o los aporta el sistema operativo, y nunca se descargan en segundo plano (APP-054). Si una función depende de un componente opcional del sistema (por ejemplo, un idioma de reconocimiento), lo indica y enlaza al ajuste del sistema; XcreenShot no lo descarga por su cuenta.
  - Ninguna función de este archivo es generativa: no inventan contenido; solo detectan, reconocen o reorganizan lo que ya está en la imagen.
  - El texto reconocido y las detecciones (rostros, datos sensibles, elementos) solo están en memoria mientras se usan. No se escriben en los registros (APP-046) ni en los eventos (APP-031), que solo llevan recuentos. Los archivos temporales de procesamiento se borran al terminar.
  - Una operación que tarda más de 500 ms muestra progreso con **Cancelar** y no bloquea el resto de la interfaz. Cancelar deja el documento como estaba (decisión de diseño).
  - Funcionan sin aceleración gráfica. Si el equipo tiene aceleración para estos cálculos, se puede usar (sugerencia, no obligatoria).
  - Cada acción de este archivo es un comando registrado del área `smart` (APP-027). Las fichas nombran los que más interesan a scripts y agentes y fijan sus parámetros. Sus resultados nunca incluyen la imagen salvo que se pida.
- **Criterios de aceptación:**
  - [ ] Con la red desconectada desde el arranque, Extraer texto, Redacción automática, Quitar fondo y el Grabador de pasos dan sobre los juegos de prueba el mismo resultado que con red.
  - [ ] Un monitor de red no registra ninguna conexión de XcreenShot mientras se usan todas las funciones de este archivo durante 30 minutos.
  - [ ] Tras extraer texto y redactar una captura que contiene un correo conocido, ese correo no aparece en los registros (tampoco con el registro detallado) ni en la salida de `xcreenshot events watch`.
- **Fuentes:** research/05-plataformas-versiones.md §2; research/01-captura.md §13; research/02-editor.md §5, §13 (AI / ML)

### SMT-002 · Activar y desactivar las funciones inteligentes
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (en Snagit las funciones de IA principales no se pueden apagar; solo los experimentos, desde el despliegue)
- **Comportamiento:**
  - Preferencias > Editor, grupo **Funciones inteligentes** (propuesta para APP-008): un interruptor por función: Extraer texto, Editar texto, Redacción automática, Separar elementos, Simplificar automático, Quitar fondo, Aplicar solo al texto, Grabador de pasos y Códigos QR. Todos activados por defecto. La lectura de QR al capturar tiene su propio ajuste (SMT-038).
  - Cada interruptor aparece cuando su función existe (regla de APP-008); el grupo completo llega en M5, aunque el interruptor de Extraer texto se puede adelantar a M3. Clave de ajuste: `smart.<función>.enabled` (por ejemplo, `smart.autoRedact.enabled`).
  - Al desactivar una función desaparecen sus botones, entradas de menú, modos de captura y atajos. Sus comandos fallan con el código `feature_disabled`, cuyo mensaje nombra el ajuste (propuesta de código para APP-027).
  - La configuración administrada puede fijar cualquiera con `locked` (APP-035); el interruptor aparece entonces con candado.
  - Los documentos que ya contienen resultados de una función desactivada (redacciones, objetos de simplificación, objetos QR) se siguen abriendo y editando como objetos normales.
  - Ninguna función de este archivo sale como Función en prueba (APP-034): todas son definitivas al llegar su hito. El equipo de construcción solo puede publicar una como prueba de forma temporal, mientras no cumpla sus criterios de aceptación, y lo anota en `PARITY.md` (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con Redacción automática desactivada, el botón no aparece en las propiedades de Desenfocar y `xcreenshot smart redact` falla con `feature_disabled`.
  - [ ] Con `"locked": {"smart.removeBackground.enabled": false}` en `managed.json`, Quitar fondo no aparece en ningún menú y su interruptor muestra el candado.
  - [ ] Un documento con redacciones automáticas se abre y se edita igual después de desactivar la función.
- **Mejora sobre Snagit:** cada función inteligente se puede apagar por separado, también por parte del administrador.
- **Fuentes:** research/01-captura.md §13 (Policy); research/04-biblioteca-compartir.md §7; research/05-plataformas-versiones.md §5

### SMT-003 · Motor de reconocimiento de texto
- **Hito:** M3 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** motor de OCR que usan Grab Text, Edit Text y Smart Redact, con las mejoras de 2026 (imágenes altas, colores, espaciado de párrafos, experimento "Optimized OCR")
- **Comportamiento:**
  - Un único motor sirve a todas las funciones que leen texto: Extraer texto (SMT-005 a SMT-008), Editar texto (SMT-009), Redacción automática (SMT-010), Separar elementos (SMT-018), Simplificar automático (SMT-019), Aplicar solo al texto (SMT-021) y el texto de reserva de los pasos (SMT-025). Así todas ven el mismo texto.
  - **Resultado** para una imagen o una zona: bloques (párrafos), líneas y palabras, cada uno con su rectángulo en píxeles de la imagen, una confianza de 0 a 1 y el idioma detectado. Por línea, además: color del texto, color del fondo, altura de letra y familia genérica (sin serifa, con serifa o monoespaciada), todos estimados.
  - **Orden de lectura:** de arriba abajo y de izquierda a derecha. Si hay columnas separadas, se lee una columna entera antes de la siguiente (decisión de diseño). Dos párrafos se separan cuando el espacio vertical entre ellos es mayor que el interlineado.
  - **Casos que debe resolver:**
    - texto claro sobre fondo oscuro (modo oscuro) y texto de colores;
    - letra pequeña de interfaz, desde unos 9 px de altura de mayúscula a una escala del 100 % (decisión de diseño);
    - capturas de pantallas de alta densidad;
    - imágenes muy altas o anchas (capturas con desplazamiento), que se procesan por tramos solapados sin perder ni repetir líneas en las uniones.
  - **Límites conocidos**, que la ayuda explica con texto propio: puede no reconocer texto girado más de unos 10°, escrito a mano, muy decorativo, con contorno o sombra marcada, o con muy poco contraste.
  - **Confianza** (decisión de diseño): las palabras por debajo de 0,5 se descartan; entre 0,5 y 0,8 se entregan marcadas como dudosas. Cada función decide qué hacer con ellas: Extraer texto las subraya (SMT-006) y la redacción automática las incluye, porque es mejor redactar de más.
  - **Rendimiento** (equipo de referencia, percentil 95, a partir del segundo uso en la sesión):
    - captura de 1920×1080 con texto de interfaz: < 1 s;
    - captura de 3840×2160: < 2 s;
    - imagen de 10.000 × 30.000 px: < 30 s, con progreso y Cancelar (SMT-001).
    - El primer uso en la sesión puede tardar hasta 3 s más. XcreenShot puede preparar el motor en segundo plano poco después de arrancar (sugerencia, no obligatoria).
  - Usar el reconocimiento de texto que ofrece el sistema operativo cuando cumpla estos requisitos, y un motor incluido en la app cuando no (por ejemplo, si al sistema le falta un idioma obligatorio) (sugerencia, no obligatoria). El usuario no elige motor.
- **Criterios de aceptación:**
  - [ ] Sobre el juego de prueba de texto (capturas de interfaces, documentos y páginas web en los cinco idiomas obligatorios, en modo claro y oscuro, a escala 100 % y 200 %), los caracteres correctos son ≥ 97 % en cada plataforma, y la diferencia entre Windows y macOS es ≤ 2 puntos.
  - [ ] En una captura con desplazamiento de 20.000 px de alto con 400 líneas numeradas, el resultado contiene las 400 líneas, en orden y sin repeticiones.
  - [ ] Una captura de 1920×1080 con una página de texto se reconoce en < 1 s (percentil 95, desde el segundo uso).
  - [ ] En una página a dos columnas, todo el texto de la columna izquierda sale antes que el de la derecha.
- **Mejora sobre Snagit:** el mismo resultado en Windows y macOS, y requisitos explícitos para imágenes altas y modo oscuro.
- **Fuentes:** research/01-captura.md §7 (OCR engine and languages, Recognition tips, Version notes); research/02-editor.md §5.2; research/05-plataformas-versiones.md §2, §5, §1A (2026.4.0), §1B (2026.1.0); research/04-biblioteca-compartir.md §7

### SMT-004 · Idiomas del reconocimiento
- **Hito:** M3 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** idiomas de OCR de Grab Text (inglés, francés, alemán, español y portugués; el japonés no está confirmado)
- **Comportamiento:**
  - **Idiomas obligatorios**, disponibles sin conexión en una instalación nueva de ambas plataformas: inglés, español, portugués, francés y alemán, con todos sus signos (ñ, á, ç, ã, õ, ü, ß, œ, ¿, ¡, «»).
  - **Idiomas adicionales** cuando el equipo los admita: japonés y chino simplificado y tradicional (decisión de diseño). Se pueden ofrecer otros idiomas que el sistema tenga disponibles.
  - Ajuste **Idioma del reconocimiento** (Preferencias > Captura, APP-008; clave `smart.ocr.languages`):
    - **Automático** (por defecto): usa el idioma de la interfaz y los idiomas preferidos del sistema, y para cada bloque se queda con el que da mejor resultado (decisión de diseño);
    - o una lista de hasta 3 idiomas elegidos por el usuario.
  - Un idioma adicional que el equipo no tiene aparece como "No disponible en este equipo", con un enlace al ajuste del sistema donde se añade.
  - El idioma de reconocimiento no depende del de la interfaz. La ventana de resultados permite repetir con otro idioma (SMT-006) y los comandos aceptan `language`.
- **Criterios de aceptación:**
  - [ ] En un equipo recién instalado y sin conexión, con la interfaz en inglés y el idioma en Automático, un texto en portugués se reconoce con sus signos (ã, ç, õ) correctos.
  - [ ] En Windows y macOS recién instalados, los cinco idiomas obligatorios aparecen como disponibles sin instalar nada más.
  - [ ] Si el japonés no está disponible, aparece como "No disponible en este equipo" con el enlace, y tenerlo en la lista no impide reconocer los demás idiomas.
- **Mejora sobre Snagit:** reconocimiento multilingüe automático, lista explícita de idiomas y soporte de japonés y chino cuando el equipo lo permite.
- **Fuentes:** research/01-captura.md §7, §15 (2021.0.1), §16 (punto 8); research/02-editor.md §5.2, §15 (punto 5)

---

## 2. Extraer y editar texto

### SMT-005 · Extraer texto como modo de captura
- **Hito:** M3 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Grab Text como tipo de Selección; Grab Text sin abrir el Editor; aviso de éxito o de fallo
- **Comportamiento:**
  - Se elige con Panel de captura > Imagen > Selección = **Extraer texto** (CAP-002). También hay una acción **Extraer texto** con atajo global propio (por defecto `Ctrl+Alt+Shift+T` / `⌃⌥⇧T`, CAP-065), el preset incorporado **Extraer texto** (CAP-064), una entrada en el menú del ícono (CAP-005) y el comando `smart.extract_text` con `source: "screen"`.
  - Opciones compatibles (completa la tabla de CAP-002): Retardo, sí; Copiar al portapapeles, sí (copia el **texto**); Capturar cursor, Intervalo, Programada, Efectos, Compartir y Abrir en, no. Las que no se aplican aparecen desactivadas, con un tooltip que explica el motivo.
  - **Flujo:**
    1. Aparece la selección normal (CAP-013, con la detección de CAP-016). El usuario arrastra sobre el texto o hace clic en una ventana o subárea detectada. `Esc` cancela.
    2. Al soltar, XcreenShot reconoce el texto (SMT-003). Si tarda más de 300 ms, un indicador junto a la zona muestra que está trabajando.
    3. No se abre el editor.
  - Ajuste **Mostrar la ventana de resultados** (Preferencias > Captura, junto a Idioma del reconocimiento, APP-008; por defecto: sí; clave `smart.extractText.showResults`):
    - **Sí:** se abre la ventana de resultados (SMT-006) cerca de la zona. Si Copiar al portapapeles está activado, el texto ya está copiado cuando aparece.
    - **No:** el texto se copia directamente y una notificación (categoría Capturas, APP-016) dice "Texto copiado (N palabras)", con la acción **Ver**, que abre la ventana de resultados.
  - Si no se reconoce texto, en ambos casos aparece el aviso "No se encontró texto en la zona seleccionada", con **Ver consejos** (SMT-006) y **Repetir**. El portapapeles no cambia.
  - Ajuste **Guardar también la imagen en la biblioteca** (Preferencias > Captura; por defecto: no; clave `smart.extractText.saveImage`) (decisión de diseño). Sin él, la imagen de la zona solo existe mientras la ventana de resultados está abierta.
  - Funciona sobre cualquier contenido de la pantalla, también sobre un video en reproducción, porque se usa la pantalla congelada.
- **Criterios de aceptación:**
  - [ ] Con Selección = Extraer texto, arrastrar sobre un párrafo de un navegador abre la ventana de resultados con ese párrafo, sin abrir el editor ni crear ninguna entrada en la biblioteca.
  - [ ] Con Mostrar la ventana de resultados desactivado, tras soltar, el texto se puede pegar en otra app y aparece la notificación con el número de palabras.
  - [ ] Seleccionar una zona sin texto (la foto de un paisaje) muestra "No se encontró texto…" y el portapapeles conserva lo que tenía.
  - [ ] Con Selección = Extraer texto, Capturar cursor aparece desactivado con su tooltip.
- **Mejora sobre Snagit:** copia directa con aviso en ambas plataformas, atajo propio y la opción de no guardar la imagen.
- **Fuentes:** research/01-captura.md §7, §4.1, §11.3

### SMT-006 · Ventana de resultados de Extraer texto
- **Hito:** M3 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** diálogo Grab Text Results (Copy All, Plain text / Make Plain Text, edición, corrector ortográfico, Esc cierra, espaciado de párrafos, sin botón de minimizar)
- **Comportamiento:**
  - Ventana **Texto extraído** con:
    - un área de texto editable con el texto reconocido;
    - **Copiar todo** (botón y `Ctrl/Cmd+Shift+C`), que copia el texto con formato y sin formato a la vez, para que cada app pegue el que admita;
    - casilla **Sin formato** (se recuerda; clave `smart.extractText.plain`): el área muestra y copia solo el texto, sin fuentes ni colores;
    - **Copiar y cerrar** (`Ctrl/Cmd+Enter`);
    - menú **Más**: **Guardar como texto…** (archivo `.txt` en UTF-8), **Reconocer en otro idioma** (SMT-004) y **Unir las líneas de cada párrafo** (por defecto: no) (decisión de diseño).
  - **Formato:** cada línea conserva su familia genérica, su tamaño relativo, su negrita y su color aproximados (SMT-003). Los párrafos se separan con una línea en blanco. Los saltos de línea dentro de un párrafo se conservan, salvo que se unan.
  - Las palabras dudosas (SMT-003) aparecen con subrayado de puntos. Al pasar el puntero, una miniatura muestra la zona de la imagen de la que salen (decisión de diseño).
  - El corrector ortográfico del editor (EDT-027; ajuste e idioma en APP-008) marca las palabras desconocidas y ofrece sugerencias con clic derecho.
  - **Sin texto:** la ventana muestra el aviso y consejos con texto propio: subir el contraste o el zoom antes de capturar, enderezar el texto, dejar fuera marcos y bordes, evitar letras decorativas y probar con otro idioma.
  - `Esc` cierra la ventana sin preguntar. No tiene botón de minimizar. Se puede redimensionar y recuerda su tamaño y posición. Se usa por completo con el teclado y con lector de pantalla (APP-043, APP-044).
  - Puede haber varias ventanas de resultados abiertas a la vez (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con Sin formato desactivado, al pegar en un procesador de textos una palabra roja de la captura sigue en rojo; con Sin formato activado, se pega texto sin formato.
  - [ ] Una imagen con tres párrafos da tres párrafos separados por una línea en blanco.
  - [ ] `Esc` cierra la ventana; `Ctrl/Cmd+Enter` copia y cierra.
  - [ ] Una palabra mal escrita en la imagen aparece marcada por el corrector, y elegir una sugerencia la sustituye en el área de texto.
- **Mejora sobre Snagit:** guardar como `.txt`, repetir con otro idioma, unir líneas y marca de palabras dudosas.
- **Fuentes:** research/01-captura.md §7; research/02-editor.md §5.2, §11 (spell check); research/05-plataformas-versiones.md §1A (2024.3.5, 2024.3.6, 2026.4.0), §3 (Editor: Plain-text copy from Grab Text)

### SMT-007 · Extraer texto en el editor
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Grab Text desde el menú contextual de la imagen o de una selección; herramienta Grab Text de la lista More
- **Comportamiento:**
  - Puntos de entrada:
    - clic derecho en el lienzo sin selección > **Extraer texto de la imagen**;
    - con una selección de la herramienta Selección (EDT-043) > **Extraer texto de la selección**;
    - clic derecho en una imagen flotante > **Extraer texto**, solo de ese objeto;
    - menú Imagen, la herramienta **Extraer texto** del menú Más (EDT-003) y el atajo del editor `Ctrl+Shift+O` / `Shift+Cmd+O` (decisión de diseño; la hoja de atajos del editor lo incluye).
  - Se reconoce lo que se ve en el lienzo en esa zona: la imagen base con sus objetos y efectos. Por ejemplo, el texto de una anotación también sale (decisión de diseño). Las zonas tapadas por una redacción nunca devuelven texto.
  - El resultado se muestra en la ventana de SMT-006. El documento no cambia.
  - Comando `smart.extract_text`:
    - `source`: documento abierto, elemento de la biblioteca o ruta de archivo; `rect` opcional; `language` opcional;
    - `format`: `plain`, `formatted` (HTML) o `json` (bloques, líneas y palabras con rectángulos y confianza);
    - no abre ninguna ventana.
- **Criterios de aceptación:**
  - [ ] En el editor, Extraer texto de la selección sobre una parte de una tabla devuelve solo el texto de esa parte.
  - [ ] El texto que está debajo de una redacción sólida no aparece en el resultado.
  - [ ] `xcreenshot smart extract-text --file captura.png --format json` devuelve las líneas con sus rectángulos y su confianza, sin abrir ninguna ventana.
- **Mejora sobre Snagit:** salida JSON con posiciones para scripts y agentes; el resultado respeta las redacciones.
- **Fuentes:** research/02-editor.md §5.2, §4.2, §2.1 (More), §12; research/01-captura.md §7 (Related Editor features)

### SMT-008 · Extraer texto de un fotograma de video
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Grab Text del fotograma de un video (solo Mac en Snagit)
- **Comportamiento:**
  - En el reproductor y editor de video (VID- (reproductor)), botón y entrada de menú **Extraer texto del fotograma**, con el mismo atajo que en el editor de imágenes (SMT-007).
  - Funciona en pausa y durante la reproducción; si el video se está reproduciendo, se pausa en ese fotograma.
  - Se reconoce el fotograma bajo el cabezal a la resolución completa del video, no a la de la vista previa. Arrastrando un rectángulo sobre el video se limita la zona (decisión de diseño).
  - El resultado se abre en la ventana de SMT-006; con formato, conserva el color del texto.
- **Criterios de aceptación:**
  - [ ] En un video de una presentación pausado en una diapositiva con título y viñetas, la acción devuelve el título y las viñetas en orden.
  - [ ] Si el video se está reproduciendo, la acción lo pausa y el texto corresponde al fotograma que se ve.
  - [ ] En un video 1080p mostrado a la mitad de tamaño, el texto pequeño se reconoce igual que en el mismo fotograma guardado como imagen.
- **Mejora sobre Snagit:** también en Windows.
- **Fuentes:** research/03-video.md (Grab Text from a video frame); research/02-editor.md §5.2; research/05-plataformas-versiones.md §2, §3 (Editor)

### SMT-009 · Editar texto de la imagen
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Edit Text (sucesor de Text Replace)
- **Comportamiento:**
  - Con una selección de la herramienta Selección sobre texto que forma parte de los píxeles > clic derecho > **Editar texto**. También desde el menú contextual de la Redacción automática (SMT-014) y desde Separar elementos (SMT-018).
  - XcreenShot reconoce las líneas de la selección y:
    1. borra sus píxeles y rellena el hueco con el fondo de alrededor (Relleno inteligente, EDT-045);
    2. crea en el mismo sitio un objeto Texto (EDT-026) por párrafo, con la familia genérica, el tamaño, la alineación y el **color original** del texto, sobre fondo transparente;
    3. deja el cursor al final del texto, listo para escribir.
  - El objeto es un Texto normal: se puede reescribir, añadir o borrar texto y cambiar fuente, tamaño, color y estilo.
  - Si la selección corta letras o líneas, XcreenShot avisa y ofrece **Ajustar la selección al texto**, que la amplía hasta abarcar las líneas completas (decisión de diseño).
  - Si la selección incluye otros elementos (íconos, bordes), solo se sustituye el texto; los demás píxeles no cambian.
  - Sobre degradados o fotos, el relleno puede dejar marcas; el aviso lo explica y sugiere una selección más ajustada.
  - Toda la operación se deshace en un paso. Sin texto en la selección: "No se encontró texto para editar".
- **Criterios de aceptación:**
  - [ ] En la captura de un botón azul con el texto blanco "Enviar", Editar texto y escribir "Guardar" deja el botón azul con "Guardar" en blanco, sin restos del texto anterior.
  - [ ] El color del texto creado coincide con el original (diferencia de color ΔE ≤ 5).
  - [ ] Un Deshacer devuelve la imagen anterior idéntica píxel a píxel.
  - [ ] Una selección que corta una palabra por la mitad muestra el aviso con **Ajustar la selección al texto**.
- **Mejora sobre Snagit:** ajuste automático de la selección al texto.
- **Fuentes:** research/02-editor.md §5.2, §4.2; research/05-plataformas-versiones.md §2

---

## 3. Redacción automática

### SMT-010 · Inicio y análisis de la Redacción automática
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Smart Redact (botón en las acciones de la herramienta Blur; panel con categorías; botón Done)
- **Comportamiento:**
  - Se abre con el botón **Redacción automática** de las propiedades de la herramienta Desenfocar (EDT-040), con la entrada del menú Imagen y con un atajo asignable (sin atajo por defecto).
  - Al abrirla, XcreenShot analiza **todo el lienzo** (imagen base e imágenes flotantes) y muestra el **panel de Redacción automática** con:
    - la lista de categorías (SMT-011) y la de palabras y patrones propios (SMT-012), cada una con su casilla y el **número de coincidencias** encontradas;
    - el estilo de las redacciones (SMT-013);
    - **Siguiente** / **Anterior**, que centran la vista en cada redacción para revisarla (decisión de diseño);
    - **Buscar en otras capturas…** (SMT-016);
    - **Listo**.
  - Cada coincidencia de una categoría activada se cubre al momento con una redacción. Un indicador de progreso aparece si el análisis dura más de 500 ms (SMT-001).
  - Cada redacción es un **objeto Desenfocar normal** (EDT-040): se puede seleccionar, mover, cambiar de tamaño, borrar y cambiar de tipo, color, forma e intensidad. Además guarda su categoría y el texto o elemento que cubre, para que el panel lo pueda gestionar mientras está abierto.
  - **Listo** cierra el panel y deja las redacciones como objetos. Volver a abrir el panel recupera la gestión de las redacciones automáticas que sigan en el documento, sin duplicarlas.
  - El análisis entero se deshace en un paso. Cada acción posterior del panel es otro paso de deshacer.
  - Si no se encuentra nada, el panel lo dice ("No se encontraron datos sensibles") y ofrece añadir redacciones a mano con un clic (SMT-014).
  - **Recordatorio de seguridad** al pulsar Listo, si hay redacciones de tipo Desenfocar o Pixelar (se puede ocultar): para datos confidenciales es más seguro el relleno sólido, porque hay herramientas que pueden revertir el desenfoque.
  - Comando `smart.redact`: documento o archivo de entrada, categorías, palabras y patrones, estilo; resultado: número de redacciones por categoría y ruta de salida. Con `flatten: true` guarda el resultado ya aplanado (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] En la captura de prueba de un formulario con un correo, un teléfono y una tarjeta, abrir la Redacción automática cubre los tres y el panel muestra 1 coincidencia en cada una de esas categorías.
  - [ ] Después de **Listo**, una redacción se puede seleccionar con la herramienta Mover y cambiar a Pixelar en sus propiedades.
  - [ ] Volver a abrir el panel sobre el mismo documento no crea redacciones duplicadas.
  - [ ] Un solo Deshacer tras el análisis quita todas las redacciones que creó.
- **Fuentes:** research/02-editor.md §5.3, §3.12; research/01-captura.md §13; research/05-plataformas-versiones.md §2, §1A (2025.0.0)

### SMT-011 · Categorías de datos sensibles
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** las nueve categorías de Smart Redact, con mejoras de detección (perfiles laterales, direcciones, sin categorías duplicadas)
- **Comportamiento:**
  - Categorías, en este orden, con su casilla:

    | Categoría | Qué detecta (decisión de diseño en los formatos) | Por defecto |
    |---|---|---|
    | Correos electrónicos | Direcciones de correo | Activada |
    | Números de teléfono | Formatos nacionales e internacionales, con o sin prefijo, espacios, guiones o paréntesis | Activada |
    | Números de tarjeta | De 13 a 19 dígitos, agrupados o seguidos, que superan la verificación de dígito de control de las tarjetas | Activada |
    | Números de identificación | Números de seguro social de EE. UU. y documentos de identidad nacionales con formato fijo de los países de los idiomas obligatorios (por ejemplo, DNI y NIE de España, CPF de Brasil, NIF de Portugal); la lista exacta es la del juego de prueba | Activada |
    | Direcciones postales | Calle y número, código postal y ciudad, en varias líneas si hace falta | Activada |
    | Direcciones IP | IPv4 e IPv6 | Activada |
    | URLs | Direcciones web, con o sin `http(s)://` | Desactivada |
    | Fechas | Fechas numéricas y escritas, en los cinco idiomas obligatorios | Desactivada |
    | Rostros | Caras humanas de frente, de perfil y de tres cuartos, en fotos o en videollamadas capturadas; un recuadro por cara | Activada |

  - Las URLs y las fechas empiezan desactivadas porque abundan en las capturas de interfaces y casi nunca son sensibles (decisión de diseño). XcreenShot recuerda las casillas de la última vez (clave `smart.autoRedact.categories`).
  - Desactivar una categoría quita sus redacciones (las que siguen gestionadas por el panel); activarla las vuelve a crear.
  - Un mismo texto nunca recibe dos redacciones. Si encaja en dos categorías, cuenta para la más específica: tarjeta antes que teléfono, IP antes que fecha (decisión de diseño).
  - La redacción cubre solo el dato, no toda la línea. En los datos con etiqueta ("Correo: ana@…"), la etiqueta queda visible.
  - La detección de texto usa el motor de SMT-003, sobre el texto de la imagen base y de las imágenes flotantes. Los rostros se detectan con un modelo local de detección de caras (SMT-001). No se reconoce a las personas ni se compara una cara con otra.
- **Criterios de aceptación:**
  - [ ] En el juego de prueba de datos sensibles, cada categoría detecta al menos el 95 % de los casos marcados y tiene como mucho un 5 % de falsos positivos sobre las capturas de control.
  - [ ] Un número de 16 dígitos que no supera la verificación de las tarjetas no se marca como tarjeta.
  - [ ] Una foto de grupo con 5 personas, dos de ellas de perfil, produce 5 redacciones de Rostros.
  - [ ] Al desactivar Números de teléfono desaparecen las redacciones de teléfonos y las demás siguen.
- **Mejora sobre Snagit:** documentos de identidad de los países de los idiomas obligatorios, además del número de seguro social de EE. UU.
- **Fuentes:** research/02-editor.md §5.3, §13 (2025); research/01-captura.md §13; research/05-plataformas-versiones.md §2, §1A (2025.1.0), §1B (2024.2.3, 2025.3.1); research/04-biblioteca-compartir.md §7

### SMT-012 · Palabras y patrones personalizados
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** — (mejora; Snagit solo ofrece categorías fijas)
- **Comportamiento:**
  - Sección **Mis términos** del panel de Redacción automática (decisión de diseño):
    - **Palabras o frases** que siempre se redactan (nombre de un cliente, de un proyecto, de un servidor). Sin distinguir mayúsculas, tildes ni espacios repetidos, salvo que se marque **Coincidencia exacta**.
    - **Patrones** con expresiones regulares (por ejemplo, `PO-\d{6}` para números de pedido), con un nombre y una prueba en vivo que muestra qué encuentra en la imagen abierta.
  - Cada término o patrón tiene su casilla y cuenta como una categoría más, con su número de coincidencias.
  - Las listas se guardan en las preferencias (clave `smart.autoRedact.customTerms`), viajan con Exportar ajustes (APP-011) y se pueden fijar con la configuración administrada (APP-035), para que toda una organización redacte los mismos términos.
  - Un patrón no válido se marca en rojo y no se aplica. Un patrón que tarda demasiado (más de 200 ms por imagen) se detiene y se avisa (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Con el término "Proyecto Orión", la captura que contiene "proyecto orion" lo redacta si Coincidencia exacta está desactivada, y no lo redacta si está activada.
  - [ ] El patrón `PO-\d{6}` redacta "PO-123456" y no "PO-12345".
  - [ ] Un término fijado en `managed.json` aparece en la lista con candado y no se puede borrar.
- **Mejora sobre Snagit:** términos y patrones propios, compartibles en toda la organización.
- **Fuentes:** research/02-editor.md §5.3 (base de categorías que se amplía)

### SMT-013 · Estilo de las redacciones
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** tipo de Smart Redact (Solid shapes, Blur, Pixelate), Shape Color, esquinas redondeadas
- **Comportamiento:**
  - Controles del panel:
    - **Tipo:** **Relleno sólido** (por defecto), **Desenfocar** o **Pixelar**;
    - **Color** del relleno sólido (por defecto: negro), con el selector de color normal (EDT-019);
    - **Esquinas redondeadas** (por defecto: sí, radio de 4 px) (decisión de diseño en el radio);
    - **Margen** alrededor del dato: de 0 a 10 px; por defecto 2 px (decisión de diseño).
  - Cambiar el estilo en el panel lo aplica a todas las redacciones que gestiona el panel. Lo que se cambie después en un objeto concreto con las propiedades de Desenfocar solo afecta a ese objeto.
  - Los rostros usan la forma de elipse (decisión de diseño); el resto, rectángulo.
  - El estilo elegido se recuerda para la próxima vez (clave `smart.autoRedact.style`).
- **Criterios de aceptación:**
  - [ ] Cambiar el Tipo a Pixelar con 10 redacciones convierte las 10 en una sola acción, que se deshace en un paso.
  - [ ] Con Color = rojo, todas las redacciones sólidas se ven rojas; al cambiar luego el color de una sola en sus propiedades, las demás siguen rojas.
  - [ ] Una redacción sólida exportada a PNG no deja ver ningún píxel del dato original (comprobado ampliando al 800 %).
- **Fuentes:** research/02-editor.md §5.3, §3.12; research/05-plataformas-versiones.md §1A (2025.1.0), §1B (2025.2.0)

### SMT-014 · Añadir y quitar redacciones con un clic
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Smart Redact: clic en el texto para añadir o quitar una redacción; clic derecho > Remove, Edit Text (solo Mac en Snagit)
- **Comportamiento:**
  - Con el panel abierto, al pasar el puntero sobre el lienzo se resaltan las palabras reconocidas.
    - **Clic** en una palabra sin redactar la redacta.
    - **Clic** en una redacción la quita.
    - `Shift`+clic o arrastrar sobre varias palabras redacta todas las palabras recorridas como una sola redacción por línea.
    - **Doble clic** redacta la línea entera (decisión de diseño).
  - Las redacciones hechas a mano se agrupan en la categoría **Manual** del panel.
  - Menú contextual de una redacción: **Quitar**, **Redactar todas las coincidencias** (SMT-015), **Editar texto** (SMT-009, que actúa sobre el texto que está debajo y deja de estar redactado) y **No volver a marcar en este documento** (decisión de diseño).
  - Fuera del panel, las redacciones son objetos normales y se borran con `Supr`.
- **Criterios de aceptación:**
  - [ ] Con el panel abierto, un clic en una palabra que no se detectó la cubre con el estilo actual y aparece en la categoría Manual.
  - [ ] Un clic en una redacción automática la quita y su categoría baja en uno.
  - [ ] Arrastrar sobre tres palabras de una misma línea crea una sola redacción que cubre las tres.
- **Mejora sobre Snagit:** también en Windows; redacción por línea con doble clic.
- **Fuentes:** research/02-editor.md §5.3, §13 (2026); research/05-plataformas-versiones.md §1B (2026.0.0), §3 (Editor)

### SMT-015 · Redactar todas las coincidencias
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Redact All Matches (solo Mac en Snagit)
- **Comportamiento:**
  - Clic derecho en una redacción o en una palabra > **Redactar todas las coincidencias**: redacta todas las apariciones del mismo texto en el documento, con las mismas reglas de comparación que los términos propios (SMT-012).
  - El panel muestra cuántas se añadieron ("Se redactaron 7 coincidencias"), con **Deshacer** en el mismo mensaje.
  - Opción del mensaje: **Añadir a Mis términos**, para que se redacte también en próximas capturas (decisión de diseño).
  - Su opuesto, **Quitar todas las coincidencias**, aparece en el menú de una redacción.
- **Criterios de aceptación:**
  - [ ] En una captura donde el nombre "Marta Ruiz" aparece 4 veces, redactar una y elegir Redactar todas las coincidencias deja las 4 cubiertas.
  - [ ] Quitar todas las coincidencias sobre una de ellas descubre las 4.
- **Mejora sobre Snagit:** también en Windows; puede convertirse en un término permanente.
- **Fuentes:** research/02-editor.md §5.3, §13 (2026); research/05-plataformas-versiones.md §1B (2026.0.0), §3 (Editor)

### SMT-016 · Redactar varias capturas a la vez
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** buscar contenido sensible parecido en varias capturas (solo Mac en Snagit)
- **Comportamiento:**
  - Entradas:
    - con varias capturas seleccionadas en la bandeja reciente (`TRY-`) o en la biblioteca (`LIB-`) > clic derecho > **Redacción automática…**;
    - desde el panel de un documento > **Buscar en otras capturas…**, que ofrece elegir capturas de la bandeja o de la biblioteca y aplicar las mismas categorías, términos, palabras redactadas a mano y estilo.
  - Una ventana de revisión muestra cada captura con sus redacciones propuestas y el número por categoría. Por cada captura: **Aceptar**, **Revisar en el editor** u **Omitir**. **Aceptar todas** aplica todas a la vez.
  - "Parecido" significa (decisión de diseño):
    - el mismo texto que se redactó a mano en el documento de origen;
    - las mismas categorías y términos;
    - y, para rostros, la misma configuración, sin comparar identidades (SMT-011).
  - Las capturas modificadas guardan una versión nueva. Hasta cerrar la ventana de revisión se puede **Deshacer todo**. Los videos no se incluyen.
  - Se procesan en segundo plano, con progreso por captura y Cancelar; cancelar conserva las ya aceptadas.
- **Criterios de aceptación:**
  - [ ] Con 10 capturas seleccionadas que muestran el mismo correo, la ventana de revisión propone redactarlo en las 10 y Aceptar todas lo redacta en todas.
  - [ ] Un nombre redactado a mano en la captura de origen se propone también en las otras capturas donde aparece.
  - [ ] Cancelar a mitad deja intactas las capturas que no se aceptaron.
- **Mejora sobre Snagit:** también en Windows, desde la bandeja o la biblioteca, con revisión por captura.
- **Fuentes:** research/02-editor.md §5.3 (Mac 2026.0.0)

### SMT-017 · Rendimiento y privacidad de la redacción
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** procesamiento local de Smart Redact; mejoras de rendimiento con muchas redacciones; precisión mejorada por el OCR optimizado
- **Comportamiento:**
  - **Rendimiento** (equipo de referencia):
    - análisis de una captura de 1920×1080: < 1,5 s; de 3840×2160: < 3 s;
    - con 500 redacciones en el documento, cambiar el tipo o el color de todas, mover una o hacer zoom responde en < 100 ms;
    - activar o desactivar una categoría con 200 coincidencias: < 200 ms.
  - **Privacidad:** se aplican las garantías de SMT-001. Además:
    - las redacciones nunca se rellenan con datos derivados del contenido (el relleno sólido es de un solo color);
    - al exportar a PNG, JPG, GIF, PDF o al compartir, todo se aplana (EDT-040, EDT-110), así que el dato no queda en el archivo;
    - el `.xshot` sí conserva los píxeles originales bajo las redacciones, porque es editable (SPEC §7). Como las redacciones son objetos Desenfocar, al compartir o exportar un `.xshot` con redacciones sin aplanar aparece el aviso de información oculta recuperable de EDT-040, y aplanarlas es irreversible (EDT-091).
  - El texto y los rostros detectados no se guardan en el documento ni en la biblioteca. Las redacciones solo guardan su categoría y su rectángulo.
- **Criterios de aceptación:**
  - [ ] Un documento con 500 redacciones mantiene ≥ 50 fps al desplazarse y hacer zoom.
  - [ ] Un PNG exportado de un documento con redacciones sólidas no contiene el texto redactado: Extraer texto sobre el PNG no lo devuelve.
  - [ ] Al enviar un `.xshot` con redacciones sin aplanar a un destino de archivo, aparece el aviso de EDT-040.
- **Mejora sobre Snagit:** requisitos de rendimiento explícitos con cientos de redacciones, en ambas plataformas.
- **Fuentes:** research/02-editor.md §5.3, §3.12 (security notes); research/05-plataformas-versiones.md §2, §1B (2026.3.0), §5; research/01-captura.md §13

---

## 4. Edición inteligente de imágenes

### SMT-018 · Separar elementos
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Smart Move (interruptor en Windows; botón y Done en Mac; control Detail; volver a escribir el texto; también en imágenes flotantes)
- **Comportamiento:**
  - Botón **Separar elementos** en las propiedades de la herramienta Mover (EDT-042) y en el menú contextual de una imagen flotante. Actúa sobre la imagen base o, si hay una imagen flotante seleccionada, solo sobre ella.
  - XcreenShot detecta en los píxeles los elementos de interfaz (botones, íconos, campos, casillas, imágenes) y los bloques de texto (SMT-003), y convierte cada uno en un objeto. Los objetos se marcan con un contorno discontinuo al pasar el puntero.
  - Cada objeto se puede:
    - seleccionar (también varios, con `Shift` o con un recuadro de selección, EDT-079), mover, cambiar de tamaño y borrar;
    - en el caso de texto, volver a escribir con doble clic: se convierte en un objeto Texto con el estilo detectado, como en SMT-009.
  - El hueco que deja un objeto al moverlo o borrarlo se rellena con el fondo de alrededor (Relleno inteligente, EDT-045).
  - Control **Detalle** (de 1 a 10; por defecto 5): con menos detalle, los elementos cercanos se agrupan (una fila de botones como un solo objeto); con más, se separan (cada botón y cada palabra). Cambiar el detalle vuelve a calcular los objetos. Si ya se movió alguno, primero pide confirmación (decisión de diseño).
  - **Restablecer** devuelve la imagen al estado de antes de separar.
  - **Listo** (o cambiar de herramienta) aplana los objetos en la imagen y sale del modo. Toda la sesión se deshace en un paso.
  - Si no detecta nada útil (por ejemplo, en una foto), lo dice y no cambia nada.
- **Criterios de aceptación:**
  - [ ] En la captura de un cuadro de diálogo, Separar elementos permite arrastrar el botón **Aceptar** a otro sitio y su hueco original queda relleno con el fondo del diálogo.
  - [ ] Con Detalle = 1, una barra de tres botones es un solo objeto; con Detalle = 10, son tres.
  - [ ] Doble clic en la etiqueta "Nombre" de un campo permite escribir "Apellidos", con un tamaño y un color iguales al original.
  - [ ] Sobre una imagen flotante, Separar elementos no cambia la imagen base.
- **Mejora sobre Snagit:** el mismo funcionamiento (botón y Listo) en ambas plataformas; confirmación antes de recalcular.
- **Fuentes:** research/02-editor.md §5.1, §4.1; research/05-plataformas-versiones.md §2, §3 (Editor: Smart Move activation), §1B (2024.4.0)

### SMT-019 · Simplificar automático
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Auto Simplify de la herramienta Simplify (control Detail; los objetos se regeneran al cambiarlo y se pierden las ediciones manuales)
- **Comportamiento:**
  - La herramienta Simplificar es de `EDT-` (EDT-041: paletas, modos Dibujar y Mover, Mostrar original, bloquear al salir). Esta ficha define solo la generación automática.
  - Casilla **Simplificar automáticamente** en las propiedades de la herramienta: XcreenShot detecta los bloques de texto (SMT-003) y los elementos de interfaz (como en SMT-018) y crea encima un objeto de simplificación por cada uno:
    - el texto se convierte en barras redondeadas de la altura de la línea, una por línea;
    - los botones, campos e íconos se convierten en formas sencillas (rectángulo, rectángulo redondeado o círculo) del color de la paleta activa más parecido al del elemento.
  - Control **Detalle** (de 1 a 10; por defecto 5): con menos detalle, se simplifica por bloques grandes; con más, por elemento y por línea.
  - Al cambiar Detalle o volver a activar la casilla, los objetos se generan de nuevo. **Antes de regenerar**, si el usuario editó, movió o borró alguno de los objetos generados, aparece el aviso "Se perderán los cambios hechos a N objetos simplificados", con **Regenerar** y **Cancelar** (decisión de diseño). Los objetos que el usuario dibujó a mano no se tocan nunca.
  - Desactivar la casilla quita los objetos generados que no se hayan editado (decisión de diseño).
  - Los objetos generados son objetos de simplificación normales: se editan con las propiedades de `EDT-` y se guardan en el `.xshot`.
- **Criterios de aceptación:**
  - [ ] En la captura de una ventana de ajustes, Simplificar automático tapa cada línea de texto con una barra y cada botón con una forma de la paleta activa.
  - [ ] Tras mover un objeto generado, cambiar Detalle muestra el aviso; **Cancelar** deja todo como estaba.
  - [ ] Un objeto de simplificación dibujado a mano sigue en su sitio tras regenerar.
- **Mejora sobre Snagit:** aviso antes de perder las ediciones; los objetos dibujados a mano se conservan.
- **Fuentes:** research/02-editor.md §3.13, §5.5; research/05-plataformas-versiones.md §2

### SMT-020 · Quitar fondo
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** Remove Background automático (solo Mac en Snagit; Windows usa métodos manuales)
- **Comportamiento:**
  - Clic derecho en la imagen base o en una imagen flotante > **Quitar fondo**. También en el menú Imagen.
  - XcreenShot detecta el sujeto principal (personas, animales, objetos) y vuelve transparentes los demás píxeles. Los bordes quedan suavizados, sin halo duro.
  - Si se aplica a la imagen base, el lienzo pasa a fondo transparente (EDT-074). Al guardar en un formato sin transparencia, el fondo toma el color del lienzo.
  - Es una sola acción que se deshace en un paso. El resultado se puede retocar con el Borrador (EDT-048) y con las herramientas de selección.
  - Opciones del aviso que aparece tras aplicarlo (decisión de diseño):
    - **Invertir**: conserva el fondo y quita el sujeto;
    - **Crear como imagen flotante**: deja la imagen original intacta y añade encima el sujeto recortado.
  - Funciona mejor con fotos y fondos no uniformes. El pelo y los bordes muy finos pueden quedar imperfectos; la ayuda lo explica.
  - Para fondos de un solo color, la herramienta Varita mágica (EDT-046) sigue siendo la opción precisa; la ayuda de Quitar fondo la menciona.
  - Si no encuentra un sujeto claro, lo dice y no cambia nada.
- **Criterios de aceptación:**
  - [ ] Con la red desconectada, Quitar fondo en la foto de prueba de una persona delante de una estantería deja a la persona sobre fondo transparente, con más del 95 % de coincidencia con la máscara de referencia.
  - [ ] El resultado guardado en PNG tiene canal alfa y el fondo es transparente.
  - [ ] En Windows y en macOS el resultado de la misma foto es equivalente: coincidencia entre ambas máscaras ≥ 95 %.
  - [ ] Un Deshacer devuelve la imagen original.
- **Notas de plataforma:** usar la segmentación de sujetos que ofrece el sistema operativo cuando exista, y un modelo local incluido en la app cuando no (sugerencia, no obligatoria).
- **Mejora sobre Snagit:** también en Windows; opciones Invertir y Crear como imagen flotante.
- **Fuentes:** research/02-editor.md §5.4, §13 (2025); research/05-plataformas-versiones.md §2, §3 (Editor)

### SMT-021 · Aplicar solo al texto
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** opción "Apply only to text" de Highlighter, Blur y Spotlight (solo Mac en Snagit; antes experimento "Smart Highlight")
- **Comportamiento:**
  - Casilla **Solo el texto** en las propiedades de las herramientas Resaltador, Desenfocar y Foco (EDT-036, EDT-040, EDT-039). Por defecto: desactivada; cada herramienta la recuerda por separado.
  - Con la casilla activada, al dibujar un área la herramienta no cubre todo el rectángulo, sino solo el texto que hay dentro (SMT-003):
    - **Resaltador:** una banda por línea, ajustada al alto y al ancho del texto de esa línea, como un resaltador de verdad;
    - **Desenfocar:** una zona por línea o palabra, solo sobre el texto;
    - **Foco:** quedan iluminadas las líneas de texto del área y se atenúa todo lo demás.
  - El objeto sigue siendo uno solo: al moverlo o cambiar su tamaño, las bandas se recalculan con el texto que queda dentro (decisión de diseño).
  - Si el área no contiene texto, el objeto se comporta como con la casilla desactivada y una pista lo indica.
  - Las propiedades normales (color, opacidad, tipo de desenfoque, intensidad) se mantienen.
- **Criterios de aceptación:**
  - [ ] Con Resaltador y Solo el texto, arrastrar sobre un párrafo de tres líneas de distinta longitud produce tres bandas, cada una del ancho de su línea.
  - [ ] Con Desenfocar y Solo el texto, un área sobre un formulario desenfoca las etiquetas y los valores, pero no los bordes de los campos.
  - [ ] Al mover el objeto a otro párrafo, las bandas se ajustan al texto nuevo.
- **Mejora sobre Snagit:** también en Windows; el objeto se recalcula al moverlo.
- **Fuentes:** research/02-editor.md §3.8, §3.11, §3.12, §5.6, §13 (2026); research/05-plataformas-versiones.md §2, §3 (Editor), §5; research/04-biblioteca-compartir.md §7

---

## 5. Grabador de pasos

### SMT-022 · Iniciar el Grabador de pasos
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** Step Capture (preset Step Capture; Selection = Steps; aviso "capturing steps"; ajustes de captura respetados)
- **Comportamiento:**
  - El Grabador de pasos crea una guía paso a paso a partir de los clics del usuario. Se inicia:
    - con Panel de captura > Imagen > Selección = **Grabador de pasos** (CAP-002) y **Capturar**;
    - con el preset incorporado **Grabador de pasos** (CAP-064);
    - con su atajo global (sin atajo por defecto; se asigna en APP-018), desde el menú del ícono (CAP-005) o con el comando `smart.steps_start`.
  - **Propiedades del modo** (engranaje de CAP-002; se guardan en los presets) (decisión de diseño):
    - **Área**: **Todas las pantallas** (por defecto), **Un monitor** (se elige al empezar) o **Una ventana** (se elige al empezar; solo los clics dentro de esa ventana crean pasos);
    - **Título de la guía**: vacío (se propone al terminar, SMT-028) o un texto fijo.
  - Opciones del Panel de captura que respeta (completa la tabla de CAP-002):
    - **Capturar cursor** (CAP-047): si está activado, cada paso lleva el puntero como objeto (SMT-026);
    - **Retardo** (CAP-048): cuenta atrás antes de empezar;
    - **Efectos** (CAP-051): se aplican a la imagen de cada paso, por ejemplo Escalar imagen en pantallas 4K;
    - **Compartir** (CAP-052), incluidos el destino Archivo y varios destinos a la vez: reciben la guía al terminar;
    - **Abrir en** (CAP-045): Editor abre la guía al terminar; Ninguno solo la guarda en la biblioteca; Anotación rápida no se aplica a las guías y se comporta como Editor;
    - **Copiar al portapapeles** (CAP-046): copia la guía al terminar (SMT-034);
    - Intervalo y Programada no se aplican y aparecen desactivados.
  - Al empezar aparece durante 2 s, en el centro del monitor principal, el aviso **"Grabando pasos: cada clic crea un paso"**, con el atajo para cancelar. Los clics mientras se ve el aviso no crean pasos. Un clic en el aviso lo cierra al momento (decisión de diseño en la duración).
  - En macOS hace falta el permiso de Accesibilidad para detectar los clics y leer los elementos (APP-024). Sin él, el modo aparece como no disponible y su tooltip explica cómo activarlo.
  - No puede haber una grabación de video y una de pasos a la vez: el segundo inicio responde `busy` (APP-028) y lo dice en pantalla (decisión de diseño).
  - **Opciones del grabador de pasos** (Preferencias > Captura, APP-008): Tecla para cancelar (SMT-023), Anotar el uso del teclado (SMT-024), Encuadre del paso (SMT-026) y Autor de las guías (SMT-028).
  - **Comandos** (APP-027): `smart.steps_start` (área, título, preset opcional), `smart.steps_pause`, `smart.steps_resume`, `smart.steps_status` (estado y número de pasos), `smart.steps_stop` (resultado: identificador de la guía en la biblioteca, número de pasos y ruta) y `smart.steps_cancel`. Mientras graba desde el CLI, los clics del usuario crean los pasos igual que desde la interfaz.
- **Criterios de aceptación:**
  - [ ] Ejecutar el preset Grabador de pasos muestra el aviso y, tras él, la barra del grabador con el contador a 0.
  - [ ] Con Efectos = Escalar imagen al 50 %, cada imagen de paso mide la mitad que la pantalla capturada.
  - [ ] Con Área = Una ventana, los clics fuera de esa ventana no suman pasos.
  - [ ] En macOS sin permiso de Accesibilidad, Selección = Grabador de pasos aparece no disponible y la captura de región sigue funcionando.
- **Mejora sobre Snagit:** limitar la grabación a un monitor o a una ventana.
- **Fuentes:** research/01-captura.md §8.1, §4.1, §4.3, §9.1, §14; research/04-biblioteca-compartir.md §4 (Built-in special presets); research/05-plataformas-versiones.md §1A (2025.0.0), §1B (2025.0.0)

### SMT-023 · Barra del grabador
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** barra de Step Capture (contador de pasos, Pause, Record, Stop; se puede arrastrar; nunca sale en las imágenes; ignora sus propios clics; Esc cancela; pausa desde el ícono de bandeja)
- **Comportamiento:**
  - Barra pequeña que aparece en la esquina inferior izquierda del monitor principal y se puede arrastrar a cualquier sitio, también a otro monitor. Recuerda su posición (decisión de diseño). Contiene:
    - **contador de pasos**, que destella brevemente cada vez que se añade uno;
    - **Pausar** / **Reanudar**: en pausa, los clics no crean pasos y la barra muestra "En pausa";
    - **Deshacer último paso** (decisión de diseño): quita el último paso grabado;
    - **Terminar**: cierra la grabación y crea la guía (SMT-028), que sigue la ruta de Abrir en, Compartir y Copiar (SMT-022);
    - **Cancelar**: descarta la grabación. Si ya hay pasos, pide confirmación en la propia barra ("¿Descartar 12 pasos?").
  - **Teclas:**
    - **Tecla para cancelar** (Opciones del grabador de pasos): `Esc` (por defecto), `Shift+Esc` o ninguna. Mientras graba, esa tecla la recibe XcreenShot, no la app que se está documentando. Quien necesite `Esc` en la app puede elegir `Shift+Esc` (decisión de diseño).
    - Atajos asignables para Pausar/Reanudar y Terminar (APP-018), sin valor por defecto.
  - Los clics en la barra, en su confirmación y en cualquier otra ventana de XcreenShot nunca crean pasos.
  - La barra, el aviso de inicio y cualquier indicador de XcreenShot nunca aparecen en las imágenes de los pasos.
  - El menú del ícono muestra en su parte de estado **Pausar/Reanudar grabación de pasos** y **Terminar** (APP-002); al salir de la app con una grabación activa se pregunta qué hacer (APP-004).
- **Criterios de aceptación:**
  - [ ] Hacer clic en Pausar, hacer 3 clics en otra app y pulsar Reanudar no añade pasos; el siguiente clic sí añade uno.
  - [ ] Con 5 pasos grabados, `Esc` muestra la confirmación; **Descartar** termina sin crear nada en la biblioteca.
  - [ ] Si la barra se arrastra encima de la zona donde se hace clic, no aparece en la imagen del paso.
  - [ ] **Deshacer último paso** con 4 pasos deja el contador en 3 y la guía final tiene 3 pasos.
- **Mejora sobre Snagit:** deshacer el último paso sin parar la grabación; tecla de cancelar configurable.
- **Fuentes:** research/01-captura.md §8.1, §1.1, §10; research/05-plataformas-versiones.md §1A (2025.1.0)

### SMT-024 · Qué crea un paso
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** cada clic o arrastre crea un paso; clics derechos detectados y etiquetados; el teclado no se graba; los pequeños temblores del cursor no se dividen en muchos pasos
- **Comportamiento:**
  - Crean un paso:
    - **clic izquierdo**;
    - **clic derecho**, que se marca como tal en el texto y en el resaltado del clic;
    - **doble clic**: un solo paso, marcado como doble clic (decisión de diseño);
    - **arrastre**: un solo paso desde que se pulsa hasta que se suelta, con una flecha editable del punto de inicio al de llegada (decisión de diseño).
  - Un movimiento de menos de 5 px entre pulsar y soltar cuenta como clic, no como arrastre. Dos pulsaciones en el tiempo de doble clic del sistema cuentan como un doble clic (decisión de diseño en los umbrales).
  - **Momento de la imagen:** la imagen muestra la pantalla tal como estaba **justo antes** del clic, para que se vea el elemento que se pulsó y no el resultado (decisión de diseño). En los arrastres, la del momento de pulsar.
  - No crean pasos: mover el puntero, la rueda del ratón, el clic central, los gestos del panel táctil que no son clics ni los clics sobre XcreenShot.
  - **Teclado:** por defecto no se graba nada de lo que se escribe ni de los atajos de teclado; el usuario lo añade en la guía. Ajuste **Anotar el uso del teclado** (Opciones del grabador de pasos; por defecto: no) (decisión de diseño):
    - un atajo con modificador (por ejemplo `Ctrl+S`) crea un paso con el texto "Pulsar Ctrl+S";
    - al escribir en un campo, se añade al texto del paso anterior "y escribir en el campo «Nombre»", **sin el texto escrito**;
    - nunca se guarda ninguna tecla escrita, tampoco en memoria más allá de detectar que hubo escritura.
  - El texto de cada paso lo crea SMT-025 y la imagen, SMT-026.
- **Criterios de aceptación:**
  - [ ] Un clic izquierdo, un clic derecho y un arrastre en el Explorador de archivos o en el Finder crean 3 pasos, y el segundo está marcado como clic derecho.
  - [ ] Un doble clic en un archivo crea un solo paso.
  - [ ] Un clic con un temblor de 3 px crea un clic, no un arrastre.
  - [ ] Con Anotar el uso del teclado activado, escribir una contraseña en un campo no deja ningún carácter de ella en la guía, en el `.xshot` ni en los registros.
- **Mejora sobre Snagit:** anotación opcional del teclado, siempre sin el contenido escrito.
- **Fuentes:** research/01-captura.md §8.1; research/05-plataformas-versiones.md §1A (2025.1.0), §2

### SMT-025 · Texto automático de cada paso
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** texto descriptivo automático de cada paso (Windows 2025.1.0; en Mac no consta) y etiqueta de clic derecho
- **Comportamiento:**
  - Cada paso recibe una frase que describe lo que se pulsó, en el idioma de la interfaz (decisión de diseño).
  - **Fuente de la información**, en este orden (decisión de diseño):
    1. **Información de accesibilidad del sistema** sobre el elemento pulsado: tipo (botón, pestaña, menú, elemento de menú, casilla, opción, campo de texto, enlace, lista, elemento de lista, archivo) y nombre visible.
    2. Si no hay información útil (apps sin accesibilidad, juegos, escritorios remotos, ventanas elevadas), el **texto reconocido** (SMT-003) bajo el punto del clic o más cercano a él.
    3. Si tampoco hay texto: una frase genérica.
  - **Plantillas de frase** (ejemplos en español; equivalentes en inglés) (decisión de diseño):

    | Situación | Texto |
    |---|---|
    | Botón | Clic en el botón **Guardar** |
    | Pestaña | Seleccionar la pestaña **Vista** |
    | Menú de la barra de menús | Abrir el menú **Archivo** |
    | Elemento de menú | Elegir **Exportar…** en el menú |
    | Casilla | Activar la casilla **Recordarme** (o "Desactivar…", según su estado después del clic) |
    | Campo de texto | Clic en el campo **Correo electrónico** |
    | Enlace | Clic en el enlace **Política de privacidad** |
    | Elemento de lista o archivo | Seleccionar **informe.docx** |
    | Clic derecho | Clic derecho en **informe.docx** |
    | Doble clic | Doble clic en **informe.docx** |
    | Arrastre | Arrastrar **informe.docx** hasta **Documentos** |
    | Solo texto reconocido | Clic en «Guardar» |
    | Sin información | Clic aquí · Clic derecho aquí · Arrastrar desde aquí hasta aquí |

  - Cuando la aplicación cambia respecto al paso anterior (y en el primer paso), la frase empieza con "En **<Aplicación>**, …".
  - El nombre del elemento va en negrita. Los nombres de más de 60 caracteres se recortan con "…".
  - La frase se genera con reglas y plantillas, sin modelos generativos (SMT-001). Es texto normal de la guía: se puede editar (SMT-029).
- **Criterios de aceptación:**
  - [ ] En el Bloc de notas o en TextEdit, clic en el menú Archivo y luego en Guardar como… produce "En Bloc de notas, abrir el menú **Archivo**" (o su equivalente en TextEdit) y "Elegir **Guardar como…** en el menú".
  - [ ] En una app sin información de accesibilidad, un clic sobre un botón con la palabra "Aceptar" produce "Clic en «Aceptar»".
  - [ ] Un clic sobre una zona de una foto sin texto produce "Clic aquí".
  - [ ] Con la interfaz en inglés, las frases salen en inglés ("Click the **Save** button").
- **Mejora sobre Snagit:** comportamiento definido y el mismo en ambas plataformas, con reserva por reconocimiento de texto.
- **Fuentes:** research/01-captura.md §8.1, §16 (punto 7); research/05-plataformas-versiones.md §1A (2025.1.0), §2

### SMT-026 · Imagen de cada paso
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** imagen ampliada y centrada en la zona detectada, con el cursor y el resaltado del clic como objetos editables; captura de pantalla completa si la zona mide menos de 200×200 px
- **Comportamiento:**
  - Cada paso guarda la imagen **completa del monitor** donde se hizo clic, sin recortar, y un **encuadre** sobre ella (como el historial de recorte de `.xshot`, SPEC §7). Así el encuadre se puede cambiar después sin perder nada (EDT-049, EDT-050).
  - Ajuste **Encuadre del paso** (Opciones del grabador de pasos):
    - **Zona del clic** (por defecto): el encuadre se ajusta al contenedor del elemento pulsado (cuadro de diálogo, menú, panel o barra de herramientas), según la información de accesibilidad o la detección de subáreas (CAP-016), con un margen de 24 px (decisión de diseño). Si la zona detectada mide menos de 200×200 px, el paso usa la pantalla completa del monitor;
    - **Ventana**: la ventana sobre la que se hizo clic;
    - **Pantalla completa**: el monitor entero.
  - **Objetos editables** sobre la imagen:
    - el **puntero**, si Capturar cursor está activado (CAP-047), con la forma que tenía en ese momento;
    - el **resaltado del clic**: un anillo semitransparente centrado en el punto del clic; con otro color para el clic derecho (decisión de diseño: naranja para el izquierdo y violeta para el derecho, tomados del tema activo si los define);
    - la **flecha** de los arrastres (SMT-024).
  - Se pueden mover, cambiar de estilo o borrar como cualquier objeto (SMT-029).
  - Pantallas de alta densidad: la imagen se guarda a la resolución real. Si el usuario quiere imágenes más pequeñas, usa el efecto Escalar imagen (SMT-022) o la escala de la guía (SMT-029).
  - Si el clic cae entre dos monitores, se usa el monitor que contiene el punto del clic.
- **Criterios de aceptación:**
  - [ ] Un clic en el botón Aceptar de un cuadro de diálogo de 500×300 px produce una imagen encuadrada en el diálogo, con el anillo del clic sobre el botón.
  - [ ] Un clic en un ícono aislado de la barra de tareas o del Dock, cuya zona mide menos de 200×200 px, produce una imagen de la pantalla completa.
  - [ ] En la guía, ampliar el encuadre de un paso recupera el contenido de alrededor del monitor original.
  - [ ] Borrar el anillo del clic y el puntero deja la imagen limpia, sin restos.
- **Mejora sobre Snagit:** encuadre configurable (zona, ventana o pantalla) y recuperable después.
- **Fuentes:** research/01-captura.md §8.1, §4.7; research/05-plataformas-versiones.md §1A (2025.4.0), §1B (2025.2.0)

### SMT-027 · Límites, rendimiento, ventanas elevadas y recuperación
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** sin máximo fijo de pasos (rendimiento recomendado hasta unos 50 en Windows y 100 en Mac); corrección de memoria; captura de apps elevadas como el Administrador de tareas; recuperación de una captura de pasos en curso tras un fallo
- **Comportamiento:**
  - **Sin máximo fijo de pasos.** Rendimiento exigido hasta **300 pasos** en el equipo de referencia (decisión de diseño):
    - el clic del usuario nunca se retrasa ni se bloquea: la app documentada responde igual que sin grabar;
    - cada paso queda registrado (contador actualizado) en < 200 ms;
    - el consumo de memoria no crece con el número de pasos: las imágenes se escriben a disco al momento (< 50 MB de memoria adicional con 300 pasos);
    - al pulsar Terminar con 300 pasos, la guía se abre en < 5 s.
  - A partir de 200 pasos, la barra muestra una pista que sugiere dividir la tarea en varias guías (decisión de diseño). No impide seguir.
  - **Ventanas elevadas (Windows):** los clics sobre ventanas que se ejecutan como administrador (por ejemplo, el Administrador de tareas) crean pasos con su imagen. Si el sistema no deja leer la información de accesibilidad, el texto se obtiene del texto reconocido (SMT-025). Si tampoco se pueden detectar los clics sobre esas ventanas, XcreenShot avisa una vez y ofrece **Reiniciar como administrador** (APP-026).
  - **Recuperación:** los pasos se guardan en la carpeta de recuperación a medida que se crean. Si la app se cierra de forma inesperada, el siguiente arranque ofrece la grabación en **Recuperar trabajo** (APP-047), con todos los pasos registrados hasta el fallo, como una guía normal.
  - **Disco lleno:** la grabación se pausa sola, avisa y conserva los pasos ya guardados.
- **Criterios de aceptación:**
  - [ ] Una prueba automática que hace 300 clics produce una guía de 300 pasos, sin retrasos de más de 200 ms en ningún paso y con < 50 MB de memoria adicional.
  - [ ] En Windows, sin ejecutar XcreenShot como administrador, 3 clics en el Administrador de tareas crean 3 pasos con imagen y texto.
  - [ ] Matar el proceso tras 20 pasos y volver a abrir la app permite recuperar una guía con esos 20 pasos.
- **Mejora sobre Snagit:** rendimiento exigido hasta 300 pasos en ambas plataformas.
- **Fuentes:** research/01-captura.md §8.1, §14; research/05-plataformas-versiones.md §3 (Step Capture: recommended step ceiling), §1A (2025.1.0, 2025.4.0), §1B (2026.3.0, 2026.4.0)

---

## 6. Guía de pasos y sus salidas

### SMT-028 · Estructura y formato de la guía
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** plantilla de la guía de Step Capture (encabezado, secciones de imagen numeradas, secciones de texto, pie; el texto largo cabe); ajustes de plantilla (título, pie con nombre del autor del sistema o propio, tema, numerar imágenes)
- **Comportamiento:**
  - Al terminar una grabación (SMT-023) se crea un **documento de guía**: un `.xshot` con tipo `guide` (SPEC §7) que se abre en el editor en la **vista de guía**. Usa el motor de plantillas del editor (EDT-095) con una plantilla propia de guía.
  - **Partes de la guía**, de arriba abajo:
    1. **Encabezado:** título (por defecto, "Pasos en <aplicación más usada en la grabación>", por ejemplo "Pasos en Excel", o el título fijado en SMT-022; siempre editable) (decisión de diseño), fecha de creación y, opcional, una descripción (vacía por defecto).
    2. **Secciones de paso**, numeradas 1, 2, 3…: el número con el estilo del objeto Paso del tema (EDT-035), el texto del paso (SMT-025) y su imagen encuadrada (SMT-026).
    3. **Secciones de texto** opcionales, que el usuario añade entre pasos (notas, advertencias, contexto). No llevan número.
    4. **Pie:** **Creado por** con el nombre del usuario del sistema (por defecto) o un texto propio, y la fecha. El nombre se configura en el ajuste **Autor de las guías** de las Opciones del grabador de pasos (SMT-022) y se puede cambiar en cada guía (decisión de diseño).
  - **Diseño** (decisión de diseño):
    - una columna, de 1200 px de ancho; las imágenes se escalan para caber y nunca se amplían por encima de su tamaño real;
    - el texto va encima de la imagen y crece en altura lo que haga falta, sin cortarse;
    - colores y fuente del **tema activo** (EDT-017). La guía se puede cambiar a otro tema después.
  - **Formato del archivo** (propuesta para SPEC §7): en `document.json`, el objeto `guide` contiene el encabezado, la lista ordenada de secciones (tipo `step`, `text` o `image`; texto con formato; referencia a la imagen; encuadre; objetos) y el pie. Cada imagen completa de paso se guarda en `images/step-NNN.png`. `thumb.png` es la imagen del primer paso.
  - **Metadatos** de la guía: número de pasos, aplicaciones que aparecen y fecha de inicio y fin de la grabación (CAP-056).
  - Al compartir o exportar, la guía se convierte en el formato de cada salida (SMT-030 a SMT-034). Al guardarla como imagen (PNG, JPG), se exporta como una sola imagen larga con todas las secciones (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Una grabación de 5 clics termina con una guía con encabezado, 5 secciones de paso numeradas del 1 al 5, con su texto y su imagen, y un pie con el nombre del usuario del sistema.
  - [ ] Un texto de paso de 400 caracteres se ve completo en la guía, sin cortarse ni solaparse con la imagen.
  - [ ] Un `.xshot` de guía creado en Windows se abre en macOS con los mismos pasos, textos, encuadres y objetos.
  - [ ] Cambiar el tema de la guía cambia el color de los números de paso y la fuente de los textos.
- **Fuentes:** research/01-captura.md §8.1 (Output); research/02-editor.md §9, §10.2, §14; research/05-plataformas-versiones.md §1A (2025.2.0)

### SMT-029 · Editar la guía
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** edición de la guía de Step Capture: anotaciones ancladas a los pasos, mover o borrar el cursor, efecto Scale Image, Capture en un marcador vacío, icono de candado antes de editar zonas de imagen y textos; renumeración
- **Comportamiento:**
  - **Panel de pasos** (lateral) con la lista de secciones en miniatura. Desde ahí y desde el menú de cada sección:
    - **reordenar** arrastrando; los números se recalculan solos;
    - **eliminar** una o varias secciones (`Supr`);
    - **duplicar** un paso;
    - **añadir** una sección de paso vacía, una de texto o una de imagen, antes o después de la elegida;
    - **unir** dos pasos de texto o **dividir** uno en dos (decisión de diseño).
  - **Texto de los pasos:** se edita en el sitio con doble clic, con negrita, cursiva, listas y enlaces. El corrector ortográfico actúa sobre él (EDT-027).
  - **Imágenes de los pasos:** todas las herramientas del editor funcionan sobre la imagen de cada paso: flecha, globo, sello, forma, desenfoque, Redacción automática (SMT-010), etc.
    - Las anotaciones quedan **ancladas** a la imagen de su paso: al reordenar, mover o cambiar el tamaño de las secciones, se mueven con ella.
    - El **puntero** y el **resaltado del clic** (SMT-026) se pueden mover, cambiar de estilo o borrar.
    - **Recortar** cambia el encuadre del paso y puede ampliarlo hasta la imagen completa del monitor.
  - **Bloqueo de la imagen:** cada sección de imagen tiene un candado. Con el candado cerrado (por defecto), arrastrar dentro de la imagen dibuja o selecciona anotaciones y la imagen no se mueve. Con el candado abierto se puede desplazar y ampliar la imagen dentro de su sección, con botones de zoom del 10 % y ajuste a los bordes y al centro, como en las zonas de imagen de las plantillas (EDT-098) (decisión de diseño).
  - **Escala de las imágenes:** control de la guía que reduce todas las imágenes de paso a un porcentaje (por ejemplo, 50 % para capturas de pantallas 4K o de alta densidad), sin perder la imagen original (decisión de diseño). Equivale al efecto Escalar imagen aplicado a todos los pasos.
  - **Nueva captura en una sección:** una sección de imagen vacía muestra el botón **Capturar**, que lanza una captura universal (CAP-009) y coloca el resultado en esa sección. También se puede pegar o arrastrar una imagen.
  - Las secciones de imagen vacías no salen al guardar, imprimir, copiar ni exportar.
  - **Añadir pasos grabando:** menú de la guía > **Grabar más pasos aquí** reanuda el grabador e inserta los pasos nuevos después de la sección elegida (decisión de diseño).
  - Todo se puede deshacer y rehacer paso a paso.
- **Criterios de aceptación:**
  - [ ] Al arrastrar el paso 4 delante del paso 2, se renumeran del 1 al N sin huecos, y la flecha dibujada sobre el antiguo paso 4 sigue en el mismo sitio de su imagen.
  - [ ] Borrar el puntero de un paso no borra el de los demás.
  - [ ] Una sección de imagen vacía no aparece en el PDF exportado.
  - [ ] Con el candado de una imagen abierto, arrastrar desplaza la imagen dentro de la sección; con el candado cerrado, arrastrar con la herramienta Flecha dibuja una flecha.
  - [ ] **Capturar** en una sección vacía coloca la nueva captura en ella.
- **Mejora sobre Snagit:** unir y dividir pasos, y grabar pasos nuevos dentro de una guía existente.
- **Fuentes:** research/02-editor.md §9, §14; research/01-captura.md §4.3, §8.1; research/05-plataformas-versiones.md §1A (2025.2.0, 2025.3.0), §1B (2025.1.0, 2025.2.0), §3 (Step Capture)

### SMT-030 · Guía a Word y PowerPoint
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** compartir la guía en Microsoft Word (imágenes por separado y texto editable) y en PowerPoint (presentación con diapositiva de título, una diapositiva por imagen y texto editable)
- **Comportamiento:**
  - **Exportar como documento de Word (`.docx`)** y **Exportar como presentación de PowerPoint (`.pptx`)**, en Archivo > Exportar guía y en el menú Compartir de la guía. XcreenShot genera los archivos por sí mismo, sin necesidad de tener Office instalado (decisión de diseño). Los destinos Word y PowerPoint de `SHR-` los abren en la app si está instalada, con sus opciones de documento nuevo o actual.
  - **Word:**
    - título de la guía como Título 1 y la descripción como párrafo normal;
    - cada paso como elemento de una **lista numerada** de Word con su texto editable (negritas incluidas), seguido de su imagen (ya con las anotaciones) como imagen independiente, ajustada al ancho de la página;
    - las secciones de texto como párrafos normales;
    - el pie como pie de página del documento;
    - estilos de Word estándar (Título 1, Normal, Lista con números), para que el usuario los cambie con la plantilla de su organización.
  - **PowerPoint:**
    - una **diapositiva de título** con el título de la guía, la descripción y el autor;
    - **una diapositiva por paso**, con el número y el texto del paso como título editable y la imagen ajustada al espacio libre, sin deformarla;
    - las secciones de texto, como diapositivas de texto;
    - formato 16:9 (decisión de diseño).
  - Las imágenes van a resolución completa, o a la de la escala de la guía (SMT-029). El texto alternativo de cada imagen es el texto del paso.
- **Criterios de aceptación:**
  - [ ] Una guía de 6 pasos exportada a `.docx` se abre en Word con el título, una lista numerada del 1 al 6 con el texto editable y 6 imágenes; cambiar el estilo Normal en Word cambia la fuente de los pasos.
  - [ ] La misma guía exportada a `.pptx` tiene 7 diapositivas (título y 6 pasos), y el título de cada una se puede editar como texto.
  - [ ] Los archivos se generan en un equipo sin Office y se abren sin errores en Word, PowerPoint y LibreOffice.
  - [ ] Cada imagen de los archivos tiene como texto alternativo el texto de su paso.
- **Mejora sobre Snagit:** genera los archivos sin Office instalado; texto alternativo para accesibilidad.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Microsoft Word, Microsoft PowerPoint); research/01-captura.md §8.1 (Output); research/02-editor.md §14; research/05-plataformas-versiones.md §1A (2025.3.0, 2026.1.0), §1B (2025.1.0, 2026.0.0), §3 (Step Capture)

### SMT-031 · Guía a Pages y Keynote (macOS)
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** macOS. **Motivo:** Pages y Keynote son apps de Apple que solo existen en macOS. En Windows, sus equivalentes son Word y PowerPoint (SMT-030).
- **Paridad con Snagit:** guía de pasos a Apple Pages y a Keynote, con texto editable
- **Comportamiento:**
  - Entradas **Pages** y **Keynote** en Archivo > Exportar guía y en el menú Compartir de la guía (destinos de `SHR-`), solo cuando esas apps están instaladas.
  - **Pages:** documento con la misma estructura que el de Word (título, lista numerada de pasos con texto editable e imágenes, secciones de texto y pie). Opción de documento nuevo o del documento abierto, como en el destino de Pages.
  - **Keynote:** presentación con la misma estructura que la de PowerPoint (diapositiva de título y una por paso, con título editable).
  - Abrir en Pages o Keynote el `.docx` o el `.pptx` de SMT-030, o controlarlos mediante la automatización del sistema, son caminos válidos (sugerencia, no obligatoria). El requisito es que el texto se pueda editar.
- **Criterios de aceptación:**
  - [ ] En macOS con Pages instalado, compartir una guía de 4 pasos en Pages abre un documento con 4 pasos numerados cuyo texto se puede editar.
  - [ ] En macOS con Keynote instalado, compartir la misma guía crea 5 diapositivas con títulos editables.
  - [ ] Sin Pages instalado, la entrada Pages no aparece ni en Exportar guía ni en el menú Compartir.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Pages / Keynote / Numbers); research/05-plataformas-versiones.md §1B (2025.2.0, 2026.0.0), §3 (Step Capture)

### SMT-032 · Guía a Markdown y HTML
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** exportación de la guía a Markdown y a HTML: una carpeta con el archivo `.md` o `.html` y las imágenes (solo Mac en Snagit)
- **Comportamiento:**
  - Archivo > Exportar guía > **Markdown** o **HTML**. Se elige una carpeta de destino y XcreenShot crea en ella una carpeta con el nombre de la guía:
    - `<nombre>.md` o `index.html`;
    - subcarpeta `images/` con `paso-01.png`, `paso-02.png`… (anotaciones incluidas).
  - **Markdown** (decisión de diseño): título como `#`, descripción como párrafo, cada paso como elemento de lista numerada (`1.`) con su texto (negritas como `**…**`) y debajo la imagen con ruta relativa y el texto del paso como texto alternativo; las secciones de texto como párrafos; el pie al final, en cursiva. Sintaxis compatible con CommonMark y con las variantes habituales de los gestores de repositorios.
  - **HTML** (decisión de diseño): una página válida, legible sin conexión, con estilos propios incluidos en la página (sin hojas de estilo ni fuentes externas), una lista ordenada `<ol>` de pasos, imágenes con `alt` y diseño adaptable que se lee bien en un móvil.
  - Opción **Un solo archivo** (por defecto: no) (decisión de diseño): en HTML, incrusta las imágenes en la página; en Markdown, no se ofrece.
  - Formato de las imágenes: PNG (por defecto) o JPG, con la escala de la guía (SMT-029).
  - Si la carpeta ya existe, se ofrece **Reemplazar** o **Crear otra** (añade `-2`, `-3`…).
- **Criterios de aceptación:**
  - [ ] Exportar a Markdown una guía de 3 pasos crea `<nombre>.md` y `images/` con 3 imágenes. El `.md`, visto en un visor de Markdown, muestra los 3 pasos numerados con sus imágenes.
  - [ ] El `index.html` exportado se abre sin conexión en un navegador con los estilos y las imágenes correctos, y no hace ninguna petición de red.
  - [ ] Con Un solo archivo, el HTML se puede mover solo a otra carpeta y sigue mostrando las imágenes.
  - [ ] La exportación funciona igual en Windows y en macOS.
- **Mejora sobre Snagit:** también en Windows; HTML en un solo archivo.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Markdown / HTML export); research/01-captura.md §8.1 (Output); research/02-editor.md §14; research/05-plataformas-versiones.md §1B (2025.3.0, 2026.3.0), §3 (Step Capture)

### SMT-033 · Guía a PDF e impresión
- **Hito:** M5 · **Prioridad:** Must
- **Plataformas:** Ambas
- **Paridad con Snagit:** imprimir la guía o guardarla como PDF (Archivo > Imprimir o Guardar como PDF), con saltos de página
- **Comportamiento:**
  - **Imprimir** (`Ctrl/Cmd+P`) y **Exportar como PDF** desde la guía (EDT-114, EDT-110, EDT-111).
  - **Paginación** (decisión de diseño):
    - un paso nunca se corta entre dos páginas: si no cabe, pasa entero a la siguiente;
    - si la imagen de un paso es más alta que una página, se reduce hasta caber;
    - el encabezado solo va en la primera página;
    - cada página lleva al pie el título de la guía y "Página N de M".
  - Tamaño de papel: el del sistema (A4 o Carta según la región), con márgenes de 15 mm; orientación vertical por defecto, que se puede cambiar.
  - El PDF tiene texto seleccionable y buscable (no son imágenes del texto), marcadores con el título de cada paso y el texto alternativo de las imágenes (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Una guía de 12 pasos exportada a PDF en A4 no tiene ningún paso partido entre dos páginas.
  - [ ] En el PDF se puede buscar una palabra del texto de un paso y el visor la encuentra.
  - [ ] El panel de marcadores del visor de PDF lista los 12 pasos.
- **Mejora sobre Snagit:** PDF con texto buscable, marcadores por paso y paginación que no corta los pasos.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (Print / PDF (Step Capture)); research/02-editor.md §9 (Output), §10.5

### SMT-034 · Imágenes de la guía por separado y portapapeles
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** guardar los pasos como imágenes separadas con el destino Archivo; exportación por lotes de las imágenes de una Step Capture (solo Mac en Snagit)
- **Comportamiento:**
  - Archivo > Exportar guía > **Imágenes por separado**, y la opción **Guardar cada paso como una imagen** del destino Archivo (CAP-053) cuando lo que se comparte es una guía:
    - una imagen por sección de paso o de imagen, con sus anotaciones aplanadas;
    - formato PNG (por defecto) o JPG, con las opciones de la exportación por lotes (EDT-113): escala, efectos y nombre;
    - nombre por defecto `<nombre de la guía>_01.png`, `_02.png`…, con ceros a la izquierda según el número de pasos;
    - casilla **Incluir también el texto** (por defecto: no) (decisión de diseño), que añade `pasos.txt` con el texto numerado de cada paso.
  - **Copiar al portapapeles** (Copiar todo en la guía o la opción Copiar al portapapeles del grabador, SMT-022) copia la guía entera como contenido con formato (texto con imágenes), para pegarla en un correo, una wiki o un chat, y como texto sin formato para las apps que no lo admiten (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Exportar como imágenes por separado una guía de 9 pasos crea 9 archivos numerados del `_01` al `_09`, cada uno con sus anotaciones.
  - [ ] Pegar en un cliente de correo una guía copiada muestra los pasos numerados con su texto y sus imágenes.
  - [ ] Con Incluir también el texto, se crea `pasos.txt` con una línea numerada por paso.
- **Mejora sobre Snagit:** también en Windows; copia de la guía completa al portapapeles.
- **Fuentes:** research/04-biblioteca-compartir.md §3.2 (File); research/02-editor.md §10.4; research/05-plataformas-versiones.md §1B (2026.1.0, 2026.3.0), §3 (Step Capture)

### SMT-035 · Guías en la biblioteca
- **Hito:** M5 · **Prioridad:** Should
- **Plataformas:** Ambas
- **Paridad con Snagit:** categoría propia de las Step Captures en la biblioteca, con insignia del número de imágenes; exportarlas desde la biblioteca
- **Comportamiento:**
  - Las guías se guardan en la biblioteca como cualquier `.xshot` (LIB- (autoguardado)) y forman un tipo propio, **Guías de pasos**, en los filtros por tipo (LIB- (filtros)).
  - Su miniatura muestra la imagen del primer paso con una **insignia con el número de pasos** (por ejemplo, "12 pasos") y un ícono de guía.
  - La búsqueda de la biblioteca encuentra las guías por su título y por el texto de sus pasos (decisión de diseño). Es una búsqueda local (LIB- (búsqueda)).
  - Menú contextual de una guía en la biblioteca y en la bandeja reciente: **Abrir**, **Exportar como** (Word, PowerPoint, Markdown, HTML, PDF, Imágenes por separado y, en macOS, Pages y Keynote) y **Compartir**, sin abrir el editor.
  - Los metadatos de la guía (número de pasos, aplicaciones) se pueden usar en los filtros de aplicación de la biblioteca.
  - Comando `smart.guide_export` (APP-027): guía (identificador o ruta), formato (`docx`, `pptx`, `md`, `html`, `pdf`, `images`; en macOS también `pages` y `key`), carpeta o archivo de salida y las opciones de cada formato; resultado: rutas escritas (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Tras grabar una guía de 7 pasos, la biblioteca la muestra con la insignia "7 pasos" y aparece al filtrar por Guías de pasos.
  - [ ] Buscar en la biblioteca una palabra que solo aparece en el texto del paso 5 encuentra la guía.
  - [ ] Clic derecho > Exportar como > Markdown desde la biblioteca crea la carpeta de SMT-032 sin abrir el editor.
- **Mejora sobre Snagit:** búsqueda por el texto de los pasos; exportación desde la biblioteca en ambas plataformas.
- **Fuentes:** research/04-biblioteca-compartir.md §1.1 (Step Capture items in Library, Search pane categories); research/01-captura.md §8.1 (Output); research/05-plataformas-versiones.md §1A (2026.3.0), §1B (2026.3.0)

---

## 7. Códigos QR

Otras funciones que en Snagit para Mac nacieron como experimentos junto a los códigos QR ya tienen dueño y este archivo no las repite: la pantalla completa con cada ventana como capa editable es CAP-029 (Ventanas como objetos), y la medición de distancias en píxeles es EDT-094 (Medición en vivo).

### SMT-036 · Crear un código QR
- **Hito:** M5 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** experimento de Mac para crear códigos QR propios
- **Comportamiento:**
  - Menú Imagen > **Insertar código QR…** y herramienta **Código QR** del menú Más (EDT-003) (decisión de diseño en la ubicación). Se abre un panel con:
    - **Contenido:** **Enlace** (por defecto) o **Texto**. Con Enlace, si el texto no lleva esquema se añade `https://` (decisión de diseño);
    - **Corrección de errores:** Baja, Media (por defecto), Alta o Máxima, con una explicación breve: más corrección resiste mejor los daños pero da un código más denso;
    - **Color del código** (por defecto: negro) y **color del fondo** (por defecto: blanco, o transparente);
    - **Margen** alrededor del código, en módulos: de 0 a 8; por defecto 4.
  - Una vista previa se actualiza al escribir. El texto que no cabe en un QR (más de unos 2900 bytes con corrección Baja) se marca en rojo, con el máximo para la corrección elegida.
  - **Aviso de contraste:** si el contraste entre el código y el fondo es bajo, o si el código es más claro que el fondo, aparece "Puede que algunos lectores no lo lean" (decisión de diseño). Antes de insertar, XcreenShot lee el código generado con su propio lector (SMT-038) y avisa si no lo consigue.
  - **Insertar** lo añade al lienzo como un objeto **QR editable**: se mueve y cambia de tamaño sin perder nitidez (se vuelve a generar al tamaño final) y conserva su contenido. Doble clic vuelve a abrir el panel para cambiarlo. Al exportar se aplana como cualquier objeto.
  - También existe como comando `smart.qr_create` (contenido, opciones y ruta de salida PNG o SVG) (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Un QR creado con el enlace `https://example.com` y colocado en una imagen exportada a PNG se lee con la cámara de un teléfono y abre esa dirección.
  - [ ] Ampliar el objeto QR al 400 % no lo pixela: los bordes de los módulos siguen nítidos.
  - [ ] Con código gris claro sobre fondo blanco aparece el aviso de contraste.
  - [ ] Doble clic en el objeto permite cambiar el enlace, y el código se actualiza.
- **Mejora sobre Snagit:** función normal (no experimental) en ambas plataformas; objeto editable y comprobación de lectura.
- **Fuentes:** research/04-biblioteca-compartir.md §7; research/05-plataformas-versiones.md §5, §1B (2025.4.0); research/01-captura.md §12.4, §11.2; research/02-editor.md §11

### SMT-037 · Código QR de un enlace compartido
- **Hito:** M5 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** experimento de Mac que da a una captura compartida su código QR
- **Comportamiento:**
  - Cuando un envío produce un enlace (Enlace para compartir u otro destino en línea que devuelva enlace, `SHR-`), el aviso de envío terminado y el historial de envíos ofrecen **Mostrar código QR** junto a **Copiar enlace**.
  - **Mostrar código QR** abre una ventana pequeña con el código del enlace, a buen tamaño para leerlo con un teléfono desde la pantalla, y estas acciones:
    - **Copiar imagen** (PNG);
    - **Guardar como…** (PNG o SVG);
    - **Insertar en la captura**: abre la captura en el editor y añade el código como objeto QR (SMT-036).
  - El código se genera en el equipo; la generación no hace ninguna conexión adicional.
  - Depende de los destinos que devuelven enlace, que llegan con `SHR-` en M6. Si al cerrar M5 aún no existen, esta ficha se completa con M6; al ser Could, no bloquea M5.
- **Criterios de aceptación:**
  - [ ] Tras crear un Enlace para compartir, **Mostrar código QR** muestra un código que, leído con un teléfono, abre ese enlace.
  - [ ] **Insertar en la captura** añade el código como objeto QR editable a la captura compartida.
- **Mejora sobre Snagit:** función normal en ambas plataformas y con cualquier destino que devuelva enlace.
- **Fuentes:** research/04-biblioteca-compartir.md §7; research/05-plataformas-versiones.md §5, §1B (2025.4.0); research/01-captura.md §12.4

### SMT-038 · Leer códigos QR
- **Hito:** M5 · **Prioridad:** Could
- **Plataformas:** Ambas
- **Paridad con Snagit:** experimento de Mac para leer códigos QR durante la captura
- **Comportamiento:**
  - Ajuste **Detectar códigos QR al capturar** (Preferencias > Captura, APP-008; por defecto: sí; clave `smart.qr.detectOnCapture`) (decisión de diseño en el valor por defecto).
  - **Durante la captura** (captura universal y modos con selección en pantalla): con la pantalla congelada (CAP-013), XcreenShot busca códigos QR en segundo plano, sin retrasar la cruceta (APP-048). En la fase de ajuste, si la selección contiene un código, la barra de captura (CAP-011) muestra el botón **QR** con el número de códigos. Al pulsarlo (o la tecla `Q`) se abre un panel junto a la selección con el contenido de cada código y estas acciones:
    - **Copiar**;
    - **Abrir enlace**, solo si es un enlace `http` o `https`. Antes de abrirlo se muestra la dirección completa, con el dominio resaltado, y se pide confirmación. XcreenShot nunca abre nada automáticamente;
    - **Cerrar**, que vuelve a la fase de ajuste: la captura puede seguir o cancelarse con `Esc` sin crear nada.
  - **Después de capturar:** si la captura se abre en el editor y contiene códigos, una pastilla discreta sobre el lienzo dice **"Código QR detectado"** y marca el código. Al pulsarla se abre el mismo panel, con una acción más: **Redactar el código**, que lo cubre con una redacción (SMT-013) (decisión de diseño). Si la captura no abre el editor, no se muestra nada (decisión de diseño).
  - También se leen desde el menú contextual del editor (**Leer códigos QR** de la imagen o de la selección), desde la ventana de resultados de Extraer texto, que añade al final los códigos de la zona (SMT-006), y con el comando `smart.qr_read`.
  - Se leen también otros códigos de barras habituales (Data Matrix, EAN, Code 128) si el lector del equipo los admite (sugerencia, no obligatoria).
  - Si un QR contiene los datos de una red Wi-Fi, la contraseña solo se muestra al pulsar **Mostrar** (decisión de diseño).
- **Criterios de aceptación:**
  - [ ] Al seleccionar en la captura universal una zona con el QR de una página web, la barra de captura muestra el botón **QR**; el panel muestra la dirección y **Copiar** la copia, y después se puede cancelar la captura sin que quede nada en la biblioteca.
  - [ ] **Abrir enlace** pide confirmación antes de abrir el navegador y no lo abre si se cancela.
  - [ ] Una captura con tres QR distintos abierta en el editor muestra la pastilla, y el panel lista los tres contenidos.
  - [ ] Con el ajuste desactivado, no aparecen ni el botón **QR** ni la pastilla, y **Leer códigos QR** sigue funcionando desde el menú contextual.
- **Mejora sobre Snagit:** también en Windows, en el editor y en Extraer texto; apertura de enlaces siempre confirmada.
- **Fuentes:** research/01-captura.md §12.4, §11.2; research/04-biblioteca-compartir.md §7; research/05-plataformas-versiones.md §5, §1B (2025.4.0)

---

## Relacionado

| Prefijo / archivo | Relación con este archivo |
|---|---|
| `SPEC.md` | Principios §2.1 (las funciones de una sola plataforma pasan a ambas) y §2.3 (local primero); alcance §3.2; hitos §4 (M3: Extraer texto; M5: el resto); formato `.xshot` de tipo `guide` §7 (este archivo propone la estructura del objeto `guide` en SMT-028); nombres de la interfaz §8 (Grabador de pasos, Redacción automática, Separar elementos, Extraer texto) |
| `CAP-` (`01-captura.md`) | Modos Extraer texto y Grabador de pasos de la lista Selección (CAP-002; este archivo completa su tabla de opciones en SMT-005 y SMT-022); menú del ícono (CAP-005); pantalla congelada, detección de subáreas y barra de captura (CAP-013, CAP-016, CAP-011; botón **QR** y tecla `Q` en la fase de ajuste, SMT-038, que conviene añadir a CAP-017); Ventanas como objetos (CAP-029, que cubre la pantalla completa por capas); opciones al capturar (CAP-045 a CAP-053); metadatos (CAP-056); presets incorporados Grabador de pasos y Extraer texto (CAP-064); atajos por defecto (CAP-065) |
| `EDT-` (`02-editor.md`) | Menú Más (EDT-003); temas, selector de color (EDT-017, EDT-019); Texto y corrector (EDT-026, EDT-027); Paso (EDT-035); Resaltador, Foco, Desenfocar y Simplificar, que llevan los botones de este archivo (EDT-036, EDT-039, EDT-040, EDT-041); Mover, Selección, relleno del hueco, Varita mágica, Borrador (EDT-042, EDT-043, EDT-044, EDT-045, EDT-046, EDT-048); Recortar (EDT-049, EDT-050); color del lienzo (EDT-074); selección múltiple (EDT-079); aplanar (EDT-091); Medición en vivo (EDT-094, que cubre Live Measure); plantillas y zonas de imagen en que se basa la guía (EDT-095, EDT-098); exportación e impresión (EDT-110, EDT-111, EDT-113, EDT-114) |
| `VID-` (`03-video.md`) | Reproductor y editor de video donde aparece Extraer texto del fotograma (SMT-008); exclusión mutua entre una grabación de video y una de pasos (SMT-022); reducción de ruido y fondo de cámara, que son funciones locales de `VID-` |
| `LIB-`, `TRY-`, `SHR-` (`04-biblioteca-compartir.md`) | Tipo Guías de pasos, insignia, búsqueda y exportación desde la biblioteca (SMT-035); redacción de varias capturas desde la bandeja o la biblioteca (SMT-016); destinos Word, PowerPoint, Pages y Keynote con la guía (SMT-030, SMT-031); destino Archivo con imágenes por separado (SMT-034); Enlace para compartir y su código QR (SMT-037) |
| `APP-` (`05-app.md`) | Ajustes en Preferencias > Captura y Editor (APP-008; este archivo propone el grupo Funciones inteligentes, SMT-002, y los ajustes de SMT-004, SMT-005, SMT-022 y SMT-038); notificaciones (APP-016); atajos (APP-018); permiso de Accesibilidad (APP-024); ventanas elevadas (APP-026); registro de comandos, catálogo y eventos (APP-027, APP-030, APP-031; propuestas: código de error `feature_disabled` y los comandos `smart.steps_pause`, `smart.steps_resume`, `smart.steps_status`, `smart.steps_cancel`, `smart.guide_export`, `smart.qr_create` y `smart.qr_read`); funciones en prueba (APP-034); configuración administrada (APP-035); registros sin contenido (APP-046); recuperación de grabaciones de pasos (APP-047); tiempos de la cruceta y de la captura (APP-048, APP-049); local y privado (APP-054) |
| `PARITY.md` | Usa los IDs SMT- de este archivo |

---

## Excluido

| Elemento de la investigación | Motivo |
|---|---|
| Salidas del Step Capture a Camtasia Editor (pasos en la línea de tiempo) y a Camtasia Online, incluido el envío del texto de las imágenes (research/01 §8.1; research/04 §3.2 Camtasia Editor y Camtasia Online; research/03 "Step Capture → video"; research/05 §1A 2026.3.0 y 2026.4.0, §1B 2026.3.0, §2) | Fuera del alcance (SPEC §3.2: Camtasia). Las salidas abiertas (Word, PowerPoint, Markdown, HTML, PDF, imágenes) cubren la necesidad |
| Pack de cursores para Step Capture que se descarga con cuenta de TechSmith (research/05 §2, fila Step Capture) | Fuera del alcance (cuenta y Assets de TechSmith, SPEC §3.2). El puntero y el resaltado de clic son objetos propios (SMT-026, EDT-093) |
| "Screentelligence" como nombre de marketing de las funciones de IA (research/01 §13; research/05 §2) | Nombre comercial de TechSmith, sin función propia (SPEC §2.4). Sus funciones están en este archivo |
| TechSmith Assets: generador de imágenes con IA en la nube (research/05 §2; research/02 §13) | Fuera del alcance (SPEC §3.2: Assets y funciones generativas). Contradice además SMT-001 |
| Navegación de Assets (sellos, plantillas, temas, fotos) (research/05 §2) | Fuera del alcance (SPEC §3.2); paquetes locales en `EDT-` |
| Interruptor experimental "Improved OCR" / "Optimized OCR" (research/01 §7; research/02 §5.2, §11; research/04 §7; research/05 §1A 2026.0.0, 2026.3.0, §5) | No se ofrece un motor alternativo que el usuario elija. La calidad que ese experimento buscaba es un requisito del motor único (SMT-003) y de la redacción (SMT-011, SMT-017) |
| Nombres de los motores de OCR, detección de caras, segmentación y detección de elementos que usa Snagit (research/01 §7; research/02 §5.1–5.4; research/05 §2) | Detalle de implementación de Snagit, no visible para el usuario (clean room). El spec fija el comportamiento y que todo sea local (SMT-001, SMT-003) |
| Actualizaciones del archivo de licencia del OCR para que Grab Text, Edit Text y Smart Redact sigan funcionando (research/01 §7 Version notes; research/05 §1A 2024.3.7, 2026.1.1) | Corrección de licencias de un componente de terceros de Snagit. XcreenShot no depende de licencias que caduquen (SMT-001) |
| Translate Text (exportar a XLIFF e importar traducciones) (research/02 §5.7; §13 2024) | Retirado por TechSmith (SPEC §3.2) |
| Recogida de datos de uso y botón Send Feedback de los experimentos; ocultar la pestaña si no hay analítica (research/04 §7; research/05 §5) | Fuera del alcance (SPEC §3.2). Mecanismo sin telemetría en APP-034; las funciones de este archivo son definitivas (SMT-002) |
| Imposibilidad de desactivar las funciones de IA principales; solo los experimentos se desactivan en el despliegue (research/01 §13 Policy) | Se sustituye por interruptores por función y bloqueo administrado (SMT-002, APP-035) |
| Corrección de pequeños temblores del cursor que dividían un paso en varios, correcciones de memoria y de fallos del Step Capture (research/01 §8.1; research/05 §1A 2025.3.1, 2025.4.0) | Correcciones de errores de Snagit. Los umbrales de clic y arrastre (SMT-024) y los requisitos de memoria (SMT-027) cubren el comportamiento esperado |
| Diálogo de Grab Text: arreglos de cierre con `Esc` y retirada del botón de minimizar como cambios de versión (research/01 §7; research/05 §1A 2024.3.6) | No se excluye la conducta, que está en SMT-006; solo se omite el historial de versiones |
| Requisito de macOS 10.15 o posterior para Grab Text (research/02 §5.2) | Requisito antiguo de Snagit. Los requisitos del sistema de XcreenShot están en APP-040 |

---

## Cobertura

| Sección de la investigación | Cubierta por |
|---|---|
| research/01-captura.md §7 — Grab Text: modo de captura, opciones incompatibles desactivadas, ventana de resultados, Copy All | SMT-005, SMT-006 |
| research/01 §7 — Texto sin formato, espaciado de párrafos, `Esc`, sin botón de minimizar, corrector ortográfico | SMT-006; corrector: EDT-027 |
| research/01 §7 — Sin abrir el editor; aviso de éxito o fallo | SMT-005 |
| research/01 §7 — Funciones relacionadas del editor (imagen o selección, Edit Text, fotograma de video) | SMT-007, SMT-009, SMT-008 |
| research/01 §7 — Motor de OCR e idiomas; experimento Optimized OCR | SMT-003, SMT-004; interruptor experimental y nombres de motores: Excluido |
| research/01 §7 — Consejos de reconocimiento, primera ejecución más lenta, familias de fuente | SMT-003, SMT-006 (consejos) |
| research/01 §7 — Notas de versión (capturas altas, "Cannot recognize text", archivo de licencia) | SMT-003, SMT-005, SMT-006; licencia: Excluido |
| research/01 §8.1 — Iniciar (preset, Selection = Steps), aviso de inicio | SMT-022; preset: CAP-064 |
| research/01 §8.1 — Barra (contador, pausa, reanudar, detener, arrastrable, ignora sus clics, nunca sale en las imágenes, `Esc`, pausa desde el ícono) | SMT-023; menú del ícono: APP-002 |
| research/01 §8.1 — Qué crea un paso (clic, arrastre, clic derecho, teclado no grabado, temblores) | SMT-024; temblores: Excluido (corrección) |
| research/01 §8.1 — Texto automático, zoom y centrado, cursor y resaltado como objetos, menos de 200×200 px, apps elevadas | SMT-025, SMT-026, SMT-027 |
| research/01 §8.1 — Ajustes respetados (cursor, destinos, efectos como Escalar imagen) | SMT-022 |
| research/01 §8.1 — Límites, memoria, recuperación tras fallo | SMT-027; APP-047 |
| research/01 §8.1 — Salida: plantilla de guía, exportaciones, categoría en la biblioteca | SMT-028, SMT-030 a SMT-035; Camtasia: Excluido |
| research/01 §12.4 — Lectura de QR al capturar, crear QR y QR de capturas compartidas | SMT-038, SMT-036, SMT-037 |
| research/01 §13 — Auto-texto y clic derecho del Step Capture | SMT-024, SMT-025 |
| research/01 §13 — Smart Redact (local; experimento previo; precisión del OCR optimizado) | SMT-010, SMT-011, SMT-017 |
| research/01 §13 — Screentelligence | Excluido |
| research/01 §13 — Política: todo en el dispositivo; IA principal no desactivable | SMT-001, SMT-002; política de Snagit: Excluido |
| research/02-editor.md §3.8 — Resaltador: Aplicar solo al texto (y experimento Smart Highlight) | SMT-021; herramienta base y forma: EDT-036 |
| research/02 §3.11 — Foco: Aplicar solo al texto | SMT-021; herramienta base: EDT-039 |
| research/02 §3.12 — Desenfocar: botón Smart Redact y Aplicar solo al texto | SMT-010, SMT-021; herramienta base y notas de seguridad: EDT-040, SMT-017 |
| research/02 §3.13 — Simplificar: Auto Simplify, Detalle, regenerar descarta ediciones | SMT-019; resto de la herramienta: EDT-041 |
| research/02 §5.1 — Smart Move | SMT-018; botón en Mover: EDT-042 |
| research/02 §5.2 — Grab Text (imagen o selección, resultados, texto sin formato, fotograma de video, párrafos, imágenes altas, corrector) | SMT-006, SMT-007, SMT-008, SMT-003; corrector: EDT-027 |
| research/02 §5.2 — Edit Text | SMT-009 |
| research/02 §5.2 — Motores, idiomas, OCR mejorado, requisito de macOS 10.15 | SMT-003, SMT-004; motores, experimento y requisito: Excluido |
| research/02 §5.3 — Smart Redact: entrada, 9 categorías, tipos, color, esquinas, Done, objetos Blur editables | SMT-010, SMT-011, SMT-013 |
| research/02 §5.3 — Clic para redactar, Redact All Matches, Remove, Edit Text, varias capturas | SMT-014, SMT-015, SMT-016, SMT-009 |
| research/02 §5.3 — Mejoras de detección y de rendimiento; procesamiento local | SMT-011, SMT-017, SMT-001; motores: Excluido |
| research/02 §5.4 — Quitar fondo automático; limpieza con el Borrador; métodos manuales en Windows | SMT-020; métodos manuales: EDT-043, EDT-045, EDT-046, EDT-047 |
| research/02 §5.5 — Auto Simplify | SMT-019 |
| research/02 §5.6 — Aplicar solo al texto | SMT-021 |
| research/02 §5.7 — Translate Text | Excluido |
| research/02 §9 — Plantillas en que se basa la guía (título, pie con autor, tema, numerar, zonas de imagen, zoom, Capturar en marcador vacío, marcadores vacíos omitidos, candado, paginación) | SMT-028, SMT-029, SMT-033; plantillas en general: EDT-095 a EDT-102 |
| research/02 §10.2 — Las guías nuevas necesitan versiones recientes | SMT-028 (formato `guide` versionado, SPEC §7) |
| research/02 §10.4 — Exportación por lotes de imágenes de Step Capture (Mac) | SMT-034; asistente general: EDT-113 |
| research/02 §11 — Experimentos: OCR mejorado, QR, Smart Highlight, Live Measure | SMT-003 (OCR; interruptor: Excluido), SMT-036 a SMT-038, SMT-021, EDT-094 |
| research/02 §13 — Novedades IA/ML 2024–2026 (Smart Redact, Remove Background, apply-only-to-text, click-to-redact, párrafos) | SMT-010 a SMT-021, SMT-006; generador de imágenes con IA: Excluido |
| research/02 §14 — Edición de la guía de Step Capture (anclas, cursor, exportar a Word, PowerPoint, HTML, Markdown) | SMT-029, SMT-030, SMT-032 |
| research/02 §15 (punto 5) — Lista completa de idiomas de OCR | SMT-004 (decisión de diseño) |
| research/03-video.md — Grab Text from a video frame | SMT-008 |
| research/03 — Step Capture → video (Camtasia) | Excluido |
| research/04-biblioteca-compartir.md §1.1 — Step Captures en la biblioteca (categoría, insignia, exportar desde la biblioteca; filtro por tipo) | SMT-035; filtros generales: `LIB-` |
| research/04 §3.2 — File: pasos como imágenes separadas | SMT-034 |
| research/04 §3.2 — Microsoft Word: guía como texto editable más imágenes | SMT-030; destino Word general: `SHR-` |
| research/04 §3.2 — Microsoft PowerPoint: guía como presentación con diapositiva de título | SMT-030; destino PowerPoint general: `SHR-` |
| research/04 §3.2 — Pages / Keynote: guías editables | SMT-031; Numbers y destinos generales de Apple: `SHR-` |
| research/04 §3.2 — Print / PDF (Step Capture) | SMT-033 |
| research/04 §3.2 — Markdown / HTML (Step Capture) | SMT-032 |
| research/04 §3.2 — Camtasia Editor y Camtasia Online con guías | Excluido |
| research/04 §4 — Presets incorporados (Step Capture) | CAP-064, SMT-022 |
| research/04 §7 — Smart Highlight | SMT-021 |
| research/04 §7 — Auto-detectar y desenfocar datos sensibles | SMT-010, SMT-011 |
| research/04 §7 — Pantalla completa con cada ventana como capa editable | CAP-029 |
| research/04 §7 — QR (crear, compartir, leer al capturar) | SMT-036, SMT-037, SMT-038 |
| research/04 §7 — Live Measure | EDT-094 |
| research/04 §7 — Improved OCR y Optimized OCR | SMT-003, SMT-017; interruptor: Excluido |
| research/04 §7 — Otros experimentos (Screen Draw, 60 fps, teclas en video, grabación de una ventana, esquinas del lienzo, HDR, Quick Image Annotation) | `VID-`, EDT-075, CAP-022, CAP-057 |
| research/04 §7 — Programa Experiments (telemetría, Send Feedback) | APP-034; telemetría: Excluido |
| research/05-plataformas-versiones.md §2 — Smart Redact | SMT-010 a SMT-017 |
| research/05 §2 — Grab Text / Edit Text | SMT-003 a SMT-009 |
| research/05 §2 — Smart Move | SMT-018 |
| research/05 §2 — Simplify / Auto Simplify | SMT-019; EDT-041 |
| research/05 §2 — Remove Background | SMT-020 |
| research/05 §2 — Step Capture (texto automático; sin conexión; elevadas; pack de cursores) | SMT-022 a SMT-027, SMT-001; pack de cursores: Excluido |
| research/05 §2 — Apply only to text | SMT-021 |
| research/05 §2 — Reducción de ruido; desenfoque y fondo virtual de cámara | `VID-` |
| research/05 §2 — HDR | CAP-022 |
| research/05 §2 — Generador de imágenes con IA; navegación de Assets | Excluido |
| research/05 §2 — Screencast / Share Link; destinos en la nube | `SHR-`; Screencast: Excluido (SPEC §3.2) |
| research/05 §2 — Camtasia Online; traspaso a Camtasia Editor | Excluido |
| research/05 §2 — Licencias; experimentos y analítica | Excluido (SPEC §3.2); APP-034, APP-054 |
| research/05 §2 — Declaración de IA local, no generativa, sin entrenamiento, sin internet | SMT-001 |
| research/05 §3 — Step Capture: salidas en ambas plataformas (Word, PowerPoint) | SMT-030 |
| research/05 §3 — Step Capture: salidas solo de Mac (Pages, Keynote, Markdown, HTML, imágenes por lotes) | SMT-031 (solo macOS, con motivo), SMT-032, SMT-034 (ambas) |
| research/05 §3 — Step Capture: efecto Escalar imagen (Windows) | SMT-022, SMT-029 |
| research/05 §3 — Step Capture: Capturar desde una zona vacía; candado antes de editar (Mac) | SMT-029 |
| research/05 §3 — Step Capture: número recomendado de pasos | SMT-027 |
| research/05 §3 — Editor: Remove Background automático (Mac) | SMT-020 |
| research/05 §3 — Editor: Aplicar solo al texto (Mac); forma del Resaltador | SMT-021; forma: EDT-036 |
| research/05 §3 — Editor: clic para redactar y Redact All Matches (Mac) | SMT-014, SMT-015 |
| research/05 §3 — Editor: Grab Text de un fotograma de video (Mac) | SMT-008 |
| research/05 §3 — Editor: Live Measure (Mac) | EDT-094 |
| research/05 §3 — Editor: activación de Smart Move (interruptor o botón) | SMT-018 |
| research/05 §3 — Editor: copia de texto sin formato desde Grab Text | SMT-006 |
| research/05 §3 — Editor: Lock Objects de Simplify (Windows) | EDT-041 |
| research/05 §3 — Editor: resto de filas (degradados, esquinas, relleno inteligente, imprimir selección, curvas y globos, Assets, fijar en la bandeja, seleccionar todos) | `EDT-`, `TRY-`; Assets: Excluido |
| research/05 §5 — Improved / Optimized OCR | SMT-003, SMT-017; interruptor: Excluido |
| research/05 §5 — QR | SMT-036, SMT-037, SMT-038 |
| research/05 §5 — Pantalla completa con cada ventana como capa | CAP-029 |
| research/05 §5 — Quick Image Annotation; HDR Capture Fidelity; Screen Draw en cualquier momento; teclas en video | CAP-057, CAP-022, `VID-` |
| research/05 §5 — Experimentos que pasaron a definitivos (Smart Redact, Apply only to text, esquinas, Pen, ventana que se mueve, Live Measure) | SMT-010, SMT-021, EDT-075, `VID-`, CAP-027, EDT-094 |
| research/05 §5 — Funcionamiento de la pestaña Experiments (envío de comentarios, analítica) | APP-034; telemetría: Excluido |
