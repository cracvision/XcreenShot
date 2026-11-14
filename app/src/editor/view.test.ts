import { describe, expect, it } from "vitest";
import {
  fitZoom,
  imagePoint,
  layout,
  openingZoom,
  originalZoom,
  parseZoom,
  scaleFor,
  stepZoom,
  zoomAround,
} from "./view";

const viewport = { width: 1000, height: 700 };

describe("zoom (EDT-007)", () => {
  it("opens large images fitted and small ones at actual size", () => {
    expect(openingZoom({ width: 6000, height: 4000 }, viewport, 1)).toBeLessThan(100);
    expect(openingZoom({ width: 400, height: 300 }, viewport, 1)).toBe(100);
    // On a 150 % display a 400 px capture still fits at 100 %.
    expect(openingZoom({ width: 400, height: 300 }, viewport, 1.5)).toBe(100);
  });

  it("actual size maps one image pixel to one physical pixel", () => {
    // 200 image px at 100 % on a 150 % display = 133.3 CSS px = 200 physical px.
    expect(200 * scaleFor(100, 1.5) * 1.5).toBeCloseTo(200);
  });

  it("fits the whole image", () => {
    const z = fitZoom({ width: 3000, height: 1000 }, viewport, 1);
    expect(3000 * scaleFor(z, 1)).toBeLessThanOrEqual(viewport.width - 48 + 1e-9);
  });

  it("steps through the fixed zoom levels", () => {
    expect(stepZoom(100, true)).toBe(150);
    expect(stepZoom(100, false)).toBe(67);
    expect(stepZoom(3200, true)).toBe(3200);
    expect(stepZoom(10, false)).toBe(10);
    expect(stepZoom(120, true)).toBe(150);
  });

  it("original size undoes the capture's display scale", () => {
    // Captured at 200 %, shown on a 100 % monitor: 50 %.
    expect(originalZoom(2, 1)).toBe(50);
  });

  it("zooming keeps the image pixel under the pointer", () => {
    const image = { width: 4000, height: 3000 };
    const view = { zoom: 100, cx: 2000, cy: 1500 };
    const p = { x: 200, y: 150 };
    const before = imagePoint(layout(view, image, viewport, 1), p);
    const after = imagePoint(
      layout(zoomAround(view, 200, p, image, viewport, 1), image, viewport, 1),
      p,
    );
    expect(after[0]).toBeCloseTo(before[0]);
    expect(after[1]).toBeCloseTo(before[1]);
  });

  it("never scrolls an image that fits, whatever the fraction of a pixel", () => {
    // Snapping the window to half the screen gave fractional sizes; a content box a
    // fraction too big made a scroll bar come and go every frame (Carlos, 2026-10-24).
    const image = { width: 2560, height: 1600 };
    for (const width of [1194.33, 1194.67, 1211, 1426.5]) {
      const vp = { width, height: 1058.67 };
      const zoom = fitZoom(image, vp, 1.5);
      const l = layout({ zoom, cx: 1280, cy: 800 }, image, vp, 1.5);
      expect(l.contentWidth).toBe(0);
      expect(l.contentHeight).toBe(0);
      expect(l.scrollLeft).toBe(0);
    }
  });

  it("scrolls large images with whole-pixel content", () => {
    const l = layout({ zoom: 100, cx: 2000, cy: 1500 }, { width: 4000, height: 3000 }, viewport, 1);
    expect(Number.isInteger(l.contentWidth)).toBe(true);
    expect(l.contentWidth).toBeGreaterThan(viewport.width);
    expect(l.scrollLeft).toBeGreaterThan(0);
  });

  it("reads typed values", () => {
    expect(parseZoom("250")).toBe(250);
    expect(parseZoom("33,5 %")).toBe(33.5);
    expect(parseZoom("99999")).toBe(3200);
    expect(parseZoom("abc")).toBeNull();
  });
});
