# Plan de M6 · Compartir y distribución (propuesta, 2026-11-15)

M6 tiene dos tipos de trabajo: lo que el spec ya define y se puede construir ya, y lo que necesita una decisión de Carlos o un spec que todavía no está escrito.

## Se construye ya (spec completo)

| Entrega | IDs | Qué |
|---|---|---|
| M6.1 · Red local | SHR-042, SHR-043, SHR-044 | Ver quién está (mDNS), emparejar con código de 6 dígitos, contactos, enviar cifrado (TLS 1.3 con claves fijadas), pendientes de 7 días, aceptar o rechazar, comandos `share.*`. Dependencias aprobadas el 2026-11-15: `mdns-sd` y `rustls` (con su motor criptográfico `ring`) |
| M6.2 · Configuración administrada | APP-035 | `managed.json` en `%ProgramData%\XcreenShot`: valores por defecto, ajustes bloqueados y directivas; Avanzado > Configuración administrada |
| M6.3 · Modo portable | APP-038 | `portable.flag` junto al ejecutable: datos y biblioteca en la carpeta; "(portable)" en el título y en Acerca de; `.zip` portable |
| M6.4 · Instalador y desinstalación | APP-037, APP-042 | Lo que falta del instalador por usuario (`/silent`, PATH y arranque al iniciar sesión) y la pregunta de borrar ajustes al desinstalar (nunca la biblioteca) |

## Necesita a Carlos

| IDs | Qué falta |
|---|---|
| SHR-001 a SHR-017 (destinos de compartir, Enlace para compartir, historial) | **El spec no está escrito** (`04-biblioteca-compartir.md` §9 y §10 solo tienen Archivo y Portapapeles). Hay que redactar las fichas y que Carlos las apruebe. D-02 (proveedor del enlace) sigue abierta |
| APP-036 · Actualizaciones | Dónde se publican las versiones (el repositorio es privado), con qué clave se firma el manifiesto y quién la guarda |
| APP-037 · Firma del instalador | Certificado de firma de código (D-06): tiene coste anual |
| APP-041 · Windows en ARM | Compilar es posible; probarlo necesita un equipo ARM64 |
| APP-039 · macOS | Etapa de macOS |

## Diseño de M6.1 (red local)

- **Identidad:** un par de claves Ed25519 por equipo, en el llavero del sistema (una entrada propia por instancia aislada). La huella (los primeros 16 caracteres hexadecimales del SHA-256 de la clave pública) es lo único que se anuncia además del nombre visible y el equipo.
- **Descubrimiento:** servicio DNS-SD `_xcreenshot._tcp.local.` con `mdns-sd`; TXT `name`, `device`, `fp`, `v`. Solo mientras la preferencia está activada. "Sin conexión" en menos de 30 s: se comprueba cada 10 s si cada persona sigue ahí.
- **Conexión:** TCP directo entre equipos con TLS 1.3 de `rustls`, con **claves públicas fijadas** (RFC 7250, sin certificados ni autoridades): cada lado comprueba la clave del otro. Para emparejar, la clave tiene que coincidir con la huella anunciada; después, con la del contacto guardado.
- **Emparejar:** los dos ven un código de 6 dígitos (SHA-256 de las dos claves ordenadas) y los dos aceptan; solo entonces cada uno guarda al otro como contacto (`contacts.json` en la carpeta de ajustes: nombre, equipo, clave pública, Aceptar siempre, bloqueado).
- **Enviar:** oferta con nombres y tamaños → el otro acepta o rechaza → los archivos van por la misma conexión cifrada, con SHA-256 de cada uno; llegan a una carpeta temporal y solo pasan a la biblioteca, con la etiqueta "De: <nombre>", si todo llegó entero. Cancelar en cualquier lado no deja archivos a medias.
- **Pendientes:** `outbox.json` con cada envío y su estado (Pendiente, Enviando, Entregado, Rechazado, Caducado, Cancelado, Falló). Se reintenta cada 15 s cuando el contacto está en línea; caduca a los 7 días.
- **Comandos** `share.lan_*` (estado, personas, emparejar, responder, contactos, enviar, envíos, cancelar, recibidos, aceptar o rechazar). Responder por automatización exige la preferencia **Permitir por automatización**.
- **Pruebas:** dos instancias del servicio en el mismo proceso, por `127.0.0.1` y con el descubrimiento simulado, para no abrir puertos hacia la red ni hacer saltar el firewall en el equipo de Carlos. La prueba entre dos equipos reales, con Carlos.
