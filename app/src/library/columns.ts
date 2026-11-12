// The columns of the Library's list (LIB-004): which show, how wide, and the grid
// template that lays out the header and the rows alike.
import type { SortKey } from "./query";

export interface Column {
  key: string;
  sort?: SortKey;
  /** The width when the user has not set one. */
  width: string;
}

export const COLUMNS: Column[] = [
  { key: "name", sort: "name", width: "minmax(180px, 2.2fr)" },
  { key: "captured", sort: "captured", width: "minmax(130px, 1.1fr)" },
  { key: "modified", sort: "modified", width: "minmax(130px, 1.1fr)" },
  { key: "size", sort: "size", width: "80px" },
  { key: "type", sort: "type", width: "120px" },
  { key: "dimensions", width: "110px" },
  { key: "app", sort: "app", width: "minmax(90px, 1fr)" },
  { key: "site", width: "minmax(90px, 1fr)" },
  { key: "tags", width: "minmax(90px, 1fr)" },
];

/** The smallest a column can be made. */
export const MIN_WIDTH = 56;

/** The columns shown: all but the hidden ones; the name always. */
export function shownColumns(hidden: string[] = []): Column[] {
  return COLUMNS.filter((c) => c.key === "name" || !hidden.includes(c.key));
}

/** The grid template of the list for these columns and widths. */
export function template(columns: Column[], widths: Record<string, number> = {}): string {
  return columns
    .map((c) => {
      const w = widths[c.key];
      return typeof w === "number" && Number.isFinite(w)
        ? `${Math.max(MIN_WIDTH, Math.round(w))}px`
        : c.width;
    })
    .join(" ");
}
