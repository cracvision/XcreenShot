// Order and movement of the recent tray (TRY-002, TRY-013).
import type { DocSummary } from "./state";

/** The tray's items: the documents shown in the editor, the most recent first. The
 * core keeps them in the order they arrived (EDT-120 restores it). */
export function trayOrder(documents: DocSummary[]): DocSummary[] {
  return documents.filter((d) => d.shown).reverse();
}

/** The item before (`-1`) or after (`1`) `active` in the tray; null at the ends
 * (no wrapping, TRY-013). */
export function neighbour(
  documents: DocSummary[],
  active: string | null,
  step: 1 | -1,
): string | null {
  const order = trayOrder(documents);
  const at = order.findIndex((d) => d.documentId === active);
  if (at < 0) return order[0]?.documentId ?? null;
  return order[at + step]?.documentId ?? null;
}

/** IDs of documents that are new in the tray (for the dot of the hidden tray, TRY-001). */
export function arrived(before: DocSummary[], after: DocSummary[]): boolean {
  const known = new Set(before.filter((d) => d.shown).map((d) => d.documentId));
  return after.some((d) => d.shown && !known.has(d.documentId));
}

/** The tray item with the keyboard focus, if the focus is in the tray. */
export function focusedTrayItem(): string | null {
  const el = document.activeElement;
  if (!(el instanceof HTMLElement)) return null;
  return el.closest<HTMLElement>(".strip [data-doc]")?.dataset.doc ?? null;
}
