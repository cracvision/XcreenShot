// Applies `general.theme` (APP-012). "system" follows prefers-color-scheme live via CSS;
// "light" and "dark" force the palette regardless of the system.
export function applyTheme(setting: unknown): void {
  const root = document.documentElement;
  if (setting === "light" || setting === "dark") {
    root.dataset.theme = setting;
  } else {
    delete root.dataset.theme;
  }
}

/** The glass of the floating panels, 0 (solid) to 1, from `general.glassIntensity`
 * (0–100, APP-012). Anything else is the default, 60. */
export function glassLevel(setting: unknown): number {
  const n = typeof setting === "number" && Number.isFinite(setting) ? setting : 60;
  return Math.min(100, Math.max(0, n)) / 100;
}

/** Applies `general.glassIntensity`: styles.css turns `--glass` into tint, blur and shine. */
export function applyGlass(setting: unknown): void {
  document.documentElement.style.setProperty("--glass", String(glassLevel(setting)));
}

const ACCENTS = ["violet", "teal", "graphite"];

/** Applies `general.accent` (APP-012): blue, the logo's, is the default palette. */
export function applyAccent(setting: unknown): void {
  const root = document.documentElement;
  if (typeof setting === "string" && ACCENTS.includes(setting)) {
    root.dataset.accent = setting;
  } else {
    delete root.dataset.accent;
  }
}
