import { describe, expect, it } from "vitest";
import { boxOf, fingerprint, hits, moved, outside, reboxed } from "./objects";
import type { DocObject } from "./state";

const shape = (fill: string | null): DocObject => ({
  id: "s",
  type: "shape",
  rect: { x: 100, y: 100, w: 200, h: 100 },
  rotation: 0,
  style: {
    shapeType: "rectangle",
    fill,
    outline: "#F2545B",
    outlineWidth: 4,
    lineStyle: "solid",
    opacity: 100,
  },
});

const text: DocObject = {
  id: "t",
  type: "text",
  at: [10, 20],
  width: null,
  text: "Hola",
  rotation: 0,
  style: {
    font: "Segoe UI",
    size: 14,
    bold: false,
    italic: false,
    color: "#000000",
    align: "left",
    opacity: 100,
  },
  layout: { width: 50, height: 22 },
};

describe("objects on the canvas (EDT-022, EDT-042)", () => {
  it("picks transparent shapes by the outline and filled ones anywhere", () => {
    expect(hits(shape(null), [200, 150], 3)).toBe(false);
    expect(hits(shape(null), [101, 150], 3)).toBe(true);
    expect(hits(shape("#FFFFFF"), [200, 150], 3)).toBe(true);
    expect(hits(shape("#FFFFFF"), [400, 150], 3)).toBe(false);
  });

  it("uses the laid-out size of text", () => {
    expect(boxOf(text)).toEqual({ x: 10, y: 20, w: 50, h: 22, rotation: 0 });
    expect(hits(text, [30, 30], 0)).toBe(true);
  });

  it("moves and reshapes objects into update sets", () => {
    const [m, set] = moved(shape(null), 5, -5);
    expect(set).toEqual({ rect: { x: 105, y: 95, w: 200, h: 100 } });
    expect(m.type === "shape" && m.rect.x).toBe(105);
    const [, tset] = reboxed(text, { x: 0, y: 20, w: 120, h: 22, rotation: 15 });
    expect(tset).toEqual({ at: [0, 20], width: 120, rotation: 15 });
  });

  it("finds objects outside the canvas, but not blurs", () => {
    const blur = {
      id: "b",
      type: "blur",
      rect: { x: 200, y: 0, w: 100, h: 50 },
      rotation: 0,
      style: { blurType: "blur", color: "#000000", intensity: 60, shapeType: "rectangle" },
    } as DocObject;
    expect(outside([shape(null), text, blur], 250, 300)).toEqual(["s"]);
    // A canvas grown to fit an arrow leaves its tip a fraction of a pixel out.
    const tip = { id: "a", type: "arrow", from: [100, 50], to: [-0.3, 50] } as DocObject;
    expect(outside([tip], 250, 300)).toEqual([]);
  });

  it("fingerprints change with the object", () => {
    expect(fingerprint(text)).not.toBe(fingerprint({ ...text, text: "Adiós" } as DocObject));
  });
});
