// Themes outside the Styles tab (EDT-017, EDT-018): importing a `.xtheme` from a menu,
// a drop or the tab itself.
import { useEffect, useState } from "react";
import i18next from "i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { run } from "../lib/commands";
import { useEditor, xsdocUrl } from "./state";

/** The themes, by id and name, again when they change (EDT-017). */
export function useThemes(): { id: string; name: string }[] {
  const version = useEditor((s) => s.themesVersion);
  const [themes, setThemes] = useState<{ id: string; name: string }[]>([]);
  useEffect(() => {
    let alive = true;
    run<{ themes: { id: string; name: string }[] }>("editor.themes")
      .then((r) => alive && setThemes(r.themes))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [version]);
  return themes;
}

/** Tools with saved styles (the core's `STYLED_TOOLS`). */
export const STYLED_TOOLS = [
  "arrow",
  "line",
  "text",
  "callout",
  "shape",
  "step",
  "highlight",
  "pen",
  "magnify",
  "spotlight",
  "blur",
] as const;

export const themeRun = (params: Record<string, unknown>) =>
  run<Record<string, unknown>>("editor.theme", params);

/** Imports a `.xtheme` (EDT-018): asks the file if not given; Replace or Keep both. */
export async function importTheme(path?: string): Promise<void> {
  const { showError, showToast } = useEditor.getState();
  try {
    const chosen =
      path ??
      ((await open({
        filters: [{ name: i18next.t("ui.editor.themes.file"), extensions: ["xtheme"] }],
      })) as string | null);
    if (!chosen) return;
    const r = await themeRun({ action: "import", path: chosen });
    if (r.status === "needs_decision") {
      useEditor.setState({ themeConflict: { path: chosen, name: String(r.name) } });
      return;
    }
    announceImport(r, showToast);
  } catch (e) {
    showError(e);
  }
}

export function announceImport(r: Record<string, unknown>, showToast: (t: string) => void) {
  if (typeof r.missingFont === "string")
    showToast(i18next.t("ui.editor.themes.missing_font", { font: r.missingFont }));
  else showToast(i18next.t("ui.editor.themes.imported"));
}

/** The same style, whatever the key order. */
export function same(a: unknown, b: unknown): boolean {
  return JSON.stringify(sorted(a)) === JSON.stringify(sorted(b));
}
function sorted(v: unknown): unknown {
  if (Array.isArray(v)) return v.map(sorted);
  if (v && typeof v === "object")
    return Object.fromEntries(
      Object.keys(v as object)
        .sort()
        .filter((k) => (v as Record<string, unknown>)[k] !== null)
        .map((k) => [k, sorted((v as Record<string, unknown>)[k])]),
    );
  return v;
}
function hashOf(v: unknown): string {
  const text = JSON.stringify(sorted(v));
  let h = 2166136261;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return (h >>> 0).toString(36);
}

export function sampleUrl(tool: string, which: string, style: unknown): string {
  return xsdocUrl(`sample/${tool}/${which}?v=${hashOf(style)}`);
}
