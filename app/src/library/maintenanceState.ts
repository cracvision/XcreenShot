// Rebuild the index and Check the library (LIB-023): what the ⋯ menu starts.
import { create } from "zustand";
import { run } from "../lib/commands";
import { useEditor } from "../editor/state";

export interface Report {
  damaged: string[];
  orphans: number;
  writable: boolean;
}

export const useMaintenance = create<{ dialog: "rebuild" | "check" | null; report: Report | null }>(
  () => ({ dialog: null, report: null }),
);

export function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

/** The conflict copy being resolved (LIB-029). */
export const useResolve = create<{ item: { id: string; name: string } | null }>(() => ({
  item: null,
}));

export function askResolve(item: { id: string; name: string }): void {
  useResolve.setState({ item });
}

export async function resolveConflict(id: string, keep: "this" | "original" | "both") {
  useResolve.setState({ item: null });
  try {
    await run("library.resolve", { id, keep });
  } catch (e) {
    fail(e);
  }
}

export function askRebuild(): void {
  useMaintenance.setState({ dialog: "rebuild" });
}

export async function checkLibrary(): Promise<void> {
  try {
    const report = await run<Report>("library.check");
    useMaintenance.setState({ dialog: "check", report });
  } catch (e) {
    fail(e);
  }
}
