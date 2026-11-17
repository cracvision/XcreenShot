// Order and movement of the recent tray (TRY-002, TRY-013).
import type { DocSummary } from "./state";

/** The tray's items, left to right: the order the core gives (`tray`: pinned ones
 * first, then the newest, as moved, TRY-004, TRY-005); without it, the documents shown
 * in the editor, the most recent first (they arrive in order, EDT-120 restores it). */
export function trayOrder(documents: DocSummary[], tray: string[] = []): DocSummary[] {
  if (tray.length === 0) return documents.filter((d) => d.shown).reverse();
  const byId = new Map(documents.map((d) => [d.documentId, d]));
  return tray.map((id) => byId.get(id)).filter((d): d is DocSummary => d !== undefined && d.shown);
}

/** Sizes of the tray's thumbnails (TRY-007). */
export const TRAY_SIZES = ["small", "medium", "large"] as const;
export type TraySize = (typeof TRAY_SIZES)[number];

/** Where an item dropped at `x` goes among the others, given their middles (TRY-005). */
export function dropIndex(middles: number[], x: number): number {
  return middles.filter((m) => m < x).length;
}

/** The item before (`-1`) or after (`1`) `active` in the tray; null at the ends
 * (no wrapping, TRY-013). */
export function neighbour(
  documents: DocSummary[],
  active: string | null,
  step: 1 | -1,
  tray: string[] = [],
): string | null {
  const order = trayOrder(documents, tray);
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
