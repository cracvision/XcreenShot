// Only what is in view is drawn (LIB-025, APP-052): which rows of a grid or list
// of fixed-height rows a scroll position shows, with a margin above and below.

export interface Span {
  /** First and last row drawn (inclusive), and the items they hold. */
  firstRow: number;
  lastRow: number;
  start: number;
  end: number;
  rows: number;
}

export function visibleSpan(
  count: number,
  columns: number,
  rowHeight: number,
  scrollTop: number,
  viewport: number,
  overscan = 2,
): Span {
  const cols = Math.max(1, columns);
  const h = Math.max(1, rowHeight);
  const rows = Math.ceil(count / cols);
  if (rows === 0) return { firstRow: 0, lastRow: -1, start: 0, end: 0, rows: 0 };
  const firstRow = Math.max(0, Math.floor(scrollTop / h) - overscan);
  const lastRow = Math.min(rows - 1, Math.ceil((scrollTop + Math.max(0, viewport)) / h) + overscan);
  return {
    firstRow,
    lastRow,
    start: firstRow * cols,
    end: Math.min(count, (lastRow + 1) * cols),
    rows,
  };
}

/** Columns of `min` px (and `gap` between) that fit in `width`. */
export function columnsFor(width: number, min: number, gap: number): number {
  return Math.max(1, Math.floor((width + gap) / (min + gap)));
}

/** The scroll position that brings `row` into view, or null when it already is. */
export function scrollToRow(
  row: number,
  rowHeight: number,
  scrollTop: number,
  viewport: number,
): number | null {
  const top = row * rowHeight;
  if (top < scrollTop) return top;
  if (top + rowHeight > scrollTop + viewport) return top + rowHeight - viewport;
  return null;
}
