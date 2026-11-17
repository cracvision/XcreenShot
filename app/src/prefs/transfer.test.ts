import { describe, expect, it } from "vitest";
import { changesOf } from "./transfer";

describe("settings transfer (APP-011)", () => {
  it("counts what each group of an import changes", () => {
    expect(changesOf("preferences", { changes: ["a", "b"], notApplied: ["c"] })).toBe(2);
    expect(changesOf("shortcuts", { changes: [], kept: true })).toBe(0);
    expect(
      changesOf("themes", {
        themes: { added: 1, replaced: 1, skipped: 3 },
        effects: { added: 2 },
        favorites: 1,
      }),
    ).toBe(5);
    expect(changesOf("toolbar", { changes: true })).toBe(1);
    expect(changesOf("toolbar", undefined)).toBe(0);
  });
});
