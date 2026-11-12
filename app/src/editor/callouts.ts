// Callouts (EDT-028): where the text goes in the body, a copy of
// crates/xs-image/src/callouts.rs (`text_box`), so typing happens where Rust draws.
import type { Rect } from "./boxes";
import type { CalloutObject } from "./state";

/** How far in the text starts, as a share of the body, for round shapes. */
export function inset(shape: CalloutObject["style"]["shape"]): number {
  switch (shape) {
    case "ellipse":
    case "cloud":
    case "thought":
      return 0.12;
    case "explosion":
      return 0.18;
    case "arrow":
      return 0.1;
    default:
      return 0;
  }
}

/** The part of the body the text goes in, unturned. */
export function textBox(c: CalloutObject): Rect {
  const r = c.rect;
  const k = inset(c.style.shape);
  const p = c.style.padding + c.style.outlineWidth;
  const ix = r.w * k + p;
  const iy = r.h * k + p;
  const left = c.style.shape === "tag" ? ix + r.h * 0.35 : ix;
  return {
    x: r.x + left,
    y: r.y + iy,
    w: Math.max(1, r.w - left - ix),
    h: Math.max(1, r.h - 2 * iy),
  };
}

/** Rectangular shapes have a corner radius to drag. */
export function hasRadius(c: CalloutObject): boolean {
  return c.style.shape === "roundedRectangle";
}

export const MAX_TAILS = 8;
