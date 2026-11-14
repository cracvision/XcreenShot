import { beforeEach, describe, expect, it, vi } from "vitest";
import { run } from "../lib/commands";
import {
  acceptItems,
  closeBatch,
  countsOf,
  pending,
  startBatch,
  useRedactBatch,
  type BatchItem,
  type BatchStatus,
} from "./redactBatchState";

vi.mock("../lib/commands", () => ({ run: vi.fn(), isCommandError: () => false }));

const item = (index: number, state: BatchItem["state"], counts = {}): BatchItem => ({
  index,
  name: `Captura ${index}`,
  path: `C:/lib/c${index}.xshot`,
  documentId: state === "ready" ? `d${index + 10}` : null,
  state,
  redactions: Object.values(counts as Record<string, number>).reduce((a, b) => a + b, 0),
  counts,
});

const status = (items: BatchItem[], running = false): BatchStatus => ({
  jobId: 7,
  running,
  done: items.length,
  total: items.length,
  texts: 1,
  items,
});

beforeEach(async () => {
  vi.mocked(run).mockReset();
  vi.mocked(run).mockResolvedValue({});
  await closeBatch();
});

describe("auto redact over several captures (SMT-016)", () => {
  it("lists a capture's counts largest first", () => {
    expect(countsOf(item(0, "ready", { manual: 1, email: 3, phone: 0 }))).toEqual([
      ["email", 3],
      ["manual", 1],
    ]);
  });

  it("only the captures ready wait for a decision", () => {
    const s = status([item(0, "ready", { email: 1 }), item(1, "accepted"), item(2, "nothing")]);
    expect(pending(s).map((i) => i.index)).toEqual([0]);
    expect(pending(null)).toEqual([]);
  });

  it("starts with the captures chosen and keeps each picture once analyzed", async () => {
    const first = status([item(0, "ready", { email: 1 }), item(1, "waiting")], true);
    vi.mocked(run).mockResolvedValueOnce(first);
    await startBatch({ paths: ["C:/lib/c0.xshot", "C:/lib/c1.xshot"], documentId: "d1" });
    expect(vi.mocked(run)).toHaveBeenCalledWith("smart.redact_batch", {
      paths: ["C:/lib/c0.xshot", "C:/lib/c1.xshot"],
      documentId: "d1",
    });
    const thumbs = useRedactBatch.getState().thumbs;
    expect(Object.keys(thumbs)).toEqual(["0"]);
    expect(thumbs[0]).toContain("d10/thumb");

    // Accepted, the document closes: its picture stays.
    vi.mocked(run).mockResolvedValueOnce(status([{ ...item(0, "accepted") }, item(1, "nothing")]));
    await acceptItems();
    expect(vi.mocked(run)).toHaveBeenCalledWith("smart.redact_batch_decide", {
      jobId: 7,
      decision: "accept",
    });
    expect(useRedactBatch.getState().thumbs[0]).toBe(thumbs[0]);
  });

  it("closing ends the job in the core", async () => {
    vi.mocked(run).mockResolvedValueOnce(status([item(0, "ready", { email: 1 })]));
    await startBatch({ documentIds: ["d4"] });
    await closeBatch();
    expect(vi.mocked(run)).toHaveBeenLastCalledWith("smart.redact_batch_close", { jobId: 7 });
    expect(useRedactBatch.getState().status).toBeNull();
  });
});
