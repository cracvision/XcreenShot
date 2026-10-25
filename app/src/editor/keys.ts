// Editor keys (EDT-115, EDT-116, EDT-117), with the Coexistence values. They become
// assignable shortcuts with the rest of the table in build 4c. They only act on the
// editor window and never while typing in a field (EDT-116).
import { useEffect } from "react";
import { run } from "../lib/commands";
import {
  applyCrop,
  closeAll,
  closeDocument,
  copyAll,
  copyObjects,
  cutObjects,
  deleteSelection,
  exportImage,
  openFiles,
  pasteObjects,
  redo,
  resetCrop,
  saveActive,
  saveAs,
  selectTool,
  startNewText,
  startTextEdit,
  undo,
} from "./actions";
import { moved } from "./objects";
import { requestZoom, useEditor, usePanel, usePointer, type Tool } from "./state";
import { TOOL_BY_CODE } from "./tools";

const isMac = navigator.userAgent.includes("Mac");
/** Holding V longer than this is a temporary Move (EDT-042). */
const HOLD_MS = 300;
/** Size of objects inserted from the keyboard (EDT-013), in canvas pixels. */
const INSERT = { w: 200, h: 120 };

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

/** Arrow keys move the selection 1 px, 10 with Shift (EDT-087, EDT-117). */
function nudge(dx: number, dy: number): void {
  const { active, doc, selection } = useEditor.getState();
  if (!active || !doc || selection.length !== 1) return;
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
    tool === "arrow"
      ? { type: "arrow", from: [c[0] - INSERT.w / 2, c[1]], to: [c[0] + INSERT.w / 2, c[1]] }
      : tool === "shape" || tool === "blur"
        ? { type: tool, rect: { x: c[0] - INSERT.w / 2, y: c[1] - INSERT.h / 2, ...INSERT } }
        : null;
  if (!object) return;
  run<{ ids: string[] }>("document.annotate", { documentId: active, objects: [object] })
    .then((r) => r.ids[0] && setSelection([r.ids[0]]))
    .catch(fail);
}

/** The style property a picked color goes to, per tool (EDT-117 `I`). */
const COLOR_KEY: Partial<Record<Tool, string>> = {
  arrow: "color",
  text: "color",
  shape: "outline",
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
  const arrows = (doc?.objects ?? []).filter((o) => o.type === "arrow" && selection.includes(o.id));
  const width = (w: number) =>
    Math.min(100, Math.max(1, Math.round(w * (up ? 1.1 : 1 / 1.1) * 10) / 10));
  if (arrows.length > 0 && active) {
    const first = arrows[0];
    if (first?.type !== "arrow") return;
    run("document.update_object", {
      documentId: active,
      objectIds: arrows.map((a) => a.id),
      set: { style: { width: width(first.style.width) } },
    }).catch(fail);
  } else if (tool === "arrow" && styles) {
    run("editor.set_style", { tool: "arrow", style: { width: width(styles.arrow.width) } }).catch(
      fail,
    );
  }
}

export function useEditorKeys(): void {
  useEffect(() => {
    let vDown: { at: number; previous: Tool } | null = null;

    const down = (e: KeyboardEvent) => {
      if (typing(e.target) || document.querySelector(".backdrop")) return;
      const primary = isMac ? e.metaKey : e.ctrlKey;
      const other = isMac ? e.ctrlKey : e.metaKey;
      const { shiftKey: shift, altKey: alt } = e;
      const state = useEditor.getState();
      let handled = true;
      if (primary && !other) {
        const digit = /^Digit([1-9])$/.exec(e.code);
        if (digit && !alt && !shift) {
          // Ctrl+1…9: the tool in that place of the bar (EDT-115).
          const tool = state.tools[Number(digit[1]) - 1];
          if (tool) selectTool(tool);
          e.preventDefault();
          e.stopImmediatePropagation();
          return;
        }
        switch (e.code) {
          case "KeyZ":
            if (shift) redo();
            else undo();
            break;
          case "KeyY":
            if (!isMac) redo();
            else handled = false;
            break;
          case "KeyS":
            if (alt && shift) void exportImage();
            else if (shift) void saveAs();
            else if (!alt) void saveActive();
            else handled = false;
            break;
          case "KeyO":
            void openFiles();
            break;
          case "KeyW":
            // In the editor, Ctrl+W closes the document, not the window (EDT-108).
            if (alt && !shift) void closeAll();
            else if (!alt && !shift) void closeDocument();
            else handled = false;
            break;
          case "KeyC":
            if (shift && !alt) void copyAll();
            else if (!shift && !alt && state.selection.length > 0) void copyObjects();
            else handled = false;
            break;
          case "KeyX":
            if (!shift && !alt && state.selection.length > 0) void cutObjects();
            else handled = false;
            break;
          case "KeyV":
            if (!alt) void pasteObjects(shift);
            else handled = false;
            break;
          case "KeyP":
            // Show or hide the side panel (EDT-005, EDT-116).
            if (alt && !shift) usePanel.getState().set({ visible: !usePanel.getState().visible });
            else handled = false;
            break;
          case "Enter":
          case "NumpadEnter":
            insert();
            break;
          case "Digit0":
          case "Numpad0":
            requestZoom(alt ? "original" : shift ? "fit" : "percent", alt || shift ? null : 100);
            break;
          case "Equal":
          case "NumpadAdd":
            requestZoom("in");
            break;
          case "Minus":
          case "NumpadSubtract":
            requestZoom("out");
            break;
          default:
            handled = false;
        }
      } else if (!primary && !other && !alt) {
        const step = shift ? 10 : 1;
        const tool = TOOL_BY_CODE[e.code];
        if (tool && tool !== "move" && !shift) {
          selectTool(tool);
        } else {
          switch (e.code) {
            case "Delete":
            case "Backspace":
              deleteSelection();
              break;
            case "Escape":
              // Cancels the crop, else deselects (EDT-117).
              if (state.tool === "crop" && state.cropRect) resetCrop();
              else if (state.selection.length > 0) state.setSelection([]);
              else handled = false;
              break;
            case "Enter":
            case "NumpadEnter": {
              // Applies the crop; on a selected text, types into it (EDT-117).
              const selected = state.doc?.objects.find((o) => o.id === state.selection[0]);
              if (state.tool === "crop") void applyCrop();
              else if (selected?.type === "text") startTextEdit(selected);
              else handled = false;
              break;
            }
            case "Tab":
              if (onCanvas()) cycle(shift);
              else handled = false;
              break;
            case "KeyV":
              if (shift || e.repeat) break;
              vDown = { at: Date.now(), previous: state.tool };
              selectTool("move");
              break;
            case "KeyI":
              pickCanvasColor();
              break;
            case "BracketLeft":
              resizeStroke(false);
              break;
            case "BracketRight":
              resizeStroke(true);
              break;
            case "ArrowLeft":
              nudge(-step, 0);
              break;
            case "ArrowRight":
              nudge(step, 0);
              break;
            case "ArrowUp":
              nudge(0, -step);
              break;
            case "ArrowDown":
              nudge(0, step);
              break;
            default:
              handled = false;
          }
        }
      } else {
        handled = false;
      }
      if (handled) {
        e.preventDefault();
        // Before the app-wide shortcuts (Ctrl+W there closes the window).
        e.stopImmediatePropagation();
      }
    };

    const up = (e: KeyboardEvent) => {
      if (e.code !== "KeyV" || !vDown) return;
      // Held: Move was temporary, the previous tool comes back.
      if (Date.now() - vDown.at > HOLD_MS && vDown.previous !== "move") selectTool(vDown.previous);
      vDown = null;
    };

    window.addEventListener("keydown", down, true);
    window.addEventListener("keyup", up, true);
    return () => {
      window.removeEventListener("keydown", down, true);
      window.removeEventListener("keyup", up, true);
    };
  }, []);
}
