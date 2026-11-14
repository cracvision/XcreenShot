import { describe, expect, it } from "vitest";
import type { WordLine } from "./redactState";
import { between, covered, lineRect, spanRects, textUnder } from "./redactWords";

const lines: WordLine[] = [
  {
    words: [
      { text: "Cliente:", rect: [10, 10, 60, 16] },
      { text: "Marta", rect: [76, 10, 40, 16] },
      { text: "Ruiz", rect: [122, 10, 34, 16] },
    ],
  },
  {
    words: [
      { text: "Firma", rect: [10, 40, 40, 16] },
      { text: "de", rect: [56, 40, 16, 16] },
      { text: "Marta", rect: [78, 40, 40, 16] },
    ],
  },
];

describe("redacting words by hand (SMT-014)", () => {
  it("covers three words of one line with one rectangle", () => {
    expect(spanRects(lines, { line: 0, word: 0 }, { line: 0, word: 2 })).toEqual([
      [10, 10, 146, 16],
    ]);
  });

  it("covers a drag backwards over two lines with one rectangle per line", () => {
    expect(spanRects(lines, { line: 1, word: 1 }, { line: 0, word: 1 })).toEqual([
      [76, 10, 80, 16],
      [10, 40, 62, 16],
    ]);
  });

  it("knows the words a drag goes over", () => {
    const a = { line: 0, word: 2 };
    const b = { line: 1, word: 0 };
    expect(between({ line: 0, word: 2 }, b, a)).toBe(true);
    expect(between({ line: 1, word: 0 }, a, b)).toBe(true);
    expect(between({ line: 0, word: 1 }, a, b)).toBe(false);
    expect(between({ line: 1, word: 1 }, a, b)).toBe(false);
  });

  it("covers the whole line with a double click", () => {
    expect(lineRect(lines, 1)).toEqual([10, 40, 108, 16]);
    expect(lineRect(lines, 5)).toBeNull();
  });

  it("finds the text under a redaction with its margin", () => {
    expect(textUnder(lines, [74, 8, 84, 20])).toBe("Marta Ruiz");
    expect(covered([76, 10, 40, 16], [[74, 8, 84, 20]])).toBe(true);
    expect(covered([10, 40, 40, 16], [[74, 8, 84, 20]])).toBe(false);
  });
});
