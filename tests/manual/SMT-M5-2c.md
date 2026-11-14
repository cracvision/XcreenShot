# Redactar varias capturas a la vez (M5.2c)

ID: SMT-016.

## Visto en la instancia de desarrollo (2026-11-14)

- Pruebas automáticas (`xs-capture`):
  - `several_captures_redacted`: el mismo correo en dos capturas se propone en las dos; el nombre redactado a mano en la captura de origen se propone donde aparece; una sin datos queda "Nada que redactar" y el video queda fuera; los archivos no cambian antes de decidir; Aceptar todas los guarda con las redacciones; Deshacer todo los deja como estaban.
  - `cancel_and_skip_leave_captures_alone`: aceptar una, omitir otra y cancelar deja la omitida y la cancelada sin cambios; una captura abierta sigue abierta sin lo que propuso el lote.
- En la instancia aislada, con cuatro formularios generados: la ventana de revisión con miniaturas y números por categoría ("Correos electrónicos: 1 · Manual: 1"), **Revisar en el editor** con la barra de vuelta, **Aceptar todas** (copia "pedido-2 (redacted).png"), **Deshacer todo** (la copia desaparece) y **Cerrar**.

## Para Carlos (en la próxima caminata)

1. [ ] En la **Biblioteca**, selecciona 10 capturas que muestren el mismo correo > clic derecho > **Redacción automática…**. La ventana de revisión propone el correo en las 10 con su miniatura. **Aceptar todas**: ábrelas y está cubierto en todas.
2. [ ] En una captura abierta, redacta a mano un nombre (clic en las palabras) > **Buscar en otras capturas…** > elige capturas donde aparezca ese nombre > **Buscar**. Se propone también en ellas ("Manual: 1").
3. [ ] Con muchas capturas, pulsa **Cancelar** a mitad: las no aceptadas quedan intactas.
4. [ ] **Revisar en el editor** en una: se abre con el panel; la barra de arriba lleva de vuelta a la revisión.
5. [ ] **Deshacer todo** antes de cerrar: las aceptadas vuelven a como estaban.
6. [ ] En la **bandeja reciente**, selecciona varias capturas (Ctrl+clic) > clic derecho > **Redacción automática…**.
