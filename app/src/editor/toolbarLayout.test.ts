import { describe, expect, it } from "vitest";
import { barTools, insertItem, moveItem, removeAt } from "./toolbarLayout";

describe("toolbar layout (EDT-004)", () => {
  const bar = ["tool:move", "tool:arrow", "tool:text", "tool:crop"];

  it("moves an item: Eraser to the second place", () => {
    const withEraser = [...bar, "tool:eraser"];
    expect(moveItem(withEraser, 4, 1)).toEqual([
      "tool:move",
      "tool:eraser",
      "tool:arrow",
      "tool:text",
      "tool:crop",
    ]);
  });

  it("inserts new items and moves ones already there", () => {
    expect(insertItem(bar, "action:print", 2)).toEqual([
      "tool:move",
      "tool:arrow",
      "action:print",
      "tool:text",
      "tool:crop",
    ]);
    expect(insertItem(bar, "tool:crop", 0)).toEqual([
      "tool:crop",
      "tool:move",
      "tool:arrow",
      "tool:text",
    ]);
    expect(insertItem(bar, "sep", 1)).toHaveLength(5);
  });

  it("removes and lists the tools in order", () => {
    expect(removeAt(bar, 1)).toEqual(["tool:move", "tool:text", "tool:crop"]);
    expect(barTools(["tool:move", "sep", "action:save", "tool:crop"])).toEqual(["move", "crop"]);
  });
});
