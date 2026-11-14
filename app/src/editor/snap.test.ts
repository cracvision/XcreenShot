import { describe, expect, it } from "vitest";
import { snapMove, snapPoint } from "./snap";

const canvas = { x: 0, y: 0, w: 800, h: 600 };

describe("snapping (EDT-084)", () => {
  it("pulls a box near the canvas center onto it, with guides", () => {
    // A 100 × 50 box whose center lands at (403, 297): 3 px off on each axis.
    const start = { x: 0, y: 0, w: 100, h: 50 };
    const r = snapMove(start, 353, 272, canvas, [], 6);
    expect(r.dx).toBe(350);
    expect(r.dy).toBe(275);
    expect(r.guides.some((g) => g.axis === "x" && g.at === 400)).toBe(true);
    expect(r.guides.some((g) => g.axis === "y" && g.at === 300)).toBe(true);
  });

  it("does not snap beyond the reach", () => {
    const r = snapMove({ x: 0, y: 0, w: 100, h: 50 }, 340, 200, canvas, [], 6);
    expect([r.dx, r.dy]).toEqual([340, 200]);
    expect(r.guides).toEqual([]);
  });

  it("lines up with another object's edge", () => {
    const other = { x: 500, y: 100, w: 80, h: 80 };
    // Left edge at 497 snaps to 500.
    const r = snapMove({ x: 0, y: 300, w: 60, h: 40 }, 497, 0, canvas, [other], 6);
    expect(r.dx).toBe(500);
    expect(r.guides[0]?.at).toBe(500);
  });

  it("leaves the same space as a pair in the same row", () => {
    const a = { x: 100, y: 100, w: 50, h: 50 };
    const b = { x: 200, y: 100, w: 50, h: 50 };
    // Gap a-b is 50; the third box after b snaps to x = 300.
    const r = snapMove({ x: 0, y: 100, w: 50, h: 50 }, 303, 0, canvas, [a, b], 6);
    expect(r.dx).toBe(300);
    expect(r.spacings.length).toBe(2);
  });

  it("centers a box between two neighbors", () => {
    const a = { x: 100, y: 400, w: 50, h: 50 };
    const b = { x: 350, y: 400, w: 50, h: 50 };
    // Room 200 for a 50 box: 75 on each side, x = 225.
    const r = snapMove({ x: 0, y: 400, w: 50, h: 50 }, 228, 0, canvas, [a, b], 6);
    expect(r.dx).toBe(225);
  });

  it("snaps a resize handle to a line", () => {
    const r = snapPoint([797, 120], { x: true, y: false }, canvas, [], 6);
    expect(r.point).toEqual([800, 120]);
    expect(r.guides[0]?.axis).toBe("x");
    const none = snapPoint([700, 120], { x: true, y: true }, canvas, [], 6);
    expect(none.point).toEqual([700, 120]);
  });
});
