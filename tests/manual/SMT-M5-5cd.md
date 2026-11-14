# Exportar la guía y guías en la biblioteca (M5.5c y M5.5d)

IDs: SMT-030, SMT-032 a SMT-035. Diseño: [ADR 0009](../../docs/adr/0009-guias-de-pasos.md).

## Visto en la instancia de desarrollo (2026-11-14)

- Pruebas automáticas (`xs-editor`):
  - Markdown con lista numerada e imágenes relativas; HTML con estilos propios, `alt` y nada de la red; un solo archivo con las 3 imágenes dentro; la carpeta repetida sale como `-2`;
  - Word con 6 pasos en Lista con números, 6 imágenes con su texto alternativo, Título 1 y pie; PowerPoint con diapositiva de título y una por paso;
  - PDF de 12 pasos sin ninguno partido, un marcador por paso y "Página N de M";
  - imágenes sueltas `Alta_de_cliente-01.png`… y `pasos.txt`;
  - una guía de 7 pasos en la biblioteca: tipo `guide`, `steps: 7`, la búsqueda la encuentra por una palabra del paso 5 y se exporta desde su archivo.
- Con herramientas externas: python-docx lee el Word (Título 1, Lista con números, 3 imágenes, pie "Creado por…"); LibreOffice abre el Word y el PowerPoint (4 diapositivas); PyMuPDF y pypdf leen el PDF (texto, marcadores y "Página 2 de 2").

## Para Carlos (en la próxima caminata)

1. [ ] En una guía, pestaña **Pasos** > **Exportar como** > **Documento de Word**: ábrelo en Word. Título, una lista numerada con el texto de cada paso (con sus negritas) y su imagen. Cambia el estilo Normal: cambia la letra de los pasos.
2. [ ] **Presentación de PowerPoint**: una diapositiva de título y una por paso; el título de cada una se puede editar.
3. [ ] **PDF**: ningún paso partido entre páginas, "Página N de M" al pie, el panel de marcadores lista los pasos y la búsqueda encuentra una palabra de un paso. `Ctrl+P` en la guía abre ese mismo PDF para imprimir.
4. [ ] **Markdown** y **Página HTML** en una carpeta: se crea una carpeta con el nombre de la guía y `images/`. Abre el HTML sin conexión en el navegador. **HTML en un solo archivo**: muévelo solo a otra carpeta y sigue mostrando las imágenes.
5. [ ] **Imágenes por separado**: un archivo por paso, numerado, con sus anotaciones.
6. [ ] **Copiar al portapapeles** (o Copiar todo en la guía) y pégalo en un correo de Outlook o en Word: los pasos numerados con su texto y sus imágenes.
7. [ ] En la **Biblioteca**, filtro Tipo > **Guías de pasos**: aparece la guía con la insignia "N pasos". Busca una palabra que solo esté en un paso: aparece.
8. [ ] Clic derecho en la guía > **Exportar como** > **Markdown**: se crea la carpeta sin abrir el editor.
