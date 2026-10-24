# XcreenShot for agents

XcreenShot is a screen capture, annotation and recording app for Windows and macOS.
Every action is a **command** with a stable ID, typed parameters and a typed result.
Agents and scripts drive the running app through the `xcreenshot` CLI. An MCP server
(`xcreenshot mcp`, stdio only) arrives in milestone M2.

> Status: milestone M0 (foundations). Capture, editing, library and sharing commands
> arrive from M1. This file is hand-written for now; from M2 it is generated from the
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
| `app.quit` | `xcreenshot app quit` | Quits the app. |
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

Every capture is also kept in the library folder (`Documents\XcreenShot` on Windows, `~/Pictures/XcreenShot` on macOS). The result has `status` (`completed`), `path`, `files`, `width`, `height`, `rect`, `copied`, `metadata` (capture time, app and version, window title, OS, display scale…) and `warnings`. A second capture while one is running fails at once with `busy` (exit 5).

## Recipes

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
