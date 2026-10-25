// What the canvas needs to know about each object: its box, whether the pointer is on
// it, and the change a move or a handle makes (EDT-022, EDT-042). Pure functions.
import { corners, type Box, type Rect } from "./boxes";
import { distanceToSegment, type Point } from "./geometry";
import type { DocObject, TextObject } from "./state";

/** The box of an object drawn in a rectangle (text, shape, blur), or null for arrows. */
export function boxOf(o: DocObject): Box | null {
  switch (o.type) {
    case "shape":
    case "blur":
      return { ...o.rect, rotation: o.rotation };
    case "text":
      return {
        x: o.at[0],
        y: o.at[1],
        w: o.width ?? o.layout.width,
        h: o.layout.height,
        rotation: o.rotation,
      };
    default:
      return null;
  }
}

/** Whether canvas point `p` is on the object; `reach` is the pointer tolerance. */
export function hits(o: DocObject, p: Point, reach: number): boolean {
  if (o.type === "arrow")
    return distanceToSegment(p, o.from, o.to) <= Math.max(o.style.width / 2, reach);
  const b = boxOf(o);
  if (!b) return false;
  const local = toLocalAbs(b, p);
  const inside = local[0] <= b.w / 2 + reach && local[1] <= b.h / 2 + reach;
  // A transparent shape is picked by its outline, so what is inside stays reachable.
  if (o.type === "shape" && o.style.fill === null) {
    const edge = Math.max(o.style.outlineWidth, 2 * reach);
    return inside && (local[0] >= b.w / 2 - edge || local[1] >= b.h / 2 - edge);
  }
  return inside;
}

function toLocalAbs(b: Box, p: Point): Point {
  const cx = b.x + b.w / 2;
  const cy = b.y + b.h / 2;
  const a = (-b.rotation * Math.PI) / 180;
  const dx = p[0] - cx;
  const dy = p[1] - cy;
  return [
    Math.abs(dx * Math.cos(a) - dy * Math.sin(a)),
    Math.abs(dx * Math.sin(a) + dy * Math.cos(a)),
  ];
}

/** The object moved by (`dx`, `dy`), and the `set` for `document.update_object`. */
export function moved(o: DocObject, dx: number, dy: number): [DocObject, Record<string, unknown>] {
  switch (o.type) {
    case "arrow": {
      const from: Point = [o.from[0] + dx, o.from[1] + dy];
      const to: Point = [o.to[0] + dx, o.to[1] + dy];
      return [
        { ...o, from, to },
        { from, to },
      ];
    }
    case "text": {
      const at: Point = [o.at[0] + dx, o.at[1] + dy];
      return [{ ...o, at }, { at }];
    }
    case "shape":
    case "blur": {
      const rect = { ...o.rect, x: o.rect.x + dx, y: o.rect.y + dy };
      return [{ ...o, rect }, { rect }];
    }
    default:
      return [o, {}];
  }
}

/** The object given a new box (resized or turned), and the `set` for the core. A
 * text box keeps its laid-out height: only its width and place change. */
export function reboxed(o: DocObject, b: Box): [DocObject, Record<string, unknown>] {
  switch (o.type) {
    case "text": {
      const at: Point = [b.x, b.y];
      const changes = { at, width: b.w, rotation: b.rotation };
      const layout = { ...o.layout, width: b.w };
      return [{ ...o, ...changes, layout } as TextObject, changes];
    }
    case "shape":
    case "blur": {
      const rect: Rect = { x: b.x, y: b.y, w: b.w, h: b.h };
      return [
        { ...o, rect, rotation: b.rotation },
        { rect, rotation: b.rotation },
      ];
    }
    default:
      return [o, {}];
  }
}

/** Canvas points an object reaches, to tell whether it is outside the canvas. */
export function extentPoints(o: DocObject): Point[] {
  if (o.type === "arrow") return [o.from, o.to];
  const b = boxOf(o);
  return b ? corners(b) : [];
}

/** Objects wholly or partly outside a `width` × `height` canvas (EDT-049, EDT-011). */
export function outside(objects: DocObject[], width: number, height: number): string[] {
  return objects
    .filter((o) =>
      extentPoints(o).some((p) => p[0] < 0 || p[1] < 0 || p[0] > width || p[1] > height),
    )
    .map((o) => o.id);
}

/** A short fingerprint of an object, so the canvas reloads its image only when it
 * changes. */
export function fingerprint(o: DocObject): string {
  const text = JSON.stringify(o);
  let h = 2166136261;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return (h >>> 0).toString(36);
}
