// Geometry of annotation objects, in canvas pixels. A copy of crates/xs-image/src/
// geometry.rs, the reference: tests/fixtures/arrow-geometry.json keeps both in step,
// so what the canvas shows is what an export writes (ADR 0004).

export type Cap =
  | "none"
  | "filled"
  | "open"
  | "thin"
  | "concave"
  | "circle"
  | "circleHollow"
  | "square"
  | "diamond"
  | "bar";

export type LineStyle = "solid" | "dash" | "dot" | "dashDot";

export const CAPS: Cap[] = [
  "none",
  "filled",
  "open",
  "thin",
  "concave",
  "circle",
  "circleHollow",
  "square",
  "diamond",
  "bar",
];
export const LINE_STYLES: LineStyle[] = ["solid", "dash", "dot", "dashDot"];

/** Arrow properties of M1 (EDT-024). */
export interface ArrowStyle {
  color: string;
  width: number;
  startCap: Cap;
  endCap: Cap;
  lineStyle: LineStyle;
  opacity: number;
}

export type Point = [number, number];

export type Shape =
  | { kind: "path"; points: number[]; closed: boolean; fill: boolean; strokeWidth: number }
  | { kind: "circle"; cx: number; cy: number; r: number; fill: boolean; strokeWidth: number };

export interface Geometry {
  shaft: [number, number, number, number] | null;
  width: number;
  dash: number[] | null;
  caps: Shape[];
}

export function dash(style: LineStyle, width: number): number[] | null {
  switch (style) {
    case "solid":
      return null;
    case "dash":
      return [3 * width, 2 * width];
    case "dot":
      return [width, width];
    case "dashDot":
      return [3 * width, 1.5 * width, width, 1.5 * width];
  }
}

function cap(kind: Cap, tip: Point, u: Point, w: number, scale: number): [Shape[], number] {
  const n: Point = [-u[1], u[0]];
  const l = (3 * w + 6) * scale;
  const h = 0.45 * (3 * w + 6) * scale;
  const r = (1.5 * w + 2) * scale;
  const at = (a: number, b: number): Point => [
    tip[0] + a * u[0] + b * n[0],
    tip[1] + a * u[1] + b * n[1],
  ];
  const path = (pts: Point[], closed: boolean, fill: boolean, strokeWidth: number): Shape => ({
    kind: "path",
    points: pts.flat(),
    closed,
    fill,
    strokeWidth,
  });
  switch (kind) {
    case "none":
      return [[], 0];
    case "filled":
      return [[path([tip, at(-l, h), at(-l, -h)], true, true, 0)], l / 2];
    case "open":
      return [[path([at(-l, h), tip, at(-l, -h)], false, false, w)], w / 2];
    case "thin": {
      const ht = 0.3 * l;
      return [[path([at(-l, ht), tip, at(-l, -ht)], false, false, w)], w / 2];
    }
    case "concave":
      return [[path([tip, at(-l, h), at(-0.65 * l, 0), at(-l, -h)], true, true, 0)], 0.45 * l];
    case "circle":
      return [[{ kind: "circle", cx: tip[0], cy: tip[1], r, fill: true, strokeWidth: 0 }], 0];
    case "circleHollow":
      return [
        [
          {
            kind: "circle",
            cx: tip[0],
            cy: tip[1],
            r,
            fill: false,
            strokeWidth: Math.max(w * 0.5, 1),
          },
        ],
        r,
      ];
    case "square":
      return [[path([at(r, r), at(r, -r), at(-r, -r), at(-r, r)], true, true, 0)], 0];
    case "diamond":
      return [[path([at(1.3 * r, 0), at(0, r), at(-1.3 * r, 0), at(0, -r)], true, true, 0)], 0];
    case "bar":
      return [[path([at(0, h), at(0, -h)], false, false, w)], 0];
  }
}

export function arrowGeometry(from: Point, to: Point, style: ArrowStyle): Geometry {
  const w = style.width;
  const dx = to[0] - from[0];
  const dy = to[1] - from[1];
  const len = Math.hypot(dx, dy);
  if (len < 1e-6) return { shaft: null, width: w, dash: null, caps: [] };
  const d: Point = [dx / len, dy / len];
  // Ends never take more than 40 % of the arrow each.
  const scale = Math.min((0.4 * len) / (3 * w + 6), 1);
  const [startCaps, startInset] = cap(style.startCap, from, [-d[0], -d[1]], w, scale);
  const [endCaps, endInset] = cap(style.endCap, to, d, w, scale);
  return {
    shaft: [
      from[0] + startInset * d[0],
      from[1] + startInset * d[1],
      to[0] - endInset * d[0],
      to[1] - endInset * d[1],
    ],
    width: w,
    dash: dash(style.lineStyle, w),
    caps: [...startCaps, ...endCaps],
  };
}

/** `to` moved so the arrow from `from` is horizontal, vertical or at 45° (EDT-022 Shift). */
export function constrain45(from: Point, to: Point): Point {
  const dx = to[0] - from[0];
  const dy = to[1] - from[1];
  const len = Math.hypot(dx, dy);
  if (len === 0) return to;
  const step = Math.PI / 4;
  const angle = Math.round(Math.atan2(dy, dx) / step) * step;
  return [from[0] + len * Math.cos(angle), from[1] + len * Math.sin(angle)];
}

/** Distance from point `p` to the segment a–b. */
export function distanceToSegment(p: Point, a: Point, b: Point): number {
  const dx = b[0] - a[0];
  const dy = b[1] - a[1];
  const len2 = dx * dx + dy * dy;
  const t =
    len2 === 0 ? 0 : Math.max(0, Math.min(1, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2));
  return Math.hypot(p[0] - (a[0] + t * dx), p[1] - (a[1] + t * dy));
}
