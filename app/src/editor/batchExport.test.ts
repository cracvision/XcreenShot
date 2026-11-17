import { describe, expect, it } from "vitest";
import { batchName, DEFAULT_BATCH, stemOf } from "./batchExport";

describe("batch export names (EDT-113)", () => {
  it("numbers a base name like the core", () => {
    const naming = { ...DEFAULT_BATCH.naming, mode: "base" as const, base: "guia" };
    expect(batchName(naming, "x", 0)).toBe("guia_01");
    expect(batchName(naming, "x", 9)).toBe("guia_10");
    expect(batchName({ ...naming, start: 7, digits: 3, prefix: "a-", suffix: "-b" }, "x", 0)).toBe(
      "a-guia_007-b",
    );
  });

  it("keeps the original name", () => {
    expect(batchName(DEFAULT_BATCH.naming, "Chrome 2026", 3)).toBe("Chrome 2026");
    expect(stemOf("C:\\caps\\Chrome 2026.xshot")).toBe("Chrome 2026");
  });
});
