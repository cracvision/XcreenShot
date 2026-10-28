import { describe, expect, it } from "vitest";
import fixture from "../../../tests/fixtures/shape-geometry.json";
import { contains, dragRect, innerRect, pixelBounds, resize, rotateTo, type Box } from "./boxes";

interface Case {
  rect: { x: number; y: number; w: number; h: number };
  outline: number;
  rotation: number;
  inner: { x: number; y: number; w: number; h: number };
  bounds: [number, number, number, number] | null;
}

const near = (a: number, b: number) => Math.abs(a - b) < 1e-9;

describe("boxes (ADR 0004, EDT-022)", () => {
  it("match the renderer of exported images", () => {
    for (const c of fixture as unknown as Case[]) {
      const inner = innerRect(c.rect, c.outline);
      expect(near(inner.x, c.inner.x) && near(inner.w, c.inner.w) && near(inner.h, c.inner.h)).toBe(
        true,
      );
      expect(pixelBounds(c.rect, c.rotation, 400, 200)).toEqual(c.bounds);
    }
  });

  const box: Box = { x: 100, y: 100, w: 200, h: 100, rotation: 0 };

  it("resizes from the opposite side", () => {
    const r = resize(box, "se", [350, 250]);
    expect([r.x, r.y, r.w, r.h]).toEqual([100, 100, 250, 150]);
    const l = resize(box, "w", [50, 999]);
    expect([l.x, l.w, l.h]).toEqual([50, 250, 100]);
  });

  it("keeps the proportion with Shift and the center with Alt", () => {
    const r = resize(box, "se", [500, 160], { keepRatio: true });
    expect(r.w / r.h).toBeCloseTo(2);
    const c = resize(box, "e", [350, 150], { fromCenter: true });
    expect(c.x + c.w / 2).toBeCloseTo(200);
    expect(c.w).toBeCloseTo(100 + 2 * 50 + 100);
  });

  it("keeps the center of a turned box when resizing from the center", () => {
    const turned: Box = { ...box, rotation: 30 };
    const r = resize(turned, "se", [400, 300], { fromCenter: true });
    expect(r.x + r.w / 2).toBeCloseTo(200);
    expect(r.y + r.h / 2).toBeCloseTo(150);
    expect(r.rotation).toBe(30);
  });

  it("turns toward the pointer, in 15° steps with Shift", () => {
    expect(rotateTo(box, [400, 150], false)).toBeCloseTo(90);
    expect(rotateTo(box, [210, 0], true) % 15).toBe(0);
  });

  it("draws squares and from the center", () => {
    expect(dragRect([0, 0], [30, 10], true, false)).toEqual({ x: 0, y: 0, w: 30, h: 30 });
    expect(dragRect([100, 100], [130, 120], false, true)).toEqual({ x: 70, y: 80, w: 60, h: 40 });
    expect(dragRect([50, 50], [10, 20], false, false)).toEqual({ x: 10, y: 20, w: 40, h: 30 });
  });

  it("knows what is inside a turned box", () => {
    const t: Box = { ...box, rotation: 90 };
    expect(contains(t, [200, 210])).toBe(true);
    expect(contains(t, [290, 150])).toBe(false);
  });
});
