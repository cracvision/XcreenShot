// Align and distribute objects (EDT-083): how much each one moves. Objects are given by
// their extent; against several selected the frame is their common box, against one the
// canvas.

export interface Extent {
  id: string;
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}

export type AlignTo = "left" | "hcenter" | "right" | "top" | "vcenter" | "bottom";
export type Moves = Record<string, [number, number]>;

/** The common box of some extents. */
export function frameOf(items: Extent[]): Extent {
  return {
    id: "",
    x0: Math.min(...items.map((i) => i.x0)),
    y0: Math.min(...items.map((i) => i.y0)),
    x1: Math.max(...items.map((i) => i.x1)),
    y1: Math.max(...items.map((i) => i.y1)),
  };
}

export function alignMoves(items: Extent[], to: AlignTo, frame: Extent): Moves {
  const out: Moves = {};
  for (const i of items) {
    const w = i.x1 - i.x0;
    const h = i.y1 - i.y0;
    let dx = 0;
    let dy = 0;
    if (to === "left") dx = frame.x0 - i.x0;
    else if (to === "right") dx = frame.x1 - i.x1;
    else if (to === "hcenter") dx = (frame.x0 + frame.x1) / 2 - (i.x0 + w / 2);
    else if (to === "top") dy = frame.y0 - i.y0;
    else if (to === "bottom") dy = frame.y1 - i.y1;
    else dy = (frame.y0 + frame.y1) / 2 - (i.y0 + h / 2);
    if (dx !== 0 || dy !== 0) out[i.id] = [dx, dy];
  }
  return out;
}

/** The same space between neighbours, left to right (or top to bottom); the first and
 *  the last stay where they are. */
export function distributeMoves(items: Extent[], axis: "h" | "v"): Moves {
  if (items.length < 3) return {};
  const lo = (i: Extent) => (axis === "h" ? i.x0 : i.y0);
  const hi = (i: Extent) => (axis === "h" ? i.x1 : i.y1);
  const sorted = [...items].sort((a, b) => lo(a) - lo(b));
  const first = sorted[0] as Extent;
  const last = sorted[sorted.length - 1] as Extent;
  const span = hi(last) - lo(first);
  const sizes = sorted.reduce((sum, i) => sum + (hi(i) - lo(i)), 0);
  const gap = (span - sizes) / (sorted.length - 1);
  const out: Moves = {};
  let at = hi(first) + gap;
  for (const i of sorted.slice(1, -1)) {
    const d = at - lo(i);
    if (d !== 0) out[i.id] = axis === "h" ? [d, 0] : [0, d];
    at += hi(i) - lo(i) + gap;
  }
  return out;
}
