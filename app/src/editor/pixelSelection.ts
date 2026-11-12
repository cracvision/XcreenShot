// Pixel selection (EDT-043 to EDT-046): what the editor knows of it and the commands
// it runs. The selection itself lives in the core; `document.info` reports its outline
// in canvas pixels.
import { isCommandError, run } from "../lib/commands";
import type { Point } from "./geometry";
import { useEditor, type PixelSelectionInfo } from "./state";

export type SelectMode = "replace" | "add" | "subtract";

/** Whether `p` (canvas pixels) is inside the selection's outline: its loops count
 * even-odd, so a hole is outside. */
export function insideSelection(sel: PixelSelectionInfo, p: Point): boolean {
  let inside = false;
  for (const loop of sel.outline) {
    for (let i = 0, j = loop.length - 1; i < loop.length; j = i++) {
      const a = loop[i];
      const b = loop[j];
      if (!a || !b) continue;
      if (a[1] > p[1] !== b[1] > p[1]) {
        const x = ((b[0] - a[0]) * (p[1] - a[1])) / (b[1] - a[1]) + a[0];
        if (p[0] < x) inside = !inside;
      }
    }
  }
  return inside;
}

/** How a press joins the selection (EDT-043): with one active, Shift adds and Alt
 * subtracts. */
export function modeOf(active: boolean, e: { shiftKey: boolean; altKey: boolean }): SelectMode {
  if (!active) return "replace";
  if (e.shiftKey) return "add";
  if (e.altKey) return "subtract";
  return "replace";
}

/** Runs `document.select_pixels` on the document shown. */
export async function selectPixels(params: Record<string, unknown>): Promise<void> {
  const { doc, showError } = useEditor.getState();
  if (!doc) return;
  try {
    await run("document.select_pixels", { documentId: doc.documentId, ...params });
  } catch (e) {
    showError(e);
  }
}

/** Runs an action on the selected pixels (EDT-044), filling the gap as the tool that
 * selected says (EDT-045). */
export async function selectionAction(
  action: "delete" | "cut" | "copy" | "move" | "float" | "crop",
  extra: Record<string, unknown> = {},
): Promise<{ changed: boolean; ids?: string[] } | null> {
  const { doc, tool, styles, showError } = useEditor.getState();
  if (!doc?.pixelSelection) return null;
  const style = tool === "wand" ? styles?.wand : styles?.select;
  const gapFill = style?.gapFill ?? "smart";
  const quality = style?.quality ?? "fast";
  try {
    return await run<{ changed: boolean; ids?: string[] }>("document.selection_action", {
      documentId: doc.documentId,
      action,
      gapFill,
      quality,
      ...extra,
    });
  } catch (e) {
    // Stopped with Esc: nothing changed, nothing to say (EDT-045).
    if (!(isCommandError(e) && e.code === "cancelled")) showError(e);
    return null;
  }
}
