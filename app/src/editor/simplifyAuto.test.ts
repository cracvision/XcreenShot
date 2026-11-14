import { beforeEach, describe, expect, it, vi } from "vitest";
import { run } from "../lib/commands";
import {
  autoSimplify,
  autoState,
  cancelRegenerate,
  confirmRegenerate,
  useAutoSimplify,
} from "./simplifyAuto";
import type { DocObject } from "./state";

vi.mock("../lib/commands", () => ({ run: vi.fn(), isCommandError: () => false }));

const simple = (auto?: { detail?: number }): DocObject =>
  ({
    id: "o1",
    type: "simplify",
    rect: { x: 0, y: 0, w: 10, h: 10 },
    rotation: 0,
    style: { kind: "block" },
    ...(auto ? { autoSimplify: auto } : {}),
  }) as unknown as DocObject;

beforeEach(() => {
  vi.mocked(run).mockReset();
  useAutoSimplify.setState({ confirm: null, busy: false, kept: 0 });
});

describe("simplify automatically (SMT-019)", () => {
  it("is on while generated objects exist, with their detail", () => {
    expect(autoState([])).toEqual({ on: false, detail: 5 });
    expect(autoState([simple()])).toEqual({ on: false, detail: 5 });
    expect(autoState([simple(), simple({ detail: 8 })])).toEqual({ on: true, detail: 8 });
  });

  it("asks before losing changes; Cancel leaves everything, Generate again forces", async () => {
    vi.mocked(run).mockResolvedValueOnce({ needsConfirm: true, edited: 2 });
    await autoSimplify("d1", 3);
    expect(useAutoSimplify.getState().confirm).toEqual({ documentId: "d1", detail: 3, edited: 2 });
    cancelRegenerate();
    expect(useAutoSimplify.getState().confirm).toBeNull();
    expect(vi.mocked(run)).toHaveBeenCalledTimes(1);

    vi.mocked(run).mockResolvedValueOnce({ needsConfirm: true, edited: 2 });
    await autoSimplify("d1", 3);
    vi.mocked(run).mockResolvedValueOnce({ added: 4 });
    await confirmRegenerate();
    expect(vi.mocked(run)).toHaveBeenLastCalledWith("smart.simplify_auto", {
      documentId: "d1",
      detail: 3,
      force: true,
    });
  });
});
