import { describe, expect, it } from "vitest";
import { nextLevel } from "./fit";

describe("a bar that fits its width (EDT-001)", () => {
  it("climbs while its content is wider than it is", () => {
    expect(nextLevel(0, 1300, 1000, 8, new Map())).toBe(1);
    expect(nextLevel(8, 1300, 1000, 8, new Map())).toBe(8);
  });

  it("comes down when the level below fits, or to try it once", () => {
    expect(nextLevel(3, 900, 1000, 8, new Map([[2, 980]]))).toBe(2);
    expect(nextLevel(3, 900, 1000, 8, new Map())).toBe(2);
    // Known not to fit: it stays, so it never goes up and down.
    expect(nextLevel(3, 900, 1000, 8, new Map([[2, 1100]]))).toBe(3);
    expect(nextLevel(0, 900, 1000, 8, new Map())).toBe(0);
  });
});

describe("the labels a level hides", () => {
  it("adds one kind of label per level, up to three", async () => {
    const { fitClasses } = await import("./fit");
    expect(fitClasses(0)).toBe("");
    expect(fitClasses(2)).toBe(" fit-1 fit-2");
    expect(fitClasses(9)).toBe(" fit-1 fit-2 fit-3");
  });
});
