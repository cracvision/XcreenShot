// Dragging tray and library items out to other apps, the desktop or a folder
// (TRY-015). The drag starts after the pointer moves a few pixels with the button
// down; the files are prepared (a `.xshot` or unsaved changes become a flat PNG) and
// then handed to the system's drag and drop.
import { invoke } from "@tauri-apps/api/core";
import i18next from "i18next";
import type { PointerEvent as ReactPointerEvent } from "react";
import { isCommandError, run } from "../lib/commands";
import { useEditor } from "../editor/state";
import { dismissNotice, notify } from "./actions";

const t = (key: string, options: Record<string, unknown> = {}) => i18next.t(key, options);

/** Pixels the pointer moves with the button down before a drag starts. */
export const DRAG_THRESHOLD = 4;
/** After this long preparing, a notice says so (TRY-015). */
const SLOW_MS = 1000;

/** What a drag carries: open documents by ID and files by path. */
export interface DragTarget {
  docs: string[];
  paths: string[];
}

let active = false;
let carrying: DragTarget | null = null;

/** A drag out is under way: drops on XcreenShot's own window are its own files. */
export function draggingOut(): boolean {
  return active;
}

/** What the drag under way carries (TRY-014: an item is not dropped on itself). */
export function dragged(): DragTarget | null {
  return active ? carrying : null;
}

/** Whether the pointer went far enough from where the button went down. */
export function pastThreshold(dx: number, dy: number): boolean {
  return Math.hypot(dx, dy) > DRAG_THRESHOLD;
}

/** A place where a drag moves the item instead of taking it out (TRY-005): the tray. */
export interface DragInside {
  area: () => DOMRect | null;
  /** The pointer is at `x` inside the area. */
  over: (x: number) => void;
  /** Let go at `x` inside the area. */
  drop: (x: number) => void;
  /** The drag left the area (or was cancelled) without a drop. */
  leave: () => void;
}

function within(r: DOMRect | null, x: number, y: number): boolean {
  return r !== null && x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
}

/**
 * Watches a press on an item: if the pointer moves past the threshold with the
 * button down, the drag starts with what `target()` returns then, and the click
 * that ends the press does nothing else. With `inside`, while the pointer stays in
 * that area the drag moves the item there; leaving it starts the drag out.
 */
export function watchDrag(
  e: ReactPointerEvent,
  target: () => DragTarget | null,
  inside?: DragInside,
): void {
  if (e.button !== 0 || active) return;
  const start = { x: e.clientX, y: e.clientY };
  const id = e.pointerId;
  let moving = false;
  const out = () => {
    stop();
    const what = target();
    if (!what || what.docs.length + what.paths.length === 0) return;
    swallowClick();
    void dragOut(what);
  };
  const move = (m: PointerEvent) => {
    if (m.pointerId !== id) return;
    if (!moving && !pastThreshold(m.clientX - start.x, m.clientY - start.y)) return;
    if (inside && within(inside.area(), m.clientX, m.clientY)) {
      moving = true;
      inside.over(m.clientX);
      return;
    }
    if (moving) inside?.leave();
    out();
  };
  const up = (u: PointerEvent) => {
    stop();
    if (moving && inside) {
      swallowClick();
      inside.drop(u.clientX);
    }
  };
  const cancel = () => {
    stop();
    if (moving) inside?.leave();
  };
  const stop = () => {
    window.removeEventListener("pointermove", move, true);
    window.removeEventListener("pointerup", up, true);
    window.removeEventListener("pointercancel", cancel, true);
  };
  window.addEventListener("pointermove", move, true);
  window.addEventListener("pointerup", up, true);
  window.addEventListener("pointercancel", cancel, true);
}

/** The click that may follow a drag is not a click on the item. */
function swallowClick(): void {
  const eat = (c: MouseEvent) => {
    c.stopPropagation();
    c.preventDefault();
  };
  window.addEventListener("click", eat, { capture: true, once: true });
  setTimeout(() => window.removeEventListener("click", eat, true), 600);
}

/** Copies the one image of the drag, for when the drag could not happen. */
async function copyInstead(what: DragTarget): Promise<void> {
  try {
    let id = what.docs[0];
    if (!id && what.paths[0]) {
      const opened = await run<{ documentId: string }>("document.open", {
        path: what.paths[0],
        show: false,
      });
      id = opened.documentId;
    }
    if (!id) return;
    await run("document.copy_all", { documentId: id });
    dismissNotice();
    useEditor.getState().showToast(t("ui.editor.copied"));
  } catch (e) {
    useEditor.getState().showError(e);
  }
}

/** Prepares the files and drags them out until they are dropped or cancelled. */
export async function dragOut(what: DragTarget): Promise<void> {
  active = true;
  carrying = what;
  let released = false;
  const up = () => (released = true);
  window.addEventListener("pointerup", up, true);
  document.body.classList.add("preparing-drag");
  const count = what.docs.length + what.paths.length;
  let slow: number | undefined;
  const timer = setTimeout(() => {
    slow = notify(t("ui.files.drag_preparing", { count }));
  }, SLOW_MS);
  try {
    const prepared = await invoke<{ files: string[]; image: string | null }>("xs_drag_prepare", {
      docs: what.docs,
      paths: what.paths,
    });
    clearTimeout(timer);
    if (slow !== undefined) dismissNotice(slow);
    document.body.classList.remove("preparing-drag");
    window.removeEventListener("pointerup", up, true);
    // The button came up while the files were being prepared: the other app did not
    // wait (TRY-015). Copy all is the way to hand over one image.
    if (released) {
      notify(
        t("ui.files.drag_too_soon"),
        count === 1 ? { label: t("ui.editor.copy_all"), run: () => copyInstead(what) } : undefined,
      );
      return;
    }
    await invoke("xs_drag_start", { files: prepared.files, image: prepared.image });
  } catch (e) {
    clearTimeout(timer);
    const reason = isCommandError(e) ? e.message : String(e);
    notify(t("ui.files.drag_failed", { reason }));
  } finally {
    clearTimeout(timer);
    window.removeEventListener("pointerup", up, true);
    document.body.classList.remove("preparing-drag");
    // The window may still hear about the drop a moment after it happened.
    setTimeout(() => (active = false), 500);
  }
}
