import { describe, expect, it } from "vitest";
import {
  NO_FILTERS,
  chips,
  dateLabel,
  defaultOrder,
  hasFilters,
  isOn,
  toggle,
  visibleValues,
} from "./query";

describe("library filters (LIB-004, LIB-005)", () => {
  it("starts newest and largest first, names A to Z", () => {
    expect(defaultOrder("modified")).toBe("desc");
    expect(defaultOrder("size")).toBe("desc");
    expect(defaultOrder("name")).toBe("asc");
  });

  it("adds values of a section and takes them away, without case", () => {
    let f = toggle(NO_FILTERS, "types", "image");
    f = toggle(f, "types", "gif");
    expect(f.types).toEqual(["image", "gif"]);
    f = toggle(f, "tags", "Cliente A");
    expect(isOn(f, "tags", "cliente a")).toBe(true);
    f = toggle(f, "tags", "CLIENTE A");
    expect(f.tags).toEqual([]);
    expect(hasFilters(f)).toBe(true);
    expect(hasFilters(NO_FILTERS)).toBe(false);
  });

  it("lists a chip per active value", () => {
    const f = { ...NO_FILTERS, dates: ["2025"], favorites: true };
    expect(chips(f)).toEqual([
      { section: "dates", value: "2025" },
      { section: "favorites", value: "" },
    ]);
  });

  it("hides values without items unless chosen", () => {
    const counts = [
      { value: "Chrome", count: 3 },
      { value: "Code", count: 0 },
    ];
    expect(visibleValues(counts, [])).toEqual([{ value: "Chrome", count: 3 }]);
    expect(visibleValues(counts, ["code", "Slack"]).map((c) => c.value)).toEqual([
      "Chrome",
      "code",
      "Slack",
    ]);
  });

  it("names years, months and days", () => {
    expect(dateLabel("2026", "en")).toBe("2026");
    expect(dateLabel("2026-11", "en")).toBe("November 2026");
    expect(dateLabel("2026-11-12", "en")).toBe("Nov 12, 2026");
  });
});
