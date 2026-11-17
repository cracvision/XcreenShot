import { describe, expect, it } from "vitest";
import { zoomOf } from "./zoom";

describe("library thumbnail size (LIB-003)", () => {
  it("stays within 50 and 300 % in steps of 10", () => {
    expect(zoomOf(undefined)).toBe(100);
    expect(zoomOf(20)).toBe(50);
    expect(zoomOf(999)).toBe(300);
    expect(zoomOf(204)).toBe(200);
    expect(zoomOf(Number.NaN)).toBe(100);
  });
});
