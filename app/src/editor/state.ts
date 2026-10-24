// Editor view state. Documents, tools and styles live in the core (ADR 0004): this
// store mirrors what the `xs://editor` events report and keeps only what belongs to
// the view (selection, views, open dialogs).
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import { isCommandError, run } from "../lib/commands";
import type { ArrowStyle, Point } from "./geometry";
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

/** Objects of later builds are kept and drawn by the core, not edited here. */
export type DocObject = ArrowObject | { id: string; type: string };

/** What `document.info` returns (EDT-119). */
export interface DocDetails extends Omit<DocSummary, "objects"> {
  canvas: { width: number; height: number; color?: string; cornerRadius: number };
  crop: { x: number; y: number; width: number; height: number };
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

interface EditorState {
  ready: boolean;
  tool: Tool;
  tools: Tool[];
  styles: { arrow: ArrowStyle } | null;
  documents: DocSummary[];
  active: string | null;
  doc: DocDetails | null;
  selection: string[];
  /** Last view of each document in this session. */
  views: Record<string, View>;
  /** A zoom asked by a command (CLI, menu) for the canvas to apply. */
  zoomRequest: { mode: string; value: number | null; seq: number } | null;
  recovery: RecoveryItem[];
  /** Documents to decide on before closing or quitting. */
  decision: { kind: "close" | "quit"; pending: Pending[] } | null;
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
  decision: null,
  flatten: null,
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
            set({ selection: [] });
            void loadDoc(active).then((doc) => {
              if (get().active === active) set({ doc });
            });
          }
          break;
        }
        case "document": {
          const doc = payload as unknown as DocDetails;
          if (doc.documentId !== get().active) break;
          // Objects that no longer exist leave the selection (undo, CLI, delete).
          const ids = new Set(doc.objects.map((o) => o.id));
          set((s) => ({ doc, selection: s.selection.filter((id) => ids.has(id)) }));
          break;
        }
        case "tool":
          set({ tool: payload.tool as Tool });
          break;
        case "styles":
          set({ styles: payload as unknown as { arrow: ArrowStyle } });
          break;
        case "zoom":
          set({
            zoomRequest: {
              mode: payload.mode as string,
              value: (payload.value as number | null) ?? null,
              seq: ++zoomSeq,
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
      styles: { arrow: ArrowStyle };
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
  useEditor.setState({ zoomRequest: { mode, value, seq: ++zoomSeq } });
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
export const usePointer = create<{ point: Point | null }>(() => ({ point: null }));

/** URL of a document's pixels for the canvas (`xsdoc` protocol, ADR 0004). */
export function docUrl(id: string, part: "base" | "thumb", version = 0): string {
  const path = `${id}/${part}?v=${version}`;
  return navigator.userAgent.includes("Windows")
    ? `http://xsdoc.localhost/${path}`
    : `xsdoc://localhost/${path}`;
}
