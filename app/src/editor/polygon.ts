// Polygons (EDT-030): their corners are kept as `[u, v]` from 0 to 1 across their box,
// so they turn and stretch with it like any shape. These helpers go between those and
// canvas points.
import { toLocal, toWorld, type Box, type Rect } from "./boxes";
import type { Point } from "./geometry";

/** The box around `pts` (canvas points) and the corners as shares of it. */
export function fromPoints(pts: Point[]): { rect: Rect; points: Point[] } {
  const xs = pts.map((p) => p[0]);
  const ys = pts.map((p) => p[1]);
  const x = Math.min(...xs);
  const y = Math.min(...ys);
  const w = Math.max(1, Math.max(...xs) - x);
  const h = Math.max(1, Math.max(...ys) - y);
  return { rect: { x, y, w, h }, points: pts.map((p): Point => [(p[0] - x) / w, (p[1] - y) / h]) };
}

/** The canvas point of each corner of a polygon in `box`. */
export function corners(box: Box, points: Point[]): Point[] {
  return points.map((p) => toWorld(box, [(p[0] - 0.5) * box.w, (p[1] - 0.5) * box.h]));
}

/**
 * The polygon with its corners at canvas points `pts`, keeping its turn: a new box
 * around them in its own frame, and the corners as shares of that box.
 */
export function refit(box: Box, pts: Point[]): { rect: Rect; points: Point[] } {
  const local = pts.map((p) => toLocal(box, p));
  const xs = local.map((p) => p[0]);
  const ys = local.map((p) => p[1]);
  const x0 = Math.min(...xs);
  const y0 = Math.min(...ys);
  const w = Math.max(1, Math.max(...xs) - x0);
  const h = Math.max(1, Math.max(...ys) - y0);
  // The new box's center, back on the canvas.
  const c = toWorld(box, [x0 + w / 2, y0 + h / 2]);
  return {
    rect: { x: c[0] - w / 2, y: c[1] - h / 2, w, h },
    points: local.map((p): Point => [(p[0] - x0) / w, (p[1] - y0) / h]),
  };
}

/** Where on the outline a point falls: the side `i` (from corner i to i + 1), or -1. */
export function sideAt(pts: Point[], p: Point, reach: number): number {
  for (let i = 0; i < pts.length; i++) {
    const a = pts[i];
    const b = pts[(i + 1) % pts.length];
    if (!a || !b) continue;
    const dx = b[0] - a[0];
    const dy = b[1] - a[1];
    const len2 = dx * dx + dy * dy;
    const t =
      len2 === 0 ? 0 : Math.max(0, Math.min(1, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2));
    if (Math.hypot(p[0] - (a[0] + t * dx), p[1] - (a[1] + t * dy)) <= reach) return i;
  }
  return -1;
}
