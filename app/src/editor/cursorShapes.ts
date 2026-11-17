// The pointer as an object (EDT-093): the standard shapes as polygons in the unit
// square, where each one points, and its highlight. The same as
// `crates/xs-image/src/cursor.rs`, which draws them for the final image.
import type { Rect } from "./boxes";

export interface CursorLook {
  shape: "captured" | "arrow" | "hand" | "text" | "cross";
  highlight: "none" | "yellow" | "accent";
  /** The captured pointer's size in pixels: 100 %. */
  natural: [number, number];
}

export const CURSOR_SHAPES = ["captured", "arrow", "hand", "text", "cross"] as const;
export const CURSOR_HIGHLIGHTS = ["none", "yellow", "accent"] as const;

const POLYGONS: Record<string, [number, number][]> = {
  arrow: [
    [0.0, 0.0],
    [0.0, 0.82],
    [0.22, 0.64],
    [0.36, 0.98],
    [0.5, 0.92],
    [0.36, 0.6],
    [0.64, 0.6],
  ],
  hand: [
    [0.36, 0.0],
    [0.48, 0.0],
    [0.5, 0.42],
    [0.62, 0.36],
    [0.74, 0.4],
    [0.86, 0.44],
    [0.96, 0.52],
    [0.94, 0.8],
    [0.8, 1.0],
    [0.36, 1.0],
    [0.12, 0.66],
    [0.06, 0.5],
    [0.16, 0.46],
    [0.34, 0.62],
  ],
  text: [
    [0.2, 0.0],
    [0.8, 0.0],
    [0.8, 0.1],
    [0.56, 0.1],
    [0.56, 0.9],
    [0.8, 0.9],
    [0.8, 1.0],
    [0.2, 1.0],
    [0.2, 0.9],
    [0.44, 0.9],
    [0.44, 0.1],
    [0.2, 0.1],
  ],
  cross: [
    [0.44, 0.0],
    [0.56, 0.0],
    [0.56, 0.44],
    [1.0, 0.44],
    [1.0, 0.56],
    [0.56, 0.56],
    [0.56, 1.0],
    [0.44, 1.0],
    [0.44, 0.56],
    [0.0, 0.56],
    [0.0, 0.44],
    [0.44, 0.44],
  ],
};

/** The shape's outline as flat points in its box (for a closed Konva line). */
export function cursorPoints(shape: string, r: Rect): number[] {
  const poly = POLYGONS[shape] ?? POLYGONS.arrow ?? [];
  return poly.flatMap(([x, y]) => [r.x + x * r.w, r.y + y * r.h]);
}

/** The highlight circle, centred where the pointer points, if it has one. */
export function highlightCircle(
  look: CursorLook,
  r: Rect,
): { x: number; y: number; r: number } | null {
  if (look.highlight === "none") return null;
  const [hx, hy] =
    look.shape === "hand"
      ? [0.42, 0.02]
      : look.shape === "text" || look.shape === "cross"
        ? [0.5, 0.5]
        : [0.04, 0.04];
  return { x: r.x + hx * r.w, y: r.y + hy * r.h, r: Math.max(r.w, r.h) * 0.75 };
}

/** The box of a cursor at `percent` of its natural size, keeping its top-left corner. */
export function cursorRect(r: Rect, look: CursorLook, percent: number): Rect {
  const k = Math.min(400, Math.max(50, percent)) / 100;
  const [w, h] = look.natural[0] > 0 ? look.natural : [r.w, r.h];
  return { x: r.x, y: r.y, w: Math.round(w * k), h: Math.round(h * k) };
}

/** Its size now, as a percentage of the natural one. */
export function cursorPercent(r: Rect, look: CursorLook): number {
  return look.natural[0] > 0 ? Math.round((r.w / look.natural[0]) * 100) : 100;
}
