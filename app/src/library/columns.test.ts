import { describe, expect, it } from "vitest";
import { shownColumns, template } from "./columns";

describe("library list columns (LIB-004)", () => {
  it("hides columns but never the name, and keeps the widths set", () => {
    const cols = shownColumns(["site", "tags", "name"]);
    expect(cols.map((c) => c.key)).toEqual([
      "name",
      "captured",
      "modified",
      "size",
      "type",
      "dimensions",
      "app",
    ]);
    const t = template(cols.slice(0, 4), { captured: 200, size: 10 });
    expect(t).toBe("minmax(180px, 2.2fr) 200px minmax(130px, 1.1fr) 56px");
  });
});
