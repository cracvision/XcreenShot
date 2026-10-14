import { describe, expect, it } from "vitest";
import {
  bounding,
  clampInside,
  fromPoints,
  intersect,
  lockAspect,
  nextZoom,
  placeBar,
  placeMagnifier,
  ratioFromKeys,
  resizeByHandle,
  toHex,
} from "./geometry";

describe("selection rectangles", () => {
  it("has exactly the dragged size, odd sizes included (CAP-009)", () => {
    expect(fromPoints({ x: 10, y: 20 }, { x: 311, y: 221 })).toEqual({
      x: 10,
      y: 20,
      width: 301,
      height: 201,
    });
    expect(fromPoints({ x: 311, y: 221 }, { x: 10, y: 20 })).toEqual({
      x: 10,
      y: 20,
      width: 301,
      height: 201,
    });
  });

  it("is at least 1×1", () => {
    expect(fromPoints({ x: 5, y: 5 }, { x: 5, y: 5 })).toEqual({ x: 5, y: 5, width: 1, height: 1 });
  });

  it("intersects and bounds monitors", () => {
    const a = { x: 0, y: 0, width: 100, height: 100 };
    const b = { x: 100, y: 20, width: 50, height: 50 };
    expect(intersect(a, b)).toBeNull();
    expect(bounding([a, b])).toEqual({ x: 0, y: 0, width: 150, height: 100 });
  });
});

describe("aspect lock (CAP-018)", () => {
  it("Shift gives a square", () => {
    const b = lockAspect({ x: 0, y: 0 }, { x: 300, y: 120 }, ratioFromKeys(true, false) ?? 0);
    const r = fromPoints({ x: 0, y: 0 }, b);
    expect(r.width).toBe(r.height);
  });

  it("Alt gives 16:9 and Alt+Shift 4:3, also dragging up-left", () => {
    const r169 = fromPoints(
      { x: 0, y: 0 },
      lockAspect({ x: 0, y: 0 }, { x: 1600, y: 300 }, 16 / 9),
    );
    expect(r169).toEqual({ x: 0, y: 0, width: 1600, height: 900 });
    const a = { x: 1000, y: 1000 };
    const r43 = fromPoints(a, lockAspect(a, { x: 600, y: 990 }, ratioFromKeys(true, true) ?? 0));
    expect(r43.width / r43.height).toBeCloseTo(4 / 3, 2);
  });
});

describe("adjusting (CAP-011)", () => {
  it("resizes by a corner handle", () => {
    const r = { x: 10, y: 10, width: 100, height: 50 };
    expect(resizeByHandle(r, "se", { x: 210, y: 110 })).toEqual({
      x: 10,
      y: 10,
      width: 200,
      height: 100,
    });
    expect(resizeByHandle(r, "w", { x: 0, y: 999 })).toEqual({
      x: 0,
      y: 10,
      width: 110,
      height: 50,
    });
  });

  it("clamps values outside the screen", () => {
    const screen = { x: 0, y: 0, width: 1920, height: 1080 };
    expect(clampInside({ x: 1800, y: 0, width: 5000, height: 100 }, screen)).toEqual({
      x: 0,
      y: 0,
      width: 1920,
      height: 100,
    });
  });

  it("puts the bar below, else above, else inside", () => {
    const screen = { x: 0, y: 0, width: 1000, height: 800 };
    const bar = { width: 300, height: 40 };
    expect(placeBar({ x: 100, y: 100, width: 400, height: 200 }, bar, screen).y).toBe(308);
    expect(placeBar({ x: 100, y: 500, width: 400, height: 280 }, bar, screen).y).toBe(452);
    const inside = placeBar({ x: 0, y: 0, width: 1000, height: 800 }, bar, screen);
    expect(inside.y).toBeGreaterThan(700);
  });
});

describe("magnifier (CAP-015)", () => {
  it("stays fully on screen near the bottom-right corner", () => {
    const screen = { x: 0, y: 0, width: 1920, height: 1080 };
    const p = placeMagnifier({ x: 1915, y: 1075 }, { width: 200, height: 220 }, screen);
    expect(p.x + 200).toBeLessThanOrEqual(1920);
    expect(p.y + 220).toBeLessThanOrEqual(1080);
  });

  it("steps through ×2, ×4, ×8, ×16 and shows hex colors", () => {
    expect(nextZoom(4, true)).toBe(8);
    expect(nextZoom(16, true)).toBe(16);
    expect(nextZoom(2, false)).toBe(2);
    expect(toHex(14, 140, 127)).toBe("#0E8C7F");
  });
});
