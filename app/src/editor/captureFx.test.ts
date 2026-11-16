import { describe, expect, it } from "vitest";
import { captureTypes, withoutTypes } from "./captureFx";

describe("effects on capture from the editor (CAP-051)", () => {
  const panel = [
    { type: "border", width: 4 },
    { type: "perspective", horizontal: 15 },
    { type: "shadow" },
  ];

  it("knows which types every capture gets", () => {
    const types = captureTypes(panel);
    expect(types.has("perspective")).toBe(true);
    expect(types.has("scale")).toBe(false);
  });

  it("removes only the types turned off, keeping the order and options", () => {
    expect(withoutTypes(panel, ["perspective"])).toEqual([
      { type: "border", width: 4 },
      { type: "shadow" },
    ]);
    expect(withoutTypes(panel, [])).toEqual(panel);
  });
});
