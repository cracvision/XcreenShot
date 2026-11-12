import { describe, expect, it } from "vitest";
import type { DocSummary } from "./state";
import { arrived, dropIndex, neighbour, trayOrder } from "./tray";

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

  it("follows the core's order with pinned items first (TRY-004, TRY-005)", () => {
    const tray = ["b", "c", "a"];
    expect(trayOrder(docs, tray).map((d) => d.documentId)).toEqual(["b", "c", "a"]);
    expect(trayOrder(docs, ["hidden", "a"]).map((d) => d.documentId)).toEqual(["a"]);
    expect(neighbour(docs, "b", 1, tray)).toBe("c");
    expect(neighbour(docs, "a", 1, tray)).toBeNull();
  });

  it("drops a dragged item among the others by their middles (TRY-005)", () => {
    expect(dropIndex([50, 150, 250], 10)).toBe(0);
    expect(dropIndex([50, 150, 250], 160)).toBe(2);
    expect(dropIndex([50, 150, 250], 400)).toBe(3);
  });

  it("knows when a capture arrived", () => {
    expect(arrived(docs, [...docs, doc("d")])).toBe(true);
    expect(arrived(docs, docs)).toBe(false);
    expect(arrived(docs, [...docs, doc("agent", false)])).toBe(false);
  });
});
