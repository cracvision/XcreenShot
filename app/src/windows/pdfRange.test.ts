import { describe, expect, it } from "vitest";
import { formatRange, parseRange } from "./pdfRange";

describe("PDF page ranges (CAP-041)", () => {
  it("reads ranges", () => {
    expect(parseRange("", 3)).toEqual([1, 2, 3]);
    expect(parseRange("2-3", 5)).toEqual([2, 3]);
    expect(parseRange("1-3, 7", 9)).toEqual([1, 2, 3, 7]);
    expect(parseRange("3-", 5)).toEqual([3, 4, 5]);
    expect(parseRange("0", 5)).toBeNull();
    expect(parseRange("6", 5)).toBeNull();
    expect(parseRange("x", 5)).toBeNull();
  });

  it("writes them short", () => {
    expect(formatRange([1, 2, 3, 7])).toBe("1-3, 7");
    expect(formatRange([5, 4, 9])).toBe("4-5, 9");
    expect(formatRange([])).toBe("");
  });
});
