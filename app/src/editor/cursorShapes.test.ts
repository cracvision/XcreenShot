import { describe, expect, it } from "vitest";
import {
  cursorPercent,
  cursorRect,
  highlightCircle,
  cursorPoints,
  type CursorLook,
} from "./cursorShapes";

const look: CursorLook = { shape: "hand", highlight: "yellow", natural: [32, 32] };

describe("cursor object (EDT-093)", () => {
  it("scales from its natural size and keeps its corner", () => {
    const r = cursorRect({ x: 10, y: 20, w: 32, h: 32 }, look, 200);
    expect(r).toEqual({ x: 10, y: 20, w: 64, h: 64 });
    expect(cursorPercent(r, look)).toBe(200);
    expect(cursorRect(r, look, 1000).w).toBe(128);
  });

  it("draws its shape in its box, with the highlight at its tip", () => {
    const pts = cursorPoints("arrow", { x: 0, y: 0, w: 100, h: 100 });
    expect(pts.slice(0, 2)).toEqual([0, 0]);
    expect(highlightCircle(look, { x: 0, y: 0, w: 100, h: 100 })).toEqual({ x: 42, y: 2, r: 75 });
    expect(highlightCircle({ ...look, highlight: "none" }, { x: 0, y: 0, w: 1, h: 1 })).toBeNull();
  });
});
