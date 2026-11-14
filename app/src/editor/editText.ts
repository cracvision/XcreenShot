// Edit text of the image (SMT-009): the text in the pixel selection (or in a place,
// from auto redact's menu) becomes a Text object of the same size and color, ready to
// type. A selection that cuts letters asks first and offers to fit it to the text.
import i18next from "i18next";
import { create } from "zustand";
import { run } from "../lib/commands";
import { startTextEdit } from "./actions";
import { useEditor, type TextObject } from "./state";

type Rect = [number, number, number, number];

interface EditTextResult {
  ids?: string[];
  cut?: boolean;
  fitRect?: Rect;
  rough?: boolean;
}

interface EditTextState {
  /** The selection cuts letters: what to ask again with. */
  ask: { documentId: string; rect?: Rect } | null;
  busy: boolean;
}

export const useEditText = create<EditTextState>(() => ({ ask: null, busy: false }));

async function waitForText(id: string): Promise<TextObject | null> {
  for (let i = 0; i < 40; i++) {
    const o = useEditor.getState().doc?.objects.find((x) => x.id === id);
    if (o?.type === "text") return o;
    await new Promise((r) => setTimeout(r, 50));
  }
  return null;
}

async function send(params: Record<string, unknown>): Promise<void> {
  const ed = useEditor.getState();
  const documentId = (params.documentId as string | undefined) ?? ed.doc?.documentId;
  if (!documentId) return;
  useEditText.setState({ busy: true });
  try {
    const r = await run<EditTextResult>("smart.edit_text", { ...params, documentId });
    if (r.cut) {
      const rect = params.rect as Rect | undefined;
      useEditText.setState({ ask: { documentId, ...(rect ? { rect } : {}) } });
      return;
    }
    if (r.rough) useEditor.getState().showToast(i18next.t("ui.edit_text.rough"));
    // Ready to type, the cursor at the end of the text.
    const first = r.ids?.[0];
    const o = first ? await waitForText(first) : null;
    if (o) startTextEdit(o);
  } catch (e) {
    useEditor.getState().showError(e);
  } finally {
    useEditText.setState({ busy: false });
  }
}

/** Edit text of the pixel selection, or of `rect` in canvas pixels. */
export function editText(rect?: Rect, fit = false): Promise<void> {
  return send({ ...(rect ? { rect } : {}), ...(fit ? { fit: true } : {}) });
}

/** The answer to "the selection cuts the text": fit it, go on as it is, or nothing. */
export function answerCut(how: "fit" | "anyway" | null): Promise<void> {
  const ask = useEditText.getState().ask;
  useEditText.setState({ ask: null });
  if (!ask || !how) return Promise.resolve();
  return send({ ...ask, [how]: true });
}
