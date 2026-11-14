// Only the text (SMT-021): a highlight, a blur or a spotlight that covers just the
// lines of text inside its area. The core finds the bands and keeps them in the object,
// relative to its corner, so they follow it while it is dragged.
import type { BlurObject, DocObject, HighlightObject, SpotlightObject } from "./state";

type Banded = HighlightObject | BlurObject | SpotlightObject;

/** The bands to draw in canvas pixels; null to draw the whole area. */
export function textBands(o: Banded): [number, number, number, number][] | null {
  if (!o.style.textOnly || !o.bands || o.bands.length === 0) return null;
  return o.bands.map(([x, y, w, h]) => [o.rect.x + x, o.rect.y + y, w, h]);
}

/** Selected objects asking for the text only found none inside: they cover their area. */
export function noTextIn(objects: DocObject[]): boolean {
  return objects.some(
    (o) =>
      (o.type === "highlight" || o.type === "blur" || o.type === "spotlight") &&
      o.style.textOnly === true &&
      o.bandsFor !== undefined &&
      (o.bands?.length ?? 0) === 0,
  );
}
