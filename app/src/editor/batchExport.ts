// Batch export (EDT-113): the dialog's state and what it remembers, and the names it
// will give (the same rule as the core's `batch_name`).
import { create } from "zustand";
import { DEFAULT_OPTIONS, type ExportFormat, type ExportOptions } from "./exportOptions";

export interface Naming {
  mode: "keep" | "base";
  base: string;
  start: number;
  digits: number;
  prefix: string;
  suffix: string;
}

export interface BatchSettings {
  format: ExportFormat;
  options: ExportOptions;
  /** A saved effect's id, or null for none. */
  effect: string | null;
  folder: string | null;
  nextToOriginal: boolean;
  naming: Naming;
  onConflict: "keepBoth" | "replace" | "skip";
}

export const DEFAULT_BATCH: BatchSettings = {
  format: "png",
  options: DEFAULT_OPTIONS,
  effect: null,
  folder: null,
  nextToOriginal: false,
  naming: { mode: "keep", base: "image", start: 1, digits: 2, prefix: "", suffix: "" },
  onConflict: "keepBoth",
};

const KEY = "xs.batch";

export function rememberedBatch(): BatchSettings {
  try {
    const saved = JSON.parse(localStorage.getItem(KEY) ?? "{}") as Partial<BatchSettings>;
    return {
      ...DEFAULT_BATCH,
      ...saved,
      options: { ...DEFAULT_OPTIONS, ...(saved.options ?? {}) },
      naming: { ...DEFAULT_BATCH.naming, ...(saved.naming ?? {}) },
    };
  } catch {
    return DEFAULT_BATCH;
  }
}

export function rememberBatch(s: BatchSettings): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(s));
  } catch {
    // Not remembered: no harm.
  }
}

/** The name (without extension) of the `index`-th file exported. */
export function batchName(naming: Naming, original: string, index: number): string {
  const core =
    naming.mode === "base"
      ? `${naming.base.trim() || "image"}_${String(naming.start + index).padStart(
          Math.min(9, Math.max(1, naming.digits)),
          "0",
        )}`
      : original;
  return `${naming.prefix}${core}${naming.suffix}`;
}

/** The file's name without folder and extension. */
export function stemOf(path: string): string {
  const name = path.split(/[\\/]/).pop() ?? path;
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(0, dot) : name;
}

/** The files the dialog is open with, or null when closed. */
export const useBatchDialog = create<{ files: string[] | null }>(() => ({ files: null }));
