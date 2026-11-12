// Editor keys. The assignable ones come from Preferences > Shortcuts, scope Editor
// (EDT-115, EDT-116); the canvas keys are fixed (EDT-013, EDT-117). They only act on
// the editor window and never while typing in a field (EDT-116).
import { useEffect } from "react";
import { fromEvent } from "../lib/accelerator";
import { run, type ShortcutRow } from "../lib/commands";
import { setting, useApp } from "../store";
import {
  applyCrop,
  performAction,
  resetCrop,
  selectTool,
  startCalloutEdit,
  startNewText,
  startTextEdit,
} from "./actions";
import { moved } from "./objects";
import { selectPixels } from "./pixelSelection";
import { isSegment, requestZoom, useEditor, usePointer, type Tool } from "./state";
import { useMainView } from "../library/state";
import { inToolbar } from "../components/arrowFocus";
import { toggleFullscreen, useFullscreen } from "./fullscreen";
import { barTools, useToolbar } from "./toolbarLayout";
import { useZones } from "./templateUi";

const isMac = navigator.userAgent.includes("Mac");

/** Number pad keys work like the main ones (`Ctrl`+numpad `+` zooms in). */
const NUMPAD: Record<string, string> = {
  NumpadAdd: "Equal",
  NumpadSubtract: "Minus",
  ...Object.fromEntries([..."0123456789"].map((d) => [`Numpad${d}`, `Digit${d}`])),
};

/** The editor shortcut row of a key press, if any. */
function editorRow(e: KeyboardEvent): ShortcutRow | null {
  const { shortcuts, recording } = useApp.getState();
  if (recording || !shortcuts) return null;
  // Not `{ ...e }`: the modifier flags are getters of the event, not its own fields.
  const accel = fromEvent({
    code: NUMPAD[e.code] ?? e.code,
    ctrlKey: e.ctrlKey,
    altKey: e.altKey,
    shiftKey: e.shiftKey,
    metaKey: e.metaKey,
  });
  if (!accel) return null;
  return shortcuts.rows.find((r) => r.scope === "editor" && r.shortcut === accel) ?? null;
}
/** Holding V longer than this is a temporary Move (EDT-042). */
const HOLD_MS = 300;
/** Size of objects inserted from the keyboard (EDT-013), in canvas pixels. */
const INSERT = { w: 200, h: 120 };

/** Keys pressed in a menu or the menu bar are the menu's: arrows, Enter, Escape (EDT-002). */
function inMenu(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest('[role="menu"], [role="menubar"]') !== null;
}

/** The library grid and the recent tray have their own keys: arrows, Enter, Delete
 * (LIB-002, TRY-013). */
function ownKeys(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest(".strip, .library-grid") !== null;
}

function typing(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  );
}

const onCanvas = () => document.activeElement?.classList.contains("canvas-stage") ?? false;

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

/** The zones `F6` goes through, in order (EDT-013). */
const ZONES = [
  ".editor-toolbar",
  ".canvas-stage",
  ".library-grid",
  ".side-panel",
  ".strip",
  ".status-bar",
];

/** `F6` / `Shift+F6`: the focus to the next or previous zone of the window. */
function nextZone(back: boolean): void {
  const zones = ZONES.map((z) => document.querySelector<HTMLElement>(z)).filter(
    (z): z is HTMLElement => z !== null,
  );
  if (zones.length === 0) return;
  const at = zones.findIndex((z) => z.contains(document.activeElement));
  const next = zones[(at + (back ? zones.length - 1 : 1) + zones.length) % zones.length];
  if (!next) return;
  // A list with one stop (the tray, the grid) is entered at that stop.
  const target =
    next.tabIndex >= 0
      ? next
      : (next.querySelector<HTMLElement>('[tabindex="0"]') ??
        next.querySelector<HTMLElement>(
          "button:not(:disabled), input, select, [tabindex]:not([tabindex='-1'])",
        ));
  target?.focus();
}

/** Arrow keys move the selection 1 px, 10 with Shift (EDT-087, EDT-117). */
function nudge(dx: number, dy: number): void {
  const { active, doc, selection } = useEditor.getState();
  if (!active || !doc || selection.length === 0) return;
  if (selection.length > 1) {
    // Several selected move together, in one undo step (EDT-079, EDT-087).
    run("document.update_object", {
      documentId: active,
      objectIds: selection,
      move: [dx, dy],
    }).catch(fail);
    return;
  }
  const o = doc.objects.find((x) => x.id === selection[0]);
  if (!o) return;
  const [, set] = moved(o, dx, dy);
  run("document.update_object", { documentId: active, objectId: o.id, set }).catch(fail);
}

/** Tab / Shift+Tab: the next or previous object, in stacking order (EDT-013). */
function cycle(back: boolean): void {
  const { doc, selection, setSelection } = useEditor.getState();
  if (!doc || doc.objects.length === 0) return;
  const ids = doc.objects.map((o) => o.id);
  const at = selection[0] ? ids.indexOf(selection[0]) : -1;
  const next = back ? (at <= 0 ? ids.length - 1 : at - 1) : (at + 1) % ids.length;
  const id = ids[next];
  if (id) setSelection([id]);
}

/** The canvas point at the middle of what the editor shows. */
function viewCenter(): [number, number] | null {
  const { active, views } = useEditor.getState();
  const v = active ? views[active] : undefined;
  return v ? [v.cx, v.cy] : null;
}

/** Ctrl+Enter inserts an object of the active tool in the middle of the view, to
 * annotate without a mouse (EDT-013). */
function insert(): void {
  const { active, tool, setSelection } = useEditor.getState();
  const c = viewCenter();
  if (!active || !c) return;
  if (tool === "text") {
    startNewText([c[0] - INSERT.w / 2, c[1]], null);
    return;
  }
  const object =
    tool === "arrow" || tool === "line"
      ? { type: tool, from: [c[0] - INSERT.w / 2, c[1]], to: [c[0] + INSERT.w / 2, c[1]] }
      : tool === "shape" || tool === "blur" || tool === "highlight" || tool === "spotlight"
        ? { type: tool, rect: { x: c[0] - INSERT.w / 2, y: c[1] - INSERT.h / 2, ...INSERT } }
        : tool === "step"
          ? { type: "step", at: c }
          : tool === "stamp"
            ? { type: "stamp", at: c }
            : tool === "magnify"
              ? { type: "magnify", source: { x: c[0] - 30, y: c[1] - 30, w: 60, h: 60 } }
              : tool === "callout"
                ? { type: "callout", rect: { x: c[0] - 100, y: c[1] - 60, w: 200, h: 60 } }
                : null;
  if (!object) return;
  run<{ ids: string[] }>("document.annotate", { documentId: active, objects: [object] })
    .then((r) => r.ids[0] && setSelection([r.ids[0]]))
    .catch(fail);
}

/** The style property a picked color goes to, per tool (EDT-117 `I`). */
const COLOR_KEY: Partial<Record<Tool, string>> = {
  arrow: "color",
  line: "color",
  text: "color",
  shape: "outline",
  step: "fill",
  callout: "fill",
  highlight: "color",
  pen: "color",
  magnify: "outline",
  spotlight: "color",
  stamp: "color",
  simplify: "fill",
  blur: "color",
};

/** `I`: takes the color under the pointer for the active tool or the selection. */
function pickCanvasColor(): void {
  const { screen } = usePointer.getState();
  const { tool, selection, active, doc } = useEditor.getState();
  const canvas = document.querySelector<HTMLCanvasElement>(".canvas-stage canvas");
  if (!screen || !canvas || !active) return;
  const ratio = canvas.width / canvas.clientWidth;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  let data: Uint8ClampedArray;
  try {
    data = ctx.getImageData(Math.floor(screen.x * ratio), Math.floor(screen.y * ratio), 1, 1).data;
  } catch {
    return;
  }
  // Outside the image the canvas is empty: nothing to take.
  if ((data[3] ?? 0) === 0) return;
  const hex =
    `#${[data[0], data[1], data[2]].map((v) => (v ?? 0).toString(16).padStart(2, "0")).join("")}`.toUpperCase();
  const selected = doc?.objects.find((o) => o.id === selection[0]);
  const kind = (selected?.type ?? tool) as Tool;
  const key = COLOR_KEY[kind];
  if (!key) return;
  const style = { [key]: hex, ...(kind === "blur" ? { blurType: "solid" } : {}) };
  const p = selected
    ? run("document.update_object", { documentId: active, objectIds: selection, set: { style } })
    : run("editor.set_style", { tool: kind, style });
  p.catch(fail);
}

/** `[` and `]`: thinner or thicker arrows (EDT-117), 10 % a step. */
function resizeStroke(up: boolean): void {
  const { tool, selection, active, doc, styles } = useEditor.getState();
  const arrows = (doc?.objects ?? []).filter((o) => isSegment(o) && selection.includes(o.id));
  const width = (w: number) =>
    Math.min(100, Math.max(1, Math.round(w * (up ? 1.1 : 1 / 1.1) * 10) / 10));
  if (arrows.length > 0 && active) {
    const first = arrows[0];
    if (!first || !isSegment(first)) return;
    run("document.update_object", {
      documentId: active,
      objectIds: arrows.map((a) => a.id),
      set: { style: { width: width(first.style.width) } },
    }).catch(fail);
  } else if ((tool === "arrow" || tool === "line") && styles) {
    run("editor.set_style", { tool, style: { width: width(styles[tool].width) } }).catch(fail);
  } else if (tool === "eraser" && styles) {
    // EDT-048: steps of 10 %, 1 to 200 px.
    const w = styles.eraser.width;
    const next = Math.min(200, Math.max(1, Math.round(w * (up ? 1.1 : 1 / 1.1) * 10) / 10));
    run("editor.set_style", { tool, style: { width: next } }).catch(fail);
  }
}

export function useEditorKeys(): void {
  useEffect(() => {
    // A tool key held down: Move or the tool is temporary (EDT-042).
    let held: { code: string; at: number; previous: Tool } | null = null;

    /** Runs an editor shortcut; false when it does not apply now. */
    const runRow = (row: ShortcutRow, e: KeyboardEvent): boolean => {
      const params = row.params as { action?: string; tool?: Tool; mode?: string; value?: number };
      switch (row.runs) {
        case "editor.action":
          return performAction(params.action ?? "");
        case "editor.select_tool": {
          const tool = params.tool;
          if (!tool) return false;
          if (!e.repeat && tool === "move") {
            held = { code: e.code, at: Date.now(), previous: useEditor.getState().tool };
          }
          if (!e.repeat) selectTool(tool);
          return true;
        }
        case "editor.zoom":
          requestZoom(params.mode ?? "actual", params.value ?? null);
          return true;
        default:
          run(row.runs, row.params, "shortcut").catch(() => undefined);
          return true;
      }
    };

    const down = (e: KeyboardEvent) => {
      if (e.code === "F6" && !e.ctrlKey && !e.altKey && !e.metaKey) {
        if (document.querySelector(".backdrop")) return;
        e.preventDefault();
        nextZone(e.shiftKey);
        return;
      }
      if (typing(e.target) || inMenu(e.target) || document.querySelector(".backdrop")) return;
      const done = () => {
        e.preventDefault();
        // Before the app-wide shortcuts (Ctrl+W there closes the window).
        e.stopImmediatePropagation();
      };
      const row = editorRow(e);
      // In the Library view, the grid and the tray only shortcuts with a modifier or a
      // function key run (Ctrl+Delete, F2…): plain keys belong to the list.
      if (useMainView.getState().view === "library" || ownKeys(e.target)) {
        const modified = e.ctrlKey || e.altKey || e.metaKey || /^F\d+$/.test(e.code);
        if (row && modified && row.runs === "editor.action" && runRow(row, e)) done();
        return;
      }
      if (row) {
        if (runRow(row, e)) done();
        return;
      }
      const primary = isMac ? e.metaKey : e.ctrlKey;
      const other = isMac ? e.ctrlKey : e.metaKey;
      const { shiftKey: shift, altKey: alt } = e;
      const state = useEditor.getState();
      let handled = true;
      if (primary && !other && !alt && !shift) {
        const digit = /^(?:Digit|Numpad)([1-9])$/.exec(e.code);
        if (digit) {
          // Ctrl+1…9: the tool in that place of the bar (EDT-115).
          const onBar = barTools(useToolbar.getState().data?.items ?? []);
          const tool = (onBar.length ? onBar : state.tools)[Number(digit[1]) - 1];
          if (tool) selectTool(tool);
        } else if (e.code === "Enter" || e.code === "NumpadEnter") {
          insert();
        } else {
          handled = false;
        }
      } else if (!primary && !other && !alt) {
        const step = shift ? 10 : 1;
        switch (e.code) {
          case "Escape":
            // Cancels the crop, else deselects (EDT-117); with nothing to cancel, closes
            // the editor if the user asked for it (EDT-012). Documents stay open.
            if (state.fillProgress !== null) run("editor.cancel").catch(() => undefined);
            // Full screen goes first, before closing the editor (EDT-009).
            else if (useFullscreen.getState().on) void toggleFullscreen();
            // Ends the framing of a template's zone (EDT-098).
            else if (useZones.getState().framing) useZones.setState({ framing: null });
            else if (state.tool === "crop" && state.cropRect) resetCrop();
            // Leaves the group a double click entered (EDT-082).
            else if (state.entered.length > 0) useEditor.setState({ entered: [] });
            else if (state.doc?.pixelSelection) void selectPixels({ shape: "none" });
            else if (state.selection.length > 0) state.setSelection([]);
            else if (setting("editor.escClosesEditor", false))
              run("app.close_window").catch(() => undefined);
            else handled = false;
            break;
          case "Enter":
          case "NumpadEnter": {
            // Applies the crop; on a selected text, types into it (EDT-117).
            const selected = state.doc?.objects.find((o) => o.id === state.selection[0]);
            if (state.tool === "crop") void applyCrop();
            else if (selected?.type === "text") startTextEdit(selected);
            else if (selected?.type === "callout") startCalloutEdit(selected);
            else handled = false;
            break;
          }
          case "Tab":
            if (onCanvas()) cycle(shift);
            else handled = false;
            break;
          case "KeyI":
            if (shift) handled = false;
            else pickCanvasColor();
            break;
          case "BracketLeft":
            resizeStroke(false);
            break;
          case "BracketRight":
            resizeStroke(true);
            break;
          // In a toolbar the arrows move between its buttons (APP-043).
          case "ArrowLeft":
            if (inToolbar(e.target)) handled = false;
            else nudge(-step, 0);
            break;
          case "ArrowRight":
            if (inToolbar(e.target)) handled = false;
            else nudge(step, 0);
            break;
          case "ArrowUp":
            if (inToolbar(e.target)) handled = false;
            else nudge(0, -step);
            break;
          case "ArrowDown":
            if (inToolbar(e.target)) handled = false;
            else nudge(0, step);
            break;
          default:
            handled = false;
        }
      } else {
        handled = false;
      }
      if (handled) done();
    };

    const up = (e: KeyboardEvent) => {
      if (!held || e.code !== held.code) return;
      // Held: Move was temporary, the previous tool comes back.
      if (Date.now() - held.at > HOLD_MS && held.previous !== "move") selectTool(held.previous);
      held = null;
    };

    window.addEventListener("keydown", down, true);
    window.addEventListener("keyup", up, true);
    return () => {
      window.removeEventListener("keydown", down, true);
      window.removeEventListener("keyup", up, true);
    };
  }, []);
}
