# XcreenShot for agents

XcreenShot is a screen capture, annotation and recording app for Windows and macOS.
Every action is a **command** with a stable ID, typed parameters and a typed result.
Agents and scripts drive the running app through the `xcreenshot` CLI. An MCP server
(`xcreenshot mcp`, stdio only) arrives in milestone M2.

> Status: milestone M1, build 4b. Capture works, and documents can be opened,
> annotated with arrows, text, shapes and blur, cropped, saved and exported
> (`document.*`, `editor.*`). Menus, the library and sharing arrive with the next builds. This file is hand-written for now; from M2 it is generated from the
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

| ID | CLI | What it does |
|---|---|---|
| `app.ping` | `xcreenshot app ping` | Version, platform, architecture, state, whether automation is on, UI language. |
| `app.commands` | `xcreenshot app commands --json` | Every command with JSON Schemas for params and result, scopes, and flags (`mutates`, `network`, `destructive`, `interactive`). |
| `app.show` | `xcreenshot app show` | Shows the main window. |
| `app.close_window` | (UI and shortcuts only) | Closes the focused XcreenShot window. |
| `app.quit` | `xcreenshot app quit [--force]` | Quits the app. If documents opened from outside the library have unsaved changes, nothing quits: the result is `status: needs_decision` with them in `pending`, and the app asks the user. `--force` quits anyway; those changes stay recoverable (`editor.recovery_list`). |
| `app.diagnostics` | `xcreenshot app diagnostics [--copy]` | Version, OS, monitors, permissions, shortcut conflicts, last 200 log lines. No user content. |
| `events.watch` | `xcreenshot events watch [--filter capture.*,file.written]` | Prints one JSON line per event until interrupted (CLI only): `capture.started`, `capture.succeeded`, `capture.cancelled`, `capture.failed`, `file.written` (path, type, size) and `app.state_changed` (`capturing` / `idle`). Each has `event`, `time`, `source` (`ui`, `shortcut`, `cli`, `mcp`) and its own data; never image content. Includes what the user does in the interface. |
| `app.welcome` | `xcreenshot app welcome` | Shows the welcome window of the first run again. |
| `app.help` | `xcreenshot app help [--topic capture]` | Opens the Help window (offline pages) at a topic: `start`, `capture`, `editor`, `library`, `shortcuts`, `permissions`, `automation`, `video`, `share` or `about`. |
| `app.about` | `xcreenshot app about --json` | Version, build and its date, architecture, channel, data and settings folders, and the only network connections the app makes. `--open true` also shows it. |
| `app.feedback` | `xcreenshot app feedback` | Opens a feedback draft in the browser with the version and the system filled in; returns its `url`. Nothing is sent by XcreenShot. |
| `app.open_logs` | `xcreenshot app open-logs` | Opens the logs folder. |
| `app.open_data_folder` | `xcreenshot app open-data-folder [--open false]` | Opens the app's local data folder; returns it (`data`) and the settings folder (`config`). |
| `app.mcp_config` | `xcreenshot app mcp-config --json` | How an MCP client starts XcreenShot: the installed CLI (`command`), `args` (`["mcp"]`), a JSON block for the client's configuration (`config`) and the command that adds it to Claude Code (`claudeCode`). `not_found` if the CLI is not installed next to the app. |
| `app.permissions` | `xcreenshot app permissions --json` | OS permission states (macOS: screen recording, accessibility, camera, microphone, notifications; Windows: whether the system blocks XcreenShot's notifications). |
| `app.request_permission` | `xcreenshot app request-permission <permission>` | Shows the OS request, or opens System Settings when the OS no longer asks. |
| `app.open_permission_settings` | `xcreenshot app open-permission-settings <permission>` | Opens the system settings page for a permission (macOS; on Windows, `notifications`, `camera` and `microphone`). |
| `settings.open` | `xcreenshot settings open [--section <id>] [--key <key>]` | Opens Preferences, optionally at a section or setting. |
| `settings.get` | `xcreenshot settings get <key>` | Reads a setting. |
| `settings.set` | `xcreenshot settings set <key> <value>` | Changes a setting. Values are parsed as JSON when possible (`true`, `5`), else as text. |
| `settings.reset` | `xcreenshot settings reset [--section capture] --yes` | Puts a section's settings back to their defaults; without `--section`, every setting, the shortcut profile and every shortcut. Captures, the library and saved styles are never touched. Destructive: needs `--yes`. |
| `app.hints` | (UI only) | Which one-time hints were seen; `seen` marks one. |
| `app.warnings` | `xcreenshot app warnings [--reset true]` | How many warnings were answered with "Don't ask again" and how many hints were seen (`hidden`); with `--reset true`, shows them all again. |
| `settings.list` | `xcreenshot settings list --json` | Every setting with type, default, current value and lock state. |
| `settings.schema` | `xcreenshot settings schema --json` | Setting definitions. |
| `shortcuts.list` | `xcreenshot shortcuts list --json` | Every assignable shortcut, its combination and status. |
| `shortcuts.set` | `xcreenshot shortcuts set <command> <combination> [--scope global\|app\|editor] [--reassign]` | Assigns a shortcut; an empty combination removes it. Editor shortcuts (`editor.*` rows) work in the editor window and win there over an app shortcut with the same keys. |
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
| `capture.multi_start` | `xcreenshot capture multi-start` | Starts multi-capture (Windows): until Esc, every click captures the window under the pointer as it looked just before the click and then reaches the app; a drag captures a rectangle (it does not reach the app); clicks on XcreenShot's windows and the taskbar are left alone. Each capture is saved in the library (not opened, not copied) and listed in the multi-capture panel. Other captures answer `busy` meanwhile. Returns the state. |
| `capture.multi_stop` | `xcreenshot capture multi-stop [--close true]` | Stops capturing clicks. The panel stays with its captures; `--close true` also closes it and forgets the session (the captures stay in the library). |
| `capture.multi_state` | `xcreenshot capture multi-state --json` | `capturing` and the session's `items` (`id`, library `path`, `width`, `height`), oldest first. |
| `capture.multi_remove` | `xcreenshot capture multi-remove --ids '["m1"]'` | Takes captures out of the panel; they stay in the library. An unknown id fails with `not_found` and removes nothing. |
| `capture.multi_clear` | `xcreenshot capture multi-clear` | Empties the panel; the captures stay in the library. |
| `capture.settings` | `xcreenshot capture settings --json` | The capture panel settings: active `tab`, `expanded`, and the `universal` and `image` tabs. |
| `capture.configure` | `xcreenshot capture configure --params '{"image":{"selection":"window"}}'` | Changes some panel settings and keeps the rest. Invalid values fail with `invalid_params` and change nothing. |
| `capture.reset_settings` | `xcreenshot capture reset-settings --yes` | Returns the Universal and Image tabs to their defaults (presets, shortcuts and preferences stay). |
| `document.open` | `xcreenshot document open <path> [--show true]` | Opens a `.xshot`, PNG or JPG file as a document and returns its `documentId`. Opening a file that is already open returns the same id. Without `--show` no window opens. |
| `document.list` | `xcreenshot document list --json` | Open documents in order, with `active` (the one in the editor), `dirty` (unsaved changes), `kind` (`library` or `external`) and size. |
| `document.info` | `xcreenshot document info [--document-id <id>] --json` | Size, resolution (`dpi`), crop, objects with their geometry and style, capture metadata, `dirty`, and the undo and redo state. |
| `document.annotate` | `xcreenshot document annotate --document-id <id> --params '{"objects":[…]}'` | Adds objects (`arrow`, `line`, `text`, `callout`, `shape`, `step`, `highlight`, `blur`), in image pixels. All or none: an invalid object fails the whole command with `invalid_params` and `details.index` (1-based). One undo step. |
| `document.arrange` | `xcreenshot document arrange --document-id <id> --object-ids '["o1"]' --to back` | Stacking order: `front`, `forward` (one place up), `backward`, `back`. Several keep their order. One undo step. |
| `document.steps` | `xcreenshot document steps --document-id <id> restart` · `renumber` | Step markers (`type: "step"`, EDT-035): `document.annotate` places one with `at` (its center) or `rect`, and without `value` it takes the next of its sequence (`style.sequence`: number, upper, lower, romanUpper, romanLower). `restart` makes the next one the first value; `renumber` numbers them again by the order they were made (one undo step). `document.info` shows the next value of each sequence in `stepNext`. |
| `document.flatten` | `xcreenshot document flatten --document-id <id> --object-ids '["o1"]'` · `--all true` | Merges objects into the image, in one undo step: they are no longer objects. `document.info` lists them in `flattened` (layers with their objects in base pixels and their `bounds` on the canvas). Returns the new `layers`: each blur gets a layer of its own that is `permanent` (when the file is written it merges into `base.png`, with the earlier layers it covers); the objects between blurs can come back. |
| `document.unflatten` | `xcreenshot document unflatten --document-id <id> --layer-id f1` · `--all true` | Makes a flattened layer (or all of them) objects again, under the others, also after saving and reopening. A `permanent` layer cannot. |
| `document.transform` | `xcreenshot document transform --document-id <id> --object-ids '["o1","o2"]' --scale '{"factor":1.5}'` · `--rotate 90` · `--equal width` | Changes several objects as one, in one undo step: `scale` stretches them from the box around them (or `from`) to `to`, or by `factor`; `strokes` (default: `editor.scaleStrokes`) decides whether line widths and text sizes follow. `rotate` turns them around the center of that box (or `around`). `equal` gives them the same `width` or `height` (default: the largest); pictures keep their proportions and arrows do not change. |
| `document.group` | `xcreenshot document group --document-id <id> --object-ids '["o1","o2"]' [--ungroup true]` | Puts objects in a new group (around the groups they were in), or takes them out of their outermost group. Objects carry `groups`, outermost first. One undo step. |
| `document.duplicate` | `xcreenshot document duplicate --document-id <id> --object-ids '["o1"]' --offset '[10,10]'` | Copies of the objects, moved by `offset` (default 10 px right and down), on top of the others. Returns their `ids`. One undo step. |
| `document.update_object` | `xcreenshot document update-object --document-id <id> --object-id o1 --set '{"to":[300,120]}'` | Changes the geometry or style of one or more objects (`--object-ids`). Style properties not given keep their value. `--move '[dx,dy]'` moves them all by that many pixels, each from where it is; `--moves '{"o1":[dx,dy],"o2":[dx,dy]}'` moves each by its own amount. One undo step. |
| `document.delete_object` | `xcreenshot document delete-object --document-id <id> --object-id o1` | Deletes one or more objects. One undo step. |
| `document.undo` | `xcreenshot document undo [--document-id <id>]` | Undoes the last change; `label` names it. At least 100 steps per document, kept while it is open. |
| `document.redo` | `xcreenshot document redo [--document-id <id>]` | Redoes the last undone change. |
| `document.save` | `xcreenshot document save [--document-id <id>] [--path <file> [--format xshot\|png\|jpg]]` | Saves in the document's own file. With `--path`: a copy for library documents (they stay linked to their `.xshot`), or the new linked file for documents opened from elsewhere. Saving a PNG or JPG flattens the objects into it. |
| `document.export` | `xcreenshot document export --format png\|jpg --path <file> [--quality 1-100] [--on-conflict keepBoth\|replace\|fail]` | Writes the final image (pixels, objects, crop) without changing the document's file. PNG keeps transparency and declares the resolution and sRGB; JPG fills transparency with white (quality 90). An existing file is never overwritten unless `--on-conflict replace`: by default the copy gets ` (2)`. |
| `document.crop` | `xcreenshot document crop --rect '{"x":10,"y":10,"w":400,"h":300}'` | Crops to a rectangle in canvas pixels. Non-destructive: the original pixels stay in the `.xshot`, and a later crop may reach past the canvas, up to the original edges (or the canvas grown for objects). Objects keep their place on the image; those it leaves outside are listed in `outside` and stay there. One undo step. `--trim` arrives in M2. |
| `document.copy_objects` | `xcreenshot document copy-objects --object-ids '["o1","o2"]' [--cut true]` | Copies objects: editable for a later `document.paste`, and as a PNG with transparency on the clipboard for other apps. `--cut true` also deletes them (one undo step). |
| `document.insert_images` | `xcreenshot document insert-images --paths '["C:/…/capture.xshot"]' [--document-id <id>] [--center '[400,300]']` | Inserts library documents (their final image) or PNG/JPG files as floating images of the document shown, at their size, the first centered on `--center` (default: the canvas center) and the next ones 24 px further each. One undo step; the canvas grows to fit them (EDT-071). With nothing open, one `.xshot` opens as itself and several make a new library document with them stacked 10 px apart (`created: true`). Floating images are moved, resized and turned like shapes (`rect`, `rotation`) and have `style.opacity`. |
| `document.rename` | `xcreenshot document rename --name "Quarterly sales chart" [--document-id <id>]` | Renames the file of an open document in its folder, keeping its extension. Invalid characters become spaces. Fails with `invalid_params` if another file has that name or the document has no file yet. Returns `path` and `name`. |
| `document.locate` | `xcreenshot document locate --path <file> [--document-id <id>]` | Links an open document to its file after it was moved or renamed outside XcreenShot (the strip showed it as not found). |
| `document.image` | `xcreenshot document image [--document-id <id>] [--max-size 1568] --json` | The final image of a document (with its objects) as a PNG in base64 (`image.mimeType`, `image.data`), scaled down to `maxSize` px on its longest side (default 1568, at most 2576). Over MCP it arrives as an image the agent can see. Read-only; nothing leaves the computer. |
| `document.paste` | `xcreenshot document paste [--in-place true] [--center '[400,300]']` | Pastes the clipboard: objects copied in XcreenShot (shifted 10 px if the originals are still there, centered on `--center` otherwise, or at their place with `--in-place`), plain text as a text object, or an image from another app as a floating image centered on `--center` (as a new library document when nothing is open). Returns `pasted` (`objects`, `text`, `image`, `document`) and the new `ids`. |
| `document.copy_all` | `xcreenshot document copy-all [--document-id <id>]` | Puts the final image on the clipboard (PNG and bitmap). The document keeps its objects. |
| `document.close` | `xcreenshot document close [--document-id <id> \| --document-ids '["d1","d2"]' \| --all true] [--save <ids>] [--discard <ids>]` | Closes documents. Library documents are saved first. Documents from elsewhere with unsaved changes must be listed in `save` or `discard`; otherwise nothing closes and the result is `status: needs_decision` with them in `pending`. Files are never deleted. |
| `editor.show` | `xcreenshot editor show [--view editor\|library] [--document-id <id>]` | Shows the main window in the Editor view (or the Library view with `--view library`), optionally with that document in front. The document shown before is saved if it is in the library. |
| `editor.state` | `xcreenshot editor state --json` | Active tool, available tools, the current style of each tool, the open documents and how many recoverable items there are. |
| `editor.fonts` | `xcreenshot editor fonts --json` | Font families installed on this computer, for the `font` of text objects. |
| `editor.select_tool` | `xcreenshot editor select-tool <move\|arrow\|line\|text\|callout\|shape\|step\|highlight\|blur\|crop>` | Makes a tool active. |
| `editor.action` | `xcreenshot editor action <open\|save\|save_as\|export\|close\|close_all\|undo\|redo\|cut\|copy\|paste\|paste_in_place\|copy_all\|delete\|deselect\|side_panel\|toolbar_labels\|recover\|ai_ask\|ai_name\|view_editor\|view_library\|tray\|prev_item\|next_item\|close_others\|rename_item\|show_in_folder\|delete_item>` | Does in the editor window what its menu entry does, with the window's selection, view and dialogs. Menus and editor shortcuts run it. To change documents without the window, use the `document.*` commands. |
| `editor.menu_state` | (UI only) | The editor window reports which menu entries can be used now and which are ticked. |
| `editor.set_style` | `xcreenshot editor set-style --tool arrow --style '{"color":"#2D7FF9","width":4}'` | Changes the tool's current style: what the next objects drawn by the user or added without a style look like. |
| `editor.zoom` | `xcreenshot editor zoom --mode actual\|fit\|original\|in\|out\|percent [--value 250]` | Changes the editor zoom (percent of physical pixels, 10–3200). The result is the expected zoom, or `null` for `fit` and `original`, which the window works out. |
| `editor.view` | (UI only) | The editor reports the zoom and position of the document it shows, so they are remembered. |
| `editor.recovery_list` | `xcreenshot editor recovery-list --json` | Documents with unsaved changes kept from a session that ended unexpectedly (recovery copies are written every 30 s). |
| `editor.recovery_restore` | `xcreenshot editor recovery-restore --ids '["<id>"]'` | Opens recoverable documents with their changes, linked to their original files and still unsaved. |
| `editor.recovery_discard` | `xcreenshot editor recovery-discard --ids '["<id>"]' --yes` | Deletes recovery copies for good. |
| `library.list` | `xcreenshot library list [--offset 0] [--limit 50] --json` | Library items, newest capture first: stable `id`, `name`, `path`, `format`, `captured`, `width`, `height`, `app`. Also `root` (the library folder), `synced` (kept by a sync service) and `total`. |
| `library.get` | `xcreenshot library get <id> --json` | One library item. `not_found` if the ID is not in the library. |
| `library.delete` | `xcreenshot library delete --ids '["<id>"]' --yes [--permanent true]` | Moves items to the system's trash and closes them in the editor (unsaved changes are lost). `--paths '["<file>"]'` names files instead, also files outside the library. Returns `undo`, valid for 30 s with `library.restore`. Without a trash (some network folders) it fails with `not_supported_on_platform` and `details.reason: no_trash`; `--permanent true` deletes for good. |
| `library.restore` | `xcreenshot library restore --undo <undo>` | Undoes a `library.delete` while it lasts: the files come back with their IDs, and the ones that were open reopen. |
| `library.rename` | `xcreenshot library rename --id <id> --name "New name"` | Renames an item's file in its folder; the ID stays. For an open document, `document.rename`. |
| `library.show_in_folder` | `xcreenshot library show-in-folder [--id <id> \| --path <file>]` | Opens Explorer or Finder with the file selected; without arguments, the library folder. |
| `ai.key_set` | (UI only) | Saves the Claude API key in the system keychain. The key never goes through the CLI or an agent. |
| `ai.key_clear` | `xcreenshot ai key-clear --yes` | Removes the Claude API key from the system keychain. |
| `ai.key_status` | `xcreenshot ai key-status --json` | Whether a key is saved (`saved`) and its last four characters (`hint`). Not over MCP. |
| `ai.key_test` | `xcreenshot ai key-test [--model claude-opus-5-5]` | Checks the key and the model with the Claude API, without sending any image. Goes out to the network. |
| `ai.ask` | `xcreenshot ai ask --prompt "Describe this image" [--document-id <id>] [--model claude-haiku-5-5] [--history '[{"prompt":…,"answer":…}]'] --json` | Sends the final image of a document (at most 2576 px on its longest side) and the request to Claude with the user's own key, and returns `text`, `model`, `stopReason` and `usage` (`inputTokens`, `outputTokens`). Needs cloud AI turned on in Preferences > AI (`feature_disabled` otherwise). Goes out to the network; not over MCP. One request at a time (`busy`). |
| `ai.suggest_name` | `xcreenshot ai suggest-name [--document-id <id>] [--avoid '["Earlier name"]'] --json` | Asks Claude (the naming model, cheap by default) for a short descriptive file name for the image; returns `name`. Does not rename: use `document.rename`. Same conditions as `ai.ask`. |
| `ai.open_console` | `xcreenshot ai open-console` | Opens the page where Claude API keys are created, in the browser. |
| `ai.cancel` | `xcreenshot ai cancel` | Stops the AI request in progress (`cancelled: true` if there was one). |

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
below, so dark text stays dark. Other object types arrive later in M2.

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
- **Cloud AI (SMT-039 to SMT-042).** Off by default; only the Preferences window can
  turn it on (`ai.enabled`) or save the API key. Agents over MCP cannot send anything
  to the cloud through XcreenShot: `ai.ask` and `ai.suggest_name` are UI and CLI
  only. An agent that wants to understand an image uses `document.image` and its own
  model (SMT-043). Logs keep the model, tokens and outcome of AI requests, never the
  image, the request, the answer or the key.
