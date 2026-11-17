import { beforeEach, describe, expect, it } from "vitest";
import { aiProgress, useAi } from "./aiState";

describe("streamed answers (SMT-040)", () => {
  beforeEach(() =>
    useAi.setState({ pending: { prompt: "Describe", answer: "", requestId: "ask-1" } }),
  );

  it("show the latest text of the request in progress only", () => {
    aiProgress("ask-1", "A blue ");
    aiProgress("ask-0", "stale");
    aiProgress("ask-1", "A blue box.");
    // Heard twice: still the same text.
    aiProgress("ask-1", "A blue box.");
    expect(useAi.getState().pending?.answer).toBe("A blue box.");
  });

  it("are ignored once nothing is in progress", () => {
    useAi.setState({ pending: null });
    aiProgress("ask-1", "late");
    expect(useAi.getState().pending).toBeNull();
  });
});
