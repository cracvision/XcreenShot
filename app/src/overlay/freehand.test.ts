import { describe, expect, it } from "vitest";
import { closes, ellipsePoints, extend } from "./freehand";

describe("freehand shapes (CAP-032)", () => {
  it("an ellipse, and a circle with Shift", () => {
    const e = ellipsePoints({ x: 0, y: 0 }, { x: 200, y: 100 }, false);
    expect(e).toHaveLength(72);
    expect(Math.max(...e.map((p) => p.x)) - Math.min(...e.map((p) => p.x))).toBeCloseTo(200, 0);
    const c = ellipsePoints({ x: 0, y: 0 }, { x: 200, y: 100 }, true);
    const h = Math.max(...c.map((p) => p.y)) - Math.min(...c.map((p) => p.y));
    expect(h).toBeCloseTo(200, 0);
    expect(ellipsePoints({ x: 0, y: 0 }, { x: 1, y: 1 }, false)).toEqual([]);
  });

  it("a free outline keeps points a little apart", () => {
    let path = extend([], { x: 0, y: 0 }, 3);
    path = extend(path, { x: 1, y: 1 }, 3);
    path = extend(path, { x: 5, y: 0 }, 3);
    expect(path).toHaveLength(2);
  });

  it("a shape must enclose something", () => {
    expect(
      closes([
        { x: 0, y: 0 },
        { x: 10, y: 0 },
      ]),
    ).toBe(false);
    expect(
      closes([
        { x: 0, y: 0 },
        { x: 10, y: 0 },
        { x: 20, y: 0 },
      ]),
    ).toBe(false);
    expect(
      closes([
        { x: 0, y: 0 },
        { x: 40, y: 0 },
        { x: 20, y: 30 },
      ]),
    ).toBe(true);
  });
});
