import { describe, expect, it } from "vitest";
import { idsOfKind } from "./selectKind";
import type { DocObject } from "./state";

const o = (id: string, type: string, extra: Record<string, unknown> = {}) =>
  ({ id, type, ...extra }) as unknown as DocObject;

describe("select all of a kind (EDT-023)", () => {
  it("takes every object of the kind, their groups, and no locked one", () => {
    const objects = [
      o("a1", "arrow"),
      o("a2", "arrow"),
      o("t1", "text"),
      o("a3", "arrow", { locked: true }),
      o("a4", "arrow", { groups: ["g1"] }),
      o("t2", "text", { groups: ["g1"] }),
      o("t3", "text", { groups: ["g2"] }),
    ];
    expect(idsOfKind(objects, "arrow")).toEqual(["a1", "a2", "a4", "t2"]);
    expect(idsOfKind(objects, "step")).toEqual([]);
  });
});
