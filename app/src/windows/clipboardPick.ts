// Picking in XcreenShot's clipboard (TRY-018): toggles, ranges and the paste order.

/** Picks or unpicks one capture. */
export function pickToggle(picked: Set<string>, id: string): Set<string> {
  const next = new Set(picked);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  return next;
}

/** Adds every capture between `from` and `to` (Shift+click), in the list shown. */
export function pickRange(
  picked: Set<string>,
  shown: string[],
  from: string,
  to: string,
): Set<string> {
  const a = shown.indexOf(from);
  const b = shown.indexOf(to);
  if (a < 0 || b < 0) return pickToggle(picked, to);
  const next = new Set(picked);
  for (const id of shown.slice(Math.min(a, b), Math.max(a, b) + 1)) next.add(id);
  return next;
}

/** The picked captures in the tray's order: the order they are pasted in. */
export function pickedInOrder(picked: Set<string>, order: string[]): string[] {
  return order.filter((id) => picked.has(id));
}
