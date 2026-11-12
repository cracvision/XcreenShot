import { describe, expect, it } from "vitest";
import { taskDone, taskProgress, totalSize, useLibraryTasks } from "./tasks";

describe("library status bar (LIB-026)", () => {
  it("adds up the size of the selection", () => {
    const items = [
      { id: "a", size: 100 },
      { id: "b", size: 250 },
      { id: "c", size: 1 },
    ];
    expect(totalSize(items, ["a", "b"])).toBe(350);
    expect(totalSize(items, [])).toBe(0);
  });

  it("keeps one entry per task and says when it ends", () => {
    taskProgress({ id: "export", label: "x", done: 1, total: 4 });
    taskProgress({ id: "export", label: "x", done: 2, total: 4 });
    expect(useLibraryTasks.getState().tasks).toHaveLength(1);
    expect(useLibraryTasks.getState().tasks[0]?.done).toBe(2);
    taskDone("export", "Export finished.");
    expect(useLibraryTasks.getState().tasks).toHaveLength(0);
    expect(useLibraryTasks.getState().said).toBe("Export finished.");
  });
});
