// Separate elements (SMT-018): the image (or a floating image) becomes objects, one per
// button, field, icon or piece of text, to move, resize or delete; the gaps they leave
// are filled with the background. Detail regroups them, Reset puts the image back, and
// Done (or another tool) flattens them into the image: the session is one undo step.
import { create } from "zustand";
import { run } from "../lib/commands";
import { startTextEdit } from "./actions";
import { useEditor, type TextObject } from "./state";

interface SeparateResult {
  pieces?: number;
  needsConfirm?: boolean;
  edited?: number;
  detail?: number;
}

interface SeparateState {
  /** The document in the mode, and the floating image it works on, if any. */
  documentId: string | null;
  objectId: string | null;
  detail: number;
  busy: boolean;
  /** Waiting for "Separate again?": with what detail, how many changes would go. */
  confirm: { detail: number; edited: number } | null;
}

export const DEFAULT_DETAIL = 5;

export const useSeparate = create<SeparateState>(() => ({
  documentId: null,
  objectId: null,
  detail: DEFAULT_DETAIL,
  busy: false,
  confirm: null,
}));

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

async function call(
  command: string,
  params: Record<string, unknown>,
): Promise<SeparateResult | null> {
  useSeparate.setState({ busy: true });
  try {
    return await run<SeparateResult>(command, params);
  } catch (e) {
    fail(e);
    return null;
  } finally {
    useSeparate.setState({ busy: false });
  }
}

/** Starts the mode on the image shown, or on the floating image `objectId`. */
export async function startSeparate(objectId?: string): Promise<void> {
  const doc = useEditor.getState().doc;
  if (!doc || doc.video) return;
  const detail = DEFAULT_DETAIL;
  const r = await call("smart.separate", {
    documentId: doc.documentId,
    detail,
    ...(objectId ? { objectId } : {}),
  });
  if (!r) return;
  useSeparate.setState({
    documentId: doc.documentId,
    objectId: objectId ?? null,
    detail,
    confirm: null,
  });
}

/** A new detail: the objects are found again; with changes made, it asks first. */
export async function setDetail(detail: number, force = false): Promise<void> {
  const { documentId } = useSeparate.getState();
  if (!documentId) return;
  const r = await call("smart.separate", { documentId, detail, ...(force ? { force } : {}) });
  if (r?.needsConfirm) {
    useSeparate.setState({ confirm: { detail, edited: r.edited ?? 0 } });
    return;
  }
  if (r) useSeparate.setState({ detail, confirm: null });
}

export function answerConfirm(again: boolean): Promise<void> {
  const c = useSeparate.getState().confirm;
  useSeparate.setState({ confirm: null });
  return again && c ? setDetail(c.detail, true) : Promise.resolve();
}

function leave(): void {
  useSeparate.setState({ documentId: null, objectId: null, confirm: null });
}

/** Back to the image as it was before separating; the mode ends. */
export async function resetSeparate(): Promise<void> {
  const { documentId } = useSeparate.getState();
  if (!documentId) return;
  await call("smart.separate_reset", { documentId });
  leave();
}

/** Flattens the objects into the image; the whole session is one undo step. */
export async function finishSeparate(): Promise<void> {
  const { documentId } = useSeparate.getState();
  if (!documentId) return;
  leave();
  await call("smart.separate_done", { documentId });
}

/** A piece of text, double-clicked: a Text object to type into (SMT-009). */
export async function separatedText(objectId: string): Promise<boolean> {
  const { documentId } = useSeparate.getState();
  if (!documentId) return false;
  try {
    const r = await run<{ id: string }>("smart.separate_text", { documentId, objectId });
    for (let i = 0; i < 40; i++) {
      const o = useEditor.getState().doc?.objects.find((x) => x.id === r.id);
      if (o?.type === "text") {
        startTextEdit(o as TextObject);
        return true;
      }
      await new Promise((res) => setTimeout(res, 50));
    }
  } catch (e) {
    fail(e);
  }
  return false;
}

/** Another tool or another document ends the mode, as Done does. */
useEditor.subscribe((s, prev) => {
  const { documentId } = useSeparate.getState();
  if (!documentId) return;
  const away = s.tool !== prev.tool && s.tool !== "move";
  const other = s.doc?.documentId !== documentId;
  if (away || other) void finishSeparate();
});
