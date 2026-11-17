import { beforeEach, describe, expect, it } from "vitest";
import { requestZoom, takeZoomRequest, useEditor } from "./state";

describe("zoom requests (EDT-007)", () => {
  beforeEach(() => useEditor.setState({ active: "d1", zoomRequest: null }));

  it("apply once, to the document they were made for", () => {
    requestZoom("in");
    // Switching to another document does not zoom it.
    expect(takeZoomRequest("d2")).toBeNull();
    expect(takeZoomRequest("d1")?.mode).toBe("in");
    // Coming back does not zoom again.
    expect(takeZoomRequest("d1")).toBeNull();
  });

  it("from the CLI target the document they name", () => {
    useEditor.setState({
      zoomRequest: { mode: "percent", value: 250, seq: 1, documentId: "d3" },
    });
    expect(takeZoomRequest("d1")).toBeNull();
    expect(takeZoomRequest("d3")?.value).toBe(250);
  });
});
