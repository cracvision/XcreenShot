// Import to the library (LIB-024): files picked or dropped on the Library view are
// copied into it; a folder's name can become their tag.
import { open } from "@tauri-apps/plugin-dialog";
import i18next from "i18next";
import { create } from "zustand";
import { run } from "../lib/commands";
import { useEditor } from "../editor/state";

/** What the library takes (the formats of its index). */
export const IMPORTABLE = ["xshot", "png", "jpg", "jpeg"];

interface ImportResult {
  imported: { source: string; path: string; id: string | null }[];
  skipped: { source: string; reason: string }[];
}

/** Paths waiting for the Import dialog (with folders, to ask about their tags). */
export const useImport = create<{ paths: string[] | null }>(() => ({ paths: null }));

/** A path that is not an image the library takes: a folder, most likely. */
export function looksLikeFolder(path: string): boolean {
  const name = path.split(/[\\/]/).pop() ?? "";
  const dot = name.lastIndexOf(".");
  return dot <= 0 || !IMPORTABLE.includes(name.slice(dot + 1).toLowerCase());
}

/** Files dropped on the Library or chosen: with a folder among them, the dialog asks
 * about tags first; else they go in at once. */
export function askImport(paths: string[]): void {
  if (paths.length === 0) return;
  if (paths.some(looksLikeFolder)) useImport.setState({ paths });
  else void importPaths(paths, false);
}

export async function importPaths(paths: string[], tagFolders: boolean): Promise<void> {
  useImport.setState({ paths: null });
  const t = i18next.t.bind(i18next);
  try {
    const r = await run<ImportResult>("library.import", { paths, tagFolders });
    const done = t("ui.library.import.done", { count: r.imported.length });
    const skipped = r.skipped.length
      ? ` ${t("ui.library.import.skipped", { count: r.skipped.length })}`
      : "";
    useEditor.getState().showToast(`${done}${skipped}`);
  } catch (e) {
    useEditor.getState().showError(e);
  }
}

export async function importFiles(): Promise<void> {
  const picked = await open({
    multiple: true,
    filters: [{ name: i18next.t("ui.library.import.images"), extensions: IMPORTABLE }],
  });
  if (Array.isArray(picked) && picked.length > 0) await importPaths(picked, false);
}

export async function importFolder(): Promise<void> {
  const picked = await open({ directory: true });
  if (typeof picked === "string") useImport.setState({ paths: [picked] });
}
