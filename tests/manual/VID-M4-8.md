# Dibujo en vivo (M4.8)

Dibujar formas sobre la pantalla mientras se graba; quedan en el video y desaparecen solas. Y dos preferencias de grabación. IDs: VID-007, VID-018, VID-021 y VID-048 a VID-051.

## Visto en la instancia de desarrollo (2026-11-13, noche)

- Modelo (pruebas automáticas de `xs-capture`):
  - cada herramienta se elige con su tecla;
  - `1` y tres clics dan los pasos 1, 2 y 3, y `1` otra vez vuelve a empezar en 1;
  - Deshacer quita solo la última forma y Supr todas;
  - con 5 s, una forma empieza a desvanecerse a los 5 s y a los 5,5 s ya no está;
  - con Nunca, la forma sigue hasta salir del modo;
  - `4` elige el tercer color.
- Grabación real de 3 s de una zona de 640 × 480 con Dibujo en vivo activado y, por comando (`record.draw_shape`), una flecha roja y un paso azul. Ambos salen en el fotograma de 1 s del video, encima de lo que había en pantalla; la barra de herramientas no sale. Se activó y se apagó por comando: unos 2 s con los ganchos puestos.

## Para Carlos (en la próxima caminata)

1. [ ] Empieza una grabación. En la barra de grabación aparece **Dibujar**; púlsalo (o `Ctrl+Alt+Mayús+D`). Arriba sale la barra de Dibujo en vivo: herramientas, seis colores, grosor y "Desaparecen tras".
2. [ ] Arrastra sobre una página: aparece una flecha roja, y la página no recibe el clic. La rueda del ratón sigue desplazando la página.
3. [ ] Prueba las teclas: `R` rectángulo (con `Mayús`, cuadrado), `O` óvalo, `P` lápiz, `1` y tres clics (1, 2, 3), `1` y otro clic (vuelve a 1), `2`–`7` colores, `Z` o `Ctrl+Z` deshacer, `Supr` borrar todo.
4. [ ] Cada forma desaparece sola a los 3 s. Cambia "Desaparecen tras" a **Nunca**: se quedan hasta que pulsas `Esc`, que apaga el modo (la grabación sigue) y las borra con un fundido.
5. [ ] Mientras dibujas, escribir no llega a las apps; los atajos globales (por ejemplo, `Ctrl+Mayús+F9`, Pausar) sí funcionan.
6. [ ] Al detener la grabación, el video muestra las formas tal como las viste, y en ningún fotograma sale la barra de Dibujo en vivo.
7. [ ] Pestaña **Video** del panel: desmarca **Dibujo en vivo**; al grabar, la barra no tiene **Dibujar** y el atajo no hace nada.
8. [ ] Preferencias > Video > **Dibujo en vivo en cualquier momento**: con ella activada y sin grabar, el atajo permite dibujar sobre cualquier app (por ejemplo, en una videollamada) y `Esc` sale.

### Empezar sin esperar y escritorio limpio (VID-007, VID-018)

9. [ ] Preferencias > Video > **Empezar a grabar sin esperar**: elige un área y, sin pulsar Grabar, empieza la cuenta atrás y la grabación.
10. [ ] Preferencias > Video > **Ocultar los íconos del escritorio al grabar**: al elegir el área, los íconos desaparecen; al detener (o cancelar), vuelven en las mismas posiciones.
11. [ ] Reproduce un video largo en el editor sin tocar el ratón más que el tiempo de apagado de pantalla: la pantalla no se apaga mientras se reproduce.

Todavía no: la Lupa (`M`), el puntero con la forma de la herramienta y el botón en el menú del ícono.
