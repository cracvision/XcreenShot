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
