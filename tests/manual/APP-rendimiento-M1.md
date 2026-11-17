# Rendimiento de M1 (APP-048, APP-049, APP-051, APP-053)

Medido por Claude el 2026-11-11 en el equipo de Carlos (Windows 11, un monitor de 5120 × 2160 al 150 %), con la compilación de *release* en una instancia aislada (`XCREENSHOT_DATA_DIR`), sin la app instalada en marcha.

## Cómo se mide

- Compilación: `npx tauri build --no-bundle` en `app/` (deja `target/release/xcreenshot-app.exe`).
- Instancia aislada de *release* con DevTools (ver `scripts/dev/README.md`); desde el 2026-11-11 ya no toca el inicio con Windows.
- Memoria: `scripts/dev/memory.ps1 -appPid <pid>` suma la app y sus procesos de WebView2 (memoria privada).
- Tiempos: el registro de la app (`…/logs/xcreenshot-AAAA-MM-DD.log`) escribe
  - `screens frozen` (congelar las pantallas al empezar una selección, APP-048),
  - `capture completed … save_ms total_ms` (guardar la captura en la biblioteca),
  - `capture to editor ms` (desde que se confirma la selección hasta que el editor la pinta, APP-049).

## Resultados

| Qué | Objetivo | Medido | Estado |
|---|---|---|---|
| Del arranque al ícono y los atajos activos (APP-051) | < 3 s | 27 ms | Cumple |
| Congelar las pantallas al empezar la selección (APP-048) | — | 75–105 ms (5120 × 2160) | — |
| De confirmar una región de 3840 × 2010 al editor (APP-049), 30 capturas | p95 < 400 ms | 96–161 ms; la primera de cada tanda, unos 150 ms | Cumple |
| Guardar la captura en la biblioteca (APP-049) | No retrasa el editor | 25–35 ms | Cumple |
| Reposo sin el editor cargado (APP-051) | < 150 MB | ~205 MB | **No cumple** (ver abajo) |
| Reposo con el editor cargado y oculto (APP-051) | < 400 MB | ~340 MB | Cumple |
| CPU en reposo (APP-051) | < 0,5 % | ~0,3 s de CPU por minuto (≈ 0,5 % de un núcleo) | Al límite |
| Tras 10 y 20 capturas 4K, cerradas (APP-051) | crece < 10 % | 340 → ~510–550 MB y se estabiliza | **No cumple** (ver abajo) |
| Copiar al portapapeles sin abrir ventana (APP-049) | < 250 ms | No medido: las pruebas automáticas no tocan el portapapeles de Carlos | En la caminata |
| Cruceta con 3 monitores 4K (APP-053) | ≥ 60 fps, p95 < 150 ms | No medible en este equipo (un solo monitor) | Pendiente |

## Lo que no cumple y por qué

- **Reposo sin el editor (≈ 205 MB):** el proceso de la app usa ~50 MB; el resto es WebView2 (navegador, GPU y las ventanas de selección precargadas para que la cruceta aparezca rápido, APP-048). Bajar a 150 MB obliga a no precargar la selección, y la cruceta tardaría más en salir. Decisión pendiente de Carlos (ver `spec/QUESTIONS.md`).
- **Memoria tras muchas capturas 4K:** la app libera sus imágenes al cerrar los documentos (su proceso vuelve a ~54 MB), pero el proceso GPU de WebView2 se queda unos 200 MB de caché de texturas. No es una fuga: tras 10 y tras 20 capturas da lo mismo. Se probó:
  - pedir a WebView2 poca memoria con el editor oculto (`MemoryUsageTargetLevel`, se queda en el código);
  - limitar la caché de la GPU (`--force-gpu-mem-discardable-limit-mb=64`): apenas cambia y no se adopta.
  Se revisa con las imágenes muy grandes (APP-050, M2).
