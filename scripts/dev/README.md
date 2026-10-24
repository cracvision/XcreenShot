# Ayudas para probar en Windows

Scripts de PowerShell para comprobar la app en marcha sin depender de que Carlos haga clic. Las coordenadas son píxeles físicos del escritorio. Varios esperan la app de desarrollo (`npx tauri dev > "$env:TEMP\xs-dev.log"` en `app/`) y el CLI en `target\debug\xcreenshot.exe`.

| Script | Para qué |
|---|---|
| `visible-windows.ps1` | Lista las ventanas visibles de XcreenShot con título, clase y rectángulo. Sirve para ver la selección y la cuenta atrás, que no salen en las capturas. |
| `win-rects.ps1` | Rectángulos de ventana, de cliente y de marco (DWM) y estilos de las ventanas visibles. |
| `click.ps1 -x X -y Y` | Clic izquierdo en un punto. |
| `shot.ps1 -x X -y Y -w W -h H -out archivo.png` | Guarda una imagen de un rectángulo de la pantalla. |
| `drag-run.ps1` | Lanza una captura interactiva, arrastra de (X1, Y1) a (X2, Y2) y pulsa teclas; imprime el resultado del CLI. |
| `esc-runs.ps1` | Lanza varias capturas de región y cancela cada una con `Esc`. |
| `esc-countdown.ps1` | Lanza una captura con retardo y pulsa `Esc` durante la cuenta atrás. |
| `enter-universal.ps1` | Lanza una captura universal y pulsa `Enter` cuando aparece la selección. |

Antes de usarlos:
- Mueven el ratón y pulsan teclas de verdad: avisa a Carlos antes si está usando el equipo.
- Para no llenar su carpeta de capturas, arranca la app con `XCREENSHOT_LIBRARY_DIR` apuntando a una carpeta temporal.
- Los avisos de Windows se quedan en pantalla mientras no hay actividad del usuario y tapan la esquina inferior derecha, donde está la cuenta atrás.
