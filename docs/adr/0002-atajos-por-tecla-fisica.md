# ADR 0002 · Atajos guardados por tecla física

**Estado:** aceptada · **Fecha:** 2026-10-12 · **IDs:** APP-017 a APP-021

## Decisión

- Un atajo se guarda con los nombres W3C de `KeyboardEvent.code` (`KeyK`, `Digit5`, `PrintScreen`) y los modificadores en orden fijo: `Ctrl+Alt+Shift+Super+KeyK`. Es la forma que entienden tanto el plugin de atajos globales como los menús nativos, sin conversión.
- La interfaz muestra la tecla con el carácter de la distribución activa cuando el sistema lo expone (`navigator.keyboard.getLayoutMap`, disponible en WebView2). En macOS (WKWebView) no existe esa API: se muestra la etiqueta de la distribución estadounidense. Provisional; ver `spec/QUESTIONS.md`.
- `shortcuts.json` solo guarda las diferencias del usuario respecto del perfil (`null` = quitado a propósito). Cambiar de perfil cambia los valores por defecto y conserva los personalizados (APP-021).
- El estado de cada atajo (activo, en pausa, en conflicto con otro comando, con el sistema o con otra app, tomado) lo calcula `xs-core` con lo que informa el shell al registrar.
- Sin ganchos de teclado de bajo nivel (APP-020): solo `RegisterHotKey` (Windows) y el registro de atajos de Carbon (macOS), a través de `tauri-plugin-global-shortcut`.
