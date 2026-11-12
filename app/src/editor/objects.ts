// What the canvas needs to know about each object: its box, whether the pointer is on
// it, and the change a move or a handle makes (EDT-022, EDT-042). Pure functions.
import { corners, type Box, type Rect } from "./boxes";
import { distanceToSegment, type Point } from "./geometry";
import { isSegment, lensOf, type DocObject, type TextObject } from "./state";

/** The box of an object drawn in a rectangle (text, shape, blur, image), or null for
 * arrows. */
export function boxOf(o: DocObject): Box | null {
  switch (o.type) {
    case "shape":
    case "blur":
    case "image":
    case "step":
    case "callout":
    case "highlight":
    case "spotlight":
      return { ...o.rect, rotation: o.rotation };
    case "magnify":
      // The lens (EDT-038).
      return { ...lensOf(o), rotation: 0 };
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
  if (isSegment(o)) return distanceToSegment(p, o.from, o.to) <= Math.max(o.style.width / 2, reach);
  // A stroke is picked along its points (EDT-037).
  if (o.type === "pen") {
    const r = Math.max(o.style.width / 2, reach);
    const pts = o.points;
    if (pts.length === 1) {
      const [only] = pts;
      return only ? Math.hypot(p[0] - only[0], p[1] - only[1]) <= r : false;
    }
    for (let i = 1; i < pts.length; i++) {
      const a = pts[i - 1];
      const b = pts[i];
      if (a && b && distanceToSegment(p, a, b) <= r) return true;
    }
    return false;
  }
  // A callout is also picked by its tails (EDT-028).
  if (o.type === "callout") {
    const c: Point = [o.rect.x + o.rect.w / 2, o.rect.y + o.rect.h / 2];
    const near = o.tails.some(
      (t) => distanceToSegment(p, c, t) <= Math.max(o.style.tailWidth / 4, reach),
    );
    if (near) return true;
  }
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
    case "arrow":
    case "line": {
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
    case "blur":
    case "step":
    case "highlight":
    case "spotlight": {
      const rect = { ...o.rect, x: o.rect.x + dx, y: o.rect.y + dy };
      return [{ ...o, rect }, { rect }];
    }
    case "pen": {
      const points = o.points.map((q): Point => [q[0] + dx, q[1] + dy]);
      return [{ ...o, points }, { points }];
    }
    case "magnify": {
      const source = { ...o.source, x: o.source.x + dx, y: o.source.y + dy };
      const at: Point = [o.at[0] + dx, o.at[1] + dy];
      return [
        { ...o, source, at },
        { source, at },
      ];
    }
    case "callout": {
      // The tails go with the body (EDT-028).
      const rect = { ...o.rect, x: o.rect.x + dx, y: o.rect.y + dy };
      const tails = o.tails.map((t): Point => [t[0] + dx, t[1] + dy]);
      return [
        { ...o, rect, tails },
        { rect, tails },
      ];
    }
    case "image": {
      // Whole pixels keep a picture sharp (EDT-078).
      const rect = { ...o.rect, x: Math.round(o.rect.x + dx), y: Math.round(o.rect.y + dy) };
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
    case "image": {
      const rect: Rect = {
        x: Math.round(b.x),
        y: Math.round(b.y),
        w: Math.max(1, Math.round(b.w)),
        h: Math.max(1, Math.round(b.h)),
      };
      return [
        { ...o, rect, rotation: b.rotation },
        { rect, rotation: b.rotation },
      ];
    }
    case "shape":
    case "blur":
    case "highlight":
    case "spotlight": {
      const rect: Rect = { x: b.x, y: b.y, w: b.w, h: b.h };
      return [
        { ...o, rect, rotation: b.rotation },
        { rect, rotation: b.rotation },
      ];
    }
    case "magnify": {
      // A bigger lens shows the same zone more enlarged (EDT-038).
      const zoom = Math.round(Math.min(800, Math.max(110, (b.w / Math.max(1, o.source.w)) * 100)));
      const at: Point = [b.x + b.w / 2, b.y + b.h / 2];
      return [
        { ...o, at, style: { ...o.style, zoom } },
        { at, style: { zoom } },
      ];
    }
    case "callout": {
      // The body changes; the tails keep pointing where they did (EDT-028).
      const rect: Rect = { x: b.x, y: b.y, w: b.w, h: b.h };
      return [
        { ...o, rect, rotation: b.rotation },
        { rect, rotation: b.rotation },
      ];
    }
    case "step": {
      // Square, 16 to 256 px; its size becomes the tool's (EDT-035, EDT-014).
      const size = Math.round(Math.min(256, Math.max(16, (b.w + b.h) / 2)));
      const rect: Rect = {
        x: b.x + b.w / 2 - size / 2,
        y: b.y + b.h / 2 - size / 2,
        w: size,
        h: size,
      };
      return [
        { ...o, rect, rotation: b.rotation, style: { ...o.style, size } },
        { rect, rotation: b.rotation, style: { size } },
      ];
    }
    default:
      return [o, {}];
  }
}

/** Canvas points an object reaches, to tell whether it is outside the canvas. */
export function extentPoints(o: DocObject): Point[] {
  if (isSegment(o)) return [o.from, o.to];
  if (o.type === "pen") return o.points;
  if (o.type === "magnify") {
    const l = lensOf(o);
    const z = o.source;
    return [
      [z.x, z.y],
      [z.x + z.w, z.y + z.h],
      [l.x, l.y],
      [l.x + l.w, l.y + l.h],
    ];
  }
  const b = boxOf(o);
  if (o.type === "callout" && b) return [...corners(b), ...o.tails];
  return b ? corners(b) : [];
}

/** Objects wholly or partly outside a `width` × `height` canvas (EDT-049, EDT-011).
 * A blur is left out: past the edge it has nothing to hide. Less than half a pixel out
 * is inside, as when the canvas grows to fit objects (EDT-071). */
export function outside(objects: DocObject[], width: number, height: number): string[] {
  return objects
    .filter(
      (o) =>
        o.type !== "blur" &&
        extentPoints(o).some(
          (p) => p[0] <= -0.5 || p[1] <= -0.5 || p[0] >= width + 0.5 || p[1] >= height + 0.5,
        ),
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

/**
 * Whether an object falls in a selection box (EDT-079): all of it, or with `touch`
 * (Alt) any part of it.
 */
export function inBand(
  o: DocObject,
  r: { x: number; y: number; w: number; h: number },
  touch: boolean,
): boolean {
  const pts = extentPoints(o);
  if (pts.length === 0) return false;
  const xs = pts.map((q) => q[0]);
  const ys = pts.map((q) => q[1]);
  const [x0, x1, y0, y1] = [Math.min(...xs), Math.max(...xs), Math.min(...ys), Math.max(...ys)];
  if (touch) return x1 >= r.x && x0 <= r.x + r.w && y1 >= r.y && y0 <= r.y + r.h;
  return x0 >= r.x && x1 <= r.x + r.w && y0 >= r.y && y1 <= r.y + r.h;
}
