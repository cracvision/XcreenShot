import { describe, expect, it } from "vitest";
import { insideSelection, modeOf } from "./pixelSelection";
import type { PixelSelectionInfo } from "./state";

const sel = (outline: [number, number][][]): PixelSelectionInfo => ({
  target: null,
  bounds: { x: 0, y: 0, w: 0, h: 0 },
  outline,
  shape: null,
});

describe("pixel selection", () => {
  it("knows what is inside, holes excluded", () => {
    const ring = sel([
      [
        [0, 0],
        [100, 0],
        [100, 100],
        [0, 100],
      ],
      [
        [40, 40],
        [40, 60],
        [60, 60],
        [60, 40],
      ],
    ]);
    expect(insideSelection(ring, [10, 10])).toBe(true);
    expect(insideSelection(ring, [50, 50])).toBe(false);
    expect(insideSelection(ring, [150, 50])).toBe(false);
  });

  it("Shift adds and Alt subtracts only with a selection", () => {
    expect(modeOf(false, { shiftKey: true, altKey: false })).toBe("replace");
    expect(modeOf(true, { shiftKey: true, altKey: false })).toBe("add");
    expect(modeOf(true, { shiftKey: false, altKey: true })).toBe("subtract");
  });
});
