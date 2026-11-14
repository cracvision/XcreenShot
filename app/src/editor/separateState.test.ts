import { beforeEach, describe, expect, it, vi } from "vitest";
import { run } from "../lib/commands";
import { answerConfirm, setDetail, startSeparate, useSeparate } from "./separateState";
import { useEditor, type DocDetails } from "./state";

vi.mock("../lib/commands", () => ({ run: vi.fn(), isCommandError: () => false }));

const doc = { documentId: "d1", objects: [] } as unknown as DocDetails;

beforeEach(() => {
  vi.mocked(run).mockReset();
  useSeparate.setState({ documentId: null, objectId: null, detail: 5, busy: false, confirm: null });
  useEditor.setState({ doc, tool: "move" });
});

describe("separate elements (SMT-018)", () => {
  it("starts on the image and asks before separating again over changes", async () => {
    vi.mocked(run).mockResolvedValueOnce({ pieces: 4 });
    await startSeparate();
    expect(vi.mocked(run)).toHaveBeenCalledWith("smart.separate", { documentId: "d1", detail: 5 });
    expect(useSeparate.getState().documentId).toBe("d1");

    vi.mocked(run).mockResolvedValueOnce({ needsConfirm: true, edited: 2 });
    await setDetail(9);
    expect(useSeparate.getState().confirm).toEqual({ detail: 9, edited: 2 });
    expect(useSeparate.getState().detail).toBe(5);

    vi.mocked(run).mockResolvedValueOnce({ pieces: 7 });
    await answerConfirm(true);
    expect(vi.mocked(run)).toHaveBeenLastCalledWith("smart.separate", {
      documentId: "d1",
      detail: 9,
      force: true,
    });
    expect(useSeparate.getState().detail).toBe(9);
  });

  it("another tool ends the mode as Done does", async () => {
    vi.mocked(run).mockResolvedValue({ pieces: 3 });
    await startSeparate("o7");
    expect(vi.mocked(run)).toHaveBeenCalledWith("smart.separate", {
      documentId: "d1",
      detail: 5,
      objectId: "o7",
    });
    useEditor.setState({ tool: "arrow" });
    await Promise.resolve();
    expect(vi.mocked(run)).toHaveBeenLastCalledWith("smart.separate_done", { documentId: "d1" });
    expect(useSeparate.getState().documentId).toBeNull();
  });
});
