import { describe, expect, it } from "vitest";
import { hsbToRgb, PALETTE, parseHex, rgbToHsb, toHex } from "./color";

describe("colors (EDT-019)", () => {
  it("reads hex as RGB", () => {
    expect(parseHex("#2D7FF9")).toEqual({ r: 45, g: 127, b: 249, a: 255 });
    expect(parseHex("2d7ff980")).toEqual({ r: 45, g: 127, b: 249, a: 128 });
    expect(parseHex("#XYZ")).toBeNull();
  });

  it("writes hex, with alpha only when not opaque", () => {
    expect(toHex({ r: 45, g: 127, b: 249, a: 255 })).toBe("#2D7FF9");
    expect(toHex({ r: 0, g: 0, b: 0, a: 128 })).toBe("#00000080");
  });

  it("round-trips through HSB", () => {
    for (const hex of ["#2D7FF9", "#F2545B", "#000000", "#FFFFFF", "#16A34A"]) {
      const c = parseHex(hex);
      if (!c) throw new Error(hex);
      expect(toHex(hsbToRgb(rgbToHsb(c)))).toBe(hex);
    }
  });

  it("has a palette of 40 distinct colors", () => {
    expect(PALETTE).toHaveLength(40);
    expect(new Set(PALETTE).size).toBe(40);
  });
});
