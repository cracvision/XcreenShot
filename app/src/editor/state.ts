// Editor view state. Documents, tools and styles live in the core (ADR 0004): this
// store mirrors what the `xs://editor` events report and keeps only what belongs to
// the view (selection, views, open dialogs).
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import { isCommandError, run } from "../lib/commands";
import type { Rect } from "./boxes";
import type { ArrowStyle, LineStyle, Point } from "./geometry";
import type { View } from "./view";

export type Tool = "move" | "arrow" | "text" | "shape" | "blur" | "crop";

export interface DocSummary {
  documentId: string;
  name: string;
  path: string | null;
  kind: "library" | "external";
  format: "xshot" | "png" | "jpg";
  width: number;
  height: number;
  objects: number;
  dirty: boolean;
  shown: boolean;
  version: number;
}

export interface ArrowObject {
  id: string;
  type: "arrow";
  from: Point;
  to: Point;
  style: ArrowStyle;
}

export type TextAlign = "left" | "center" | "right" | "justify";

/** Text properties of M1 (EDT-026). Sizes are points at the document's resolution. */
export interface TextStyle {
  font: string;
  size: number;
  bold: boolean;
  italic: boolean;
  color: string;
  align: TextAlign;
  opacity: number;
}

export interface TextObject {
  id: string;
  type: "text";
  at: Point;
  /** Fixed width (wraps), or null: the box grows with the text. */
  width: number | null;
  text: string;
  rotation: number;
  style: TextStyle;
  /** Laid-out size, computed by the core. */
  layout: { width: number; height: number };
}

export type ShapeKind = "rectangle" | "ellipse";

/** Shape properties of M1 (EDT-029). */
export interface ShapeStyle {
  shapeType: ShapeKind;
  /** null = transparent: only the outline. */
  fill: string | null;
  outline: string;
  outlineWidth: number;
  lineStyle: LineStyle;
  opacity: number;
}

export interface ShapeObject {
  id: string;
  type: "shape";
  rect: Rect;
  rotation: number;
  style: ShapeStyle;
}

export type BlurType = "blur" | "pixelate" | "solid";

/** Blur properties of M1 (EDT-040). */
export interface BlurStyle {
  blurType: BlurType;
  color: string;
  intensity: number;
  shapeType: ShapeKind;
}

export interface BlurObject {
  id: string;
  type: "blur";
  rect: Rect;
  rotation: number;
  style: BlurStyle;
}

/** Objects of later builds are kept and drawn by the core, not edited here. */
export type DocObject =
  ArrowObject | TextObject | ShapeObject | BlurObject | { id: string; type: "other" };

export interface Styles {
  arrow: ArrowStyle;
  text: TextStyle;
  shape: ShapeStyle;
  blur: BlurStyle;
}

/** A text box being typed (EDT-026): a new one (`id` null) or an existing one. */
export interface TextEdit {
  id: string | null;
  at: Point;
  width: number | null;
  text: string;
  rotation: number;
  style: TextStyle;
}

/** What `document.info` returns (EDT-119). */
export interface DocDetails extends Omit<DocSummary, "objects"> {
  canvas: { width: number; height: number; color?: string; cornerRadius: number };
  crop: { x: number; y: number; width: number; height: number };
  /** Objects a crop left outside the canvas: they do not make it grow (EDT-071). */
  outside?: string[];
  dpi: number;
  objects: DocObject[];
  base: { width: number; height: number; mime: string };
  metadata: { display?: { scale?: number } };
  canUndo: boolean;
  canRedo: boolean;
  undoLabel: string | null;
  redoLabel: string | null;
  /** Remembered view (EDT-007): zoom and the image point at the center. */
  view: { zoom?: number; x?: number; y?: number } | null;
}

export interface Pending {
  documentId: string;
  name: string;
  path: string | null;
}

export interface RecoveryItem {
  id: string;
  name: string;
  originalPath: string | null;
  savedAt: string;
}

/** A zoom asked for one document: applied once, by that document's canvas. */
export interface ZoomRequest {
  mode: string;
  value: number | null;
  seq: number;
  documentId: string | null;
}

interface EditorState {
  ready: boolean;
  tool: Tool;
  tools: Tool[];
  styles: Styles | null;
  documents: DocSummary[];
  active: string | null;
  doc: DocDetails | null;
  selection: string[];
  /** Last view of each document in this session. */
  views: Record<string, View>;
  /** A zoom asked by a command (CLI, menu) for the canvas to apply. */
  zoomRequest: ZoomRequest | null;
  recovery: RecoveryItem[];
  /** Recover work is on screen: at start, or from Help (APP-047). */
  recoveryShown: boolean;
  /** Documents to decide on before closing or quitting. */
  decision: { kind: "close" | "quit"; pending: Pending[] } | null;
  /** The text box being typed, if any (EDT-026). */
  textEdit: TextEdit | null;
  /** The crop being adjusted, in canvas pixels (EDT-049). */
  cropRect: Rect | null;
  /** Installed font families, loaded when first needed. */
  fonts: string[];
  /** Document waiting for the "objects will be flattened" confirmation (EDT-107). */
  flatten: string | null;
  toast: string | null;
  error: string | null;
  boot: () => Promise<void>;
  setSelection: (ids: string[]) => void;
  setView: (id: string, view: View) => void;
  showToast: (text: string) => void;
  showError: (e: unknown) => void;
}

let zoomSeq = 0;
let toastTimer: ReturnType<typeof setTimeout> | undefined;

async function loadDoc(id: string | null): Promise<DocDetails | null> {
  if (!id) return null;
  try {
    return await run<DocDetails>("document.info", { documentId: id });
  } catch {
    return null;
  }
}

export const useEditor = create<EditorState>((set, get) => ({
  ready: false,
  tool: "move",
  tools: ["move", "arrow"],
  styles: null,
  documents: [],
  active: null,
  doc: null,
  selection: [],
  views: {},
  zoomRequest: null,
  recovery: [],
  recoveryShown: true,
  decision: null,
  flatten: null,
  textEdit: null,
  cropRect: null,
  fonts: [],
  toast: null,
  error: null,

  boot: async () => {
    await listen<{ kind: string; payload: Record<string, unknown> }>("xs://editor", (e) => {
      const { kind, payload } = e.payload;
      switch (kind) {
        case "documents": {
          const active = (payload.active as string | null) ?? null;
          const changed = active !== get().active;
          set({ documents: payload.documents as DocSummary[], active });
          if (changed) {
            set({ selection: [], cropRect: null, textEdit: null });
            void loadDoc(active).then((doc) => {
              if (get().active === active) set({ doc });
            });
          }
          break;
        }
        case "document": {
          const doc = payload as unknown as DocDetails;
          if (doc.documentId !== get().active) break;
          // When the canvas moves over the image (a crop, or growing to fit objects,
          // EDT-049, EDT-071), the view keeps the image in place.
          const before = get().doc;
          const view = get().views[doc.documentId];
          if (before?.documentId === doc.documentId && view) {
            const dx = before.crop.x - doc.crop.x;
            const dy = before.crop.y - doc.crop.y;
            if (dx !== 0 || dy !== 0)
              get().setView(doc.documentId, { ...view, cx: view.cx + dx, cy: view.cy + dy });
          }
          // Objects that no longer exist leave the selection (undo, CLI, delete).
          const ids = new Set(doc.objects.map((o) => o.id));
          set((s) => ({ doc, selection: s.selection.filter((id) => ids.has(id)) }));
          break;
        }
        case "tool": {
          const tool = payload.tool as Tool;
          const doc = get().doc;
          // The crop box starts on the whole canvas; leaving the tool cancels it (EDT-049).
          set({
            tool,
            cropRect:
              tool === "crop" && doc
                ? { x: 0, y: 0, w: doc.canvas.width, h: doc.canvas.height }
                : null,
            // Cropping works on the image, not on objects.
            ...(tool === "crop" ? { selection: [] } : {}),
          });
          break;
        }
        case "styles":
          set({ styles: payload as unknown as Styles });
          break;
        case "zoom":
          set({
            zoomRequest: {
              mode: payload.mode as string,
              value: (payload.value as number | null) ?? null,
              seq: ++zoomSeq,
              documentId: (payload.documentId as string | null) ?? null,
            },
          });
          break;
        case "quit":
          set({ decision: { kind: "quit", pending: payload.pending as Pending[] } });
          break;
      }
    });
    const state = await run<{
      tool: Tool;
      tools: Tool[];
      styles: Styles;
      active: string | null;
      documents: DocSummary[];
    }>("editor.state");
    const recovery = await run<{ items: RecoveryItem[] }>("editor.recovery_list");
    set({
      tool: state.tool,
      tools: state.tools,
      styles: state.styles,
      documents: state.documents,
      active: state.active,
      doc: await loadDoc(state.active),
      recovery: recovery.items,
      ready: true,
    });
  },

  setSelection: (selection) => set({ selection }),
  setView: (id, view) => set((s) => ({ views: { ...s.views, [id]: view } })),
  showToast: (text) => {
    clearTimeout(toastTimer);
    set({ toast: text });
    toastTimer = setTimeout(() => set({ toast: null }), 2500);
  },
  showError: (e) => set({ error: isCommandError(e) ? e.message : String(e) }),
}));

/** Asks the canvas for a zoom: `actual`, `fit`, `original`, `in`, `out` or `percent`
 * with a value (EDT-007). Commands from the CLI arrive the same way. */
export function requestZoom(mode: string, value: number | null = null): void {
  useEditor.setState((s) => ({
    zoomRequest: { mode, value, seq: ++zoomSeq, documentId: s.active },
  }));
}

/** The pending zoom for document `id`, removed so it is applied only once. A request
 * kept after use was applied again to every document shown next (Carlos, 2026-10-24). */
export function takeZoomRequest(id: string): ZoomRequest | null {
  const r = useEditor.getState().zoomRequest;
  if (!r || (r.documentId !== null && r.documentId !== id)) return null;
  useEditor.setState({ zoomRequest: null });
  return r;
}

/** Side panel layout (EDT-005): shown or hidden, width (260–480 px) and tab. A view
 * convenience kept in this window's storage. */
export interface PanelLayout {
  visible: boolean;
  width: number;
  tab: "props" | "styles";
}

const PANEL_KEY = "xs.editor.panel";

function loadPanel(): PanelLayout {
  const fallback: PanelLayout = { visible: true, width: 280, tab: "props" };
  try {
    const v = JSON.parse(localStorage.getItem(PANEL_KEY) ?? "null") as Partial<PanelLayout> | null;
    return { ...fallback, ...v };
  } catch {
    return fallback;
  }
}

export const usePanel = create<PanelLayout & { set: (p: Partial<PanelLayout>) => void }>(
  (set, get) => ({
    ...loadPanel(),
    set: (p) => {
      set(p);
      const { visible, width, tab } = get();
      try {
        localStorage.setItem(PANEL_KEY, JSON.stringify({ visible, width, tab }));
      } catch {
        // Not remembered; the panel still changes.
      }
    },
  }),
);

/** Image pixel under the pointer, for the status bar (EDT-011). Kept apart so moving
 * the mouse does not redraw the whole editor. */
export const usePointer = create<{
  point: Point | null;
  /** The same place in the canvas area, in CSS pixels (the `I` color picker). */
  screen: { x: number; y: number } | null;
}>(() => ({ point: null, screen: null }));

/** URL of an object Rust draws for the canvas (text, blur); `version` busts caches. */
export function objectUrl(documentId: string, objectId: string, version: string): string {
  const path = `${documentId}/object/${objectId}?v=${version}`;
  return navigator.userAgent.includes("Windows")
    ? `http://xsdoc.localhost/${path}`
    : `xsdoc://localhost/${path}`;
}

/** URL of a document's pixels for the canvas (`xsdoc` protocol, ADR 0004). */
export function docUrl(id: string, part: "base" | "thumb", version = 0): string {
  const path = `${id}/${part}?v=${version}`;
  return navigator.userAgent.includes("Windows")
    ? `http://xsdoc.localhost/${path}`
    : `xsdoc://localhost/${path}`;
}
