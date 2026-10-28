// The editor's tools and the shortcut each button shows (EDT-003, EDT-115, APP-014).
import { display } from "../lib/accelerator";
import { useApp } from "../store";

/** The current shortcut of a row of Preferences > Shortcuts, as the user sees it. */
export function useShortcut(id: string): string | undefined {
  return useApp((s) => {
    const row = s.shortcuts?.rows.find((r) => r.command === id);
    if (!row?.shortcut) return undefined;
    return display(row.shortcut, s.shortcuts?.platform ?? "windows", s.layout);
  });
}

/** "Name (shortcut)", for tooltips. */
export function hint(label: string, shortcut: string | undefined): string {
  return shortcut ? `${label} (${shortcut})` : label;
}
