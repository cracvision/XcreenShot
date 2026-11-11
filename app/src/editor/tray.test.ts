import { describe, expect, it } from "vitest";
import type { DocSummary } from "./state";
import { arrived, neighbour, trayOrder } from "./tray";

function doc(id: string, shown = true): DocSummary {
  return {
    documentId: id,
    name: id,
    path: `C:/lib/${id}.xshot`,
    kind: "library",
    format: "xshot",
    width: 10,
    height: 10,
    objects: 0,
    dirty: false,
    shown,
    version: 1,
    missing: false,
  };
}

describe("recent tray (TRY-002, TRY-013)", () => {
  const docs = [doc("a"), doc("b"), doc("hidden", false), doc("c")];

  it("shows the newest first and leaves out documents never shown", () => {
    expect(trayOrder(docs).map((d) => d.documentId)).toEqual(["c", "b", "a"]);
  });

  it("next and previous follow the tray and stop at the ends", () => {
    expect(neighbour(docs, "c", 1)).toBe("b");
    expect(neighbour(docs, "b", -1)).toBe("c");
    expect(neighbour(docs, "a", 1)).toBeNull();
    expect(neighbour(docs, "c", -1)).toBeNull();
  });

  it("knows when a capture arrived", () => {
    expect(arrived(docs, [...docs, doc("d")])).toBe(true);
    expect(arrived(docs, docs)).toBe(false);
    expect(arrived(docs, [...docs, doc("agent", false)])).toBe(false);
  });
});
