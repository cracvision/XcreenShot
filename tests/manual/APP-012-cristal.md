# Aspecto Cristal · APP-012, EDT-001, EDT-005, EDT-011, TRY-002

Pedido por Carlos el 2026-11-12 (maqueta aprobada ese mismo día): un aspecto limpio y moderno, con paneles de cristal de intensidad ajustable.

Pruebas automáticas:
- `app/src/editor/insets.test.ts`: lo que tapa cada barra flotante y las barras de desplazamiento del lienzo (tamaño, posición y arrastre).
- `app/src/theme.test.ts`: la intensidad (0 a 100, con valores fuera de rango) y el acento.
- `crates/xs-core/src/settings.rs`: los valores por defecto de `general.glassIntensity` (60) y `general.accent` (azul).

Visto en la instancia de desarrollo (2026-11-12, ventana de 2549 × 1743 px a 150 %):
- La barra, el panel lateral, la barra de estado y la bandeja flotan; con una imagen de 1920 × 1080 a 100 % queda centrada en la parte visible.
- Con el zoom alto, la captura pasa por debajo de la barra y del panel, desenfocada.
- Una flecha dibujada con el ratón (zoom de ajuste, 115 %) empieza y acaba justo donde se arrastró. La posición del puntero en la barra de estado coincide (100 %).
- Los menús de la barra (Más) ya no se cortan. El selector de color del panel se abre en su sitio.
- Claro y oscuro; intensidad 100 y 60; acentos azul, violeta y turquesa, también con las flechas del teclado en Preferencias.
- Alto contraste (emulado): paneles sólidos con borde.
- No probado aún: los efectos de transparencia de Windows desactivados (el navegador integrado reconoce la consulta `prefers-reduced-transparency`; falta verlo con el ajuste real).
- La barra se compacta sin montar nada a 1920, 1366, 1100 y 900 px de ancho.

## Para Carlos (en la próxima caminata)

1. [ ] Abre una captura grande y acércala: pasa por debajo de la barra, del panel y de la bandeja, borrosa.
2. [ ] Preferencias > General > **Intensidad del cristal**: muévela de 0 a 100; a 0, todo es sólido.
3. [ ] **Color de acento**: prueba los cuatro, en claro y en oscuro.
4. [ ] Dibuja una flecha y un rectángulo con zoom alto y con la imagen desplazada: caen donde arrastras.
5. [ ] Las barras de desplazamiento finas del lienzo: arrástralas y haz clic en su carril.
6. [ ] Abre los menús de la barra (Más, Compartir): se ven enteros.
7. [ ] Windows > Personalización > Colores > **Efectos de transparencia** desactivados: los paneles quedan sólidos.
