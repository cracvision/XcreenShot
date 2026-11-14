// The quick view (LIB-013): a large preview over the window, from the Library or the
// recent tray, without opening the editor. ← and → go through the items (only the
// selection when there are several); Space or Esc close it and Enter opens the item.
import i18next from "i18next";
import { create } from "zustand";
import { xsdocUrl } from "../editor/state";
import { openItems } from "./actions";
import { useLibrary, type LibraryItem } from "./state";

export interface QuickEntry {
  id: string;
  name: string;
  /** The large picture, and the small one shown until it arrives. */
  src: string;
  thumb: string;
  /** Pixels of the image, when known: it is never shown larger than that. */
  width: number | null;
  height: number | null;
  /** When it was captured (ISO), when known. */
  date: string | null;
}

export interface QuickView {
  entries: QuickEntry[];
  at: number;
  /** Enter: open the entry in the editor. */
  open: (id: string) => void;
  /** On closing, with the entry last seen, which becomes the selection. */
  done: (id: string) => void;
}

export const useQuickView = create<{ view: QuickView | null }>(() => ({ view: null }));

export function showQuickView(view: QuickView): void {
  if (view.entries.length === 0) return;
  const at = Math.min(Math.max(0, view.at), view.entries.length - 1);
  useQuickView.setState({ view: { ...view, at } });
}

/** The next (1) or previous (-1) entry; it stops at the ends. */
export function stepQuickView(delta: number): void {
  const view = useQuickView.getState().view;
  if (!view) return;
  const at = Math.min(Math.max(0, view.at + delta), view.entries.length - 1);
  if (at !== view.at) useQuickView.setState({ view: { ...view, at } });
}

/** Closes the view; with `open`, the entry shown opens in the editor. */
export function closeQuickView(open = false): void {
  const view = useQuickView.getState().view;
  if (!view) return;
  useQuickView.setState({ view: null });
  const entry = view.entries[view.at];
  if (!entry) return;
  view.done(entry.id);
  if (open) view.open(entry.id);
}

/** The longest side, in physical pixels, that fills this window; in steps, so that the
 * same preview serves after a small resize. */
export function previewSide(width = window.innerWidth, height = window.innerHeight): number {
  const side = Math.max(width, height) * (window.devicePixelRatio || 1);
  return Math.min(8192, Math.max(512, Math.ceil(side / 512) * 512));
}

export function libraryThumb(item: LibraryItem): string {
  return xsdocUrl(`library/${item.id}/thumb?v=${encodeURIComponent(item.modified)}`);
}

/** The steps of a library guide, for the quick view (LIB-019). */
export function guideEntries(item: LibraryItem): QuickEntry[] {
  const n = item.steps ?? 0;
  return Array.from({ length: n }, (_, i) => {
    const src = xsdocUrl(`library/${item.id}/step/${i + 1}?v=${encodeURIComponent(item.modified)}`);
    return {
      id: `${item.id}#${i + 1}`,
      name: i18next.t("ui.library.step_of", { name: item.name, n: i + 1, count: n }),
      src,
      thumb: src,
      width: null,
      height: null,
      date: item.captured,
    };
  });
}

/** The entries of library items. */
export function libraryEntries(items: LibraryItem[], side = previewSide()): QuickEntry[] {
  return items.map((item) => ({
    id: item.id,
    name: item.name,
    src: xsdocUrl(`library/${item.id}/view/${side}?v=${encodeURIComponent(item.modified)}`),
    thumb: libraryThumb(item),
    width: item.width,
    height: item.height,
    date: item.captured,
  }));
}

/** The size, in CSS pixels, an image of `width` × `height` pixels takes in a box: as
 * large as fits, never larger than one image pixel per screen pixel. */
export function fitted(
  width: number | null,
  height: number | null,
  box: { width: number; height: number },
  ratio: number,
): { width: number; height: number } | null {
  if (!width || !height || box.width <= 0 || box.height <= 0) return null;
  const w = width / ratio;
  const h = height / ratio;
  const scale = Math.min(1, box.width / w, box.height / h);
  return { width: Math.round(w * scale), height: Math.round(h * scale) };
}

/** Space in the Library (LIB-013): the selection when there are several items, else
 * every item shown, from the one with the focus. On closing, the item last seen gets
 * the focus (and is the selection, unless the walk was through the selection). */
export function openLibraryQuickView(): void {
  const { items, selection, focus } = useLibrary.getState();
  const chosen = new Set(selection);
  const several = selection.length > 1;
  // A guide alone: its steps, one after the other (LIB-019).
  const guide = !several ? items.find((i) => i.id === (focus ?? selection[0])) : undefined;
  if (guide?.kind === "guide" && guide.steps) {
    showQuickView({
      entries: guideEntries(guide),
      at: 0,
      open: () => void openItems([guide]),
      done: () => useLibrary.setState({ reveal: guide.id }),
    });
    return;
  }
  const walk = several ? items.filter((i) => chosen.has(i.id)) : items;
  const start = focus ?? selection[0];
  const at = Math.max(
    0,
    walk.findIndex((i) => i.id === start),
  );
  if (walk.length === 0 || (!start && !several)) return;
  showQuickView({
    entries: libraryEntries(walk),
    at,
    open: (id) => {
      const item = walk.find((i) => i.id === id);
      if (item) void openItems([item]);
    },
    done: (id) => {
      const lib = useLibrary.getState();
      if (several) lib.select(lib.selection, id, lib.anchor);
      else lib.select([id], id, id);
      useLibrary.setState({ reveal: id });
      requestAnimationFrame(() =>
        document.querySelector<HTMLElement>(`[data-item="${id}"]`)?.focus({ preventScroll: true }),
      );
    },
  });
}
