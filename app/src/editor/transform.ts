// Several objects changed as one (EDT-080): stretched to a new box or turned around a
// point. A copy of crates/xs-editor/src/transform.rs for the canvas preview; the core
// does the real change (`document.transform`), and the tests of both check the same
// cases. A text's new height is only estimated here: the core lays it out again.
import type { Rect } from "./boxes";
import type { Point } from "./geometry";
import { boxOf, extentPoints } from "./objects";
import { isSegment, type DocObject } from "./state";

export type TransformOp =
  | { kind: "scale"; from: Rect; to: Rect; strokes: boolean }
  | { kind: "rotate"; center: Point; degrees: number };

/** The unturned box around `objects` (EDT-079), or null. */
export function boundsOf(objects: DocObject[]): Rect | null {
  const pts = objects.flatMap(extentPoints);
  if (pts.length === 0) return null;
  const xs = pts.map((p) => p[0]);
  const ys = pts.map((p) => p[1]);
  const x = Math.min(...xs);
  const y = Math.min(...ys);
  return { x, y, w: Math.max(...xs) - x, h: Math.max(...ys) - y };
}

/** Degrees in (-180, 180]. */
export function normalized(degrees: number): number {
  let r = degrees % 360;
  if (r > 180) r -= 360;
  else if (r <= -180) r += 360;
  return r;
}

function turn(p: Point, c: Point, degrees: number): Point {
  const a = (degrees * Math.PI) / 180;
  const [dx, dy] = [p[0] - c[0], p[1] - c[1]];
  return [c[0] + dx * Math.cos(a) - dy * Math.sin(a), c[1] + dx * Math.sin(a) + dy * Math.cos(a)];
}

function factors(from: Rect, to: Rect): [number, number] {
  const f = (a: number, b: number) => (Math.abs(a) < 1e-9 ? 1 : Math.abs(b / a));
  return [f(from.w, to.w), f(from.h, to.h)];
}

function mapped(p: Point, from: Rect, to: Rect): Point {
  const [sx, sy] = factors(from, to);
  return [to.x + (p[0] - from.x) * sx, to.y + (p[1] - from.y) * sy];
}

const grown = (v: number, k: number, min: number, max: number) =>
  Math.min(max, Math.max(min, Math.round(v * k * 10) / 10));

/** `o` after `op`, as the canvas shows it while the gesture goes on. */
export function transformed(o: DocObject, op: TransformOp): DocObject {
  if (isSegment(o)) {
    if (op.kind === "rotate")
      return {
        ...o,
        from: turn(o.from, op.center, op.degrees),
        to: turn(o.to, op.center, op.degrees),
      };
    const [sx, sy] = factors(op.from, op.to);
    const width = op.strokes ? grown(o.style.width, Math.sqrt(sx * sy), 1, 100) : o.style.width;
    return {
      ...o,
      from: mapped(o.from, op.from, op.to),
      to: mapped(o.to, op.from, op.to),
      style: { ...o.style, width },
    };
  }
  const b = boxOf(o);
  if (!b) return o;
  const a = (b.rotation * Math.PI) / 180;
  const [sin, cos] = [Math.sin(a), Math.cos(a)];
  const c0: Point = [b.x + b.w / 2, b.y + b.h / 2];
  let c: Point;
  let w = b.w;
  let h = b.h;
  let rotation = b.rotation;
  let k = 1;
  if (op.kind === "scale") {
    const [sx, sy] = factors(op.from, op.to);
    // Each side stretched along the object's own turned axes.
    w = b.w * Math.hypot(sx * cos, sy * sin);
    h = b.h * Math.hypot(sx * sin, sy * cos);
    c = mapped(c0, op.from, op.to);
    if (op.strokes) k = Math.sqrt(sx * sy);
  } else {
    c = turn(c0, op.center, op.degrees);
    rotation = normalized(b.rotation + op.degrees);
  }
  const rect = { x: c[0] - w / 2, y: c[1] - h / 2, w: Math.max(1, w), h: Math.max(1, h) };
  switch (o.type) {
    case "text": {
      const size = grown(o.style.size, k, 1, 1000);
      // About as many lines as before, each grown with the letters.
      const height = o.layout.height * (size / o.style.size);
      return {
        ...o,
        at: [c[0] - w / 2, c[1] - height / 2],
        width: w,
        rotation,
        layout: { width: w, height },
        style: { ...o.style, size },
      };
    }
    case "shape":
      return {
        ...o,
        rect,
        rotation,
        style: { ...o.style, outlineWidth: grown(o.style.outlineWidth, k, 0, 100) },
      };
    case "blur":
    case "image":
      return { ...o, rect, rotation };
    case "callout": {
      const tails = o.tails.map((t) =>
        op.kind === "rotate" ? turn(t, op.center, op.degrees) : mapped(t, op.from, op.to),
      );
      const style =
        op.kind === "scale" && op.strokes
          ? {
              ...o.style,
              outlineWidth: grown(o.style.outlineWidth, k, 0, 20),
              size: grown(o.style.size, k, 1, 1000),
              tailWidth: grown(o.style.tailWidth, k, 2, 200),
            }
          : o.style;
      return { ...o, rect, rotation, tails, style };
    }
    case "step": {
      // Square: the core keeps it so (EDT-035).
      const size = Math.min(256, Math.max(16, (rect.w + rect.h) / 2));
      const sq = { x: c[0] - size / 2, y: c[1] - size / 2, w: size, h: size };
      return { ...o, rect: sq, rotation, style: { ...o.style, size } };
    }
    default:
      return o;
  }
}
