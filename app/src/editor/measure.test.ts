import { describe, expect, it } from "vitest";
import {
  dimensionObjects,
  edges,
  gap,
  rulerDetails,
  rulerLabel,
  rulerOf,
  type Pixels,
} from "./measure";

describe("live measurement (EDT-094)", () => {
  it("measures a ruler: length, angle and its parts", () => {
    expect(rulerOf([10, 10], [110, 10])).toEqual({ length: 100, angle: 0, dx: 100, dy: 0 });
    expect(rulerOf([0, 100], [0, 0]).angle).toBe(90);
    expect(rulerOf([0, 0], [30, 40]).length).toBe(50);
    expect(rulerOf([0, 0], [-10, 0]).angle).toBe(180);
    expect(rulerLabel([10, 10], [110, 10])).toBe("100 px");
    expect(rulerDetails([10, 10], [110, 10])).toBe("100 px · 0° · ΔX 100 · ΔY 0");
    expect(rulerDetails([0, 0], [100, -100])).toBe("141 px · 45° · ΔX 100 · ΔY -100");
  });

  it("finds the edges of what is under the pointer", () => {
    // 100 × 60 white, with a gray box from (20, 10) to (69, 39).
    const width = 100;
    const height = 60;
    const data = new Uint8ClampedArray(width * height * 4).fill(255);
    for (let y = 10; y < 40; y++)
      for (let x = 20; x < 70; x++) {
        const i = (y * width + x) * 4;
        data.set([120, 120, 120, 255], i);
      }
    const px: Pixels = { width, height, data };
    expect(edges(px, 30, 20)).toEqual({ left: 10, right: 39, up: 10, down: 19 });
    // Outside the box: up to it, or to the image's edge.
    expect(edges(px, 5, 20)).toEqual({ left: 5, right: 14, up: 20, down: 39 });
    expect(edges(px, 200, 20)).toBeNull();
  });

  it("measures the space between two objects", () => {
    const a = { x: 10, y: 10, w: 100, h: 50 };
    expect(gap(a, { x: 150, y: 20, w: 30, h: 30 })).toEqual({ dx: 40, dy: null });
    expect(gap(a, { x: 10, y: 80, w: 30, h: 30 })).toEqual({ dx: null, dy: 20 });
    expect(gap({ x: 150, y: 20, w: 30, h: 30 }, a)).toEqual({ dx: 40, dy: null });
    expect(gap(a, { x: 50, y: 30, w: 10, h: 10 })).toEqual({ dx: null, dy: null });
  });

  it("makes a ruler a dimension: a line with bars and its length, inside the canvas", () => {
    const [line, text] = dimensionObjects([10, 10], [110, 10], 800, 500) as [
      { type: string; from: number[]; style: { startCap: string; endCap: string } },
      { type: string; text: string; at: number[] },
    ];
    expect(line.type).toBe("line");
    expect(line.from).toEqual([10, 10]);
    expect([line.style.startCap, line.style.endCap]).toEqual(["bar", "bar"]);
    expect(text).toMatchObject({ type: "text", text: "100 px" });
    // No room above at the top edge: below the line, never left of the canvas.
    expect(text.at).toEqual([0, 20]);
    const [, lower] = dimensionObjects([300, 200], [500, 200], 800, 500) as [
      unknown,
      { at: number[] },
    ];
    expect(lower.at).toEqual([300, 164]);
  });
});
