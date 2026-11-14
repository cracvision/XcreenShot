// Colors of the color selector (EDT-019): `#RRGGBB` or `#RRGGBBAA`, RGB and HSB.

export interface Rgba {
  r: number;
  g: number;
  b: number;
  /** 0–255. */
  a: number;
}

export interface Hsb {
  /** 0–360. */
  h: number;
  /** 0–100. */
  s: number;
  /** 0–100. */
  b: number;
}

export function parseHex(text: string): Rgba | null {
  const t = text.trim().replace(/^#/, "");
  if (!/^[0-9a-fA-F]{6}([0-9a-fA-F]{2})?$/.test(t)) return null;
  const n = (i: number) => parseInt(t.slice(i, i + 2), 16);
  return { r: n(0), g: n(2), b: n(4), a: t.length === 8 ? n(6) : 255 };
}

const hex2 = (v: number) =>
  Math.round(Math.min(255, Math.max(0, v)))
    .toString(16)
    .padStart(2, "0")
    .toUpperCase();

/** `#RRGGBB`, or `#RRGGBBAA` when not opaque — the form documents keep. */
export function toHex(c: Rgba): string {
  const rgb = `#${hex2(c.r)}${hex2(c.g)}${hex2(c.b)}`;
  return c.a >= 255 ? rgb : `${rgb}${hex2(c.a)}`;
}

export function rgbToHsb({ r, g, b }: Rgba): Hsb {
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const d = max - min;
  let h = 0;
  if (d > 0) {
    if (max === r) h = ((g - b) / d) % 6;
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
    h *= 60;
    if (h < 0) h += 360;
  }
  return { h, s: max === 0 ? 0 : (d / max) * 100, b: (max / 255) * 100 };
}

export function hsbToRgb({ h, s, b }: Hsb, a = 255): Rgba {
  const v = b / 100;
  const c = v * (s / 100);
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = v - c;
  const [r, g, bl] =
    h < 60
      ? [c, x, 0]
      : h < 120
        ? [x, c, 0]
        : h < 180
          ? [0, c, x]
          : h < 240
            ? [0, x, c]
            : h < 300
              ? [x, 0, c]
              : [c, 0, x];
  return {
    r: Math.round((r + m) * 255),
    g: Math.round((g + m) * 255),
    b: Math.round((bl + m) * 255),
    a,
  };
}

/** Colors of the default theme "Base" (EDT-017). */
export const THEME_COLORS = ["#1F2430", "#F2545B", "#2D7FF9", "#F5A524", "#FFFFFF"];

/** The basic palette of 40 colors (EDT-019): 8 hues in 5 tones. */
export const PALETTE: string[] = (() => {
  const hues = [0, 30, 50, 120, 180, 210, 260, 320];
  const tones: [number, number][] = [
    [30, 100],
    [60, 100],
    [90, 90],
    [90, 65],
    [80, 40],
  ];
  const out: string[] = [];
  for (const [s, b] of tones) for (const h of hues) out.push(toHex(hsbToRgb({ h, s, b })));
  // The first row is grays instead of the palest tints.
  return [
    ...["#FFFFFF", "#E6E6E6", "#BFBFBF", "#999999", "#666666", "#404040", "#1A1A1A", "#000000"],
    ...out.slice(8),
  ];
})();
