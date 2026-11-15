import { describe, expect, it } from "vitest";
import { shownZoom, zoomAfter } from "./videoZoom";

describe("the zoom on a video (VID-056)", () => {
  it("follows the editor's zoom requests", () => {
    expect(zoomAfter("fit", null, 150)).toBeNull();
    expect(zoomAfter("actual", null, 37)).toBe(100);
    expect(zoomAfter("percent", 250, 100)).toBe(250);
    expect(zoomAfter("in", null, 150)).toBe(200);
    expect(zoomAfter("out", null, 150)).toBe(100);
  });

  it("says what is shown, in video pixels per screen pixel", () => {
    // 320 px of video in 320 CSS px at 150 % display scale: 150 %.
    expect(shownZoom(320, 320, 1.5)).toBe(150);
    expect(shownZoom(960, 1920, 1)).toBe(50);
  });
});
