// Themes outside the Styles tab (EDT-017, EDT-018): importing a `.xtheme` from a menu,
// a drop or the tab itself.
import i18next from "i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { run } from "../lib/commands";
import { useEditor } from "./state";

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
