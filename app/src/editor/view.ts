// Zoom and scrolling of the editor canvas (EDT-007). Zoom is a percentage of physical
// pixels: at 100 % one image pixel is one screen pixel, whatever the display scale.

export const ZOOM_STEPS = [
  10, 25, 33, 50, 67, 100, 150, 200, 300, 400, 600, 800, 1200, 1600, 2400, 3200,
];
export const MIN_ZOOM = 10;
export const MAX_ZOOM = 3200;
/** Space kept around the canvas, in CSS pixels. */
export const MARGIN = 24;

export interface Size {
  width: number;
  height: number;
}

/** What the editor shows of a document: zoom, and the image point at the center. */
export interface View {
  zoom: number;
  cx: number;
  cy: number;
}

export function clampZoom(zoom: number): number {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom));
}

/** Next step up or down from `current` (Zoom in / Zoom out). */
export function stepZoom(current: number, up: boolean): number {
  if (up) return ZOOM_STEPS.find((s) => s > current + 0.01) ?? MAX_ZOOM;
  return [...ZOOM_STEPS].reverse().find((s) => s < current - 0.01) ?? MIN_ZOOM;
}

/** CSS pixels per image pixel. */
export function scaleFor(zoom: number, dpr: number): number {
  return zoom / 100 / dpr;
}

/** Fit to window: the whole image fits in the visible area. */
export function fitZoom(image: Size, viewport: Size, dpr: number): number {
  const w = Math.max(1, viewport.width - 2 * MARGIN);
  const h = Math.max(1, viewport.height - 2 * MARGIN);
  return clampZoom(Math.min((w * dpr) / image.width, (h * dpr) / image.height) * 100);
}

/** Zoom when a document opens: fitted if it does not fit, else actual size, never
 * enlarged (Shrink to fit, `editor.shrinkToFit`). */
export function openingZoom(image: Size, viewport: Size, dpr: number): number {
  return Math.min(100, fitZoom(image, viewport, dpr));
}

/** Original on-screen size: the size the capture had on its monitor (EDT-007). */
export function originalZoom(captureScale: number, dpr: number): number {
  return clampZoom((dpr / (captureScale || 1)) * 100);
}

/** Layout of the scrollable area for a view: content size, image origin, scroll. */
export interface Layout {
  scale: number;
  contentWidth: number;
  contentHeight: number;
  /** Image origin inside the content, in CSS pixels. */
  originX: number;
  originY: number;
  scrollLeft: number;
  scrollTop: number;
}

export function layout(view: View, image: Size, viewport: Size, dpr: number): Layout {
  const scale = scaleFor(view.zoom, dpr);
  const w = image.width * scale;
  const h = image.height * scale;
  // Small images sit centered; large ones keep a margin all around.
  const originX = Math.max(MARGIN, (viewport.width - w) / 2);
  const originY = Math.max(MARGIN, (viewport.height - h) / 2);
  const contentWidth = Math.max(viewport.width, w + 2 * originX);
  const contentHeight = Math.max(viewport.height, h + 2 * originY);
  const clamp = (v: number, max: number) => Math.min(Math.max(0, v), Math.max(0, max));
  return {
    scale,
    contentWidth,
    contentHeight,
    originX,
    originY,
    scrollLeft: clamp(
      originX + view.cx * scale - viewport.width / 2,
      contentWidth - viewport.width,
    ),
    scrollTop: clamp(
      originY + view.cy * scale - viewport.height / 2,
      contentHeight - viewport.height,
    ),
  };
}

/** The view after the user scrolled the area to `scrollLeft`, `scrollTop`. */
export function viewFromScroll(
  zoom: number,
  l: Pick<Layout, "scale" | "originX" | "originY">,
  scrollLeft: number,
  scrollTop: number,
  viewport: Size,
): View {
  return {
    zoom,
    cx: (scrollLeft + viewport.width / 2 - l.originX) / l.scale,
    cy: (scrollTop + viewport.height / 2 - l.originY) / l.scale,
  };
}

/** Image point under viewport point `p` (CSS pixels from the viewport's corner). */
export function imagePoint(l: Layout, p: { x: number; y: number }): [number, number] {
  return [(l.scrollLeft + p.x - l.originX) / l.scale, (l.scrollTop + p.y - l.originY) / l.scale];
}

/** Zooms to `zoom` keeping the image point under viewport point `p` in place
 * (Ctrl+wheel, pinch, EDT-007). */
export function zoomAround(
  view: View,
  zoom: number,
  p: { x: number; y: number },
  image: Size,
  viewport: Size,
  dpr: number,
): View {
  const before = layout(view, image, viewport, dpr);
  const [ix, iy] = imagePoint(before, p);
  const z = clampZoom(zoom);
  const scale = scaleFor(z, dpr);
  // The point stays under the pointer: center = point + (viewport center - pointer).
  return {
    zoom: z,
    cx: ix + (viewport.width / 2 - p.x) / scale,
    cy: iy + (viewport.height / 2 - p.y) / scale,
  };
}

/** Parses a typed zoom ("250", "250 %", "33,5"). */
export function parseZoom(text: string): number | null {
  const n = Number(text.replace("%", "").replace(",", ".").trim());
  return Number.isFinite(n) && n > 0 ? clampZoom(n) : null;
}
