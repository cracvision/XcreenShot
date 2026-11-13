// Freehand (CAP-032): the shapes drawn on the frozen screens, in desktop pixels.

export interface Pt {
  x: number;
  y: number;
}

export type FreehandShape = "free" | "polygon" | "ellipse";

/** An ellipse in the box from `a` to `b` (a circle with `circle`), as 72 points. */
export function ellipsePoints(a: Pt, b: Pt, circle: boolean): Pt[] {
  let w = b.x - a.x;
  let h = b.y - a.y;
  if (circle) {
    const side = Math.max(Math.abs(w), Math.abs(h));
    w = Math.sign(w || 1) * side;
    h = Math.sign(h || 1) * side;
  }
  const cx = a.x + w / 2;
  const cy = a.y + h / 2;
  const rx = Math.abs(w) / 2;
  const ry = Math.abs(h) / 2;
  if (rx < 2 || ry < 2) return [];
  return Array.from({ length: 72 }, (_, k) => {
    const t = (k / 72) * Math.PI * 2;
    return { x: cx + rx * Math.cos(t), y: cy + ry * Math.sin(t) };
  });
}

/** Adds `p` to a free outline when it moved far enough from the last point. */
export function extend(path: Pt[], p: Pt, step: number): Pt[] {
  const last = path[path.length - 1];
  if (last && Math.hypot(p.x - last.x, p.y - last.y) < step) return path;
  return [...path, p];
}

/** Whether a shape encloses something: three points and some area. */
export function closes(points: Pt[]): boolean {
  if (points.length < 3) return false;
  let area = 0;
  for (let i = 0; i < points.length; i++) {
    const a = points[i] as Pt;
    const b = points[(i + 1) % points.length] as Pt;
    area += a.x * b.y - b.x * a.y;
  }
  return Math.abs(area) / 2 >= 16;
}

/** The cursor of Freehand: scissors (CAP-032). */
export const SCISSORS =
  "url(\"data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24'><text x='2' y='19' font-size='20'>%E2%9C%82</text></svg>\") 4 12, crosshair";
