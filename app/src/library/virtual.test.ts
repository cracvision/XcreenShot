import { describe, expect, it } from "vitest";
import { columnsFor, scrollToRow, visibleSpan } from "./virtual";

describe("drawing only what is in view (LIB-025)", () => {
  it("draws the rows in view and a margin, of 50,000 items", () => {
    const s = visibleSpan(50_000, 6, 200, 100_000, 800, 2);
    expect(s.rows).toBe(8334);
    expect(s.firstRow).toBe(498);
    expect(s.lastRow).toBe(506);
    expect(s.end - s.start).toBe(54);
  });

  it("stops at the ends", () => {
    expect(visibleSpan(10, 4, 100, 0, 1000)).toMatchObject({
      firstRow: 0,
      lastRow: 2,
      start: 0,
      end: 10,
    });
    expect(visibleSpan(0, 4, 100, 0, 1000).end).toBe(0);
  });

  it("fits columns and scrolls a row into view", () => {
    expect(columnsFor(1000, 176, 16)).toBe(5);
    expect(columnsFor(100, 176, 16)).toBe(1);
    expect(scrollToRow(10, 100, 0, 500)).toBe(600);
    expect(scrollToRow(2, 100, 500, 500)).toBe(200);
    expect(scrollToRow(6, 100, 500, 500)).toBeNull();
  });
});
