# XcreenShot for agents

XcreenShot is a screen capture, annotation and recording app for Windows and macOS.
Every action is a **command** with a stable ID, typed parameters and a typed result.
Agents and scripts drive the running app through the `xcreenshot` CLI. An MCP server
(`xcreenshot mcp`, stdio only) arrives in milestone M2.

> Status: milestone M1, build 4a. Capture works, and documents can be opened,
> annotated with arrows, saved and exported (`document.*`, `editor.*`). More tools,
> the library and sharing arrive with the next builds. This file is hand-written for now; from M2 it is generated from the
> command registry (APP-033). A test fails if a registered command is missing here.

## Connecting

```sh
xcreenshot app ping --json
```

- The CLI talks to the app over a local channel only: a named pipe on Windows, a Unix
  domain socket on macOS, both restricted to the current user. **No network port is
  ever opened.**
- If the app is not running, the CLI starts it in the background (no windows). Use
  `--no-launch` to fail instead (exit code 3).
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
| 6 | Blocked: missing OS permission, automation off, or policy | `permission_denied`, `disabled_by_policy`, `not_supported_on_platform` |
| 7 | Not found | `not_found` |
| 8 | Timed out | `timeout` |

## Commands

| ID | CLI | What it does |
|---|---|---|
| `app.ping` | `xcreenshot app ping` | Version, platform, architecture, state, whether automation is on, UI language. |
| `app.commands` | `xcreenshot app commands --json` | Every command with JSON Schemas for params and result, scopes, and flags (`mutates`, `network`, `destructive`, `interactive`). |
| `app.show` | `xcreenshot app show` | Shows the main window. |
| `app.close_window` | (UI and shortcuts only) | Closes the focused XcreenShot window. |
| `app.quit` | `xcreenshot app quit [--force]` | Quits the app. If documents opened from outside the library have unsaved changes, nothing quits: the result is `status: needs_decision` with them in `pending`, and the app asks the user. `--force` quits anyway; those changes stay recoverable (`editor.recovery_list`). |
| `app.diagnostics` | `xcreenshot app diagnostics [--copy]` | Version, OS, monitors, permissions, shortcut conflicts, last 200 log lines. No user content. |
| `app.open_logs` | `xcreenshot app open-logs` | Opens the logs folder. |
| `app.permissions` | `xcreenshot app permissions --json` | OS permission states (macOS: screen recording, accessibility, camera, microphone). |
| `app.request_permission` | `xcreenshot app request-permission <permission>` | Shows the OS request, or opens System Settings when the OS no longer asks. |
| `app.open_permission_settings` | `xcreenshot app open-permission-settings <permission>` | Opens the System Settings page for a permission (macOS). |
| `settings.open` | `xcreenshot settings open [--section <id>] [--key <key>]` | Opens Preferences, optionally at a section or setting. |
| `settings.get` | `xcreenshot settings get <key>` | Reads a setting. |
| `settings.set` | `xcreenshot settings set <key> <value>` | Changes a setting. Values are parsed as JSON when possible (`true`, `5`), else as text. |
| `settings.list` | `xcreenshot settings list --json` | Every setting with type, default, current value and lock state. |
| `settings.schema` | `xcreenshot settings schema --json` | Setting definitions. |
| `shortcuts.list` | `xcreenshot shortcuts list --json` | Every assignable shortcut, its combination and status. |
| `shortcuts.set` | `xcreenshot shortcuts set <command> <combination> [--scope global\|app] [--reassign]` | Assigns a shortcut; an empty combination removes it. |
| `shortcuts.reset` | `xcreenshot shortcuts reset [<command>] [--scope …] --yes` | Restores one or all shortcuts to the profile default. |
| `shortcuts.pause` | `xcreenshot shortcuts pause [--paused true\|false]` | Pauses or resumes global shortcuts (toggles without a value). |
| `shortcuts.set_profile` | `xcreenshot shortcuts set-profile <coexistence\|compatibility> [--reset-custom] [--dry-run]` | Switches the shortcut profile; `--dry-run` lists the changes. |
| `shortcuts.retry` | `xcreenshot shortcuts retry` | Retries global shortcuts that failed to register. |
| `shortcuts.open_system_settings` | `xcreenshot shortcuts open-system-settings` | Opens the OS keyboard shortcut settings. |
| `shortcuts.recording` | (UI only) | Suspends global shortcuts while the user records one. |
| `capture.monitors` | `xcreenshot capture monitors --json` | Monitors with position and size in physical pixels, scale, name and which is primary. |
| `capture.fullscreen` | `xcreenshot capture fullscreen [--monitor pointer\|primary\|all\|<name>]` | Captures a monitor (under the pointer by default), the primary one, or the whole desktop (gaps between monitors are transparent). |
| `capture.universal` | `xcreenshot capture universal` | Interactive, with the Universal tab settings: freezes the screen and shows the crosshair; the user drags a region, clicks a window or presses F, then Capture (`Enter`). Esc cancels (exit 4, `cancelled`). |
| `capture.region` | `xcreenshot capture region --rect '{"x":0,"y":0,"width":800,"height":600}'` | Captures a rectangle in physical desktop pixels. Without `--rect` the user selects on screen (interactive; Esc gives exit 4). `--target interactive` on `capture.window` lets the user click a window. |
| `capture.window` | `xcreenshot capture window [--target active\|point\|title] [--x N --y N] [--title text]` | Captures a whole window, even if covered: the active one, the one at a point, or the first whose title or process matches. |
| `capture.capture` | `xcreenshot capture capture` | Captures with the settings of the capture panel's active tab (the "Capture" button and shortcut). May be interactive. |
| `capture.image` | `xcreenshot capture image` | Captures with the Image tab settings, whatever tab is active. May be interactive. |
| `capture.repeat_last` | `xcreenshot capture repeat-last` | Runs the last capture again with the same mode and settings. With none yet it opens the capture panel and returns `status: none`. |
| `capture.repeat_last_area` | `xcreenshot capture repeat-last-area` | Captures at once, without the crosshair, the rectangle of the last region or window capture (a window still open is taken where it is now). With none yet it opens the capture panel and returns `status: none`. |
| `capture.panel` | `xcreenshot capture panel [--show true\|false]` | Shows or hides the capture panel; without `--show` it toggles. Returns `visible`. |
| `capture.settings` | `xcreenshot capture settings --json` | The capture panel settings: active `tab`, `expanded`, and the `universal` and `image` tabs. |
| `capture.configure` | `xcreenshot capture configure --params '{"image":{"selection":"window"}}'` | Changes some panel settings and keeps the rest. Invalid values fail with `invalid_params` and change nothing. |
| `capture.reset_settings` | `xcreenshot capture reset-settings --yes` | Returns the Universal and Image tabs to their defaults (presets, shortcuts and preferences stay). |
| `document.open` | `xcreenshot document open <path> [--show true]` | Opens a `.xshot`, PNG or JPG file as a document and returns its `documentId`. Opening a file that is already open returns the same id. Without `--show` no window opens. |
| `document.list` | `xcreenshot document list --json` | Open documents in order, with `active` (the one in the editor), `dirty` (unsaved changes), `kind` (`library` or `external`) and size. |
| `document.info` | `xcreenshot document info [--document-id <id>] --json` | Size, resolution (`dpi`), crop, objects with their geometry and style, capture metadata, `dirty`, and the undo and redo state. |
| `document.annotate` | `xcreenshot document annotate --document-id <id> --params '{"objects":[…]}'` | Adds objects (arrows in this build), in image pixels. All or none: an invalid object fails the whole command with `invalid_params` and `details.index` (1-based). One undo step. |
| `document.update_object` | `xcreenshot document update-object --document-id <id> --object-id o1 --set '{"to":[300,120]}'` | Changes the geometry or style of one or more objects (`--object-ids`). Style properties not given keep their value. One undo step. |
| `document.delete_object` | `xcreenshot document delete-object --document-id <id> --object-id o1` | Deletes one or more objects. One undo step. |
| `document.undo` | `xcreenshot document undo [--document-id <id>]` | Undoes the last change; `label` names it. At least 100 steps per document, kept while it is open. |
| `document.redo` | `xcreenshot document redo [--document-id <id>]` | Redoes the last undone change. |
| `document.save` | `xcreenshot document save [--document-id <id>] [--path <file> [--format xshot\|png\|jpg]]` | Saves in the document's own file. With `--path`: a copy for library documents (they stay linked to their `.xshot`), or the new linked file for documents opened from elsewhere. Saving a PNG or JPG flattens the objects into it. |
| `document.export` | `xcreenshot document export --format png\|jpg --path <file> [--quality 1-100] [--on-conflict keepBoth\|replace\|fail]` | Writes the final image (pixels, objects, crop) without changing the document's file. PNG keeps transparency and declares the resolution and sRGB; JPG fills transparency with white (quality 90). An existing file is never overwritten unless `--on-conflict replace`: by default the copy gets ` (2)`. |
| `document.copy_all` | `xcreenshot document copy-all [--document-id <id>]` | Puts the final image on the clipboard (PNG and bitmap). The document keeps its objects. |
| `document.close` | `xcreenshot document close [--document-id <id> \| --all true] [--save <ids>] [--discard <ids>]` | Closes documents. Library documents are saved first. Documents from elsewhere with unsaved changes must be listed in `save` or `discard`; otherwise nothing closes and the result is `status: needs_decision` with them in `pending`. Files are never deleted. |
| `editor.show` | `xcreenshot editor show [--document-id <id>]` | Shows the editor window, optionally with that document in front. The document shown before is saved if it is in the library. |
| `editor.state` | `xcreenshot editor state --json` | Active tool, available tools, the current style of each tool, the open documents and how many recoverable items there are. |
| `editor.select_tool` | `xcreenshot editor select-tool <move\|arrow>` | Makes a tool active. Tools of later builds (`text`, `shape`, `blur`, `crop`) answer `not_found`. |
| `editor.set_style` | `xcreenshot editor set-style --tool arrow --style '{"color":"#2D7FF9","width":4}'` | Changes the tool's current style: what the next objects drawn by the user or added without a style look like. |
| `editor.zoom` | `xcreenshot editor zoom --mode actual\|fit\|original\|in\|out\|percent [--value 250]` | Changes the editor zoom (percent of physical pixels, 10–3200). The result is the expected zoom, or `null` for `fit` and `original`, which the window works out. |
| `editor.view` | (UI only) | The editor reports the zoom and position of the document it shows, so they are remembered. |
| `editor.recovery_list` | `xcreenshot editor recovery-list --json` | Documents with unsaved changes kept from a session that ended unexpectedly (recovery copies are written every 30 s). |
| `editor.recovery_restore` | `xcreenshot editor recovery-restore --ids '["<id>"]'` | Opens recoverable documents with their changes, linked to their original files and still unsaved. |
| `editor.recovery_discard` | `xcreenshot editor recovery-discard --ids '["<id>"]' --yes` | Deletes recovery copies for good. |

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
| `--cursor true` | `false` | Include the pointer. Not yet: it arrives with the editor and is reported in `warnings`. |

`universal`, `image` and `capture` start from the capture panel settings (read them with `capture.settings`; Open in is usually `editor`) and these options override them, so pass `--open none` to keep windows closed. `fullscreen`, `region` and `window` always use the defaults above when called from the CLI or MCP.

Every capture is saved in the library as an editable `.xshot` document, in a folder per month: `Documents\XcreenShot\2026-10\Chrome 2026-10-12 14.03.22.xshot` on Windows, `~/Pictures/XcreenShot/…` on macOS. The result has `status` (`completed`), `documentId` (for the `document.*` commands), `path` (the `.xshot`, or the `--out` file when given), `files`, `width`, `height`, `rect`, `copied`, `metadata` (capture time, app and version, window title, OS, display scale…) and `warnings`. A second capture while one is running fails at once with `busy` (exit 5).

## Documents and objects

Coordinates and sizes are image pixels, with the origin at the top-left corner of the
canvas (after any crop). `documentId` defaults to the document shown in the editor.

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

Missing style properties take the tool's current style (`editor.state`). Unknown
properties or types fail with `invalid_params`. Other object types (`text`, `shape`,
`blur`…) arrive in build 4b.

`.xshot` is a ZIP with `manifest.json`, `document.json`, `base.png` (the original
pixels, never cropped), `meta.json` and `thumb.png`. The schema of `document.json` is
in `docs/formats/xshot-document.schema.json`.

## Recipes

Capture, annotate and export without opening any window:

```sh
id=$(xcreenshot capture fullscreen --copy false --json | jq -r .result.documentId)
xcreenshot document annotate --document-id "$id" \
  --params '{"objects":[{"type":"arrow","from":[40,40],"to":[200,120],"style":{"color":"#F2545B","width":6}}]}'
xcreenshot document export --document-id "$id" --format png --path C:/temp/annotated.png
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
