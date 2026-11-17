// The editor's zoom on a video (VID-056): fitted to the stage by default, or a percent
// of its pixels (as the canvas's zoom, EDT-007: at 100 % one video pixel is one screen
// pixel) to look at details, scrolling the stage.
import { create } from "zustand";
import { clampZoom, stepZoom } from "./view";

interface VideoZoom {
  /** The document the zoom belongs to. */
  documentId: string | null;
  /** A percent asked for, or `null`: fitted. */
  zoom: number | null;
  /** The percent shown now, fitted or not. */
  shown: number;
}

export const useVideoZoom = create<VideoZoom>(() => ({ documentId: null, zoom: null, shown: 100 }));

/** The zoom after a request (`fit`, `actual`, `original`, `percent`, `in`, `out`). */
export function zoomAfter(mode: string, value: number | null, shown: number): number | null {
  switch (mode) {
    case "fit":
      return null;
    case "actual":
    case "original":
      return 100;
    case "percent":
      return value == null ? null : clampZoom(value);
    case "in":
      return stepZoom(shown, true);
    case "out":
      return stepZoom(shown, false);
    default:
      return null;
  }
}

/** The percent of `naturalWidth` video pixels shown in `cssWidth` CSS pixels. */
export function shownZoom(cssWidth: number, naturalWidth: number, dpr: number): number {
  if (!naturalWidth) return 100;
  return Math.round(((cssWidth * dpr) / naturalWidth) * 1000) / 10;
}
