import { describe, expect, it } from "vitest";
import { expand, unitOf } from "./groups";
import type { DocObject } from "./state";

const o = (id: string, groups?: string[]) =>
  ({ id, type: "other", groups }) as unknown as DocObject;

describe("groups (EDT-082)", () => {
  const objects = [o("a", ["g2", "g1"]), o("b", ["g2", "g1"]), o("c", ["g2"]), o("d")];

  it("a click takes the whole outermost group", () => {
    expect(unitOf(objects[0] as DocObject, [])).toBe("g2");
    expect(expand(objects, ["a"], [])).toEqual(["a", "b", "c"]);
    expect(expand(objects, ["d"], [])).toEqual(["d"]);
  });

  it("inside a group, the next group or the object itself", () => {
    expect(expand(objects, ["a"], ["g2"])).toEqual(["a", "b"]);
    expect(expand(objects, ["c"], ["g2"])).toEqual(["c"]);
    expect(unitOf(objects[3] as DocObject, ["g2"])).toBeUndefined();
  });
});
