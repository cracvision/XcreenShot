// The main window's two views (LIB-001), the Library view's items and selection
// (LIB-002), and the recent tray's own state (TRY-001). Items live in the core's index
// (LIB-022); this store mirrors `library.list` and keeps what belongs to the view.
import { create } from "zustand";
import { run } from "../lib/commands";
import {
  NO_FACETS,
  NO_FILTERS,
  SORT_KEYS,
  defaultOrder,
  type Facets,
  type Filters,
  type Order,
  type SortKey,
} from "./query";

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
  count: number;
  items: LibraryItem[];
  facets: Facets;
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

/** How the Library view shows the items (LIB-004, LIB-005): remembered. */
export interface LibraryShape {
  sort: SortKey;
  order: Order;
  layout: "grid" | "list";
  /** The filter pane is open. */
  pane: boolean;
}

const SHAPE_KEY = "xs.library.view";
const isShape = (v: unknown): v is LibraryShape =>
  typeof v === "object" &&
  v !== null &&
  SORT_KEYS.includes((v as LibraryShape).sort) &&
  ((v as LibraryShape).order === "asc" || (v as LibraryShape).order === "desc") &&
  ((v as LibraryShape).layout === "grid" || (v as LibraryShape).layout === "list") &&
  typeof (v as LibraryShape).pane === "boolean";

interface LibraryState {
  loaded: boolean;
  items: LibraryItem[];
  root: string;
  synced: boolean;
  /** Items in the library, and those that pass the filters and the search. */
  total: number;
  count: number;
  facets: Facets;
  shape: LibraryShape;
  filters: Filters;
  query: string;
  setShape: (shape: Partial<LibraryShape>) => void;
  setFilters: (filters: Filters) => void;
  setQuery: (query: string) => void;
  selection: string[];
  /** Where Shift+click and Shift+arrows extend from. */
  anchor: string | null;
  /** The item with the keyboard focus. */
  focus: string | null;
  load: () => Promise<void>;
  select: (selection: string[], focus?: string | null, anchor?: string | null) => void;
}

let loading: Promise<void> | null = null;

let searchTimer: ReturnType<typeof setTimeout> | undefined;

export const useLibrary = create<LibraryState>((set, get) => ({
  loaded: false,
  items: [],
  root: "",
  synced: false,
  total: 0,
  count: 0,
  facets: NO_FACETS,
  shape: load(SHAPE_KEY, { sort: "modified", order: "desc", layout: "grid", pane: true }, isShape),
  filters: NO_FILTERS,
  query: "",
  setShape: (change) => {
    const shape = { ...get().shape, ...change };
    // A new sort starts in its own direction unless one is given.
    if (change.sort && change.sort !== get().shape.sort && !change.order)
      shape.order = defaultOrder(change.sort);
    set({ shape });
    keep(SHAPE_KEY, shape);
    if (change.sort || change.order)
      void get()
        .load()
        .catch(() => undefined);
  },
  setFilters: (filters) => {
    set({ filters });
    void get()
      .load()
      .catch(() => undefined);
  },
  setQuery: (query) => {
    set({ query });
    // Searches while typing, once the keys pause (LIB-007).
    clearTimeout(searchTimer);
    searchTimer = setTimeout(
      () =>
        void get()
          .load()
          .catch(() => undefined),
      150,
    );
  },
  selection: [],
  anchor: null,
  focus: null,
  load: () => {
    // Changes arriving while a list is on its way ask for one more.
    const next = (loading ?? Promise.resolve()).then(async () => {
      const { shape, filters, query } = get();
      const r = await run<LibraryList>("library.list", {
        sort: shape.sort,
        order: shape.order,
        filters,
        query,
      });
      const ids = new Set(r.items.map((i) => i.id));
      const { selection, focus, anchor } = get();
      set({
        loaded: true,
        items: r.items,
        root: r.root,
        synced: r.synced,
        total: r.total,
        count: r.count,
        facets: r.facets,
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
