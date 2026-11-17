import { beforeEach, describe, expect, it, vi } from "vitest";
import { run } from "../lib/commands";
import { useFiles } from "../library/actions";
import { removeBackground } from "./removeBg";
import { useEditor, type DocDetails } from "./state";

vi.mock("../lib/commands", () => ({ run: vi.fn(), isCommandError: () => false }));

const doc = { documentId: "d1", objects: [] } as unknown as DocDetails;

beforeEach(() => {
  vi.mocked(run).mockReset();
  useFiles.setState({ notice: null });
  useEditor.setState({ doc });
});

describe("remove background (SMT-020)", () => {
  it("removes it, then offers Invert and As a floating image", async () => {
    vi.mocked(run).mockResolvedValue({});
    expect(await removeBackground()).toBe(true);
    expect(vi.mocked(run)).toHaveBeenCalledWith("smart.remove_background", { documentId: "d1" });
    const notice = useFiles.getState().notice;
    expect(notice?.action).toBeDefined();
    expect(notice?.more).toHaveLength(1);
    // Invert: the first undone, then again inverted; no more choices after it.
    await notice?.action?.run();
    expect(vi.mocked(run)).toHaveBeenNthCalledWith(2, "document.undo", { documentId: "d1" });
    expect(vi.mocked(run)).toHaveBeenNthCalledWith(3, "smart.remove_background", {
      documentId: "d1",
      invert: true,
    });
    expect(useFiles.getState().notice?.action).toBeUndefined();
  });

  it("works on a floating image, as a floating image", async () => {
    vi.mocked(run).mockResolvedValue({});
    await removeBackground("o3");
    await useFiles.getState().notice?.more?.[0]?.run();
    expect(vi.mocked(run)).toHaveBeenLastCalledWith("smart.remove_background", {
      documentId: "d1",
      objectId: "o3",
      floating: true,
    });
  });

  it("says why when there is no clear subject, and offers nothing", async () => {
    const showError = vi.fn();
    useEditor.setState({ showError });
    vi.mocked(run).mockRejectedValueOnce(new Error("no subject"));
    expect(await removeBackground()).toBe(false);
    expect(showError).toHaveBeenCalled();
    expect(useFiles.getState().notice).toBeNull();
  });

  it("does nothing on a video", async () => {
    useEditor.setState({ doc: { ...doc, video: {} } as unknown as DocDetails });
    expect(await removeBackground()).toBe(false);
    expect(vi.mocked(run)).not.toHaveBeenCalled();
  });
});
