// Snapping (EDT-084): a moved or resized box lines up its edges and center with the
// canvas and the other objects, or leaves the same space as between other objects.
// Pure geometry in canvas pixels; the canvas draws the guides it returns.
import type { Rect } from "./boxes";

/** A line to draw: vertical at `x` (or horizontal at `y`) from `from` to `to`. */
export interface Guide {
  axis: "x" | "y";
  at: number;
  from: number;
  to: number;
}

/** The same space shown between boxes: a measure from `a` to `b` on one axis, at `cross`. */
export interface Spacing {
  axis: "x" | "y";
  a: number;
  b: number;
  cross: number;
}

export interface Snapped {
  /** The move after snapping. */
  dx: number;
  dy: number;
  guides: Guide[];
  spacings: Spacing[];
}

const lines = (r: Rect, axis: "x" | "y"): number[] =>
  axis === "x" ? [r.x, r.x + r.w / 2, r.x + r.w] : [r.y, r.y + r.h / 2, r.y + r.h];

const span = (r: Rect, axis: "x" | "y"): [number, number] =>
  axis === "x" ? [r.x, r.x + r.w] : [r.y, r.y + r.h];

/** Whether two boxes face each other across `axis` (they overlap on the other one). */
const facing = (a: Rect, b: Rect, axis: "x" | "y"): boolean => {
  const [a0, a1] = span(a, axis === "x" ? "y" : "x");
  const [b0, b1] = span(b, axis === "x" ? "y" : "x");
  return a0 < b1 && b0 < a1;
};

interface Hit {
  shift: number;
  guides: Guide[];
  spacings: Spacing[];
}

/** The best snap of `box` on one axis: to edges and centers, or to an equal space. */
function snapAxis(
  box: Rect,
  canvas: Rect,
  others: Rect[],
  axis: "x" | "y",
  reach: number,
): Hit | null {
  const own = lines(box, axis);
  const cross = axis === "x" ? "y" : "x";
  let best: Hit | null = null;
  const consider = (shift: number, make: () => Omit<Hit, "shift">) => {
    if (Math.abs(shift) > reach) return;
    if (best && Math.abs(shift) > Math.abs(best.shift) + 1e-9) return;
    const found = { shift, ...make() };
    if (best && Math.abs(shift - best.shift) < 1e-9) {
      best = {
        shift,
        guides: [...best.guides, ...found.guides],
        spacings: [...best.spacings, ...found.spacings],
      };
    } else best = found;
  };
  // Edges and centers of the canvas and of the other objects.
  for (const target of [canvas, ...others]) {
    const [c0, c1] = span(target, cross);
    for (const t of lines(target, axis))
      for (const o of own)
        consider(t - o, () => {
          const [b0, b1] = span(box, cross);
          return {
            guides: [{ axis, at: t, from: Math.min(c0, b0), to: Math.max(c1, b1) }],
            spacings: [],
          };
        });
  }
  // The same space: between two neighbors, or continuing a row of two or more.
  const row = others.filter((o) => facing(o, box, axis));
  const [s0, s1] = span(box, axis);
  const before = row
    .filter((o) => span(o, axis)[1] <= s0 + reach)
    .sort((a, b) => span(b, axis)[1] - span(a, axis)[1]);
  const after = row
    .filter((o) => span(o, axis)[0] >= s1 - reach)
    .sort((a, b) => span(a, axis)[0] - span(b, axis)[0]);
  const mid = (a: Rect, b: Rect) => {
    const [a0, a1] = span(a, cross);
    const [b0, b1] = span(b, cross);
    return (Math.max(a0, b0) + Math.min(a1, b1)) / 2;
  };
  const prev = before[0];
  const next = after[0];
  if (prev && next) {
    // Centered between its two neighbors.
    const room = span(next, axis)[0] - span(prev, axis)[1] - (s1 - s0);
    if (room >= 0) {
      const target = span(prev, axis)[1] + room / 2;
      consider(target - s0, () => ({
        guides: [],
        spacings: [
          { axis, a: span(prev, axis)[1], b: target, cross: mid(prev, box) },
          { axis, a: target + (s1 - s0), b: span(next, axis)[0], cross: mid(box, next) },
        ],
      }));
    }
  }
  const prev2 = before[1];
  if (prev && prev2) {
    // After a pair, with their gap.
    const gap = span(prev, axis)[0] - span(prev2, axis)[1];
    if (gap >= 0) {
      const target = span(prev, axis)[1] + gap;
      consider(target - s0, () => ({
        guides: [],
        spacings: [
          { axis, a: span(prev2, axis)[1], b: span(prev, axis)[0], cross: mid(prev2, prev) },
          { axis, a: span(prev, axis)[1], b: target, cross: mid(prev, box) },
        ],
      }));
    }
  }
  const next2 = after[1];
  if (next && next2) {
    // Before a pair, with their gap.
    const gap = span(next2, axis)[0] - span(next, axis)[1];
    if (gap >= 0) {
      const target = span(next, axis)[0] - gap - (s1 - s0);
      consider(target - s0, () => ({
        guides: [],
        spacings: [
          { axis, a: target + (s1 - s0), b: span(next, axis)[0], cross: mid(box, next) },
          { axis, a: span(next, axis)[1], b: span(next2, axis)[0], cross: mid(next, next2) },
        ],
      }));
    }
  }
  return best;
}

/**
 * Snaps a box moved by (`dx`, `dy`) from `start` (EDT-084). `reach` is the snapping
 * distance in canvas pixels (6 screen pixels at the current zoom).
 */
export function snapMove(
  start: Rect,
  dx: number,
  dy: number,
  canvas: Rect,
  others: Rect[],
  reach: number,
): Snapped {
  const box = { ...start, x: start.x + dx, y: start.y + dy };
  const hx = snapAxis(box, canvas, others, "x", reach);
  const hy = snapAxis(box, canvas, others, "y", reach);
  const sx = hx?.shift ?? 0;
  const sy = hy?.shift ?? 0;
  // Guides along one axis span the box as it ends up on the other.
  const moved = { ...box, x: box.x + sx, y: box.y + sy };
  const fit = (g: Guide): Guide => {
    const [b0, b1] = span(moved, g.axis === "x" ? "y" : "x");
    return { ...g, from: Math.min(g.from, b0), to: Math.max(g.to, b1) };
  };
  return {
    dx: dx + sx,
    dy: dy + sy,
    guides: [...(hx?.guides ?? []), ...(hy?.guides ?? [])].map(fit),
    spacings: [...(hx?.spacings ?? []), ...(hy?.spacings ?? [])],
  };
}

/**
 * Snaps a point being dragged by a resize handle (EDT-084): its x (or y) goes to the
 * nearest edge or center line within `reach`, for the sides that the handle moves.
 */
export function snapPoint(
  p: [number, number],
  moves: { x: boolean; y: boolean },
  canvas: Rect,
  others: Rect[],
  reach: number,
): { point: [number, number]; guides: Guide[] } {
  const out: [number, number] = [p[0], p[1]];
  const guides: Guide[] = [];
  for (const axis of ["x", "y"] as const) {
    if (!moves[axis]) continue;
    const i = axis === "x" ? 0 : 1;
    let best: { t: number; d: number; target: Rect } | null = null;
    for (const target of [canvas, ...others])
      for (const t of lines(target, axis)) {
        const d = Math.abs(t - p[i]);
        if (d <= reach && (!best || d < best.d)) best = { t, d, target };
      }
    if (best) {
      out[i] = best.t;
      const [c0, c1] = span(best.target, axis === "x" ? "y" : "x");
      const q = p[axis === "x" ? 1 : 0];
      guides.push({ axis, at: best.t, from: Math.min(c0, q), to: Math.max(c1, q) });
    }
  }
  return { point: out, guides };
}
