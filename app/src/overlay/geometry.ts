// Geometry of the on-screen selection (CAP-011, CAP-013, CAP-015, CAP-018). Pure
// functions in physical desktop pixels, tested without a screen.

export interface Pt {
  x: number;
  y: number;
}

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type Handle = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

export const contains = (r: Rect, p: Pt): boolean =>
  p.x >= r.x && p.x < r.x + r.width && p.y >= r.y && p.y < r.y + r.height;

export function intersect(a: Rect, b: Rect): Rect | null {
  const x = Math.max(a.x, b.x);
  const y = Math.max(a.y, b.y);
  const right = Math.min(a.x + a.width, b.x + b.width);
  const bottom = Math.min(a.y + a.height, b.y + b.height);
  return right > x && bottom > y ? { x, y, width: right - x, height: bottom - y } : null;
}

export function bounding(rects: Rect[]): Rect | null {
  if (rects.length === 0) return null;
  const x = Math.min(...rects.map((r) => r.x));
  const y = Math.min(...rects.map((r) => r.y));
  const right = Math.max(...rects.map((r) => r.x + r.width));
  const bottom = Math.max(...rects.map((r) => r.y + r.height));
  return { x, y, width: right - x, height: bottom - y };
}

/** Rectangle dragged from `a` to `b`, at least 1×1 (CAP-011). */
export function fromPoints(a: Pt, b: Pt): Rect {
  return {
    x: Math.min(a.x, b.x),
    y: Math.min(a.y, b.y),
    width: Math.max(1, Math.abs(b.x - a.x)),
    height: Math.max(1, Math.abs(b.y - a.y)),
  };
}

/**
 * Moves `b` so that the rectangle from `a` keeps `ratio` (width / height), following
 * the axis the user dragged most (CAP-018).
 */
export function lockAspect(a: Pt, b: Pt, ratio: number): Pt {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const sx = dx < 0 ? -1 : 1;
  const sy = dy < 0 ? -1 : 1;
  if (Math.abs(dx) / ratio >= Math.abs(dy)) {
    return { x: b.x, y: a.y + sy * Math.round(Math.abs(dx) / ratio) };
  }
  return { x: a.x + sx * Math.round(Math.abs(dy) * ratio), y: b.y };
}

/** Aspect held by modifier keys (CAP-018): Shift 1:1, Alt 16:9, Alt+Shift 4:3. */
export function ratioFromKeys(shift: boolean, alt: boolean): number | null {
  if (alt && shift) return 4 / 3;
  if (alt) return 16 / 9;
  if (shift) return 1;
  return null;
}

/** Keeps a rectangle inside `bounds` (same size if it fits). */
export function clampInside(r: Rect, bounds: Rect): Rect {
  const width = Math.min(r.width, bounds.width);
  const height = Math.min(r.height, bounds.height);
  const x = Math.min(Math.max(r.x, bounds.x), bounds.x + bounds.width - width);
  const y = Math.min(Math.max(r.y, bounds.y), bounds.y + bounds.height - height);
  return { x, y, width, height };
}

/** Resizes `r` by dragging one of its 8 handles to `p` (CAP-011). */
export function resizeByHandle(r: Rect, handle: Handle, p: Pt): Rect {
  let left = r.x;
  let top = r.y;
  let right = r.x + r.width;
  let bottom = r.y + r.height;
  if (handle.includes("w")) left = p.x;
  if (handle.includes("e")) right = p.x;
  if (handle.includes("n")) top = p.y;
  if (handle.includes("s")) bottom = p.y;
  return fromPoints({ x: left, y: top }, { x: right, y: bottom });
}

/**
 * Where the capture bar goes (CAP-011): below the selection, else above, else inside
 * at the bottom-right. `bounds` is the monitor; all values in the same units.
 */
export function placeBar(
  sel: Rect,
  bar: { width: number; height: number },
  bounds: Rect,
  gap = 8,
): Pt {
  const x = Math.min(
    Math.max(sel.x + sel.width - bar.width, bounds.x + gap),
    bounds.x + bounds.width - bar.width - gap,
  );
  const below = sel.y + sel.height + gap;
  if (below + bar.height <= bounds.y + bounds.height - gap) return { x, y: below };
  const above = sel.y - gap - bar.height;
  if (above >= bounds.y + gap) return { x, y: above };
  return {
    x: Math.max(bounds.x + gap, sel.x + sel.width - bar.width - gap),
    y: Math.max(bounds.y + gap, sel.y + sel.height - bar.height - gap),
  };
}

/** Magnifier box next to the pointer, flipped so it is always fully visible (CAP-015). */
export function placeMagnifier(
  p: Pt,
  size: { width: number; height: number },
  bounds: Rect,
  offset = 24,
): Pt {
  let x = p.x + offset;
  let y = p.y + offset;
  if (x + size.width > bounds.x + bounds.width) x = p.x - offset - size.width;
  if (y + size.height > bounds.y + bounds.height) y = p.y - offset - size.height;
  return { x: Math.max(bounds.x, x), y: Math.max(bounds.y, y) };
}

export const MAGNIFIER_ZOOMS = [2, 4, 8, 16] as const;

export function nextZoom(current: number, up: boolean): number {
  const i = MAGNIFIER_ZOOMS.indexOf(current as (typeof MAGNIFIER_ZOOMS)[number]);
  const at = i < 0 ? 1 : i;
  return (
    MAGNIFIER_ZOOMS[Math.min(MAGNIFIER_ZOOMS.length - 1, Math.max(0, at + (up ? 1 : -1)))] ?? 4
  );
}

export const toHex = (r: number, g: number, b: number): string =>
  "#" +
  [r, g, b]
    .map((v) => v.toString(16).padStart(2, "0"))
    .join("")
    .toUpperCase();
