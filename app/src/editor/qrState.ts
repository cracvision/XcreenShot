// QR codes (SMT-036, SMT-038): the panel that creates or changes a QR object, and the
// codes read from the image. Links open only after the user confirms.
import { create } from "zustand";
import { run } from "../lib/commands";
import { useEditor, type QrObject, type QrStyle } from "./state";
import type { QrCode } from "../lib/qr";

export const DEFAULT_QR: QrStyle = {
  kind: "link",
  ecc: "M",
  color: "#000000",
  background: "#FFFFFF",
  margin: 4,
};

export interface QrCheck {
  content: string;
  modules: number;
  bytes: number;
  max: number;
  readable: boolean;
  hardToRead: boolean;
  svg?: string;
}

export { splitUrl, type QrCode } from "../lib/qr";

interface QrState {
  /** The create panel: new (objectId null) or changing a QR object. */
  editing: { objectId: string | null; content: string; style: QrStyle } | null;
  check: QrCheck | null;
  /** Too long, with how many bytes fit. */
  tooLong: { bytes: number; max: number } | null;
  /** Codes read: the panel lists them; the canvas marks them. */
  codes: QrCode[] | null;
  /** A link waiting for the user's yes before it opens. */
  confirmOpen: QrCode | null;
  /** "QR code detected" on a capture just opened. */
  pill: { documentId: string; codes: QrCode[] } | null;
}

export const useQr = create<QrState>(() => ({
  editing: null,
  check: null,
  tooLong: null,
  codes: null,
  confirmOpen: null,
  pill: null,
}));

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

/** Opens the panel: a new code, or the QR object `o` to change (double click). */
export function openQr(o?: QrObject): void {
  useQr.setState({
    editing: o
      ? { objectId: o.id, content: o.content, style: { ...DEFAULT_QR, ...o.style } }
      : { objectId: null, content: "", style: DEFAULT_QR },
    check: null,
    tooLong: null,
  });
}

export function closeQr(): void {
  useQr.setState({ editing: null, check: null, tooLong: null });
}

let seq = 0;

/** The preview and the checks of what the panel has now. */
export async function previewQr(content: string, style: QrStyle): Promise<void> {
  const mine = ++seq;
  if (!content.trim()) {
    useQr.setState({ check: null, tooLong: null });
    return;
  }
  try {
    const r = await run<QrCheck>("smart.qr_create", { content, ...style, preview: true });
    if (mine === seq) useQr.setState({ check: r, tooLong: null });
  } catch (e) {
    const details = (e as { details?: { bytes?: number; max?: number } }).details;
    if (mine !== seq) return;
    if (details?.max) {
      useQr.setState({ check: null, tooLong: { bytes: details.bytes ?? 0, max: details.max } });
    } else {
      useQr.setState({ check: null });
    }
  }
}

/** Inserts the code, or changes the QR object being edited. */
export async function insertQr(content: string, style: QrStyle): Promise<void> {
  const doc = useEditor.getState().doc;
  const editing = useQr.getState().editing;
  if (!doc || !editing) return;
  try {
    const r = await run<{ objectId?: string }>("smart.qr_create", {
      content,
      ...style,
      documentId: doc.documentId,
      ...(editing.objectId ? { objectId: editing.objectId } : {}),
    });
    closeQr();
    if (r.objectId) useEditor.getState().setSelection([r.objectId]);
  } catch (e) {
    fail(e);
  }
}

/** Reads the codes of the image, or of `rect` (canvas pixels). */
export async function readQr(rect?: [number, number, number, number]): Promise<void> {
  const doc = useEditor.getState().doc;
  if (!doc) return;
  try {
    const r = await run<{ codes: QrCode[] }>("smart.qr_read", {
      documentId: doc.documentId,
      ...(rect ? { rect } : {}),
    });
    useQr.setState({ codes: r.codes, pill: null });
  } catch (e) {
    fail(e);
  }
}

export function closeCodes(): void {
  useQr.setState({ codes: null, confirmOpen: null });
}

/** A link opens only after the user's yes, never by itself. */
export function askOpen(code: QrCode): void {
  useQr.setState({ confirmOpen: code });
}

export async function openConfirmed(yes: boolean): Promise<void> {
  const code = useQr.getState().confirmOpen;
  useQr.setState({ confirmOpen: null });
  if (!yes || !code || code.kind !== "url") return;
  try {
    await run("smart.qr_open", { url: code.text });
  } catch (e) {
    fail(e);
  }
}

/** Covers a code with a solid redaction (SMT-013). */
export async function redactCode(code: QrCode): Promise<void> {
  const doc = useEditor.getState().doc;
  if (!doc) return;
  const [x, y, w, h] = code.rect;
  try {
    await run("document.annotate", {
      documentId: doc.documentId,
      objects: [
        {
          type: "blur",
          rect: { x: x - 4, y: y - 4, w: w + 8, h: h + 8 },
          style: { blurType: "solid", color: "#000000" },
        },
      ],
    });
    useQr.setState((s) => ({ codes: s.codes?.filter((c) => c !== code) ?? null }));
  } catch (e) {
    fail(e);
  }
}

/** After a capture opens in the editor: its codes, quietly (SMT-038). */
export async function detectOnOpen(documentId: string): Promise<void> {
  try {
    const r = await run<{ codes: QrCode[] }>("smart.qr_read", { documentId });
    if (r.codes.length > 0 && useEditor.getState().doc?.documentId === documentId) {
      useQr.setState({ pill: { documentId, codes: r.codes } });
    }
  } catch {
    // Nothing to say when it cannot read.
  }
}
