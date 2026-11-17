// Whether the Print dialog is open (EDT-114).
import { create } from "zustand";

/** With `paths`, library files print one per page without opening them (LIB-017). */
export const usePrintDialog = create<{ open: boolean; paths?: string[] | undefined }>(() => ({
  open: false,
}));

export function printFiles(paths: string[]): void {
  if (paths.length > 0) usePrintDialog.setState({ open: true, paths });
}
