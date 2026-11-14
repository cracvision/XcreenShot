import { describe, expect, it } from "vitest";
import { alignMoves, distributeMoves, frameOf, type Extent } from "./align";

const e = (id: string, x0: number, y0: number, w: number, h: number): Extent => ({
  id,
  x0,
  y0,
  x1: x0 + w,
  y1: y0 + h,
});

describe("align and distribute (EDT-083)", () => {
  it("aligning to the top leaves them with the top edge of the highest", () => {
    const items = [e("a", 0, 30, 10, 10), e("b", 20, 10, 10, 10), e("c", 40, 50, 10, 10)];
    expect(alignMoves(items, "top", frameOf(items))).toEqual({ a: [0, -20], c: [0, -40] });
  });

  it("one object centers on the canvas", () => {
    const canvas = e("", 0, 0, 200, 100);
    expect(alignMoves([e("a", 0, 0, 50, 20)], "hcenter", canvas)).toEqual({ a: [75, 0] });
  });

  it("distributing four leaves the same space and keeps the first and the last", () => {
    const items = [
      e("a", 0, 0, 10, 10),
      e("b", 15, 0, 20, 10),
      e("c", 40, 0, 10, 10),
      e("d", 90, 0, 10, 10),
    ];
    const moves = distributeMoves(items, "h");
    expect(moves.a).toBeUndefined();
    expect(moves.d).toBeUndefined();
    // Span 100, sizes 50: three gaps of 50/3.
    const gap = 50 / 3;
    expect(moves.b?.[0]).toBeCloseTo(10 + gap - 15);
    expect(moves.c?.[0]).toBeCloseTo(10 + gap + 20 + gap - 40);
  });
});
