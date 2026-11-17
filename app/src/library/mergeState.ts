// Merge another library (LIB-031): from the Library's ⋯ menu, choose the other
// library's folder, confirm, and its items are copied here in the background (the
// status bar shows the progress and Cancel); a report says what happened.
import { open } from "@tauri-apps/plugin-dialog";
import type { TFunction } from "i18next";
import i18next from "i18next";
import { create } from "zustand";
import { run } from "../lib/commands";
import { fail } from "./maintenanceState";

export interface MergeReport {
  added: number;
  duplicates: number;
  enriched: number;
  renamed: { from: string; path: string }[];
  failed: string[];
  cancelled: boolean;
  path?: string;
}

export interface MergeAsk {
  path: string;
  items: number;
  /** It is an XcreenShot library (its items have their tags). */
  library: boolean;
}

export const useMerge = create<{
  asking: MergeAsk | null;
  report: MergeReport | null;
  running: boolean;
}>(() => ({ asking: null, report: null, running: false }));

/** Choose the other library's folder and look at it before asking. */
export async function askMerge(chosen?: string): Promise<void> {
  const dir =
    chosen ?? (await open({ directory: true, title: i18next.t("ui.library.merge.choose") }));
  if (typeof dir !== "string") return;
  try {
    const i = await run<{ path: string; items: number; library: boolean; same: boolean }>(
      "library.location",
      { action: "inspect", path: dir },
    );
    if (i.same) {
      fail(i18next.t("ui.library.merge.same"));
      return;
    }
    useMerge.setState({ asking: { path: i.path, items: i.items, library: i.library } });
  } catch (e) {
    fail(e);
  }
}

/** Starts the merge; its end arrives as a `library_merge` event. */
export async function startMerge(path: string): Promise<void> {
  useMerge.setState({ asking: null, running: true });
  try {
    await run("library.merge", { path, background: true });
  } catch (e) {
    useMerge.setState({ running: false });
    fail(e);
  }
}

export function cancelMerge(): void {
  void run("library.merge", { cancel: true }).catch(() => undefined);
}

/** The report's sentences, in order. */
export function mergeSummary(r: MergeReport, t: TFunction): string[] {
  if (r.cancelled) return [t("ui.library.merge.cancelled")];
  const lines = [t("ui.library.merge.added", { count: r.added })];
  if (r.duplicates > 0) lines.push(t("ui.library.merge.duplicates", { count: r.duplicates }));
  if (r.enriched > 0) lines.push(t("ui.library.merge.enriched", { count: r.enriched }));
  if (r.renamed.length > 0) lines.push(t("ui.library.merge.renamed", { count: r.renamed.length }));
  if (r.failed.length > 0) lines.push(t("ui.library.merge.failed", { count: r.failed.length }));
  return lines;
}

/** The file name at the end of a path. */
export function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}
