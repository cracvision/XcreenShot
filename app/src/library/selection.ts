// Selection and keyboard movement in a grid or strip of items (LIB-002, TRY-013):
// plain functions over IDs in display order, so they can be tested without a window.

export interface ClickMods {
  /** Ctrl (Windows) or Cmd (macOS): add or remove one item. */
  toggle: boolean;
  /** Shift: everything from the anchor to the item. */
  range: boolean;
}

/** Items from `from` to `to`, both included, in display order. */
export function range(order: string[], from: string | null, to: string): string[] {
  const a = from ? order.indexOf(from) : -1;
  const b = order.indexOf(to);
  if (b < 0) return [];
  if (a < 0) return [to];
  return order.slice(Math.min(a, b), Math.max(a, b) + 1);
}

/** The selection after a click on `id`, and the new anchor. */
export function clickSelection(
  order: string[],
  current: string[],
  anchor: string | null,
  id: string,
  mods: ClickMods,
): { selection: string[]; anchor: string } {
  if (mods.range) {
    const span = range(order, anchor ?? id, id);
    // Ctrl+Shift adds the range to what was selected.
    const selection = mods.toggle
      ? order.filter((x) => current.includes(x) || span.includes(x))
      : span;
    return { selection, anchor: anchor ?? id };
  }
  if (mods.toggle) {
    const selection = current.includes(id)
      ? current.filter((x) => x !== id)
      : order.filter((x) => current.includes(x) || x === id);
    return { selection, anchor: id };
  }
  return { selection: [id], anchor: id };
}

/** Where a key moves the focus in a grid of `columns` columns, `page` items a screen.
 * Null: the key does not move anything. Moves stop at the ends (no wrapping). */
export function moveFocus(
  order: string[],
  focus: string | null,
  key: string,
  columns: number,
  page: number,
  primary = false,
): string | null {
  if (order.length === 0) return null;
  const last = order.length - 1;
  const at = focus ? order.indexOf(focus) : -1;
  if (at < 0) return order[0] ?? null;
  const cols = Math.max(1, columns);
  let next: number;
  switch (key) {
    case "ArrowLeft":
      next = at - 1;
      break;
    case "ArrowRight":
      next = at + 1;
      break;
    case "ArrowUp":
      next = primary ? 0 : at - cols;
      break;
    case "ArrowDown":
      next = primary ? last : at + cols;
      break;
    case "Home":
      next = 0;
      break;
    case "End":
      next = last;
      break;
    case "PageUp":
      next = at - Math.max(cols, page);
      break;
    case "PageDown":
      next = at + Math.max(cols, page);
      break;
    default:
      return null;
  }
  return order[Math.min(last, Math.max(0, next))] ?? null;
}

/** How many items share the first row: the columns of a wrapping grid. */
export function columnsOf(tops: number[]): number {
  const first = tops[0];
  if (first === undefined) return 1;
  const n = tops.findIndex((top) => Math.abs(top - first) > 2);
  return n < 0 ? tops.length : Math.max(1, n);
}

export interface Box {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

/** Whether two boxes overlap (the rubber band over an item). */
export function overlaps(a: Box, b: Box): boolean {
  return a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom;
}

/** The box between two points, whichever way the pointer went. */
export function boxOf(x1: number, y1: number, x2: number, y2: number): Box {
  return {
    left: Math.min(x1, x2),
    top: Math.min(y1, y2),
    right: Math.max(x1, x2),
    bottom: Math.max(y1, y2),
  };
}
