import { describe, expect, it } from "vitest";
import { boxOf, clickSelection, columnsOf, moveFocus, overlaps, range } from "./selection";

const order = Array.from({ length: 12 }, (_, i) => `i${i + 1}`);
const none = { toggle: false, range: false };

describe("library selection (LIB-002)", () => {
  it("Shift+click from the 3rd to the 10th selects 8 items", () => {
    const first = clickSelection(order, [], null, "i3", none);
    const r = clickSelection(order, first.selection, first.anchor, "i10", {
      toggle: false,
      range: true,
    });
    expect(r.selection).toHaveLength(8);
    expect(r.selection[0]).toBe("i3");
    expect(r.selection[7]).toBe("i10");
  });

  it("a range works backwards too", () => {
    expect(range(order, "i5", "i2")).toEqual(["i2", "i3", "i4", "i5"]);
  });

  it("Ctrl+click adds and removes, keeping the grid order", () => {
    let s = clickSelection(order, ["i4"], "i4", "i2", { toggle: true, range: false });
    expect(s.selection).toEqual(["i2", "i4"]);
    s = clickSelection(order, s.selection, s.anchor, "i4", { toggle: true, range: false });
    expect(s.selection).toEqual(["i2"]);
  });

  it("Ctrl+Shift+click adds a range to the selection", () => {
    const s = clickSelection(order, ["i1"], "i5", "i7", { toggle: true, range: true });
    expect(s.selection).toEqual(["i1", "i5", "i6", "i7"]);
  });

  it("a plain click selects only that item", () => {
    expect(clickSelection(order, ["i1", "i2"], "i1", "i9", none).selection).toEqual(["i9"]);
  });
});

describe("keyboard movement (LIB-002, TRY-013)", () => {
  it("arrows move by one and by a row, and stop at the ends", () => {
    expect(moveFocus(order, "i5", "ArrowRight", 4, 8)).toBe("i6");
    expect(moveFocus(order, "i5", "ArrowDown", 4, 8)).toBe("i9");
    expect(moveFocus(order, "i2", "ArrowUp", 4, 8)).toBe("i1");
    expect(moveFocus(order, "i12", "ArrowRight", 4, 8)).toBe("i12");
    expect(moveFocus(order, "i1", "ArrowLeft", 4, 8)).toBe("i1");
  });

  it("Home, End, Ctrl+arrows and pages", () => {
    expect(moveFocus(order, "i6", "Home", 4, 8)).toBe("i1");
    expect(moveFocus(order, "i6", "End", 4, 8)).toBe("i12");
    expect(moveFocus(order, "i6", "ArrowDown", 4, 8, true)).toBe("i12");
    expect(moveFocus(order, "i6", "ArrowUp", 4, 8, true)).toBe("i1");
    expect(moveFocus(order, "i2", "PageDown", 4, 8)).toBe("i10");
  });

  it("starts on the first item and ignores other keys", () => {
    expect(moveFocus(order, null, "ArrowRight", 4, 8)).toBe("i1");
    expect(moveFocus(order, "i3", "KeyA", 4, 8)).toBeNull();
    expect(moveFocus([], null, "ArrowRight", 4, 8)).toBeNull();
  });

  it("counts the columns of a wrapping grid", () => {
    expect(columnsOf([10, 10, 10, 200, 200])).toBe(3);
    expect(columnsOf([10, 10])).toBe(2);
    expect(columnsOf([])).toBe(1);
  });
});

describe("rubber band", () => {
  it("selects what the box touches, drawn either way", () => {
    const band = boxOf(100, 100, 10, 10);
    expect(overlaps(band, { left: 50, top: 50, right: 150, bottom: 150 })).toBe(true);
    expect(overlaps(band, { left: 101, top: 0, right: 150, bottom: 50 })).toBe(false);
  });
});
