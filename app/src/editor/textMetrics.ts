// Text constants shared with crates/xs-image/src/text.rs.

/** Line height as a multiple of the font size (EDT-026). */
export const LINE_SPACING = 1.2;

/** Font size in image pixels for `points` at the document's `dpi`. */
export function textPixels(points: number, dpi: number): number {
  return (points * Math.max(dpi, 1)) / 72;
}
