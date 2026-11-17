import { describe, expect, it } from "vitest";
import { pickRange, pickToggle, pickedInOrder } from "./clipboardPick";

describe("XcreenShot's clipboard picking (TRY-018)", () => {
  it("toggles a capture", () => {
    const one = pickToggle(new Set(), "d1");
    expect([...one]).toEqual(["d1"]);
    expect([...pickToggle(one, "d1")]).toEqual([]);
  });

  it("adds a range in the list shown, either way", () => {
    const shown = ["d1", "d2", "d3", "d4"];
    expect([...pickRange(new Set(["d1"]), shown, "d1", "d3")].sort()).toEqual(["d1", "d2", "d3"]);
    expect([...pickRange(new Set(), shown, "d4", "d2")].sort()).toEqual(["d2", "d3", "d4"]);
  });

  it("pastes in the tray's order, not the order picked", () => {
    const picked = new Set(["d3", "d1"]);
    expect(pickedInOrder(picked, ["d1", "d2", "d3"])).toEqual(["d1", "d3"]);
  });
});
