import { describe, expect, it } from "vitest";
import type { HighlightObject } from "./state";
import { noTextIn, textBands } from "./textBands";

const highlight = (extra: Partial<HighlightObject> = {}): HighlightObject => ({
  id: "o1",
  type: "highlight",
  rect: { x: 100, y: 50, w: 200, h: 60 },
  rotation: 0,
  style: { color: "#FFE14D", opacity: 50, shape: "rectangle", textOnly: true },
  ...extra,
});

describe("only the text (SMT-021)", () => {
  it("draws the bands where the object is, following a drag", () => {
    const o = highlight({ bands: [[2, 3, 80, 14]], bandsFor: [100, 50, 200, 60] });
    expect(textBands(o)).toEqual([[102, 53, 80, 14]]);
    expect(textBands({ ...o, rect: { ...o.rect, x: 150 } })).toEqual([[152, 53, 80, 14]]);
  });

  it("covers the whole area without bands or when not asked", () => {
    expect(textBands(highlight())).toBeNull();
    expect(textBands(highlight({ bands: [] }))).toBeNull();
    const off = highlight({ bands: [[2, 3, 80, 14]] });
    expect(textBands({ ...off, style: { ...off.style, textOnly: false } })).toBeNull();
  });

  it("says when there was no text in the area", () => {
    expect(noTextIn([highlight({ bands: [], bandsFor: [100, 50, 200, 60] })])).toBe(true);
    expect(noTextIn([highlight()])).toBe(false);
    expect(noTextIn([highlight({ bands: [[0, 0, 5, 5]], bandsFor: [100, 50, 200, 60] })])).toBe(
      false,
    );
  });
});
