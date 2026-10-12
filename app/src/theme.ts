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
