# XcreenShot for agents

XcreenShot is a screen capture, annotation and recording app for Windows and macOS.
Every action is a **command** with a stable ID, typed parameters and a typed result.
Agents and scripts drive the running app through the `xcreenshot` CLI or its MCP
server (`xcreenshot mcp`, stdio only; see "MCP" below).

> Status: milestone M2 (editor, library and recent tray). Capture, the full editor,
> the library with tags, search, details, quick view, import and export, and the tray
> work; sharing arrives with M6. The command list is generated from the command
> registry of this version, so it always matches the installed app.

Get this guide with `xcreenshot agents-md` (it prints it), from the MCP resource
`xcreenshot://agents`, or as `AGENTS.md` in the installation folder.

## Connecting

```sh
xcreenshot app ping --json
```

- The CLI talks to the app over a local channel only: a named pipe on Windows, a Unix
  domain socket on macOS, both restricted to the current user. **No network port is
  ever opened.**
- If the app is not running, the CLI starts it in the background (no windows). Use
  `--no-launch` to fail instead (exit code 3).
- At startup the app reopens the documents that were in the editor's strip when it
  last ended (EDT-120, setting `editor.reopenDocuments`). `document.*` and `editor.*`
  commands wait for that, up to 10 s, so `document.list` right after a launch already
  lists them.
- `xcreenshot <area> <action> [positional…] [--param value…]` runs `<area>.<action>`.
  `-` may replace `_` in action names. Parameter names are kebab-case
  (`resetCustom` → `--reset-custom`).
- `xcreenshot run <id> --params '<json>'` runs any command by full ID; `--params -`
  reads JSON from standard input.
- `xcreenshot help <id>` shows a command's parameters.

### Global options

| Option | Effect |
|---|---|
| `--json` | Print exactly one JSON object on stdout: `{"ok": true, "result": …}` or `{"ok": false, "error": {"code", "message", "details"}}`. Progress and warnings go to stderr. |
| `--quiet` | Print nothing on success. |
| `--timeout <s>` | Give up after this many seconds (exit 8). Default: no limit. |
| `--no-launch` | Do not start the app. |
| `--lang en\|es` | Language of human-readable messages. Machine fields are always English. |
| `--yes` | Required by commands that change or delete data in bulk. |

### Exit codes

| Code | Meaning | Error codes |
|---|---|---|
| 0 | Success | — |
| 1 | General or internal error | `internal` |
| 2 | Bad usage | `invalid_params`, `unknown_command` |
| 3 | App not running and could not start | `not_running` |
| 4 | Cancelled by the user | `cancelled` |
| 5 | Busy | `busy` |
| 6 | Blocked: missing OS permission, automation off, policy, or a feature that is off or not set up | `permission_denied`, `disabled_by_policy`, `feature_disabled`, `not_supported_on_platform` |
| 7 | Not found | `not_found` |
| 8 | Timed out | `timeout` |

## Commands

<!-- commands -->

Shortcut combinations are written like `Ctrl+Alt+Shift+K`, `Cmd+Shift+5`, `PrtScn`.
They are stored by physical key (`Ctrl+Alt+Shift+KeyK`), so they survive keyboard
layout changes.

## Capture options

The commands that take a capture (`fullscreen`, `region`, `window`, `universal`, `image`, `capture`) accept:

| Option | Default from the CLI or MCP | Effect |
|---|---|---|
| `--copy true\|false` | `true` | Copy the image to the clipboard (PNG and bitmap). |
| `--open editor\|none` | `none` | Show the capture in XcreenShot. Agents never open windows unless they ask. |
| `--out <path>` | — | Also write a PNG there (folders are created). |
| `--delay <s>` | `0` | Wait before capturing (0–60). The app shows a countdown; Esc or its Cancel button ends the capture with `cancelled`. |
| `--cursor true` | `false` | Include the pointer, with its real shape: a floating image object of the document (move or delete it with `document.update_object` / `document.delete_object`), laid flat on the image copied to the clipboard and on `--out`. Not on macOS yet (reported in `warnings`). |

`universal`, `image` and `capture` start from the capture panel settings (read them with `capture.settings`; Open in is usually `editor`) and these options override them, so pass `--open none` to keep windows closed. `fullscreen`, `region` and `window` always use the defaults above when called from the CLI or MCP.

Every capture is saved in the library as an editable `.xshot` document, in a folder per month: `Documents\XcreenShot\2026-10\Chrome 2026-10-12 14.03.22.xshot` on Windows, `~/Pictures/XcreenShot/…` on macOS. The result has `status` (`completed`), `documentId` (for the `document.*` commands), `path` (the `.xshot`, or the `--out` file when given), `files`, `width`, `height`, `rect`, `copied`, `metadata` (capture time, app and version, window title, OS, display scale…) and `warnings` (for example `blank:` when the system returned only black pixels, as with protected content). A second capture while one is running fails at once with `busy` (exit 5).

## Documents and objects

Coordinates and sizes are image pixels, with the origin at the top-left corner of the
canvas (after any crop). `documentId` defaults to the document shown in the editor.

The canvas grows to fit objects that stick out of it (`editor.autoExpandCanvas`, on by
default) and shrinks back when they return, never below the image or the last crop. When
it grows to the left or the top, every object's coordinates shift with the image, so
read them again with `document.info` (`crop` is then partly outside the original pixels,
and `frame` is the crop the user chose). Blurs, and objects a crop left outside
(`outside`), do not make it grow.

An arrow:

```json
{"type": "arrow", "from": [40, 40], "to": [200, 120],
 "style": {"color": "#F2545B", "width": 6, "startCap": "none", "endCap": "filled",
           "lineStyle": "solid", "opacity": 100}}
```

| Style property | Values |
|---|---|
| `color` | `#RRGGBB` or `#RRGGBBAA` |
| `width` | 1–100 px, decimals allowed |
| `startCap`, `endCap` | `none`, `filled`, `open`, `thin`, `concave`, `circle`, `circleHollow`, `square`, `diamond`, `bar` |
| `lineStyle` | `solid`, `dash`, `dot`, `dashDot` |
| `opacity` | 0–100 % |

A text box. `at` is its top-left corner; `width` wraps the text at that many pixels,
or `null` makes the box grow with the text. `rotation` is in degrees, clockwise, around
the box center. `document.info` adds the laid-out size as `layout`.

```json
{"type": "text", "at": [40, 40], "width": null, "text": "Click here", "rotation": 0,
 "style": {"font": "Segoe UI", "size": 14, "bold": false, "italic": false,
           "color": "#F2545B", "align": "left", "opacity": 100}}
```

| Style property | Values |
|---|---|
| `font` | A family from `editor.fonts`. One that is not installed is drawn with a fallback and kept. |
| `size` | 1–1000 points; pixels = points × the document's dpi / 72 |
| `bold`, `italic` | `true`, `false` |
| `color` | `#RRGGBB` or `#RRGGBBAA` |
| `align` | `left`, `center`, `right`, `justify` |
| `opacity` | 0–100 % |

A shape and a blur. `rect` is `{"x", "y", "w", "h"}`; `rotation` turns it clockwise
around its center.

```json
{"type": "shape", "rect": {"x": 40, "y": 40, "w": 200, "h": 120}, "rotation": 0,
 "style": {"shapeType": "rectangle", "fill": null, "outline": "#F2545B",
           "outlineWidth": 4, "lineStyle": "solid", "opacity": 100}}
{"type": "blur", "rect": {"x": 300, "y": 40, "w": 200, "h": 40}, "rotation": 0,
 "style": {"blurType": "solid", "color": "#000000", "intensity": 60, "shapeType": "rectangle"}}
```

| Style property | Values |
|---|---|
| `shapeType` | `rectangle`, `ellipse` |
| `fill` | `#RRGGBB(AA)`, or `null` for no fill |
| `outline`, `outlineWidth` | Outline color, and its thickness 0–100 px (0 = none) |
| `blurType` | `blur`, `pixelate`, or `solid` (an opaque color: the one to use for confidential data) |
| `intensity` | 1–100, for `blur` and `pixelate` |
| `color` | The `solid` color |

Exported images and copies never keep what a blur hides; a `.xshot` keeps the original
pixels under it.

A callout (EDT-028): a text box with a body and tails. `tails` are the points the tips
reach (up to 8; without `tails`, one is added below the body). With `fitText` the body
grows to show all its text.

```json
{"type": "callout", "rect": {"x": 40, "y": 40, "w": 220, "h": 70}, "rotation": 0,
 "text": "Click here", "tails": [[90, 190]],
 "style": {"shape": "roundedRectangle", "fill": "#FFFFFF", "outline": "#F2545B",
           "outlineWidth": 3, "cornerRadius": 12, "tailStyle": "straight", "tailWidth": 24}}
```

| Style property | Values |
|---|---|
| `shape` | `rectangle`, `roundedRectangle`, `ellipse`, `cloud`, `thought`, `explosion`, `tag`, `arrow`, `text` |
| `fill`, `shading` | `#RRGGBB(AA)` or `null`; a soft gradient to a darker tone |
| `outline`, `outlineWidth`, `lineStyle` | 0–20 px; `solid`, `dash`, `dot`, `dashDot` |
| `cornerRadius` | 0–200 px |
| `tailStyle`, `tailWidth`, `tailColor` | `straight`, `curved`, `arrow`; 2–200 px; `null` = the outline's |
| `font`, `size`, `bold`, `italic`, `textColor`, `align`, `valign`, `padding` | As text; `valign` `top`, `middle`, `bottom`; padding 0–100 px |
| `fitText`, `opacity` | `true`/`false`; 0–100 % |

A step marker (EDT-035): `at` (its center) or a square `rect`, and `value` (up to 6
characters; without it, the next of its sequence). Style: `sequence` (`number`,
`upper`, `lower`, `romanUpper`, `romanLower`), `shape` (`circle`, `square`,
`roundedSquare`, `diamond`, `marker`, `text`), `fill`, `textColor`, `outline`,
`outlineWidth` (0–10), `size` (16–256), `opacity`.

Missing style properties take the tool's current style (`editor.state`). Unknown
properties or types fail with `invalid_params`. A `line` is stored like an arrow (`from`, `to`, the
same style) and its tool starts without ends (EDT-025). A `highlight` (EDT-036) is a
`rect` with `rotation` and a style of `color`, `opacity` (default 50) and `shape`
(`rectangle`, `roundedRectangle`, `ellipse`); it multiplies its color into what is
below, so dark text stays dark. A `pen` stroke (EDT-037) is a list of `points`, smoothed
when it is made by `style.smooth` (0–100 %); its style has `color`, `width` (1–100),
`opacity`, `tip` (`circle`, `square`, `slash`, `bar`), `antialias` and `editable`
(with `false` it is painted into the image: a permanent flattened layer). A `magnify`
lens (EDT-038) shows its zone `source` (`{x, y, w, h}`) enlarged by `style.zoom`
(110–800 %) around `at` (default: the zone's center; elsewhere, the zone keeps an outline
and `connectors` join them); style `shape` (`circle`, `square`, `rectangle`,
`roundedRectangle`), `outline`, `outlineWidth` (0–20), `opacity`. A `spotlight`
(EDT-039) is a `rect` with `rotation` and `shape` (`rectangle`, `roundedRectangle`,
`ellipse`); everything outside the zones of all spotlights changes once, with their
shared `color`, `opacity` (default 60), `blur` (0–100) and `feather` (0–50 px): a change
to one passes to all. A `stamp` (EDT-031) names one of `editor.stamps` (`stamp`, default
the tool's), with `at` (its center) or `rect` (its height counts; the width follows the
stamp); the document keeps its drawing (`art`), so it shows the same anywhere. Setting
another `stamp` keeps its place and height. Style: `color` (`null` = its own) and
`opacity`. A `simplify` shape (EDT-041) is a `rect` with `rotation`; style `kind`
(`block`, `roundedBlock`, `textLine`, `paragraph`, `circle`, `button`, `icon`), `fill`
(transparency allowed), `background` (text lines and paragraphs), `textHeight`,
`lineSpacing`, `opacity`. The tool's style also keeps `palettes` and the `palette` in use.

`.xshot` is a ZIP with `manifest.json`, `document.json`, `base.png` (the original
pixels, never cropped), `meta.json` and `thumb.png`. The schema of `document.json` is
in `docs/formats/xshot-document.schema.json`.

## Recipes

Capture a window, annotate it and export a PNG, without opening any window:

```sh
id=$(xcreenshot capture window --target active --copy false --json | jq -r .result.documentId)
xcreenshot document annotate --document-id "$id" \
  --params '{"objects":[{"type":"arrow","from":[40,40],"to":[200,120],"style":{"color":"#F2545B","width":6}},{"type":"text","at":[210,110],"text":"Click here"}]}'
xcreenshot document export --document-id "$id" --format png --path C:/temp/annotated.png
```

Search the library (by words, tag, app, dates) and open what it finds:

```sh
xcreenshot library search "invoice" --tag "Client A" --app chrome --from 2026-11-01 --json
xcreenshot tray open <id>        # puts it in the editor's recent tray
xcreenshot document image --document-id <documentId> --max-size 1568 --json
```

Capture the active window to a file, without touching the clipboard:

```sh
xcreenshot capture window --target active --copy false --out C:/temp/window.png --json
```

Check that the app answers and automation is on:

```sh
xcreenshot app ping --json
# {"ok":true,"result":{"version":"0.1.0","platform":"windows",…,"automationEnabled":true}}
```

Switch to dark theme and Spanish:

```sh
xcreenshot settings set general.theme dark
xcreenshot settings set general.language es
```

Preview and apply a shortcut profile:

```sh
xcreenshot shortcuts set-profile compatibility --dry-run --json
xcreenshot shortcuts set-profile compatibility
```

Recording (`record.*`) and text recognition (`smart.*`) arrive in later milestones;
their recipes come with them.

## MCP (APP-029)

`xcreenshot mcp` speaks JSON-RPC over standard input and output, one message per line;
the client starts it as a child process (Preferences > Advanced > Copy, or
`xcreenshot app mcp-config --json`). Nothing listens on the network.

- **Tools.** Every command marked for MCP, named with `_` for `.` (`capture.fullscreen`
  → `capture_fullscreen`), with its description, input schema and hints (read-only,
  destructive, network). The cloud AI commands are not tools.
- **Images.** A tool whose result has a `documentId` (captures, `document_open`,
  `document_new`…) also takes `include_image: true`: the result then carries the final
  image, at most 1568 px on its longest side, as image content the agent can see.
  `document_image` gives it for any open document.
- **Resources** (read-only): `xcreenshot://agents` (this guide), `xcreenshot://library/recent`
  (the 20 most recent captures), `xcreenshot://tray` (the recent tray) and
  `xcreenshot://events` (the latest 50 events).
- **Events.** `resources/subscribe` to `xcreenshot://events`: each event of
  `events.watch` (captures, files written, app state) arrives as
  `notifications/resources/updated`; `resources/read` gives the latest ones.
  `resources/unsubscribe` stops them.

## Safety rules (APP-032)

- **Local only.** Same-user local channel; no network port, not even localhost.
- **Master switch.** Preferences > Advanced > "Allow control from the CLI and agents"
  (`automation.enabled`). When off, every CLI/MCP command fails with
  `disabled_by_policy` (exit 6) except `app.ping`, which reports
  `"automationEnabled": false`.
- **What an agent cannot do:** change `automation.*` settings (they can only be
  changed in the Preferences window), or capture/record without the normal visible
  indicators.
- **Logging.** Every CLI/MCP command is logged locally with time, command ID and
  outcome — never its parameters. Logs never leave the machine.
- **Cloud AI (SMT-039 to SMT-042).** Off by default; only the Preferences window can
  turn it on (`ai.enabled`) or save the API key. Agents over MCP cannot send anything
  to the cloud through XcreenShot: `ai.ask` and `ai.suggest_name` are UI and CLI
  only. An agent that wants to understand an image uses `document.image` and its own
  model (SMT-043). Logs keep the model, tokens and outcome of AI requests, never the
  image, the request, the answer or the key.
