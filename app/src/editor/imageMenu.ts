// The Image menu (EDT-069 to EDT-077): which dialog is open, and the units of
// sizes. The dialogs themselves are in ImageDialogs.tsx; they run document.* commands.
import { create } from "zustand";

export type ImageDialog =
  "resize" | "canvas_size" | "rotate" | "corners" | "resolution" | "canvas_color" | "revert";

export const useImageDialog = create<{ open: ImageDialog | null }>(() => ({ open: null }));

export function openImageDialog(dialog: ImageDialog): void {
  useImageDialog.setState({ open: dialog });
}

export function closeImageDialog(): void {
  useImageDialog.setState({ open: null });
}

/** Units of Resize image and Canvas size (EDT-069, EDT-070). */
export type Unit = "percent" | "px" | "in" | "cm";
export const UNITS: Unit[] = ["percent", "px", "in", "cm"];

const CM_PER_INCH = 2.54;

/** `px` in `unit`: `whole` is 100 %, `dpi` turns pixels into inches (EDT-077). */
export function fromPx(px: number, unit: Unit, whole: number, dpi: number): number {
  switch (unit) {
    case "percent":
      return whole > 0 ? (px / whole) * 100 : 100;
    case "px":
      return px;
    case "in":
      return px / dpi;
    case "cm":
      return (px / dpi) * CM_PER_INCH;
  }
}

/** The inverse of {@link fromPx}. */
export function toPx(value: number, unit: Unit, whole: number, dpi: number): number {
  switch (unit) {
    case "percent":
      return (value / 100) * whole;
    case "px":
      return value;
    case "in":
      return value * dpi;
    case "cm":
      return (value / CM_PER_INCH) * dpi;
  }
}

/** What a field shows: whole pixels, two decimals otherwise. */
export function shown(value: number, unit: Unit): string {
  if (unit === "px") return String(Math.round(value));
  return String(Math.round(value * 100) / 100);
}

/** With the proportion locked, the other side keeps the ratio of `w0` × `h0`. */
export function keepRatio(side: number, from: number, to: number): number {
  return from > 0 ? (side * to) / from : side;
}

/** A rough size on disk of a `w` × `h` capture saved as PNG: screenshots compress to
 * about one byte per pixel (provisional, spec/QUESTIONS.md). */
export function aboutSize(w: number, h: number, locale = "en"): string {
  const bytes = Math.max(1, w * h);
  if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  const mb = Math.round((bytes / (1024 * 1024)) * 10) / 10;
  return `${mb.toLocaleString(locale, { maximumFractionDigits: 1 })} MB`;
}

/** Resolutions offered in a menu (EDT-077). */
export const COMMON_DPI = [72, 96, 144, 150, 192, 300];

/** The 3 × 3 anchor of Canvas size (EDT-070), row by row. */
export const ANCHORS = [
  "topLeft",
  "top",
  "topRight",
  "left",
  "center",
  "right",
  "bottomLeft",
  "bottom",
  "bottomRight",
] as const;
export type Anchor = (typeof ANCHORS)[number];

/** Degrees in (-360, 360) with one decimal, as `document.rotate` takes them. */
export function angleOf(value: number): number {
  if (!Number.isFinite(value)) return 0;
  const r = Math.round(value * 10) / 10;
  return Math.max(-359.9, Math.min(359.9, r));
}

/** The wheel's angle for a pointer at (dx, dy) from its center: 0 straight up,
 * clockwise, in (-180, 180]. */
export function wheelAngle(dx: number, dy: number): number {
  const deg = (Math.atan2(dx, -dy) * 180) / Math.PI;
  return Math.round(deg * 10) / 10;
}
