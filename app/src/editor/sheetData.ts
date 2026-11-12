// The data of the shortcut sheet (APP-023): what the core sends, the keys that only
// work in a context, and the search.
import { create } from "zustand";

export interface SheetRow {
  title: string;
  keys: string;
  inactive: boolean;
}

export interface Sheet {
  title: string;
  profile: string;
  platform: string;
  date: string;
  groups: { id: string; title: string; rows: SheetRow[] }[];
}

export const useShortcutSheet = create<{ open: boolean }>(() => ({ open: false }));

export function openShortcutSheet(): void {
  useShortcutSheet.setState({ open: true });
}

const IS_MAC = typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");
const MOD = IS_MAC ? "⌘" : "Ctrl";

/** Keys that are not shortcuts of an action: they work in one place (CAP-017, EDT-013,
 * LIB-012, LIB-013). Labels are i18n keys; keys starting with "@" too. */
export const CONTEXTUAL: { id: string; rows: { keys: string; label: string }[] }[] = [
  {
    id: "contextual",
    rows: [
      { keys: "Enter · I", label: "ui.overlay.keys.capture" },
      { keys: "← ↑ → ↓", label: "ui.overlay.keys.move" },
      { keys: "Alt + ← ↑ → ↓", label: "ui.overlay.keys.resize" },
      { keys: "G", label: "ui.overlay.keys.guides" },
      { keys: "F", label: "ui.overlay.keys.fullscreen" },
      { keys: "A", label: "ui.overlay.keys.all_monitors" },
      { keys: "C", label: "ui.overlay.keys.color" },
      { keys: "M", label: "ui.overlay.keys.magnifier" },
      { keys: "H · F1", label: "ui.overlay.keys.help" },
      { keys: "Esc", label: "ui.overlay.keys.cancel" },
    ],
  },
  {
    id: "canvas",
    rows: [
      { keys: `${MOD}+1…9`, label: "ui.shortcuts.k_tool" },
      { keys: "← ↑ → ↓", label: "ui.shortcuts.k_move" },
      { keys: "Shift + ← ↑ → ↓", label: "ui.shortcuts.k_move_more" },
      { keys: "Tab", label: "ui.shortcuts.k_next" },
      { keys: "[ · ]", label: "ui.shortcuts.k_size" },
      { keys: "I", label: "ui.shortcuts.k_eyedropper" },
      { keys: "Delete", label: "ui.shortcuts.k_delete" },
      { keys: "Esc", label: "ui.shortcuts.k_deselect" },
      { keys: "F6", label: "ui.shortcuts.k_zones" },
      { keys: "@ui.library.quick.key", label: "ui.shortcuts.k_quick" },
      { keys: IS_MAC ? "⌘I" : "Alt+Enter", label: "ui.shortcuts.k_details" },
      { keys: `${MOD}+/`, label: "ui.shortcuts.k_sheet" },
    ],
  },
];

/** Without accents or case, the row's name or keys contain the search. */
export function matches(row: { title: string; keys: string }, query: string): boolean {
  const fold = (s: string) => s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
  const q = fold(query.trim());
  return q === "" || fold(row.title).includes(q) || fold(row.keys).includes(q);
}
