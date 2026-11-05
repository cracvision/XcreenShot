// Objects drawn in a box (text, shapes, blur): the box turned around its center, its
// handles and how dragging them resizes or turns it (EDT-022). `innerRect` and
// `pixelBounds` copy crates/xs-image/src/shapes.rs, checked against
// tests/fixtures/shape-geometry.json.
import type { Point } from "./geometry";

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** A rectangle turned by `rotation` degrees, clockwise, around its center. */
export interface Box extends Rect {
  rotation: number;
}

export type Handle = "nw" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w" | "rotate";

export const RESIZE_HANDLES: Handle[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];

/** Where the outline goes: inside the shape's size (EDT-029). */
export function innerRect(r: Rect, outline: number): Rect {
  const o = Math.max(0, Math.min(outline, r.w, r.h));
  return { x: r.x + o / 2, y: r.y + o / 2, w: r.w - o, h: r.h - o };
}

const rad = (deg: number) => (deg * Math.PI) / 180;

export function center(b: Rect): Point {
  return [b.x + b.w / 2, b.y + b.h / 2];
}

/** A point given in the box's own frame (offset from its center, unturned). */
export function toWorld(b: Box, local: Point): Point {
  const [cx, cy] = center(b);
  const s = Math.sin(rad(b.rotation));
  const c = Math.cos(rad(b.rotation));
  return [cx + local[0] * c - local[1] * s, cy + local[0] * s + local[1] * c];
}

/** A canvas point in the box's own frame. */
export function toLocal(b: Box, p: Point): Point {
  const [cx, cy] = center(b);
  const s = Math.sin(rad(-b.rotation));
  const c = Math.cos(rad(-b.rotation));
  const dx = p[0] - cx;
  const dy = p[1] - cy;
  return [dx * c - dy * s, dx * s + dy * c];
}

export function corners(b: Box): Point[] {
  return [
    [-b.w / 2, -b.h / 2],
    [b.w / 2, -b.h / 2],
    [b.w / 2, b.h / 2],
    [-b.w / 2, b.h / 2],
  ].map((p) => toWorld(b, p as Point));
}

/** Whole pixels the turned box covers inside the canvas: [x, y, w, h], or null. */
export function pixelBounds(
  r: Rect,
  rotation: number,
  width: number,
  height: number,
): [number, number, number, number] | null {
  const pts = corners({ ...r, rotation });
  const xs = pts.map((p) => p[0]);
  const ys = pts.map((p) => p[1]);
  const x0 = Math.max(Math.floor(Math.min(...xs)), 0);
  const y0 = Math.max(Math.floor(Math.min(...ys)), 0);
  const x1 = Math.min(Math.ceil(Math.max(...xs)), width);
  const y1 = Math.min(Math.ceil(Math.max(...ys)), height);
  return x1 > x0 && y1 > y0 ? [x0, y0, x1 - x0, y1 - y0] : null;
}

/** Whether `p` is inside the turned box, with `slack` pixels of tolerance. */
export function contains(b: Box, p: Point, slack = 0): boolean {
  const [x, y] = toLocal(b, p);
  return Math.abs(x) <= b.w / 2 + slack && Math.abs(y) <= b.h / 2 + slack;
}

const SIGNS: Record<Exclude<Handle, "rotate">, [number, number]> = {
  nw: [-1, -1],
  n: [0, -1],
  ne: [1, -1],
  e: [1, 0],
  se: [1, 1],
  s: [0, 1],
  sw: [-1, 1],
  w: [-1, 0],
};

/** Canvas position of each handle; the turning handle sits `stem` pixels above. */
export function handlePoint(b: Box, h: Handle, stem: number): Point {
  if (h === "rotate") return toWorld(b, [0, -b.h / 2 - stem]);
  const [sx, sy] = SIGNS[h];
  return toWorld(b, [(sx * b.w) / 2, (sy * b.h) / 2]);
}

/**
 * The box after dragging handle `h` to canvas point `p` (EDT-022): the opposite side
 * stays put; `keepRatio` (Shift) keeps the proportion, `fromCenter` (Alt) keeps the
 * center. Works on turned boxes too. Sizes never go below `min`.
 */
export function resize(
  b: Box,
  h: Exclude<Handle, "rotate">,
  p: Point,
  opts: { keepRatio?: boolean; fromCenter?: boolean; min?: number } = {},
): Box {
  const min = opts.min ?? 1;
  const [sx, sy] = SIGNS[h];
  const [lx, ly] = toLocal(b, p);
  // Edges in the box frame (center at 0).
  let left = -b.w / 2;
  let right = b.w / 2;
  let top = -b.h / 2;
  let bottom = b.h / 2;
  if (sx < 0) left = Math.min(lx, right - min);
  if (sx > 0) right = Math.max(lx, left + min);
  if (sy < 0) top = Math.min(ly, bottom - min);
  if (sy > 0) bottom = Math.max(ly, top + min);
  if (opts.fromCenter) {
    if (sx !== 0) {
      const half = Math.max(sx < 0 ? -left : right, min / 2);
      left = -half;
      right = half;
    }
    if (sy !== 0) {
      const half = Math.max(sy < 0 ? -top : bottom, min / 2);
      top = -half;
      bottom = half;
    }
  }
  let w = right - left;
  let hgt = bottom - top;
  if (opts.keepRatio && b.w > 0 && b.h > 0) {
    const ratio = b.w / b.h;
    if (sx === 0) w = hgt * ratio;
    else if (sy === 0) hgt = w / ratio;
    else if (w / hgt > ratio) hgt = w / ratio;
    else w = hgt * ratio;
    // Grow from the fixed side (or the center).
    const anchorX = opts.fromCenter ? 0 : sx < 0 ? right : sx > 0 ? left : (left + right) / 2;
    const anchorY = opts.fromCenter ? 0 : sy < 0 ? bottom : sy > 0 ? top : (top + bottom) / 2;
    left = sx < 0 ? anchorX - w : sx > 0 ? anchorX : anchorX - w / 2;
    right = left + w;
    top = sy < 0 ? anchorY - hgt : sy > 0 ? anchorY : anchorY - hgt / 2;
    bottom = top + hgt;
  }
  // New center in the canvas: the frame's offset turned back into the canvas.
  const mid = toWorld(b, [(left + right) / 2, (top + bottom) / 2]);
  return { x: mid[0] - w / 2, y: mid[1] - hgt / 2, w, h: hgt, rotation: b.rotation };
}

/** Rotation that points the turning handle at `p`; `snap` (Shift) uses 15° steps. */
export function rotateTo(b: Box, p: Point, snap: boolean): number {
  const [cx, cy] = center(b);
  let deg = (Math.atan2(p[0] - cx, -(p[1] - cy)) * 180) / Math.PI;
  if (snap) deg = Math.round(deg / 15) * 15;
  return ((deg % 360) + 360) % 360;
}

/** The box dragged from `a` to `b` (EDT-022): Shift makes a square, Alt draws from the center. */
export function dragRect(a: Point, b: Point, square: boolean, fromCenter: boolean): Rect {
  let dx = b[0] - a[0];
  let dy = b[1] - a[1];
  if (square) {
    const side = Math.max(Math.abs(dx), Math.abs(dy));
    dx = Math.sign(dx || 1) * side;
    dy = Math.sign(dy || 1) * side;
  }
  if (fromCenter) {
    return {
      x: a[0] - Math.abs(dx),
      y: a[1] - Math.abs(dy),
      w: 2 * Math.abs(dx),
      h: 2 * Math.abs(dy),
    };
  }
  return {
    x: Math.min(a[0], a[0] + dx),
    y: Math.min(a[1], a[1] + dy),
    w: Math.abs(dx),
    h: Math.abs(dy),
  };
}
