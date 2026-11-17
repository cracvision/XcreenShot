import { describe, expect, it } from "vitest";
import { corners, fromPoints, refit, sideAt } from "./polygon";
import type { Point } from "./geometry";

describe("polygons (EDT-030)", () => {
  const tri: Point[] = [
    [10, 10],
    [30, 10],
    [20, 40],
  ];

  it("keeps its corners as shares of its box", () => {
    const { rect, points } = fromPoints(tri);
    expect(rect).toEqual({ x: 10, y: 10, w: 20, h: 30 });
    expect(points).toEqual([
      [0, 0],
      [1, 0],
      [0.5, 1],
    ]);
    expect(corners({ ...rect, rotation: 0 }, points)).toEqual(tri);
  });

  it("a corner moved out of the box makes the box bigger, the others stay", () => {
    const { rect, points } = fromPoints(tri);
    const box = { ...rect, rotation: 0 };
    const moved = corners(box, points).map((p, i): Point => (i === 2 ? [20, 60] : p));
    const next = refit(box, moved);
    expect(next.rect).toEqual({ x: 10, y: 10, w: 20, h: 50 });
    const back = corners({ ...next.rect, rotation: 0 }, next.points);
    expect(back[0]?.[0]).toBeCloseTo(10);
    expect(back[2]?.[1]).toBeCloseTo(60);
  });

  it("finds the side under the pointer", () => {
    expect(sideAt(tri, [20, 10], 1)).toBe(0);
    expect(sideAt(tri, [0, 0], 1)).toBe(-1);
  });
});
