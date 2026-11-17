// The Library's thumbnail size (LIB-003): 50 to 300 % of a 160 px thumbnail, kept
// with the rest of the view's shape.
import { useLibrary } from "./state";

export const ZOOM_MIN = 50;
export const ZOOM_MAX = 300;
export const ZOOM_STEP = 10;

/** A zoom within 50-300 %, in steps of 10; 100 when there is none. */
export function zoomOf(value: number | undefined): number {
  if (typeof value !== "number" || !Number.isFinite(value)) return 100;
  return Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, Math.round(value / ZOOM_STEP) * ZOOM_STEP));
}

export function setZoom(value: number): void {
  const zoom = zoomOf(value);
  if (zoom !== zoomOf(useLibrary.getState().shape.zoom)) useLibrary.getState().setShape({ zoom });
}
