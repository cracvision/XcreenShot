import { describe, expect, it } from "vitest";
import { evaluate, keepRatio } from "./positionMath";

describe("position and size fields (EDT-086)", () => {
  it("take numbers and simple expressions on the current value", () => {
    expect(evaluate("100", 7)).toBe(100);
    expect(evaluate("+10", 90)).toBe(100);
    expect(evaluate("-5", 90)).toBe(85);
    expect(evaluate("*2", 150)).toBe(300);
    expect(evaluate("/2", 300)).toBe(150);
    expect(evaluate("=-5", 90)).toBe(-5);
    expect(evaluate("12,5", 0)).toBe(12.5);
  });

  it("refuse text and division by zero", () => {
    expect(evaluate("abc", 10)).toBeNull();
    expect(evaluate("", 10)).toBeNull();
    expect(evaluate("/0", 10)).toBeNull();
  });

  it("keep the proportion when the lock is closed", () => {
    expect(keepRatio(300, 200, "w", 600)).toEqual({ w: 600, h: 400 });
    expect(keepRatio(300, 200, "h", 100)).toEqual({ w: 150, h: 100 });
  });
});
