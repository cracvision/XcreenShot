import { describe, expect, it } from "vitest";
import { aboutSize, angleOf, fromPx, keepRatio, shown, toPx, wheelAngle } from "./imageMenu";

describe("image dialogs", () => {
  it("converts sizes between units (EDT-069)", () => {
    expect(fromPx(1000, "percent", 2000, 96)).toBe(50);
    expect(toPx(50, "percent", 2000, 96)).toBe(1000);
    expect(fromPx(960, "in", 0, 96)).toBe(10);
    expect(toPx(5, "in", 0, 96)).toBe(480);
    expect(shown(fromPx(300, "cm", 0, 300), "cm")).toBe("2.54");
    expect(Math.round(toPx(2.54, "cm", 0, 300))).toBe(300);
  });

  it("keeps the proportion: 800 wide in 1600 × 900 is 450 high (EDT-069)", () => {
    expect(keepRatio(800, 1600, 900)).toBe(450);
  });

  it("without resampling, 10 in to 5 in doubles the resolution (EDT-069)", () => {
    // 960 px at 96 dpi are 10 in; the same pixels in 5 in are 192 dpi.
    const px = 960;
    expect(px / 5).toBe(192);
    expect(fromPx(px, "in", 0, 192)).toBe(5);
  });

  it("rounds angles to what document.rotate takes (EDT-072)", () => {
    expect(angleOf(15.04)).toBe(15);
    expect(angleOf(400)).toBe(359.9);
    expect(angleOf(Number.NaN)).toBe(0);
  });

  it("reads the wheel clockwise from the top", () => {
    expect(wheelAngle(0, -10)).toBe(0);
    expect(wheelAngle(10, 0)).toBe(90);
    expect(wheelAngle(-10, 0)).toBe(-90);
    expect(Math.abs(wheelAngle(0, 10))).toBe(180);
  });

  it("estimates a size on disk", () => {
    expect(aboutSize(100, 100)).toBe("10 KB");
    expect(aboutSize(2048, 1024)).toBe("2 MB");
    expect(aboutSize(2048, 1100, "es")).toBe("2,1 MB");
  });
});
