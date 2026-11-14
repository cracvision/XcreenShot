// Export from the Library (LIB-018, LIB-033): one item with Save as, several with the
// batch export dialog, the whole library (or what the filters show) as plain files.
import { save } from "@tauri-apps/plugin-dialog";
import i18next from "i18next";
import { create } from "zustand";
import { run } from "../lib/commands";
import { useBatchDialog } from "../editor/batchExport";
import { FORMATS, rememberedOptions, type ExportFormat } from "../editor/exportOptions";
import { useEditor } from "../editor/state";
import { hasFilters } from "./query";
import { useLibrary, type LibraryItem } from "./state";

/** Export… of the selection (LIB-018): library items or tray items, by name and file. */
export async function exportItems(items: Pick<LibraryItem, "name" | "path">[]): Promise<void> {
  const [one] = items;
  if (!one) return;
  if (items.length > 1) {
    useBatchDialog.setState({ files: items.map((i) => i.path) });
    return;
  }
  const t = i18next.t.bind(i18next);
  const path = await save({
    defaultPath: `${one.name}.png`,
    filters: FORMATS.map((f) => ({
      name: t(`ui.editor.export.format_${f}`),
      extensions: f === "jpg" ? ["jpg", "jpeg"] : f === "tiff" ? ["tif", "tiff"] : [f],
    })),
  });
  if (!path) return;
  const { folder, stem, format } = splitTarget(path);
  try {
    await run("document.batch_export", {
      paths: [one.path],
      format,
      options: rememberedOptions(format),
      folder,
      naming: { mode: "name", name: stem },
      // The save dialog already asked before replacing.
      onConflict: "replace",
    });
    useEditor.getState().showToast(t("ui.library.export.saved", { name: `${stem}.${format}` }));
  } catch (e) {
    useEditor.getState().showError(e);
  }
}

/** The folder, name and format of a path chosen in Save as; an unknown extension is PNG. */
export function splitTarget(path: string): { folder: string; stem: string; format: ExportFormat } {
  const cut = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  const folder = path.slice(0, Math.max(0, cut));
  const file = path.slice(cut + 1);
  const dot = file.lastIndexOf(".");
  const ext = dot > 0 ? file.slice(dot + 1).toLowerCase() : "";
  const known = ext === "jpeg" ? "jpg" : ext === "tif" ? "tiff" : ext;
  const format = (FORMATS as readonly string[]).includes(known) ? (known as ExportFormat) : "png";
  const stem = dot > 0 && known === format ? file.slice(0, dot) : file;
  return { folder, stem, format };
}

/** `path` is the folder `root` or inside it (case and slashes aside on Windows). */
export function insideFolder(root: string, path: string): boolean {
  if (!root || !path) return false;
  const norm = (p: string) => {
    const s = p.replace(/\\/g, "/").replace(/\/+$/, "");
    return navigator.userAgent.includes("Windows") ? s.toLowerCase() : s;
  };
  const r = norm(root);
  const p = norm(path);
  return p === r || p.startsWith(`${r}/`);
}

export interface LibraryExportResult {
  folder: string;
  exported: number;
  copied: number;
  failed: { source: string; name: string; reason: string }[];
  cancelled: boolean;
  csv: string | null;
}

/** The Export the whole library dialog (LIB-033). */
export const useLibraryExport = create<{ open: boolean }>(() => ({ open: false }));

export function askExportLibrary(): void {
  useLibraryExport.setState({ open: true });
}

/** Whether the filters or the search narrow what the Library shows. */
export function libraryFiltered(): boolean {
  const { filters, query } = useLibrary.getState();
  return hasFilters(filters) || query.trim() !== "";
}

export async function exportLibrary(
  folder: string,
  format: "png" | "jpg",
  filtered: boolean,
): Promise<LibraryExportResult> {
  const { filters, query } = useLibrary.getState();
  return run<LibraryExportResult>("library.export", {
    folder,
    format,
    ...(filtered ? { filters, query } : {}),
  });
}
