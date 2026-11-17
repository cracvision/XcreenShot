// Help pages (APP-015): local, offline, in the language of the interface, one per area.
// A small markup: blank lines between paragraphs, "## " for a subtitle, "- " for a list
// item, **bold** and `keys` (shown as keys). Shortcuts are the defaults on Windows;
// Preferences > Shortcuts shows the ones in use.

export interface HelpPage {
  id: string;
  title: string;
  body: string;
}

export const TOPICS = [
  "start",
  "capture",
  "editor",
  "library",
  "shortcuts",
  "permissions",
  "automation",
  "video",
  "share",
] as const;
export type Topic = (typeof TOPICS)[number] | "about";

const en: Record<(typeof TOPICS)[number], Omit<HelpPage, "id">> = {
  start: {
    title: "Getting started",
    body: `XcreenShot captures your screen, lets you mark it up and keeps every capture in your library. It runs in the background: look for its icon next to the clock.

## Your first capture
- Press \`Ctrl+Shift+PrtScn\`, or click the icon and choose **Capture**.
- Drag over what you want, or click a window to take all of it.
- Press \`Enter\` or click **Capture**. The capture opens in the editor.

## Then
- Mark it up with an arrow, text, a shape or a blur, and copy it with \`Ctrl+Shift+C\`.
- Drag it from the recent tray at the bottom into a chat, an email or a folder.
- Find it later in the **Library** view (\`Ctrl+Shift+2\`).

Everything stays on this computer. XcreenShot only connects to the internet when you share something online, when it checks for updates, or when you turn on the cloud AI and ask it something.`,
  },
  capture: {
    title: "Capture",
    body: `## Ways to capture
- **Capture** (\`Ctrl+Shift+PrtScn\`): uses the tab that is active in the capture panel.
- **Universal capture**: drag an area or click a window, then choose what to do.
- **Window**, **Active window**, **Region** and **Full screen**: one step each, from the icon menu or with a shortcut.
- **Repeat last capture** (\`Ctrl+Alt+Shift+R\`) and **Repeat last area**.
- **Multi-capture**: every click captures the window under the pointer; \`Esc\` ends it.

## While you select
- The screen freezes, so menus and tooltips stay in the picture.
- Arrow keys move the crosshair one pixel; with \`Shift\`, ten.
- \`M\` shows or hides the magnifier; \`C\` copies the color under the pointer.
- \`Esc\` cancels.

## Options in the capture panel
- **Copy to clipboard**, **Open in** (the editor or nothing), **Delay** with a countdown.
- **Capture cursor**: the pointer arrives as its own object, which you can move or delete.

## When it is done
The capture is saved in your library. Preferences > Capture has the sound and the flash; Preferences > General, the notifications. If a window protects its content, the capture comes out black and XcreenShot tells you.`,
  },
  editor: {
    title: "Editor",
    body: `## Tools
- **Move** (\`V\`): select, move and resize objects.
- **Arrow** (\`A\`), **Text** (\`T\`), **Shape** (\`U\`): drag on the capture.
- **Blur** (\`B\`): hides what is underneath (blur, pixelate or solid).
- **Crop** (\`R\`): nothing is lost; crop again to bring it back.

## Working with objects
- Each mark is an object you can select and change later in the Properties panel.
- \`Shift\` keeps the proportions; \`Alt\` resizes from the center.
- \`Ctrl+C\`, \`Ctrl+X\` and \`Ctrl+V\` copy, cut and paste objects; \`Delete\` removes them.
- \`Ctrl+Z\` undoes and \`Ctrl+Y\` redoes.

## Saving and sharing
- Captures in the library save themselves as editable .xshot files.
- **Copy all** (\`Ctrl+Shift+C\`) copies the finished image.
- **Export** (\`Ctrl+Alt+Shift+E\`) writes a PNG or JPG.
- Saving a PNG or JPG with marks flattens it; an editable copy stays in the library.

## Keyboard
\`F6\` moves between the toolbar, the canvas, the side panel and the tray. \`Esc\` can close the editor (Preferences > Editor).`,
  },
  library: {
    title: "Library and recent tray",
    body: `## Library
Every capture is a file in your library folder (Documents\\XcreenShot, one folder per month). The **Library** view (\`Ctrl+Shift+2\`) shows them all, newest first.
- Click to select; \`Ctrl\` and \`Shift\` add more; drag on an empty spot to select a group.
- Double-click or \`Enter\` opens in the editor.
- \`F2\` renames, \`Delete\` moves to the Recycle Bin, with **Undo** for a few seconds.
- Files you copy into the folder appear in the library by themselves.

## Recent tray
The strip at the bottom of the editor holds the captures you are working on.
- \`Ctrl+PageUp\` and \`Ctrl+PageDown\` move between them; \`Ctrl+Alt+R\` shows or hides it.
- A dot marks changes that are not saved yet.
- Right-click for Close, Close others, Rename, Show in folder and Move to trash.
- Drag an item into another app, the desktop or a folder: it arrives as a PNG with its marks.

Preferences > Library shows where the library is and what happens when you save or delete.`,
  },
  shortcuts: {
    title: "Shortcuts",
    body: `Preferences > Shortcuts lists every shortcut, and you can change any of them: click the field and press the new keys.

## Three kinds
- **Global**: work from any app (the capture shortcuts).
- **In the app**: work in any XcreenShot window (\`Ctrl+,\` opens Preferences).
- **Editor**: work in the editor, and win over the others there.

## Profiles
- **Coexistence** (default) uses keys that do not clash with other capture apps.
- **Compatibility** uses the keys people know from other capture tools, starting with \`PrtScn\`.

If another app already uses a shortcut, XcreenShot tells you and offers to take it while it is open. You can pause all global shortcuts from the icon menu.`,
  },
  permissions: {
    title: "Permissions and notifications",
    body: `## Windows
XcreenShot needs no special permission to capture. It runs without administrator rights; windows of apps running as administrator can still be captured.

Preferences > General > Permissions shows whether Windows lets XcreenShot show notifications, with a button to its settings.

## macOS
macOS asks for **Screen Recording** before the first capture. If a capture fails, open Preferences > General > Permissions or Help > Fix permissions.

## Notifications
Notifications tell you about captures that open no window, about problems, and about captures made by scripts or agents. Each kind has its switch in Preferences > General. Clicking a capture notification opens it in the editor.`,
  },
  automation: {
    title: "Automation (command line and agents)",
    body: `Every action of XcreenShot is a command, and scripts and AI agents can run the same commands.

## Command line
The installer adds the \`xcreenshot\` command (Preferences > Advanced shows where). For example:
- \`xcreenshot capture fullscreen\` takes the screen and saves it in the library.
- \`xcreenshot app commands\` lists every command; add \`--help\` to any of them.
- Commands that delete or reset need \`--yes\`.

## Agents (MCP)
\`xcreenshot mcp\` lets Claude Desktop or Claude Code use XcreenShot. Preferences > AI has the configuration to copy.

## Safety
Preferences > Advanced can turn automation off, and a notification can tell you whenever a script or an agent captures the screen. Captures made this way never send anything anywhere by themselves.`,
  },
  video: {
    title: "Video",
    body: `Screen recording arrives in a later version of XcreenShot. This version captures images.`,
  },
  share: {
    title: "Share",
    body: `Sharing to other services arrives in a later version. Today you can:
- copy the finished image with **Copy all** (\`Ctrl+Shift+C\`);
- export a PNG or JPG (\`Ctrl+Alt+Shift+E\`);
- drag an item of the recent tray or the library into another app or a folder.`,
  },
};

const es: typeof en = {
  start: {
    title: "Primeros pasos",
    body: `XcreenShot captura tu pantalla, te deja marcarla y guarda cada captura en tu biblioteca. Funciona en segundo plano: busca su ícono junto al reloj.

## Tu primera captura
- Pulsa \`Ctrl+Shift+PrtScn\`, o haz clic en el ícono y elige **Capturar**.
- Arrastra sobre lo que quieras, o haz clic en una ventana para tomarla entera.
- Pulsa \`Enter\` o haz clic en **Capturar**. La captura se abre en el editor.

## Después
- Márcala con una flecha, un texto, una forma o un desenfoque, y cópiala con \`Ctrl+Shift+C\`.
- Arrástrala desde la bandeja reciente, abajo, a un chat, un correo o una carpeta.
- Encuéntrala más tarde en la vista **Biblioteca** (\`Ctrl+Shift+2\`).

Todo se queda en este equipo. XcreenShot solo se conecta a internet cuando compartes algo en línea, cuando busca actualizaciones o cuando activas la IA en la nube y le preguntas algo.`,
  },
  capture: {
    title: "Captura",
    body: `## Formas de capturar
- **Capturar** (\`Ctrl+Shift+PrtScn\`): usa la pestaña activa del panel de captura.
- **Captura universal**: arrastra un área o haz clic en una ventana, y después elige qué hacer.
- **Ventana**, **Ventana activa**, **Región** y **Pantalla completa**: un solo paso cada una, desde el menú del ícono o con un atajo.
- **Repetir última captura** (\`Ctrl+Alt+Shift+R\`) y **Repetir última área**.
- **Multicaptura**: cada clic captura la ventana bajo el puntero; \`Esc\` la termina.

## Mientras seleccionas
- La pantalla se congela, así que los menús y los tooltips salen en la imagen.
- Las flechas mueven la cruceta un píxel; con \`Shift\`, diez.
- \`M\` muestra u oculta la lupa; \`C\` copia el color bajo el puntero.
- \`Esc\` cancela.

## Opciones del panel de captura
- **Copiar al portapapeles**, **Abrir en** (el editor o nada), **Retardo** con cuenta atrás.
- **Capturar cursor**: el puntero llega como un objeto aparte, que puedes mover o borrar.

## Al terminar
La captura se guarda en tu biblioteca. En Preferencias > Captura están el sonido y el destello; en Preferencias > General, los avisos. Si una ventana protege su contenido, la captura sale en negro y XcreenShot te lo dice.`,
  },
  editor: {
    title: "Editor",
    body: `## Herramientas
- **Mover** (\`V\`): selecciona, mueve y cambia el tamaño de los objetos.
- **Flecha** (\`A\`), **Texto** (\`T\`), **Forma** (\`U\`): arrastra sobre la captura.
- **Desenfocar** (\`B\`): oculta lo que hay debajo (desenfoque, pixelado o sólido).
- **Recortar** (\`R\`): no se pierde nada; vuelve a recortar para recuperarlo.

## Trabajar con objetos
- Cada marca es un objeto que puedes seleccionar y cambiar después en el panel Propiedades.
- \`Shift\` mantiene las proporciones; \`Alt\` cambia el tamaño desde el centro.
- \`Ctrl+C\`, \`Ctrl+X\` y \`Ctrl+V\` copian, cortan y pegan objetos; \`Supr\` los borra.
- \`Ctrl+Z\` deshace y \`Ctrl+Y\` rehace.

## Guardar y compartir
- Las capturas de la biblioteca se guardan solas como archivos .xshot editables.
- **Copiar todo** (\`Ctrl+Shift+C\`) copia la imagen terminada.
- **Exportar** (\`Ctrl+Alt+Shift+E\`) escribe un PNG o un JPG.
- Guardar un PNG o un JPG con marcas lo aplana; en la biblioteca queda una copia editable.

## Teclado
\`F6\` pasa entre la barra de herramientas, el lienzo, el panel lateral y la bandeja. \`Esc\` puede cerrar el editor (Preferencias > Editor).`,
  },
  library: {
    title: "Biblioteca y bandeja reciente",
    body: `## Biblioteca
Cada captura es un archivo en la carpeta de tu biblioteca (Documentos\\XcreenShot, una carpeta por mes). La vista **Biblioteca** (\`Ctrl+Shift+2\`) las muestra todas, la más nueva primero.
- Haz clic para seleccionar; \`Ctrl\` y \`Shift\` añaden más; arrastra en un hueco para seleccionar un grupo.
- Doble clic o \`Enter\` abre en el editor.
- \`F2\` renombra y \`Supr\` manda a la Papelera de reciclaje, con **Deshacer** durante unos segundos.
- Los archivos que copies a la carpeta aparecen solos en la biblioteca.

## Bandeja reciente
La tira de abajo del editor tiene las capturas con las que estás trabajando.
- \`Ctrl+RePág\` y \`Ctrl+AvPág\` pasan de una a otra; \`Ctrl+Alt+R\` la muestra u oculta.
- Un punto marca los cambios que aún no están guardados.
- Clic derecho para Cerrar, Cerrar los demás, Renombrar, Mostrar en la carpeta y Mover a la papelera.
- Arrastra un elemento a otra app, al escritorio o a una carpeta: llega como PNG con sus marcas.

En Preferencias > Biblioteca ves dónde está la biblioteca y qué pasa al guardar o eliminar.`,
  },
  shortcuts: {
    title: "Atajos",
    body: `Preferencias > Atajos lista todos los atajos, y puedes cambiar cualquiera: haz clic en el campo y pulsa las teclas nuevas.

## Tres tipos
- **Globales**: funcionan desde cualquier app (los atajos de captura).
- **En la app**: funcionan en cualquier ventana de XcreenShot (\`Ctrl+,\` abre Preferencias).
- **Editor**: funcionan en el editor, y allí ganan a los demás.

## Perfiles
- **Coexistencia** (por defecto) usa teclas que no chocan con otras apps de captura.
- **Compatibilidad** usa las teclas que la gente conoce de otras herramientas de captura, empezando por \`PrtScn\`.

Si otra app ya usa un atajo, XcreenShot te lo dice y te ofrece tomarlo mientras esté abierto. Puedes pausar todos los atajos globales desde el menú del ícono.`,
  },
  permissions: {
    title: "Permisos y notificaciones",
    body: `## Windows
XcreenShot no necesita ningún permiso especial para capturar. Se ejecuta sin derechos de administrador; las ventanas de apps que se ejecutan como administrador se pueden capturar igual.

Preferencias > General > Permisos dice si Windows deja a XcreenShot mostrar notificaciones, con un botón a su configuración.

## macOS
macOS pide **Grabación de pantalla** antes de la primera captura. Si una captura falla, abre Preferencias > General > Permisos o Ayuda > Reparar permisos.

## Notificaciones
Las notificaciones te avisan de las capturas que no abren ninguna ventana, de los problemas y de las capturas que hacen los scripts o los agentes. Cada tipo tiene su interruptor en Preferencias > General. Un clic en el aviso de una captura la abre en el editor.`,
  },
  automation: {
    title: "Automatización (línea de comandos y agentes)",
    body: `Cada acción de XcreenShot es un comando, y los scripts y los agentes de IA pueden usar esos mismos comandos.

## Línea de comandos
El instalador añade el comando \`xcreenshot\` (Preferencias > Avanzado dice dónde está). Por ejemplo:
- \`xcreenshot capture fullscreen\` toma la pantalla y la guarda en la biblioteca.
- \`xcreenshot app commands\` lista todos los comandos; añade \`--help\` a cualquiera.
- Los comandos que borran o restablecen necesitan \`--yes\`.

## Agentes (MCP)
\`xcreenshot mcp\` deja que Claude Desktop o Claude Code usen XcreenShot. En Preferencias > IA está la configuración para copiar.

## Seguridad
En Preferencias > Avanzado puedes desactivar la automatización, y un aviso puede decirte cada vez que un script o un agente captura la pantalla. Las capturas hechas así nunca se envían a ningún sitio por sí solas.`,
  },
  video: {
    title: "Video",
    body: `La grabación de pantalla llega en una próxima versión de XcreenShot. Esta versión captura imágenes.`,
  },
  share: {
    title: "Compartir",
    body: `Compartir con otros servicios llega en una próxima versión. Hoy puedes:
- copiar la imagen terminada con **Copiar todo** (\`Ctrl+Shift+C\`);
- exportar un PNG o un JPG (\`Ctrl+Alt+Shift+E\`);
- arrastrar un elemento de la bandeja reciente o de la biblioteca a otra app o a una carpeta.`,
  },
};

export function helpPage(lang: string, id: (typeof TOPICS)[number]): HelpPage {
  const pages = lang.startsWith("es") ? es : en;
  return { id, ...pages[id] };
}

/** Pieces of one line: plain text, **bold** and `keys`. */
export type Piece = { kind: "text" | "bold" | "keys"; text: string };

export function pieces(line: string): Piece[] {
  const out: Piece[] = [];
  const re = /\*\*([^*]+)\*\*|`([^`]+)`/g;
  let at = 0;
  for (let m = re.exec(line); m; m = re.exec(line)) {
    if (m.index > at) out.push({ kind: "text", text: line.slice(at, m.index) });
    out.push(
      m[1] !== undefined ? { kind: "bold", text: m[1] } : { kind: "keys", text: m[2] ?? "" },
    );
    at = m.index + m[0].length;
  }
  if (at < line.length) out.push({ kind: "text", text: line.slice(at) });
  return out;
}

/** Blocks of a page: subtitles, paragraphs and lists. */
export type Block =
  { kind: "h2"; text: string } | { kind: "p"; text: string } | { kind: "ul"; items: string[] };

export function blocks(body: string): Block[] {
  const out: Block[] = [];
  for (const chunk of body.split(/\n\s*\n/)) {
    const lines = chunk.split("\n").filter((l) => l.trim());
    let list: string[] | null = null;
    for (const line of lines) {
      if (line.startsWith("## ")) {
        list = null;
        out.push({ kind: "h2", text: line.slice(3) });
      } else if (line.startsWith("- ")) {
        if (!list) {
          list = [];
          out.push({ kind: "ul", items: list });
        }
        list.push(line.slice(2));
      } else {
        list = null;
        out.push({ kind: "p", text: line });
      }
    }
  }
  return out;
}
