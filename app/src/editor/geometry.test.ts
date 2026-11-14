import { describe, expect, it } from "vitest";
import fixture from "../../../tests/fixtures/arrow-geometry.json";
import {
  arrowGeometry,
  constrain45,
  distanceToSegment,
  type ArrowStyle,
  type Geometry,
  type Point,
} from "./geometry";

interface Case {
  input: { from: Point; to: Point; style: ArrowStyle };
  geometry: Geometry;
}

function close(a: unknown, b: unknown): boolean {
  if (typeof a === "number" && typeof b === "number") return Math.abs(a - b) < 1e-9;
  if (Array.isArray(a) && Array.isArray(b))
    return a.length === b.length && a.every((x, i) => close(x, b[i]));
  if (a && b && typeof a === "object" && typeof b === "object") {
    const ka = Object.keys(a);
    return (
      ka.length === Object.keys(b).length &&
      ka.every((k) => close((a as Record<string, unknown>)[k], (b as Record<string, unknown>)[k]))
    );
  }
  return a === b;
}

describe("arrow geometry (ADR 0004)", () => {
  it("matches the renderer of exported images", () => {
    const cases = fixture as unknown as Case[];
    expect(cases.length).toBeGreaterThan(10);
    for (const c of cases) {
      const g = arrowGeometry(c.input.from, c.input.to, c.input.style);
      expect(close(g, c.geometry), JSON.stringify(c.input)).toBe(true);
    }
  });

  it("constrains to 0°, 45° and 90° with Shift", () => {
    const [x, y] = constrain45([0, 0], [100, 8]);
    expect(Math.round(x)).toBe(100);
    expect(Math.round(y)).toBe(0);
    const [x2, y2] = constrain45([0, 0], [50, 46]);
    expect(Math.abs(x2 - y2)).toBeLessThan(1e-9);
  });

  it("measures distance to a segment", () => {
    expect(distanceToSegment([5, 3], [0, 0], [10, 0])).toBe(3);
    expect(distanceToSegment([-4, 3], [0, 0], [10, 0])).toBe(5);
  });
});
