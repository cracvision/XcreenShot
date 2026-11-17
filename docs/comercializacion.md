# Comercializar XcreenShot: análisis y plan por etapas

**Fecha:** 2026-11-14 · **Estado:** propuesta para Carlos (nada de esto cambia el spec hasta que él lo apruebe) · **Origen:** decisión de Carlos del 2026-11-14 de vender XcreenShot (ver `docs/ESTADO.md`).

Este documento responde a una pregunta: **¿qué hace falta para vender XcreenShot en las tiendas de Microsoft, Apple y Google, y en qué orden?** Los requisitos de las tiendas se comprobaron el 2026-11-14 en la documentación oficial de Microsoft y Apple (lista al final, §10). Lo que no se pudo comprobar está marcado como **(suposición)** o **(sin verificar)**. Los temas legales son preguntas para un abogado: aquí se explica por qué importan, no se dan respuestas legales.

---

## 1. Resumen

1. **La tienda que encaja es la Microsoft Store.** Admite apps de escritorio como XcreenShot, con su comando de terminal, sus atajos globales y sus ganchos de teclado y ratón. El registro de desarrollador es gratis.
2. **En macOS, lo razonable es vender fuera de la tienda**, con la app firmada (Developer ID) y notarizada por Apple. El spec ya lo decidía así (APP-039). La Mac App Store exige el *App Sandbox*, que deja fuera el CLI y el MCP, el actualizador, las claves de licencia y probablemente el Grabador de pasos y el desplazamiento automático. Una edición recortada para esa tienda sería un producto aparte, y solo vale la pena si los números lo justifican.
3. **Google Play no aplica:** es una tienda de Android y XcreenShot es de escritorio. La App Store de iPhone y iPad tampoco: allí una app no puede capturar ni grabar la pantalla de otras apps como lo hace XcreenShot. Propuesta: dejarlas fuera.
4. **Para vender en la Microsoft Store recomiendo el paquete MSIX:** Microsoft lo firma (no hace falta comprar un certificado), lo actualiza la tienda y la tienda cobra y gestiona la prueba gratuita, a cambio del **15 %** de cada venta. La alternativa (subir el instalador `.exe` firmado) exige certificado y montar el cobro y las licencias por nuestra cuenta.
5. **Antes de vender hace falta un abogado** para cinco temas: el objetivo de paridad con el producto de referencia y el método de sala limpia (incluidas las **patentes**, que la sala limpia no cubre), la marca "XcreenShot" (parece la palabra genérica *screenshot*, lo que puede impedir registrarla), las licencias de las dependencias (hay 5 componentes con licencia MPL-2.0 y una fuente con OFL), la IA en la nube y la autoría del código escrito con IA.
6. **El producto cambia:** necesita licencia y prueba gratuita (hoy el spec lo deja fuera), política de privacidad, términos de licencia, aviso de componentes de terceros, soporte y una decisión explícita sobre los informes de errores. La regla "sin telemetría" **se puede mantener**: es un buen argumento de venta.
7. **El plan va por etapas** (§7): primero lo legal y las cuentas, sin tocar código; después terminar Windows como ya está decidido; luego preparar la versión vendible y publicarla en la Microsoft Store; la venta directa y macOS vienen después.

---

## 2. Qué tiendas aplican de verdad

| Tienda | ¿Aplica? | Por qué | Propuesta |
|---|---|---|---|
| **Microsoft Store** (Windows) | **Sí** | Acepta apps de escritorio de Windows, empaquetadas (MSIX) o con su instalador (EXE/MSI) | Primera tienda (§4) |
| **Mac App Store** (macOS) | Solo con una edición recortada | Exige el *App Sandbox* y otras reglas que chocan con funciones centrales (§3) | Vender fuera de la tienda con Developer ID y notarización; la tienda, más adelante y solo si compensa |
| **App Store de iPhone/iPad** | No | Una app de iOS no puede capturar ni grabar la pantalla de otras apps como lo hace XcreenShot, ni tiene atajos globales **(suposición, no verificada en esta sesión)** | Fuera |
| **Google Play** | No | Es la tienda de Android (y de juegos en PC). XcreenShot no tiene versión de Android y el spec no la contempla | Fuera. Si algún día se quiere un acompañante móvil (ver la biblioteca, recibir capturas), sería una idea nueva en `IDEAS.md` con su propio análisis |

**Otros canales que sí encajan** (sin comisión de tienda):
- **Venta directa** desde una web propia: instalador firmado, cobro con un proveedor de pagos y una clave de licencia (§5).
- **winget**, el gestor de paquetes de Windows: permite instalar XcreenShot con un comando y es habitual en empresas **(suposición: no se revisaron sus requisitos)**.

Si "Google" se refería a otra cosa (la Chrome Web Store o Google Workspace Marketplace), tampoco encaja: el spec excluye la extensión de navegador (`SPEC.md` §3.2).

---

## 3. macOS: Mac App Store frente a venta directa

### 3.1 Qué exige la Mac App Store

La guía de revisión de Apple (punto **2.4.5**) pone requisitos propios a las apps de la Mac App Store. Los que afectan a XcreenShot:

| Requisito de Apple (2.4.5) | Qué choca en XcreenShot |
|---|---|
| (i) La app tiene que ir dentro del *App Sandbox* | Casi todo lo de la tabla siguiente |
| (ii) Un único paquete autocontenido; no instalar código ni recursos en ubicaciones compartidas; sin instaladores de terceros | Instalar el comando `xcreenshot` en el PATH (APP-028, APP-009) |
| (iii) No arrancar al iniciar sesión ni dejar procesos en marcha tras salir **sin consentimiento** | Iniciar al iniciar sesión (APP-003) tiene que pedir permiso; no puede venir activado sin más |
| (vi) No pedir claves de licencia ni tener protección anticopia propia | La licencia la gestiona la tienda (también lo dice el punto 3.1.1) |
| (vii) Las actualizaciones solo por la Mac App Store | El actualizador propio (APP-036) no puede estar en esa edición |

Y para todas las apps de las tiendas de Apple, el punto **2.5.14** pide consentimiento explícito y un indicador visible o sonoro al grabar la pantalla o registrar la actividad del usuario. XcreenShot ya lo cumple por diseño: indicadores de grabación que no se pueden ocultar (APP-032) y aviso al empezar el Grabador de pasos (SMT-022).

### 3.2 El *App Sandbox*, función por función

| Función de XcreenShot | Dentro del sandbox | Grado de certeza |
|---|---|---|
| Capturar y grabar la pantalla | Posible, con el permiso de Grabación de pantalla (APP-024) | **(Suposición)**: no se pudo leer la página de ScreenCaptureKit |
| Biblioteca en `Documentos\XcreenShot` | Posible, pero el usuario tiene que elegir la carpeta una vez y la app guardar ese permiso (marcador de seguridad). El sandbox solo da acceso directo a Descargas, Música, Películas e Imágenes, **no a Documentos** | Verificado (documentación de entitlements de Apple, archivada pero vigente) |
| Guardar en cualquier carpeta (destino Archivo, carpeta de recepción, biblioteca en red) | Posible si el usuario elige cada carpeta en un diálogo; la app guarda el permiso | Verificado |
| **CLI `xcreenshot` desde el Terminal y servidor MCP** que lanzan Claude Desktop o Claude Code | **Prácticamente fuera.** Una herramienta de línea de comandos dentro de una app del sandbox tiene que ir también en el sandbox y está pensada para que **la lance la propia app**, no el Terminal ni otra app. No podría escribir en `--out ~/a.png` ni instalarse en el PATH | Verificado que debe ir en el sandbox y heredar el de la app; lo de lanzarla desde el Terminal es **(suposición)** basada en esa documentación |
| Atajos globales | Posibles | **(Suposición)** |
| Teclas pulsadas en el video (VID-047), clics del Grabador de pasos | Probablemente posibles con el permiso de Monitorización de entrada | **(Sin verificar):** Apple no lo documenta de forma clara; hay que probarlo con un prototipo |
| Leer los nombres de los botones de otras apps (texto automático de cada paso, SMT-025), desplazamiento automático (CAP-034 a CAP-036), captura de menús (CAP-031) | Probablemente **fuera**: usan la API de Accesibilidad para leer y controlar otras apps | **(Sin verificar):** no hay una página oficial clara; prototipo o consulta al soporte técnico de Apple |
| Dibujo en vivo (VID-048) | Su ventana sobre la pantalla, posible; depende del mismo punto de la monitorización de entrada | **(Sin verificar)** |
| Actualizaciones propias (APP-036) | Fuera (2.4.5 vii) | Verificado |
| Claves de licencia propias | Fuera (2.4.5 vi y 3.1.1) | Verificado |

### 3.3 Venta directa con Developer ID y notarización

Es lo que ya decía el spec (APP-039): un `.dmg` firmado con un certificado **Developer ID** y **notarizado** por Apple. Apple revisa automáticamente el software y Gatekeeper lo abre sin avisos. Requisitos verificados:
- Membresía del **Apple Developer Program: 99 USD al año** (la misma que para la tienda).
- Toda app distribuida con Developer ID tiene que estar notarizada, y para notarizar hay que activar el *Hardened Runtime* en la app **y en sus herramientas de línea de comandos**.
- Sin sandbox obligatorio: XcreenShot mantiene el CLI, el MCP, el actualizador propio y todas las funciones, con los permisos normales de macOS (APP-024).
- El cobro y las licencias van por cuenta propia, como en la venta directa de Windows (§5.3).

### 3.4 Propuesta para macOS

1. Mantener APP-039: **venta directa con Developer ID y notarización**, la misma licencia y el mismo cobro que en la venta directa de Windows.
2. No hacer la edición de la Mac App Store en v1. Si más adelante se quiere, se plantea como **edición reducida** (sin CLI ni MCP, sin Grabador de pasos automático ni desplazamiento automático si el prototipo lo confirma), con su propio análisis de costos.
3. Recordatorio: **macOS aún no se ha probado.** Antes de vender allí hay que pasar la segunda ronda de pruebas ya decidida (Compartir, actualizaciones y macOS).

---

## 4. Microsoft Store

### 4.1 Dos formas de publicar

La Microsoft Store acepta dos tipos de envío. Tauri, el marco con el que está hecha la app, **solo genera instaladores EXE y MSI**; el paquete MSIX necesitaría un paso de empaquetado adicional.

| | **MSIX** (paquete de la tienda) | **EXE/MSI** (nuestro instalador, enlazado desde la tienda) |
|---|---|---|
| Firma | **Microsoft firma el paquete** al publicarlo. No hace falta comprar certificado | **Hace falta un certificado de firma de código** de una autoridad del programa de raíces de confianza de Microsoft. La tienda no vuelve a firmar |
| Instalador | Lo instala Windows, por usuario, sin permisos de administrador | Instalador sin conexión (no un descargador), con instalación silenciosa, alojado en una dirección HTTPS con la versión en la ruta, que no puede cambiar después del envío |
| Actualizaciones | Las entrega la tienda. El actualizador propio (APP-036) se apaga en esta edición | Las hacemos nosotros: cada versión es una dirección nueva y un envío nuevo. El actualizador propio sigue funcionando |
| Cobro | Precio, prueba gratuita (1, 7, 15 o 30 días, o sin límite) y licencia, todo por la tienda. **Comisión: 15 %** | La página de envío permite elegir "de pago", pero **no explica cómo cobra Microsoft en ese caso**: hay que confirmarlo en Partner Center. Con cobro propio dentro de la app, la comisión es **0 %** |
| Avisos de SmartScreen | Ninguno (lo firma Microsoft) | Un instalador nuevo, aunque esté firmado, puede mostrar el aviso de "app no reconocida" hasta que acumula reputación (semanas y cientos de instalaciones). Un certificado EV **ya no** evita este aviso |
| Trabajo extra | Empaquetar en MSIX y adaptar la app (§4.2) | Firmar, alojar el instalador y montar cobro y licencias |

**Recomendación:** MSIX para la tienda. Es lo que menos infraestructura le exige a Carlos (sin certificado, sin servidor de licencias, sin cobro propio) y Microsoft se queda con un 15 %. Antes, una **prueba técnica corta** (unos días de trabajo, **suposición**) para confirmar que todo funciona empaquetado. Si fallara algo que no tiene arreglo, se publica el EXE firmado.

### 4.2 Qué cambia en la app con MSIX

| Tema | Con MSIX | Qué hay que hacer |
|---|---|---|
| Comando `xcreenshot` en la terminal (APP-028) | Se ofrece con un **alias de ejecución** declarado en el paquete. El usuario lo puede apagar en Configuración | Declarar el alias; probar que el CLI encuentra la app en marcha (tubería con nombre) |
| Servidor MCP (APP-029, SMT-043) | El bloque de configuración no puede apuntar a la carpeta interna del paquete (`WindowsApps`) | Que **Copiar la configuración** use la ruta del alias. **(Suposición:** Microsoft no dice en la página consultada si otros procesos pueden ejecutar el `.exe` directamente desde `WindowsApps`) |
| Iniciar al iniciar sesión (APP-003) | Se declara una tarea de inicio; aparece en el Administrador de tareas y el usuario la puede apagar allí | Cambiar el complemento actual por la tarea de inicio del paquete |
| Datos en `%APPDATA%` y `%LOCALAPPDATA%` | En las apps "virtualizadas", los archivos nuevos se guardan en una carpeta privada del paquete y se borran al desinstalar. Modificar archivos que ya existían está permitido | Comprobar en la prueba técnica dónde quedan los ajustes y qué pasa con IDEA-004 (escribir la configuración de Claude Desktop) |
| Asociación de archivos (`.xshot`, `.xpreset`…) | Se declaran en el paquete | Pasarlas del instalador al manifiesto |
| Ganchos de teclado y ratón (Dibujo en vivo, teclas en el video, Grabador de pasos) | Una app de escritorio empaquetada con "confianza total" funciona como ahora **(suposición:** no hay una página que lo diga para este caso; entra en la prueba técnica) | Probar |
| Motor web (WebView2) | Windows 11 lo trae instalado **(suposición)**. Para el instalador EXE de la tienda, Tauri pide incluir el instalador completo de WebView2 (sin conexión) | Comprobar en Windows 10 22H2 |
| Licencia y prueba gratuita | Las da la tienda; la app las consulta con la API de la tienda | Nuevo: comprobar la licencia al arrancar y mostrar el estado de la prueba |
| Actualizaciones (APP-036) | Las da la tienda | La edición de la tienda no busca actualizaciones propias; Acerca de dice "Actualizada por Microsoft Store" |

### 4.3 Requisitos de la tienda (verificados)

- **Cuenta de desarrollador:** gratis, como persona o como empresa. La persona se verifica con documento de identidad y una foto; la empresa, con su número DUNS o documentos oficiales y un correo de su dominio (de 2 a 5 días). **Una cuenta personal no se puede convertir después en cuenta de empresa:** conviene decidir antes (§6, decisión 2).
- **Política de privacidad (política 10.5.1):** obligatoria para las apps Win32 y de escritorio empaquetadas, aunque no recojan datos. Tiene que estar en una dirección pública. Debe explicar, como mínimo: qué queda en el equipo, qué sale (actualizaciones, compartir, página web por URL y la IA en la nube solo si el usuario la activa) y que no hay telemetría.
- **Clasificación por edades:** cuestionario de la IARC en el primer envío. Se repite si una actualización cambia el contenido. **(Suposición:** XcreenShot debería quedar en una clasificación baja, pero depende de cómo se respondan las preguntas sobre la IA y sobre la captura de cualquier página web, CAP-037).
- **IA generativa (política 11.16):** si la app genera contenido con IA en respuesta al usuario (Preguntar a la IA, SMT-040), hay que **decirlo en la ficha** y **marcarlo en Partner Center** al enviar.
- **Nombres y marcas de otros (10.1.1 y 11.2):** la ficha no puede usar nombres, imágenes ni metadatos de otros productos. Nada de "alternativa a…" con la marca del producto de referencia sin el visto bueno del abogado.
- **Cambios en el sistema (10.2.8):** cambiar ajustes de Windows del usuario exige métodos admitidos y su consentimiento. **Desinstalación limpia** (10.2.7). **No instalar software de terceros** que no aporte a la app (10.2.3).
- **Teclado y grabación de pantalla:** las políticas de Microsoft no tienen una regla específica sobre capturar teclas o grabar la pantalla (no se encontró ninguna). Aplican las reglas generales de datos personales (10.5).
- **Informes de fallos en Partner Center:** para los paquetes MSIX, Microsoft muestra análisis de fallos si se sube el archivo de símbolos (`.appxsym`). Para EXE/MSI no se encontró nada.

---

## 5. Modelo de negocio y cambios en el producto

### 5.1 Comisiones y costos fijos

| Canal | Comisión | Costos fijos | Fuente |
|---|---|---|---|
| Microsoft Store con el cobro de Microsoft | **15 %** (12 % en juegos) | Cuenta gratis | Verificado |
| Microsoft Store con cobro propio (apps que no son juegos) | **0 %** a Microsoft; lo que cobre el proveedor de pagos | Cuenta gratis | Verificado |
| Mac App Store | **15 %** con el programa para pequeñas empresas (hasta 1 millón USD de ingresos el año anterior); 30 % la tarifa estándar | 99 USD/año | El 15 % y el umbral, verificados en la página del programa; el 30 %, en informes de Apple, no en la página del programa |
| Venta directa (Windows y macOS) | Lo que cobre el proveedor de pagos. Los que actúan como "comerciante registrado" (cobran, emiten la factura y declaran el IVA o el impuesto de venta de cada país) suelen cobrar alrededor de un 5 % más unos céntimos por venta **(sin verificar; orientativo)** | Certificado de firma para Windows (§5.3) y la membresía de Apple para macOS | — |

Ejemplo con un precio de 40 USD (sin impuestos, **orientativo**): por la Microsoft Store con su cobro llegan unos 34 USD; por venta directa con un comerciante registrado, unos 37,50 USD. La tienda aporta visibilidad, confianza y la gestión del cobro; la venta directa, margen y relación directa con el cliente.

### 5.2 Precio y tipo de licencia

Opciones (sin estudio de mercado: es una propuesta para discutir):

| Opción | Cómo funciona | A favor | En contra |
|---|---|---|---|
| **A. Pago único por versión mayor** (recomendada para empezar) | Se paga una vez la versión 1.x, con todas sus actualizaciones. La 2.0, con descuento para quien ya tiene la 1 | Fácil de explicar; encaja con "sin cuenta y sin conexión"; la tienda lo gestiona como app de pago | Ingresos irregulares |
| B. Suscripción anual | Pago cada año mientras se use | Ingresos estables | Más resistencia del comprador; obliga a comprobar la licencia periódicamente |
| C. Gratis con versión Pro | Lo básico gratis; video, funciones inteligentes, CLI/MCP y plantillas, de pago | Más usuarios | Hay que decidir qué va en cada versión y mantener dos experiencias |

En todas: **prueba gratuita** (en los envíos EXE/MSI, la Microsoft Store la pide para las apps de pago y de suscripción; en MSIX es opcional) y **licencias para empresas** (varios puestos y la configuración administrada de APP-035, que es un buen argumento para ellas).

### 5.3 Qué cambia en el producto

| Tema | Hoy (spec) | Qué cambia al vender | Propuesta |
|---|---|---|---|
| **Licencia y prueba** | Fuera del alcance (`SPEC.md` §3.2; "Excluido" de `05-app.md`) | Hace falta | En la tienda, la licencia de la tienda. En venta directa, un **archivo de licencia firmado que la app comprueba sin conexión**, sin servidor de activación y sin enviar nada. Así se mantiene "sin telemetría" y "funciona sin conexión" |
| **Firma de código** | D-06 (pendiente) | Obligatoria para el EXE (y para la venta directa) | **Artifact Signing** de Microsoft (antes Trusted Signing): desde 9,99 USD/mes. Para personas, solo en EE. UU. y Canadá; para empresas, en EE. UU., Canadá, la UE, el Reino Unido y otros países. Si no se cumple, un certificado OV de una autoridad (precio sin verificar). En la tienda con MSIX no hace falta |
| **Actualizaciones** (APP-036) | Actualizador propio, firmado | Depende del canal | Tienda: las da la tienda. Venta directa: el actualizador de Tauri, que exige su propia firma; **perder esa clave impide publicar más actualizaciones**, así que se guarda en el llavero de Carlos y con copia de seguridad |
| **Informes de errores** | Sin telemetría; "Copiar información del fallo" (APP-046, APP-047, APP-054) | Con muchos usuarios, hace falta saber qué falla | Ver §5.4 |
| **Soporte** | Enviar comentarios abre una incidencia o un correo (APP-015, destino pendiente) | Clientes que pagan esperan respuesta | Un correo de soporte y una página de preguntas frecuentes. Enviar comentarios abre ese correo con la versión y el sistema rellenados. La ayuda local ya existe |
| **Idiomas** | Inglés y español (D-07) | Más mercados, más idiomas | Lanzar en inglés y español (app y ficha). Añadir portugués, francés y alemán según la demanda; el sistema de traducciones ya lo permite. Ojo: Extraer texto solo reconoce los idiomas instalados en Windows (SMT-004) |
| **Textos legales** | "Sin EULA" (Excluido de `05-app.md`) | Hacen falta | Términos de licencia (EULA), política de privacidad y aviso de componentes de terceros (Acerca de ya lo pide, APP-015), redactados o revisados por el abogado |
| **Web** | No hay | Las tiendas piden la dirección de la política de privacidad y de soporte | Una página sencilla con descripción, precios, privacidad, soporte y descargas |
| **Nombre del perfil de atajos** | "Compatibilidad" (APP-021) | Igual | No usar la marca del producto de referencia en la app ni en la ficha |
| **Objetivo del spec** | "Paridad funcional para Carlos y sus colegas" (`SPEC.md` §1) | Un producto para clientes | Reescribir el objetivo (§8) |

### 5.4 Informes de errores sin romper "sin telemetría"

| Opción | Cómo funciona | Cambia el spec |
|---|---|---|
| **A. Como hoy** | El usuario copia la información del fallo y la manda por correo si quiere | No |
| **B. Los informes de la tienda** | Windows recoge los fallos según los ajustes de diagnóstico del propio usuario y Microsoft se los muestra al desarrollador en Partner Center. XcreenShot no envía nada | No (la app no conecta con nada) |
| C. Envío opcional desde la app | Tras un fallo, la app pregunta "¿Enviar este informe?", enseña exactamente qué se envía y solo entonces lo manda a un servidor propio | Sí: APP-054 y la política de privacidad; además, un servidor |

**Propuesta:** A + B al lanzar. C solo si hace falta, y siempre con consentimiento en cada envío.

---

## 6. Temas para el abogado

Preguntas concretas, para llevar a la reunión. **Lo que sigue no es asesoramiento legal.**

### 6.1 Paridad con el producto de referencia y sala limpia
XcreenShot se diseña para igualar las funciones de Snagit (de TechSmith) a partir de un spec escrito con documentación pública, sin mirar su código ni su instalación (`CLAUDE.md`, Regla 1).
- **Derechos de autor:** la sala limpia protege contra copiar código, textos, íconos y diseño. ¿Es suficiente nuestra documentación (`spec/`, `spec/sources.md`, `spec/research/`) como prueba? ¿Debe conservarse así?
- **Separación de equipos:** en una sala limpia clásica, quien estudia el producto y quien construye son personas distintas. Aquí Carlos escribe y aprueba el spec y además usa el producto de referencia a diario; el código lo escribe una IA. ¿Debilita eso la defensa?
- **Patentes: la sala limpia no protege contra ellas.** Si el producto de referencia tiene patentes vigentes sobre alguna función (captura con desplazamiento, grabador de pasos, redacción automática…), implementarla de forma independiente no basta. ¿Hace falta un estudio de libertad de operación sobre las funciones principales?
- **Aspecto y presentación:** el aspecto Cristal es propio; ¿hay riesgo por parecido de la interfaz o de los flujos?
- **Publicidad comparativa:** ¿se puede decir "alternativa a…" o comparar funciones en la web? En las tiendas, Apple (2.3.7 y 4.1 c) y Microsoft (10.1.1) prohíben usar marcas ajenas en el nombre, los metadatos o las palabras clave.

### 6.2 La marca "XcreenShot"
- Suena exactamente como *screenshot*, la palabra genérica del producto. Las oficinas de marcas suelen rechazar como descriptivas las variantes ortográficas de palabras genéricas. **Riesgo:** no poder registrarla, o poder registrarla pero no defenderla.
- Pedir una búsqueda de anterioridad en las oficinas de los países donde se venderá (por ejemplo, EE. UU., la UE y el país de Carlos), en las tiendas y en los dominios. **No la hice:** buscar el nombre en servicios externos es enviar datos del proyecto fuera, y eso lo decide Carlos.
- Si hay que cambiar el nombre, mejor antes de publicar. El identificador interno (`com.cracvision.xcreenshot`) puede quedarse; el nombre visible se cambia en los textos, el ícono y la ficha.

### 6.3 Licencias de las dependencias
Inventario hecho el 2026-11-14 sobre lo que va dentro de la app (sin herramientas de desarrollo):

| Parte | Paquetes | Licencias |
|---|---|---|
| Rust, Windows | 386 | Casi todos MIT, Apache-2.0 o BSD (permisivas). Además: Unicode-3.0, Zlib, BSL-1.0, CC0, 0BSD (permisivas) |
| Rust, macOS | 393 | Igual |
| Interfaz (npm) | 22 | MIT y Apache-2.0, más la fuente **Outfit (OFL-1.1)** |

Para revisar:
- **5 componentes con MPL-2.0** (`cssparser`, `cssparser-macros`, `dtoa-short`, `selectors`, que vienen con Tauri, y `option-ext`). La MPL exige ofrecer el código fuente **de esos archivos** (no de toda la app) si se modifican, e incluir su aviso. No los modificamos **(suposición: confirmarlo con el abogado)**.
- **`self_cell`:** licencia doble "Apache-2.0 o GPL-2.0"; se usa bajo Apache-2.0.
- **Fuente Outfit (OFL-1.1):** se puede incluir en una app de pago, pero no venderse sola; hay que incluir su licencia.
- **Componentes del sistema:** el motor web (WebView2), el reconocimiento de texto, el detector de caras y los codificadores de video H.264 y AAC son de Windows. **(Suposición:** las licencias de patentes de H.264 y AAC las cubre el sistema operativo; preguntarlo.) El spec menciona H.265: en Windows, ese códec es una extensión de pago de la tienda.
- **Sin FFmpeg** (D-04): bien, evita sus licencias.
- **Pendiente técnico:** generar en cada compilación el aviso de componentes de terceros (lista y textos de las licencias) para Acerca de y la carpeta de instalación, con una prueba que falle si aparece una licencia nueva no aprobada.

### 6.4 La IA en la nube con la clave de cada usuario (SMT-039 a SMT-043)
- Cada usuario contrata con Anthropic y usa su clave; XcreenShot no intermedia ni cobra. ¿Hay algo en los términos de Anthropic que limite ofrecer esta función en una app de pago? ¿Necesita la app sus propios términos sobre el uso de la IA?
- **Nombre "Claude":** la app dice "Claude, de Anthropic" para explicar qué servicio usa. ¿Es un uso descriptivo aceptable o hace falta permiso?
- **Privacidad:** las capturas pueden contener datos personales de terceros. La función ya pide activarla con un aviso y solo envía cuando el usuario lo pide (SMT-042). Apple exige explicar con claridad cuándo se comparten datos con una IA de terceros y pedir permiso antes (5.1.2 i, **verificado solo a nivel de resumen**); Microsoft exige declarar la IA generativa en la ficha (11.16). ¿Basta con lo que hay?
- **Ley de IA de la UE:** si se vende en la UE, ¿le aplican las obligaciones de transparencia del artículo 50 (contenido generado por IA), y desde cuándo? **(Sin verificar)**
- **Responsabilidad** por lo que responde la IA (por ejemplo, un texto insertado en una imagen).

### 6.5 Código escrito con IA
- Casi todo el código lo escribe Claude Code. ¿Quién es el titular de ese código y cómo se protege? En algunos países (por ejemplo, EE. UU.) el contenido generado por IA sin suficiente aportación humana puede no tener derechos de autor **(sin verificar en esta sesión)**. Revisar también los términos comerciales de Anthropic sobre la titularidad de los resultados.
- El repositorio es privado (D-01). Para vender, la protección práctica es mantenerlo privado y que la licencia de uso (EULA) lo diga.

### 6.6 Empresa, impuestos y consumo
- ¿Vender como persona o con una empresa? Afecta a la responsabilidad, a la cuenta de Microsoft (no se puede pasar de personal a empresa) y a la firma de código (para personas, Artifact Signing solo existe en EE. UU. y Canadá).
- Impuestos de las ventas en las tiendas y en la venta directa (un contador).
- Términos de licencia (EULA), garantía, limitación de responsabilidad, reembolsos y, en la UE, el derecho de desistimiento en contenidos digitales.
- Política de privacidad (RGPD y leyes de privacidad de EE. UU.), aunque la app no recoja datos: la web, el correo de soporte y los pagos sí tratan datos.

---

## 7. Plan por etapas

Cada etapa termina con algo que Carlos aprueba antes de pasar a la siguiente. La etapa 0 no toca código, así que no interfiere con la sesión que está programando.

| Etapa | Qué | Quién | Termina cuando |
|---|---|---|---|
| **0. Decidir y preparar** (ya) | Las decisiones de §8; reunión con el abogado (§6); búsqueda de la marca; decidir persona o empresa; abrir la cuenta de Partner Center (gratis); reservar el dominio | Carlos y el abogado; Claude prepara los textos y las preguntas | Carlos aprueba el modelo de negocio y la marca está despejada (o hay nombre nuevo) |
| **1. Terminar Windows** (en marcha) | Lo que ya está decidido: M4, M5 y los flecos, y la **primera prueba de extremo a extremo** en Windows | Sesión de desarrollo y Carlos | Pasa la prueba de extremo a extremo |
| **2. Versión vendible para Windows** | Prueba técnica de MSIX (§4.2); licencia y prueba gratuita; aviso de componentes de terceros automático; política de privacidad, EULA y textos de la IA; soporte y Enviar comentarios; ficha de la tienda en inglés y español con capturas; clasificación por edades | Desarrollo, con los textos legales del abogado | Un paquete MSIX que pasa la certificación de la tienda en una publicación privada **(suposición: la tienda permite publicar solo para un grupo de prueba; confirmarlo en Partner Center)** |
| **3. Lanzamiento en la Microsoft Store** | Publicación; vigilar los informes de fallos de Partner Center y el correo de soporte | Carlos | Primeras ventas y primeras respuestas a clientes |
| **4. Venta directa en Windows** (opcional) | Instalador firmado (Artifact Signing u OV), actualizador APP-036 con su clave, cobro con un comerciante registrado y archivo de licencia; winget | Desarrollo y Carlos | Para empresas y para quien no usa la tienda |
| **5. macOS** | Segunda ronda ya decidida (Compartir, actualizaciones y macOS); membresía de Apple (99 USD/año); `.dmg` con Developer ID y notarización; la misma licencia que en la etapa 4 | Desarrollo y Carlos, con un Mac | Pasa las pruebas en Mac |
| 6. Mac App Store (opcional, sin fecha) | Edición reducida, solo si los números lo justifican | — | Decisión aparte |

**Lo que no entra:** Google Play, App Store de iPhone/iPad, servidor propio de IA (la IA sigue con la clave de cada usuario) y telemetría.

**Riesgos principales:**
- **Patentes y marca**: pueden obligar a cambiar funciones o el nombre. Por eso van primero.
- **MSIX**: si la prueba técnica falla, se publica el EXE firmado (más trabajo de cobro y licencias).
- **Alcance**: vender añade trabajo que no es de paridad (licencias, empaquetado, soporte, web). Retrasa el gate de paridad de M6 si se hace antes.
- **macOS sin probar**: no conviene anunciar macOS hasta pasar la etapa 5.

---

## 8. Decisiones para Carlos

Cada una con mi recomendación. Si las apruebas, las anoto en `spec/QUESTIONS.md` como propuestas de cambio del spec (no cambio el spec directamente).

1. **Tiendas:** Microsoft Store sí; Mac App Store no en v1 (venta directa en Mac); Google Play y la App Store de iPhone/iPad, fuera. *Recomendado.*
2. **Persona o empresa:** decidirlo con el abogado y el contador **antes** de abrir la cuenta de Microsoft. Dime además en qué país facturarías: cambia la firma de código disponible.
3. **Paquete de la Microsoft Store:** MSIX, con una prueba técnica previa; si falla, EXE firmado. *Recomendado.*
4. **Modelo de licencia:** pago único por versión mayor con prueba gratuita (opción A de §5.2). El precio, tras mirar el mercado.
5. **Licencia en la venta directa:** archivo firmado que se comprueba sin conexión, sin servidor de activación. *Recomendado.*
6. **Informes de errores:** mantener "sin telemetría" y usar los informes de la tienda (opciones A + B de §5.4). *Recomendado.*
7. **Idiomas al lanzar:** inglés y español; otros, según la demanda. *Recomendado.*
8. **Orden:** etapa 0 ya, en paralelo; la etapa 2 después de la prueba de extremo a extremo de Windows. *Recomendado.*
9. **Marca:** ¿autorizas que busque el nombre "XcreenShot" en las bases de marcas públicas y en las tiendas, o lo hace el abogado?

## 9. Cambios del spec que habría que proponer (si Carlos aprueba)

| Dónde | Cambio |
|---|---|
| `SPEC.md` §1 | El objetivo pasa de "para Carlos y sus colegas" a un producto a la venta; la paridad sigue siendo la meta funcional |
| `SPEC.md` §3.2 y "Excluido" de `05-app.md` | Licencias y prueba gratuita pasan a estar dentro del alcance (sin cuenta obligatoria ni activación en línea) |
| IDs nuevos `APP-` | Licencia y prueba gratuita; edición de la Microsoft Store (MSIX); aviso de componentes de terceros generado; textos legales en la app (EULA, privacidad) |
| APP-036 | Según el canal: la tienda actualiza la edición MSIX; el actualizador propio queda para la venta directa |
| APP-037 | Añadir la edición MSIX; el instalador EXE sigue para la venta directa |
| APP-028, APP-029, APP-003 | En MSIX: alias de ejecución para el CLI, ruta del bloque de configuración MCP y tarea de inicio |
| APP-039 | Sin cambios: venta directa con Developer ID y notarización; la Mac App Store sigue fuera en v1 |
| APP-015 | Destino de Enviar comentarios: el correo de soporte |
| APP-054 | Sin cambios si se elige A + B en §5.4 |
| SMT-039, SMT-040 | Texto del aviso de la IA revisado por el abogado; declarar la IA generativa en la ficha de la tienda |
| D-01, D-06, D-07, D-09 | Cerrarlas: repositorio privado con EULA propia; firma (Artifact Signing o certificado OV; Developer ID en Mac); idiomas al lanzar; nombre según la búsqueda de marca |

---

## 10. Fuentes (consultadas el 2026-11-14)

**Microsoft**
- Requisitos de los envíos EXE/MSI: <https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/app-package-requirements> (actualizada 2026-08-24), <https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/upload-app-packages>, <https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/create-app-submission>
- Requisitos de MSIX (firma de Microsoft, símbolos para fallos): <https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements>
- Precio y prueba gratuita en MSIX: <https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/price-and-availability>
- Políticas de la tienda, versión 7.20 (en vigor desde 2026-10-22): <https://learn.microsoft.com/en-us/windows/apps/publish/store-policies> (10.1.1, 10.2.3, 10.2.7, 10.2.8, 10.2.9, 10.5.1, 10.8.1, 11.2, 11.11, 11.16)
- Cuenta de desarrollador y verificación: <https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/account-types-locations-and-fees>
- Comisión (15 % apps, 12 % juegos, 100 % con cobro propio): <https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/why-distribute-through-store>
- Clasificación por edades (IARC): <https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/age-ratings>
- Informes de salud: <https://learn.microsoft.com/en-us/windows/apps/publish/health-report>
- Apps de escritorio empaquetadas (carpetas, AppData, registro): <https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes> (actualizada 2026-10-12)
- Extensiones del paquete (alias, tarea de inicio, tipos de archivo): <https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/desktop-to-uwp-extensions> (actualizada 2026-09-28)
- SmartScreen y firma (EV ya no evita el aviso; Artifact Signing desde 9,99 USD/mes): <https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation>
- Artifact Signing (países y personas): <https://learn.microsoft.com/en-us/azure/artifact-signing/overview>, <https://learn.microsoft.com/en-us/azure/artifact-signing/faq>, <https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart> (actualizada 2026-10-08)

**Apple**
- Guía de revisión (2.3.7, 2.4.5, 2.5.14, 3.1.1, 3.1.3 b, 4.1 c; 5.1.1 y 5.1.2 leídos solo a nivel de resumen): <https://developer.apple.com/app-store/review/guidelines/>
- Programa para pequeñas empresas (15 %): <https://developer.apple.com/app-store/small-business-program/>
- Apple Developer Program (99 USD/año): <https://developer.apple.com/programs/>
- Developer ID: <https://developer.apple.com/developer-id/>
- Notarización (obligatoria con Developer ID; *Hardened Runtime* en la app y en las herramientas de línea de comandos): <https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution>
- Herramienta de línea de comandos dentro de una app del sandbox: <https://developer.apple.com/documentation/xcode/embedding-a-helper-tool-in-a-sandboxed-app>
- Entitlements del sandbox (carpetas estándar, archivos elegidos por el usuario, herencia del sandbox): <https://developer.apple.com/library/archive/documentation/Miscellaneous/Reference/EntitlementKeyReference/Chapters/EnablingAppSandbox.html> (archivada)
- Detalles de privacidad de la App Store (política de privacidad obligatoria): <https://developer.apple.com/app-store/app-privacy-details/>
- Comisión estándar del 30 %: informe de Apple sobre el ecosistema de la App Store 2025 (<https://www.apple.com/newsroom/pdfs/Apples_Global_App_Store_Ecosystem_and_Its_Growth_2025.pdf>), no leído directamente en esta sesión

**Tauri**
- Microsoft Store (solo genera EXE y MSI; WebView2 sin conexión; instalación silenciosa; el editor no puede llamarse igual que el producto): <https://v2.tauri.app/distribute/microsoft-store/>
- Firma en Windows: <https://v2.tauri.app/distribute/sign/windows/>
- Mac App Store: <https://v2.tauri.app/distribute/app-store/>
- Actualizador (firma propia obligatoria): <https://v2.tauri.app/plugin/updater/>

**No se pudo verificar en la documentación oficial** (marcado en el texto): si ScreenCaptureKit, la monitorización de entrada y la API de Accesibilidad funcionan dentro del sandbox de macOS; la clasificación de edad exacta de Apple; cómo cobra Microsoft una app EXE/MSI "de pago"; si otros procesos pueden ejecutar el `.exe` desde la carpeta `WindowsApps`; las comisiones de los proveedores de pago.
