// Quick annotation (CAP-057), its marks: after selecting an area, mark it in place with a few of the
// editor's tools and copy it, without opening the editor. The marks are drawn here as a
// preview; they become the editor's own objects (EDT-024, EDT-029, EDT-034, EDT-035,
// EDT-036, EDT-040), drawn for real by Rust when the capture is made.
import type { Rect } from "./geometry";

interface Pt {
  x: number;
  y: number;
}

export type Tool = "arrow" | "shape" | "text" | "highlight" | "blur" | "step";
export const TOOLS: Tool[] = ["arrow", "shape", "text", "highlight", "blur", "step"];

/** The marks' colors: the default theme's (EDT-017). */
export const COLORS = ["#F2545B", "#FFB000", "#2563EB", "#16A34A", "#111318", "#FFFFFF"];
/** Thin, medium, thick, in physical pixels at 100 %. */
export const WIDTHS = [2, 4, 8];

/** A mark, in physical pixels of the desktop. */
export type Mark =
  | { kind: "arrow"; from: Pt; to: Pt; color: string; width: number }
  | { kind: "shape"; rect: Rect; ellipse: boolean; color: string; width: number }
  | { kind: "highlight"; rect: Rect; color: string }
  | { kind: "blur"; rect: Rect }
  | { kind: "step"; at: Pt; value: number; color: string; size: number }
  | { kind: "text"; at: Pt; text: string; color: string; size: number };

const normal = (a: Pt, b: Pt): Rect => ({
  x: Math.min(a.x, b.x),
  y: Math.min(a.y, b.y),
  width: Math.abs(b.x - a.x),
  height: Math.abs(b.y - a.y),
});

/** The mark a drag from `a` to `b` makes with `tool`; null when too small. */
export function markFromDrag(
  tool: Tool,
  a: Pt,
  b: Pt,
  color: string,
  width: number,
  ellipse: boolean,
): Mark | null {
  const r = normal(a, b);
  const tiny = Math.hypot(b.x - a.x, b.y - a.y) < 4;
  if (tool === "arrow") return tiny ? null : { kind: "arrow", from: a, to: b, color, width };
  if (tiny || r.width < 2 || r.height < 2) return null;
  if (tool === "shape") return { kind: "shape", rect: r, ellipse, color, width };
  if (tool === "highlight") return { kind: "highlight", rect: r, color };
  if (tool === "blur") return { kind: "blur", rect: r };
  return null;
}

/** The number's color on a step of `fill`: dark on the light colors. */
export function inkOn(fill: string): string {
  return luminance(fill) > 0.45 ? "#111318" : "#FFFFFF";
}

/** The relative luminance of a `#RRGGBB` color, 0 (black) to 1 (white). */
export function luminance(hex: string): number {
  const m = /^#?([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})/i.exec(hex);
  if (!m) return 0;
  const lin = (h: string) => {
    const c = parseInt(h, 16) / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * lin(m[1] as string) + 0.7152 * lin(m[2] as string) + 0.0722 * lin(m[3] as string);
}

/** The color a mark starts with: the theme's first that is neither near black nor
 * near white, else its first. */
export function firstColor(colors: string[]): string {
  const vivid = colors.find((c) => {
    const l = luminance(c);
    return l > 0.04 && l < 0.85;
  });
  return vivid ?? colors[0] ?? (COLORS[0] as string);
}

/** The marks as the editor's objects, in the capture's pixels (`sel` is its corner).
 * Text sizes are points at the capture's resolution (`scale` × 96 dpi). */
export function toObjects(marks: Mark[], sel: Rect, scale: number): Record<string, unknown>[] {
  const points = (px: number) => Math.max(1, Math.round((px * 72) / (96 * Math.max(0.1, scale))));
  const px = (p: Pt) => [Math.round(p.x - sel.x), Math.round(p.y - sel.y)];
  const box = (r: Rect) => ({
    x: Math.round(r.x - sel.x),
    y: Math.round(r.y - sel.y),
    w: Math.max(1, Math.round(r.width)),
    h: Math.max(1, Math.round(r.height)),
  });
  return marks.map((m) => {
    switch (m.kind) {
      case "arrow":
        return {
          type: "arrow",
          from: px(m.from),
          to: px(m.to),
          style: { color: m.color, width: m.width },
        };
      case "shape":
        return {
          type: "shape",
          rect: box(m.rect),
          style: {
            shapeType: m.ellipse ? "ellipse" : "rectangle",
            fill: null,
            outline: m.color,
            outlineWidth: m.width,
          },
        };
      case "highlight":
        return { type: "highlight", rect: box(m.rect), style: { color: m.color } };
      case "blur":
        return { type: "blur", rect: box(m.rect) };
      case "step": {
        const half = m.size / 2;
        return {
          type: "step",
          rect: box({ x: m.at.x - half, y: m.at.y - half, width: m.size, height: m.size }),
          // The editor keeps a step's value as text (EDT-035).
          value: String(m.value),
          style: { fill: m.color, textColor: inkOn(m.color), size: m.size },
        };
      }
      case "text":
        return {
          type: "text",
          at: px(m.at),
          text: m.text,
          style: { color: m.color, size: points(m.size) },
        };
    }
  });
}

/** Draws the marks on the selection layer of the monitor whose corner is `M`. */
export function drawMarks(ctx: CanvasRenderingContext2D, marks: Mark[], M: Rect) {
  for (const m of marks) {
    ctx.save();
    switch (m.kind) {
      case "arrow": {
        const [x1, y1, x2, y2] = [m.from.x - M.x, m.from.y - M.y, m.to.x - M.x, m.to.y - M.y];
        const angle = Math.atan2(y2 - y1, x2 - x1);
        const head = Math.max(12, m.width * 4);
        ctx.strokeStyle = m.color;
        ctx.fillStyle = m.color;
        ctx.lineWidth = m.width;
        ctx.lineCap = "round";
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2 - Math.cos(angle) * head * 0.8, y2 - Math.sin(angle) * head * 0.8);
        ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(x2, y2);
        ctx.lineTo(x2 - Math.cos(angle - 0.45) * head, y2 - Math.sin(angle - 0.45) * head);
        ctx.lineTo(x2 - Math.cos(angle + 0.45) * head, y2 - Math.sin(angle + 0.45) * head);
        ctx.closePath();
        ctx.fill();
        break;
      }
      case "shape": {
        const r = m.rect;
        ctx.strokeStyle = m.color;
        ctx.lineWidth = m.width;
        ctx.beginPath();
        if (m.ellipse)
          ctx.ellipse(
            r.x - M.x + r.width / 2,
            r.y - M.y + r.height / 2,
            r.width / 2,
            r.height / 2,
            0,
            0,
            Math.PI * 2,
          );
        else ctx.rect(r.x - M.x, r.y - M.y, r.width, r.height);
        ctx.stroke();
        break;
      }
      case "highlight":
        ctx.globalAlpha = 0.4;
        ctx.fillStyle = m.color;
        ctx.fillRect(m.rect.x - M.x, m.rect.y - M.y, m.rect.width, m.rect.height);
        break;
      case "blur": {
        // A preview: a frosted box with a hatch (Rust blurs the real pixels).
        const r = m.rect;
        ctx.fillStyle = "rgba(200, 200, 210, 0.85)";
        ctx.fillRect(r.x - M.x, r.y - M.y, r.width, r.height);
        ctx.strokeStyle = "rgba(90, 90, 100, 0.35)";
        ctx.lineWidth = 1;
        for (let k = -r.height; k < r.width; k += 10) {
          ctx.beginPath();
          ctx.moveTo(r.x - M.x + k, r.y - M.y + r.height);
          ctx.lineTo(r.x - M.x + k + r.height, r.y - M.y);
          ctx.stroke();
        }
        break;
      }
      case "step": {
        const [cx, cy] = [m.at.x - M.x, m.at.y - M.y];
        ctx.fillStyle = m.color;
        ctx.beginPath();
        ctx.arc(cx, cy, m.size / 2, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = inkOn(m.color);
        ctx.font = `700 ${Math.round(m.size * 0.55)}px "Segoe UI", system-ui, sans-serif`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(String(m.value), cx, cy + 1);
        break;
      }
      case "text":
        ctx.fillStyle = m.color;
        ctx.font = `600 ${m.size}px "Segoe UI", system-ui, sans-serif`;
        ctx.textBaseline = "top";
        m.text
          .split("\n")
          .forEach((line, i) => ctx.fillText(line, m.at.x - M.x, m.at.y - M.y + i * m.size * 1.3));
        break;
    }
    ctx.restore();
  }
}

/** The box a mark takes, in desktop pixels (text: an estimate of its lines). */
export function markBounds(m: Mark): Rect {
  switch (m.kind) {
    case "arrow":
      return normal(m.from, m.to);
    case "shape":
    case "highlight":
    case "blur":
      return m.rect;
    case "step":
      return { x: m.at.x - m.size / 2, y: m.at.y - m.size / 2, width: m.size, height: m.size };
    case "text": {
      const lines = m.text.split("\n");
      const longest = Math.max(...lines.map((l) => l.length));
      return {
        x: m.at.x,
        y: m.at.y,
        width: longest * m.size * 0.6,
        height: lines.length * m.size * 1.3,
      };
    }
  }
}

const distanceToSegment = (p: Pt, a: Pt, b: Pt) => {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const t = Math.max(
    0,
    Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / (dx * dx + dy * dy || 1)),
  );
  return Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
};

/** The topmost mark under `p` (within `slack` pixels), or -1 (CAP-057). */
export function hitMark(marks: Mark[], p: Pt, slack: number): number {
  for (let i = marks.length - 1; i >= 0; i--) {
    const m = marks[i] as Mark;
    if (m.kind === "arrow") {
      if (distanceToSegment(p, m.from, m.to) <= Math.max(slack, m.width)) return i;
      continue;
    }
    if (m.kind === "step") {
      if (Math.hypot(p.x - m.at.x, p.y - m.at.y) <= m.size / 2 + slack / 2) return i;
      continue;
    }
    const r = markBounds(m);
    if (
      p.x >= r.x - slack / 2 &&
      p.x <= r.x + r.width + slack / 2 &&
      p.y >= r.y - slack / 2 &&
      p.y <= r.y + r.height + slack / 2
    )
      return i;
  }
  return -1;
}

/** A mark moved by (`dx`, `dy`). */
export function moveMark(m: Mark, dx: number, dy: number): Mark {
  const pt = (q: Pt) => ({ x: q.x + dx, y: q.y + dy });
  const box = (r: Rect) => ({ ...r, x: r.x + dx, y: r.y + dy });
  switch (m.kind) {
    case "arrow":
      return { ...m, from: pt(m.from), to: pt(m.to) };
    case "shape":
    case "highlight":
    case "blur":
      return { ...m, rect: box(m.rect) };
    case "step":
    case "text":
      return { ...m, at: pt(m.at) };
  }
}
