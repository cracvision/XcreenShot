// Live measurement (EDT-094): distances from the pointer to the edges of what is
// under it, the gap between two objects, and a ruler. Pure helpers; what they show
// is drawn over the canvas and never exported.
import type { Point } from "./geometry";

/** A ruler's length (px), angle (degrees, counterclockwise from the right, 0–360) and
 * its horizontal and vertical parts. */
export function rulerOf(
  from: Point,
  to: Point,
): { length: number; angle: number; dx: number; dy: number } {
  const dx = to[0] - from[0];
  const dy = to[1] - from[1];
  const length = Math.hypot(dx, dy);
  const degrees = (Math.atan2(-dy, dx) * 180) / Math.PI;
  const angle = (((Math.round(degrees * 10) / 10) % 360) + 360) % 360;
  return {
    length: Math.round(length * 10) / 10,
    angle,
    dx: Math.round(dx),
    dy: Math.round(dy),
  };
}

/** The pixels of an image: RGBA rows, as an `ImageData` has them. */
export interface Pixels {
  width: number;
  height: number;
  data: Uint8ClampedArray;
}

/** How far the color under (`x`, `y`) goes left, right, up and down before another
 * color starts (more than `tolerance` apart in a channel, transparency too), up to the
 * image's edge. */
export function edges(
  px: Pixels,
  x: number,
  y: number,
  tolerance = 24,
): { left: number; right: number; up: number; down: number } | null {
  const ix = Math.floor(x);
  const iy = Math.floor(y);
  if (ix < 0 || iy < 0 || ix >= px.width || iy >= px.height) return null;
  const at = (cx: number, cy: number) => (cy * px.width + cx) * 4;
  const start = at(ix, iy);
  const same = (i: number) =>
    Math.abs((px.data[i] ?? 0) - (px.data[start] ?? 0)) <= tolerance &&
    Math.abs((px.data[i + 1] ?? 0) - (px.data[start + 1] ?? 0)) <= tolerance &&
    Math.abs((px.data[i + 2] ?? 0) - (px.data[start + 2] ?? 0)) <= tolerance &&
    Math.abs((px.data[i + 3] ?? 0) - (px.data[start + 3] ?? 0)) <= tolerance;
  const walk = (sx: number, sy: number) => {
    let n = 0;
    let cx = ix + sx;
    let cy = iy + sy;
    while (cx >= 0 && cy >= 0 && cx < px.width && cy < px.height && same(at(cx, cy))) {
      n += 1;
      cx += sx;
      cy += sy;
    }
    return n;
  };
  return { left: walk(-1, 0), right: walk(1, 0), up: walk(0, -1), down: walk(0, 1) };
}

export interface Box {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** The space between two boxes: across (`dx`) when one is left of the other, down
 * (`dy`) when one is above; null when they overlap that way. */
export function gap(a: Box, b: Box): { dx: number | null; dy: number | null } {
  const across = b.x >= a.x + a.w ? b.x - (a.x + a.w) : a.x >= b.x + b.w ? a.x - (b.x + b.w) : null;
  const down = b.y >= a.y + a.h ? b.y - (a.y + a.h) : a.y >= b.y + b.h ? a.y - (b.y + b.h) : null;
  const round = (v: number | null) => (v === null ? null : Math.round(v));
  return { dx: round(across), dy: round(down) };
}

/** A dimension's words: its length, "100 px". */
export function rulerLabel(from: Point, to: Point): string {
  return `${Math.round(rulerOf(from, to).length)} px`;
}

/** A ruler's words while it is laid: "100 px · 0° · ΔX 100 · ΔY 0". */
export function rulerDetails(from: Point, to: Point): string {
  const r = rulerOf(from, to);
  return `${Math.round(r.length)} px · ${Math.round(r.angle) % 360}° · ΔX ${r.dx} · ΔY ${r.dy}`;
}

/** A ruler made a dimension (EDT-094): a line with a bar at each end and its length in
 * a text centered above the middle (below it at the top edge), inside the canvas
 * (`width` × `height`), in the current styles of Line and Text. Objects for
 * `document.annotate`. */
export function dimensionObjects(from: Point, to: Point, width: number, height: number): unknown[] {
  const mid: Point = [(from[0] + to[0]) / 2, (from[1] + to[1]) / 2];
  const box = Math.min(200, width);
  const above = mid[1] - 36;
  const y = above >= 0 ? above : Math.min(mid[1] + 10, Math.max(0, height - 30));
  const x = Math.max(0, Math.min(mid[0] - box / 2, width - box));
  return [
    {
      type: "line",
      from,
      to,
      style: { startCap: "bar", endCap: "bar", lineStyle: "solid" },
    },
    {
      type: "text",
      at: [x, y],
      width: box,
      text: rulerLabel(from, to),
      style: { align: "center" },
    },
  ];
}
