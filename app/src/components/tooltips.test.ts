import { describe, expect, it } from "vitest";
import { hint } from "../editor/tools";

describe("tooltips (APP-014)", () => {
  it("brief is the name and the shortcut; detailed adds what it does", () => {
    expect(hint("Arrow", "A")).toBe("Arrow (A)");
    expect(hint("Arrow", undefined)).toBe("Arrow");
    expect(hint("Arrow", "A", "Drag to point.")).toBe("Arrow (A)\nDrag to point.");
  });
});
