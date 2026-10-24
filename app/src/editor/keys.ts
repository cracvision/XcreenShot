// Editor keys of build 4a (EDT-115, EDT-116, EDT-117), with the Coexistence values.
// They become assignable shortcuts with the rest of the table in build 4c. They only
// act on the editor window and never while typing in a field (EDT-116).
import { useEffect } from "react";
import { run } from "../lib/commands";
import {
  closeAll,
  closeDocument,
  copyAll,
  deleteSelection,
  exportImage,
  openFiles,
  redo,
  saveActive,
  saveAs,
  selectTool,
  undo,
} from "./actions";
import { requestZoom, useEditor, usePanel, type ArrowObject, type Tool } from "./state";

const isMac = navigator.userAgent.includes("Mac");
/** Holding V longer than this is a temporary Move (EDT-042). */
const HOLD_MS = 300;

function typing(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  );
}

function nudge(dx: number, dy: number): void {
  const { active, doc, selection, showError } = useEditor.getState();
  if (!active || !doc || selection.length !== 1) return;
  const a = doc.objects.find((o): o is ArrowObject => o.type === "arrow" && o.id === selection[0]);
  if (!a) return;
  run("document.update_object", {
    documentId: active,
    objectId: a.id,
    set: { from: [a.from[0] + dx, a.from[1] + dy], to: [a.to[0] + dx, a.to[1] + dy] },
  }).catch(showError);
}

export function useEditorKeys(): void {
  useEffect(() => {
    let vDown: { at: number; previous: Tool } | null = null;

    const down = (e: KeyboardEvent) => {
      if (typing(e.target) || document.querySelector(".backdrop")) return;
      const primary = isMac ? e.metaKey : e.ctrlKey;
      const other = isMac ? e.ctrlKey : e.metaKey;
      const { shiftKey: shift, altKey: alt } = e;
      let handled = true;
      if (primary && !other) {
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
          case "KeyP":
            // Show or hide the side panel (EDT-005, EDT-116).
            if (alt && !shift) usePanel.getState().set({ visible: !usePanel.getState().visible });
            else handled = false;
            break;
          case "KeyC":
            if (shift && !alt) void copyAll();
            else handled = false;
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
        switch (e.code) {
          case "Delete":
          case "Backspace":
            deleteSelection();
            break;
          case "Escape":
            if (useEditor.getState().selection.length > 0) useEditor.getState().setSelection([]);
            else handled = false;
            break;
          case "KeyA":
            if (!shift) selectTool("arrow");
            else handled = false;
            break;
          case "KeyV":
            if (shift || e.repeat) break;
            vDown = { at: Date.now(), previous: useEditor.getState().tool };
            selectTool("move");
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
