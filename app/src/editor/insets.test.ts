import { describe, expect, it } from "vitest";
import { insetsFrom, scrollForDrag, thumbOf } from "./insets";

describe("the part of the canvas the floating bars leave (Cristal)", () => {
  it("each bar covers its size plus its gap to the edge", () => {
    expect(insetsFrom(54, 300, 130)).toEqual({ top: 66, right: 312, bottom: 142, left: 0 });
  });

  it("a bar that is not there covers nothing", () => {
    expect(insetsFrom(54, 0, 0)).toEqual({ top: 66, right: 0, bottom: 0, left: 0 });
  });
});

describe("the canvas's thin scroll bars", () => {
  it("show nothing when the image fits", () => {
    expect(thumbOf(800, 800, 0, 700)).toBeNull();
    expect(thumbOf(500, 800, 0, 700)).toBeNull();
  });

  it("size the thumb by how much is visible and place it by the scroll", () => {
    expect(thumbOf(2000, 1000, 0, 1000)).toEqual({ start: 0, length: 500 });
    expect(thumbOf(2000, 1000, 1000, 1000)).toEqual({ start: 500, length: 500 });
    expect(thumbOf(2000, 1000, 500, 1000)).toEqual({ start: 250, length: 500 });
  });

  it("never make the thumb too small to grab", () => {
    expect(thumbOf(100000, 1000, 0, 1000)?.length).toBe(32);
  });

  it("turn a drag of the thumb into a scroll", () => {
    expect(scrollForDrag(2000, 1000, 1000, 500, 250)).toBe(500);
    expect(scrollForDrag(2000, 1000, 500, 500, 10)).toBe(0);
  });
});
