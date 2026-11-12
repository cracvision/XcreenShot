// Whether the Print dialog is open (EDT-114).
import { create } from "zustand";

export const usePrintDialog = create<{ open: boolean }>(() => ({ open: false }));
