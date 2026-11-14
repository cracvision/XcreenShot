// Auto redact in the editor (SMT-010 to SMT-013): what its panel shows. Every change
// runs `smart.redact`, which keeps the redactions still wanted, removes the others and
// never doubles them; the reading of the image is kept by the core, so a change is
// quick. The text found never reaches the panel: only categories, counts and places.
import { create } from "zustand";
import { run } from "../lib/commands";
import { settingValue, useApp } from "../store";
import { usePanel, useEditor } from "./state";

/** The categories, in the panel's order (SMT-011). */
export const CATEGORIES = [
  "email",
  "phone",
  "card",
  "id",
  "address",
  "ip",
  "url",
  "date",
  "face",
] as const;
export type Category = (typeof CATEGORIES)[number];

/** One of the user's words or patterns (SMT-012). */
export interface Term {
  name?: string;
  text?: string;
  pattern?: string;
  exact?: boolean;
  on?: boolean;
}

export type RedactType = "solid" | "blur" | "pixelate";

export interface RedactStyle {
  type: RedactType;
  color: string;
  margin: number;
}

export interface RedactItem {
  objectId: string;
  category: string;
  rect: [number, number, number, number];
}

export interface RedactResult {
  documentId: string;
  redactions: number;
  counts: Record<string, number>;
  categories: string[];
  invalidPatterns: string[];
  textRead: boolean;
  items: RedactItem[];
}

interface RedactState {
  /** The document whose panel is open. */
  documentId: string | null;
  busy: boolean;
  /** The analysis takes over 500 ms: a progress indicator shows (SMT-001). */
  slow: boolean;
  result: RedactResult | null;
  categories: string[];
  terms: Term[];
  style: RedactStyle;
  /** The redaction Next / Previous last showed. */
  index: number;
  /** Done waits for the security reminder (SMT-010). */
  reminder: boolean;
  error: string | null;
}

export const DEFAULT_ON: string[] = ["email", "phone", "card", "id", "address", "ip", "face"];

export const useRedact = create<RedactState>(() => ({
  documentId: null,
  busy: false,
  slow: false,
  result: null,
  categories: DEFAULT_ON,
  terms: [],
  style: { type: "solid", color: "#000000", margin: 2 },
  index: -1,
  reminder: false,
  error: null,
}));

/** The name a term shows and counts under: `term:<name>`. */
export function termName(t: Term): string {
  const name = t.name?.trim();
  return name || t.pattern || t.text || "";
}

function remembered(): Pick<RedactState, "categories" | "terms" | "style"> {
  const settings = useApp.getState().settings;
  const cats = settingValue(settings, "smart.autoRedact.categories");
  const terms = settingValue(settings, "smart.autoRedact.customTerms");
  const style = settingValue(settings, "smart.autoRedact.style") as
    Partial<RedactStyle> | undefined;
  const type = style?.type;
  return {
    categories: Array.isArray(cats) ? (cats as string[]) : DEFAULT_ON,
    terms: Array.isArray(terms) ? (terms as Term[]) : [],
    style: {
      type: type === "blur" || type === "pixelate" ? type : "solid",
      color: typeof style?.color === "string" ? style.color : "#000000",
      margin: typeof style?.margin === "number" ? style.margin : 2,
    },
  };
}

let seq = 0;

/** Runs `smart.redact` on the panel's document with `params`. */
async function analyze(params: Record<string, unknown>): Promise<void> {
  const documentId = useRedact.getState().documentId;
  if (!documentId) return;
  const mine = ++seq;
  useRedact.setState({ busy: true, error: null });
  const timer = setTimeout(() => {
    if (mine === seq) useRedact.setState({ slow: true });
  }, 500);
  try {
    const result = await run<RedactResult>("smart.redact", { documentId, ...params });
    if (mine !== seq || useRedact.getState().documentId !== documentId) return;
    useRedact.setState({ result, categories: result.categories, index: -1 });
  } catch (e) {
    if (mine === seq) useEditor.getState().showError(e);
  } finally {
    clearTimeout(timer);
    if (mine === seq) useRedact.setState({ busy: false, slow: false });
  }
}

/** Opens the panel on the document shown and analyzes it (SMT-010). Opening it again
 * takes over the redactions still in the document, without doubling them. */
export async function openRedact(): Promise<void> {
  const doc = useEditor.getState().doc;
  if (!doc || doc.video) return;
  usePanel.getState().set({ visible: true, tab: "props" });
  useRedact.setState({
    ...remembered(),
    documentId: doc.documentId,
    result: null,
    index: -1,
    reminder: false,
  });
  await analyze({});
}

/** Switches a category on or off (SMT-011): its redactions go or come back. */
export function toggleCategory(category: string): Promise<void> {
  const now = useRedact.getState().categories;
  const categories = now.includes(category)
    ? now.filter((c) => c !== category)
    : CATEGORIES.filter((c) => c === category || now.includes(c));
  useRedact.setState({ categories });
  return analyze({ categories });
}

/** Changes the user's terms (SMT-012): remembered, and applied at once. */
export function setTerms(terms: Term[]): Promise<void> {
  useRedact.setState({ terms });
  return analyze({ terms });
}

/** Changes the style of all the managed redactions, in one undo step (SMT-013). */
export function setStyle(changes: Partial<RedactStyle>): Promise<void> {
  const style = { ...useRedact.getState().style, ...changes };
  useRedact.setState({ style });
  return analyze({ style });
}

/** Removes the managed redactions. */
export function clearRedactions(): Promise<void> {
  return analyze({ clear: true });
}

/** Selects the next (`1`) or previous (`-1`) redaction and centers the view on it. */
export function stepRedaction(dir: 1 | -1): void {
  const { result, index, documentId } = useRedact.getState();
  const items = result?.items ?? [];
  if (!documentId || items.length === 0) return;
  const next =
    index < 0 ? (dir > 0 ? 0 : items.length - 1) : (index + dir + items.length) % items.length;
  const item = items[next];
  if (!item) return;
  const ed = useEditor.getState();
  const [x, y, w, h] = item.rect;
  const view = ed.views[documentId];
  if (view) ed.setView(documentId, { ...view, cx: x + w / 2, cy: y + h / 2 });
  ed.setSelection([item.objectId]);
  useRedact.setState({ index: next });
}

/** The managed redactions that a tool could undo (blur or pixelate): Done reminds that a
 * solid fill is safer (SMT-010). */
export function softRedactions(): number {
  const { result } = useRedact.getState();
  const doc = useEditor.getState().doc;
  if (!result || !doc) return 0;
  const ids = new Set(result.items.map((i) => i.objectId));
  return doc.objects.filter(
    (o) =>
      ids.has(o.id) &&
      o.type === "blur" &&
      (o.style.blurType === "blur" || o.style.blurType === "pixelate"),
  ).length;
}

const REMINDER_HINT = "redact_blur_reminder";

/** Done (SMT-010): the redactions stay as objects; first the security reminder, unless
 * it was hidden. */
export async function finishRedact(): Promise<void> {
  if (softRedactions() > 0) {
    try {
      const hints = await run<{ seen: string[] }>("app.hints");
      if (!hints.seen.includes(REMINDER_HINT)) {
        useRedact.setState({ reminder: true });
        return;
      }
    } catch {
      // Without the hints, the reminder shows.
      useRedact.setState({ reminder: true });
      return;
    }
  }
  closeRedact();
}

/** Closes the reminder and the panel; `hide` keeps the reminder from showing again. */
export function acknowledgeReminder(hide: boolean): void {
  if (hide) void run("app.hints", { seen: REMINDER_HINT }).catch(() => undefined);
  closeRedact();
}

export function closeRedact(): void {
  seq += 1;
  useRedact.setState({
    documentId: null,
    result: null,
    busy: false,
    slow: false,
    index: -1,
    reminder: false,
  });
}
