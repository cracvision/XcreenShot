// The same cases as the tests of crates/xs-editor/src/transform.rs (EDT-080).
import { describe, expect, it } from "vitest";
import { boundsOf, normalized, transformed } from "./transform";
import type { DocObject } from "./state";

const arrow: DocObject = {
  id: "a",
  type: "arrow",
  from: [0, 0],
  to: [100, 0],
  style: {
    color: "#000000",
    width: 4,
    startCap: "none",
    endCap: "filled",
    lineStyle: "solid",
    opacity: 100,
  },
};
const image: DocObject = {
  id: "i",
  type: "image",
  image: "x.png",
  rect: { x: 100, y: 50, w: 100, h: 50 },
  rotation: 0,
  style: { opacity: 100 },
};
const shape = (id: string, x: number): DocObject => ({
  id,
  type: "shape",
  rect: { x, y: 0, w: 20, h: 10 },
  rotation: 0,
  style: {
    shapeType: "rectangle",
    fill: null,
    outline: "#000000",
    outlineWidth: 2,
    lineStyle: "solid",
    opacity: 100,
    cornerRadius: 0,
  },
});

describe("several objects as one (EDT-080)", () => {
  it("growing the box half again grows each object and keeps their places", () => {
    const from = boundsOf([arrow, image]);
    expect(from).toEqual({ x: 0, y: 0, w: 200, h: 100 });
    if (!from) return;
    const to = { x: 0, y: 0, w: 300, h: 150 };
    const a = transformed(arrow, { kind: "scale", from, to, strokes: true });
    expect(a.type === "arrow" && a.to).toEqual([150, 0]);
    expect(a.type === "arrow" && a.style.width).toBe(6);
    const i = transformed(image, { kind: "scale", from, to, strokes: true });
    expect(i.type === "image" && i.rect).toEqual({ x: 150, y: 75, w: 150, h: 75 });
    // Without "Scale line widths and text" the arrow keeps its width.
    const kept = transformed(arrow, { kind: "scale", from, to, strokes: false });
    expect(kept.type === "arrow" && kept.style.width).toBe(4);
  });

  it("a quarter turn goes around the common center", () => {
    const shapes = [shape("s1", 0), shape("s2", 100), shape("s3", 200)];
    const all = boundsOf(shapes);
    expect(all).toEqual({ x: 0, y: 0, w: 220, h: 10 });
    const first = transformed(shapes[0] as DocObject, {
      kind: "rotate",
      center: [110, 5],
      degrees: 90,
    });
    if (first.type !== "shape") throw new Error("shape");
    expect(first.rect.x + first.rect.w / 2).toBeCloseTo(110);
    expect(first.rect.y + first.rect.h / 2).toBeCloseTo(-95);
    expect(first.rotation).toBe(90);
  });

  it("angles stay within a half turn", () => {
    expect(normalized(270)).toBe(-90);
    expect(normalized(-190)).toBe(170);
    expect(normalized(180)).toBe(180);
  });
});
