import { beforeEach, describe, expect, it, vi } from "vitest";
import { run } from "../lib/commands";
import { answerCut, editText, useEditText } from "./editText";
import { useEditor, type DocDetails } from "./state";

vi.mock("../lib/commands", () => ({ run: vi.fn(), isCommandError: () => false }));

beforeEach(() => {
  vi.mocked(run).mockReset();
  useEditText.setState({ ask: null, busy: false });
  useEditor.setState({ doc: { documentId: "d1", objects: [] } as unknown as DocDetails });
});

describe("edit text (SMT-009)", () => {
  it("asks when the selection cuts the text, and fits it when told", async () => {
    vi.mocked(run).mockResolvedValueOnce({ cut: true, fitRect: [0, 0, 10, 10] });
    await editText();
    expect(useEditText.getState().ask).toEqual({ documentId: "d1" });
    vi.mocked(run).mockResolvedValueOnce({ ids: [] });
    await answerCut("fit");
    expect(vi.mocked(run)).toHaveBeenLastCalledWith("smart.edit_text", {
      documentId: "d1",
      fit: true,
    });
    expect(useEditText.getState().ask).toBeNull();
  });

  it("from auto redact's menu: the redaction's place, fitted to the lines", async () => {
    vi.mocked(run).mockResolvedValueOnce({ ids: [] });
    await editText([5, 6, 70, 20], true);
    expect(vi.mocked(run)).toHaveBeenCalledWith("smart.edit_text", {
      documentId: "d1",
      rect: [5, 6, 70, 20],
      fit: true,
    });
  });

  it("Cancel leaves everything", async () => {
    useEditText.setState({ ask: { documentId: "d1" } });
    await answerCut(null);
    expect(vi.mocked(run)).not.toHaveBeenCalled();
    expect(useEditText.getState().ask).toBeNull();
  });
});
