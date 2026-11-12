// Templates in the editor (EDT-095 to EDT-101, ADR 0007): a document's template, its
// sections and zones, the picker's filter and the framing of a zone's picture. Pure
// helpers and the small stores the canvas, the Template tab and the picker share.
import { create } from "zustand";
import type { Point } from "./geometry";
import type { DocDetails, DocObject, Fill, StepShapeKind, ZoneObject } from "./state";

export interface TemplateSettings {
  scale: "fit" | "fill";
  title: boolean;
  captions: boolean;
  footer: boolean;
  author: "system" | "custom";
  authorText: string;
  font: string | null;
  textColor: string | null;
  background: Fill | null;
  numbering: boolean;
  numberColor: string | null;
  numberShape: StepShapeKind;
  reversed: boolean;
  theme: string | null;
}

export interface TemplateSection {
  id: string;
  kind: string;
  height: number;
  zones?: number;
}

export interface Template {
  id: string;
  settings: TemplateSettings;
  sections: TemplateSection[];
  width: number;
  hidden?: string[];
}

/** Section kinds a user can add (EDT-099), in the menu's order. */
export const SECTION_KINDS = ["step", "imageText", "text", "twoImages", "title", "footer"] as const;

/** The template of a document, if it is one. */
export function templateOf(doc: Pick<DocDetails, "template"> | null): Template | null {
  const t = doc?.template as Template | undefined;
  return t && Array.isArray(t.sections) && t.settings ? t : null;
}

/** Whether a section shows: a title or footer turned off does not (EDT-097). */
export function shown(t: Template, s: TemplateSection): boolean {
  return !(
    (s.kind === "title" && !t.settings.title) ||
    (s.kind === "footer" && !t.settings.footer)
  );
}

/** The sections shown, each with where it starts. */
export function bands(t: Template): { section: TemplateSection; top: number }[] {
  let y = 0;
  return t.sections
    .filter((s) => shown(t, s))
    .map((section) => {
      const band = { section, top: y };
      y += section.height;
      return band;
    });
}

export function zonesOf(objects: DocObject[]): ZoneObject[] {
  return objects.filter((o): o is ZoneObject => o.type === "zone");
}

/** The topmost zone under canvas point `p`. */
export function zoneAt(objects: DocObject[], p: Point): ZoneObject | null {
  const zones = zonesOf(objects);
  for (let i = zones.length - 1; i >= 0; i--) {
    const z = zones[i];
    if (!z) continue;
    const r = z.rect;
    if (p[0] >= r.x && p[0] <= r.x + r.w && p[1] >= r.y && p[1] <= r.y + r.h) return z;
  }
  return null;
}

/** Where a picture of `size` goes in its zone: top-left corner and scale (as the core
 * draws it). */
export function placement(
  z: ZoneObject,
  size: { w: number; h: number },
  offset: [number, number] = z.offset ?? [0, 0],
): { x: number; y: number; k: number } {
  const iw = Math.max(1, size.w);
  const ih = Math.max(1, size.h);
  const kx = z.rect.w / iw;
  const ky = z.rect.h / ih;
  const k = (z.fit === "fill" ? Math.max(kx, ky) : Math.min(kx, ky)) * (z.zoom ?? 1);
  return {
    x: z.rect.x + (z.rect.w - iw * k) / 2 + offset[0],
    y: z.rect.y + (z.rect.h - ih * k) / 2 + offset[1],
    k,
  };
}

/** A framing offset held to the zone's center and edges when near them (EDT-098):
 * centered, or the picture's side on the zone's side. */
export function snapOffset(
  z: ZoneObject,
  size: { w: number; h: number },
  offset: [number, number],
  reach: number,
): [number, number] {
  const { k } = placement(z, size, [0, 0]);
  const axis = (value: number, picture: number, zone: number): number => {
    const edge = (picture * k - zone) / 2;
    const targets = [0, edge, -edge];
    let best = value;
    let dist = reach;
    for (const t of targets) {
      const d = Math.abs(value - t);
      if (d <= dist) {
        dist = d;
        best = t;
      }
    }
    // No negative zero.
    return Math.round(best) || 0;
  };
  return [axis(offset[0], size.w, z.rect.w), axis(offset[1], size.h, z.rect.h)];
}

/** The divider under canvas `y` (the bottom edge of a section, not the last one), to
 * resize its section (EDT-099). */
export function dividerAt(t: Template, y: number, reach: number): TemplateSection | null {
  const all = bands(t);
  for (let i = 0; i < all.length - 1; i++) {
    const b = all[i];
    if (b && Math.abs(b.top + b.section.height - y) <= reach) return b.section;
  }
  return null;
}

/** A template of the picker (`editor.templates`). */
export interface PickerTemplate {
  id: string;
  name: string;
  family: string;
  min: number;
  max: number;
  own: boolean;
  path?: string;
  zones?: number;
  version?: number;
}

export const FAMILIES = ["steps", "timeline", "compare", "collage", "mine"] as const;

/** "For N images" (EDT-095): templates with at least N zones or a variable number. */
export function fits(t: PickerTemplate, n: number): boolean {
  return n <= 0 || t.max >= n;
}

const fold = (s: string): string =>
  s
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase();

/** The picker's groups: by family, filtered by the search and by "For N images". */
export function grouped(
  list: PickerTemplate[],
  query: string,
  n: number,
  filter: boolean,
): { family: string; items: PickerTemplate[] }[] {
  const q = fold(query.trim());
  const keep = list.filter((t) => (!filter || fits(t, n)) && (!q || fold(t.name).includes(q)));
  const known = (f: string) => (FAMILIES as readonly string[]).includes(f);
  return FAMILIES.map((family) => ({
    family,
    items: keep.filter((t) => t.family === family || (family === "mine" && !known(t.family))),
  })).filter((g) => g.items.length > 0);
}

/** How many pictures a thumbnail shows: the images chosen, within the template's
 * range (3 when none). */
export function thumbCount(t: PickerTemplate, n: number): number {
  return Math.min(t.max, Math.max(t.min, n > 0 ? n : 3));
}

/** An image for a template: a file, or an open document. */
export type ImageRef = string | { documentId: string };

/** The template picker (EDT-095). */
export const usePicker = create<{ open: boolean; images: ImageRef[] }>(() => ({
  open: false,
  images: [],
}));

export function openPicker(images: ImageRef[] = []): void {
  usePicker.setState({ open: true, images });
}

export function closePicker(): void {
  usePicker.setState({ open: false, images: [] });
}

/** What the canvas shares about zones: the zone clicked last (paste goes into it), the
 * one being framed, and how a window point maps to the canvas (files dropped on a
 * zone). */
export const useZones = create<{
  active: string | null;
  framing: string | null;
  pointAt: ((clientX: number, clientY: number) => Point | null) | null;
}>(() => ({ active: null, framing: null, pointAt: null }));

/** Where tray items being dragged would land on the canvas (TRY-014); none when the
 * drag is elsewhere. */
export const useDropPoint = create<{ at: Point | null }>(() => ({ at: null }));

/** The canvas point under a window position in physical pixels (a drop's). */
export function canvasPointAt(at: { x: number; y: number }): Point | null {
  const pointAt = useZones.getState().pointAt;
  const dpr = window.devicePixelRatio || 1;
  return pointAt ? pointAt(at.x / dpr, at.y / dpr) : null;
}

/** The Save as template dialog (EDT-101). */
export const useSaveTemplate = create<{ open: boolean }>(() => ({ open: false }));
