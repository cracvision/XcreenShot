# Redacciones a mano y todas las coincidencias (M5.2b)

IDs: SMT-014, SMT-015.

## Visto en la instancia de desarrollo (2026-11-14)

- Pruebas automáticas:
  - `xs-capture` `redactions_by_hand`: clic en "Marta" la redacta en **Manual**, un análisis nuevo la conserva; "Marta Ruiz" en todas sus apariciones deja 3 redacciones (la primera sustituye a la de la palabra); Quitar todas las coincidencias las descubre, Deshacer las devuelve; el correo quitado a mano no vuelve con otro análisis y Deshacer lo devuelve.
  - UI: tres palabras de una línea dan un solo rectángulo; un arrastre hacia atrás por dos líneas da uno por línea; doble clic da la línea; el texto bajo una redacción; el aviso de coincidencias y Añadir a Mis términos; Quitar con el teclado.

## Para Carlos (en la próxima caminata)

1. [ ] Abre una captura con texto > **Redacción automática…**. Pasa el puntero por el lienzo: cada palabra leída se resalta.
2. [ ] Clic en una palabra que no se detectó: queda cubierta con el estilo actual y aparece la fila **Manual** con 1.
3. [ ] Clic en una redacción automática: desaparece y su categoría baja en uno. Cambia una categoría: no vuelve.
4. [ ] Arrastra sobre tres palabras de una línea: una sola redacción las cubre. Arrastra sobre dos líneas: una por línea.
5. [ ] Doble clic en una palabra: se cubre la línea entera.
6. [ ] En una captura donde un nombre aparece 4 veces: clic derecho en él > **Redactar todas las coincidencias**. Quedan las 4 cubiertas y el panel dice "Se redactaron…". Prueba **Deshacer** y después **Añadir a Mis términos**.
7. [ ] Clic derecho en una de ellas > **Quitar todas las coincidencias**: se descubren las 4.
8. [ ] Solo con el teclado: **Siguiente** hasta una redacción y **Quitar**.
