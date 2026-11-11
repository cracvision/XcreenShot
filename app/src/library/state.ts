// The main window's two views (LIB-001), the Library view's items and selection
// (LIB-002), and the recent tray's own state (TRY-001). Items live in the core's index
// (LIB-022); this store mirrors `library.list` and keeps what belongs to the view.
import { create } from "zustand";
import { run } from "../lib/commands";

export type MainView = "editor" | "library";

export interface LibraryItem {
  id: string;
  name: string;
  path: string;
  kind: "image";
  format: "xshot" | "png" | "jpg" | "jpeg";
  size: number;
  modified: string;
  captured: string;
  width: number | null;
  height: number | null;
  app: string | null;
  url: string | null;
  favorite: boolean;
  tags: string[];
}

interface LibraryList {
  root: string;
  synced: boolean;
  total: number;
  items: LibraryItem[];
}

function load<T>(key: string, fallback: T, valid: (v: unknown) => v is T): T {
  try {
    const v: unknown = JSON.parse(localStorage.getItem(key) ?? "null");
    return valid(v) ? v : fallback;
  } catch {
    return fallback;
  }
}

function keep(key: string, value: unknown): void {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // Not remembered; the view still changes.
  }
}

const VIEW_KEY = "xs.main.view";
const isView = (v: unknown): v is MainView => v === "editor" || v === "library";

/** The active view, remembered between sessions (LIB-001). */
export const useMainView = create<{ view: MainView; setView: (v: MainView) => void }>((set) => ({
  view: load(VIEW_KEY, "editor", isView),
  setView: (view) => {
    set({ view });
    keep(VIEW_KEY, view);
  },
}));

export function showView(view: MainView): void {
  useMainView.getState().setView(view);
}

const TRAY_KEY = "xs.editor.tray";
const isBool = (v: unknown): v is boolean => typeof v === "boolean";

/** The recent tray: shown or hidden (remembered), and whether captures arrived while
 * it was hidden, for the dot on the Recent button (TRY-001). */
export const useTray = create<{ visible: boolean; fresh: boolean; toggle: () => void }>(
  (set, get) => ({
    visible: load(TRAY_KEY, true, isBool),
    fresh: false,
    toggle: () => {
      const visible = !get().visible;
      set({ visible, ...(visible ? { fresh: false } : {}) });
      keep(TRAY_KEY, visible);
    },
  }),
);

interface LibraryState {
  loaded: boolean;
  items: LibraryItem[];
  root: string;
  synced: boolean;
  selection: string[];
  /** Where Shift+click and Shift+arrows extend from. */
  anchor: string | null;
  /** The item with the keyboard focus. */
  focus: string | null;
  load: () => Promise<void>;
  select: (selection: string[], focus?: string | null, anchor?: string | null) => void;
}

let loading: Promise<void> | null = null;

export const useLibrary = create<LibraryState>((set, get) => ({
  loaded: false,
  items: [],
  root: "",
  synced: false,
  selection: [],
  anchor: null,
  focus: null,
  load: () => {
    // Changes arriving while a list is on its way ask for one more.
    const next = (loading ?? Promise.resolve()).then(async () => {
      const r = await run<LibraryList>("library.list");
      const ids = new Set(r.items.map((i) => i.id));
      const { selection, focus, anchor } = get();
      set({
        loaded: true,
        items: r.items,
        root: r.root,
        synced: r.synced,
        // Items that left the library leave the selection.
        selection: selection.filter((id) => ids.has(id)),
        focus: focus && ids.has(focus) ? focus : null,
        anchor: anchor && ids.has(anchor) ? anchor : null,
      });
    });
    loading = next.catch(() => undefined);
    return next;
  },
  select: (selection, focus, anchor) =>
    set((s) => ({
      selection,
      focus: focus === undefined ? s.focus : focus,
      anchor: anchor === undefined ? s.anchor : anchor,
    })),
}));

/** The items selected, in the order of the grid. */
export function selectedItems(): LibraryItem[] {
  const { items, selection } = useLibrary.getState();
  const chosen = new Set(selection);
  return items.filter((i) => chosen.has(i.id));
}
