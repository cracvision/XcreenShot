import { describe, expect, it } from "vitest";
import { hitMark, inkOn, markFromDrag, moveMark, toObjects, type Mark } from "./quickMarks";

const sel = { x: 100, y: 200, width: 400, height: 300 };

describe("quick annotation marks (CAP-057)", () => {
  it("a drag makes the tool's mark, and a click makes none", () => {
    const a = { x: 110, y: 210 };
    const b = { x: 210, y: 260 };
    expect(markFromDrag("arrow", a, b, "#F2545B", 4, false)).toMatchObject({ kind: "arrow" });
    expect(markFromDrag("shape", a, b, "#F2545B", 4, true)).toMatchObject({
      kind: "shape",
      ellipse: true,
      rect: { x: 110, y: 210, width: 100, height: 50 },
    });
    expect(markFromDrag("blur", b, a, "#F2545B", 4, false)).toMatchObject({
      kind: "blur",
      rect: { x: 110, y: 210, width: 100, height: 50 },
    });
    expect(markFromDrag("arrow", a, { x: 111, y: 211 }, "#F2545B", 4, false)).toBeNull();
  });

  it("marks become the editor's objects, in the capture's pixels", () => {
    const marks: Mark[] = [
      {
        kind: "arrow",
        from: { x: 110, y: 210 },
        to: { x: 300, y: 400 },
        color: "#F2545B",
        width: 6,
      },
      {
        kind: "shape",
        rect: { x: 120, y: 220, width: 50, height: 40 },
        ellipse: false,
        color: "#2563EB",
        width: 4,
      },
      { kind: "step", at: { x: 150, y: 250 }, value: 1, color: "#FFB000", size: 48 },
      { kind: "text", at: { x: 130, y: 300 }, text: "Aquí", color: "#111318", size: 33 },
    ];
    const objects = toObjects(marks, sel, 1.5);
    expect(objects[0]).toEqual({
      type: "arrow",
      from: [10, 10],
      to: [200, 200],
      style: { color: "#F2545B", width: 6 },
    });
    expect(objects[1]).toMatchObject({
      type: "shape",
      rect: { x: 20, y: 20, w: 50, h: 40 },
      style: { shapeType: "rectangle", fill: null, outline: "#2563EB", outlineWidth: 4 },
    });
    // A step by its box, with a dark number on a light color.
    expect(objects[2]).toMatchObject({
      type: "step",
      rect: { x: 26, y: 26, w: 48, h: 48 },
      value: "1",
      style: { fill: "#FFB000", textColor: "#111318", size: 48 },
    });
    // Text sizes are points: 33 px at 150 % are 16.5 pt at 144 dpi.
    expect(objects[3]).toMatchObject({
      type: "text",
      at: [30, 100],
      text: "Aquí",
      style: { size: 17 },
    });
  });

  it("the number of a step is light on dark colors", () => {
    expect(inkOn("#2563EB")).toBe("#FFFFFF");
    expect(inkOn("#FFFFFF")).toBe("#111318");
  });

  it("a mark is found under the pointer and moves", () => {
    const marks: Mark[] = [
      {
        kind: "shape",
        rect: { x: 100, y: 100, width: 200, height: 100 },
        ellipse: false,
        color: "#000",
        width: 4,
      },
      { kind: "arrow", from: { x: 0, y: 0 }, to: { x: 100, y: 100 }, color: "#000", width: 4 },
    ];
    expect(hitMark(marks, { x: 50, y: 52 }, 8)).toBe(1);
    expect(hitMark(marks, { x: 200, y: 150 }, 8)).toBe(0);
    expect(hitMark(marks, { x: 500, y: 500 }, 8)).toBe(-1);
    const moved = moveMark(marks[0] as Mark, 10, -5);
    expect(moved).toMatchObject({ rect: { x: 110, y: 95, width: 200, height: 100 } });
  });
});
