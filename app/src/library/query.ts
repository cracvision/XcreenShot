// What the Library view asks for (LIB-004 to LIB-007, LIB-009, LIB-011): the order,
// the filters of the left pane and the search, and the counts the core sends back.
// Pure helpers.

export type SortKey = "modified" | "captured" | "name" | "size" | "type" | "app";
export type Order = "asc" | "desc";

export const SORT_KEYS: SortKey[] = ["modified", "captured", "name", "size", "type", "app"];

/** The direction a sort starts with: newest and largest first, the rest A to Z. */
export function defaultOrder(sort: SortKey): Order {
  return sort === "modified" || sort === "captured" || sort === "size" ? "desc" : "asc";
}

export interface Filters {
  types: string[];
  dates: string[];
  apps: string[];
  sites: string[];
  tags: string[];
  favorites: boolean;
}

export type FilterSection = "types" | "dates" | "apps" | "sites" | "tags";

export const NO_FILTERS: Filters = {
  types: [],
  dates: [],
  apps: [],
  sites: [],
  tags: [],
  favorites: false,
};

export interface Count {
  value: string;
  count: number;
}

export interface MonthCount extends Count {
  days: Count[];
}
export interface YearCount extends Count {
  months: MonthCount[];
}

export interface Facets {
  types: Count[];
  dates: YearCount[];
  quickDates: Count[];
  apps: Count[];
  sites: Count[];
  tags: Count[];
  favorites: number;
}

export const NO_FACETS: Facets = {
  types: [],
  dates: [],
  quickDates: [],
  apps: [],
  sites: [],
  tags: [],
  favorites: 0,
};

export function hasFilters(f: Filters): boolean {
  return (
    f.favorites ||
    f.types.length > 0 ||
    f.dates.length > 0 ||
    f.apps.length > 0 ||
    f.sites.length > 0 ||
    f.tags.length > 0
  );
}

/** A value of a section turned on or off (values of one section add up). */
export function toggle(f: Filters, section: FilterSection, value: string): Filters {
  const now = f[section];
  const on = now.some((v) => v.toLowerCase() === value.toLowerCase());
  return {
    ...f,
    [section]: on ? now.filter((v) => v.toLowerCase() !== value.toLowerCase()) : [...now, value],
  };
}

export function isOn(f: Filters, section: FilterSection, value: string): boolean {
  return f[section].some((v) => v.toLowerCase() === value.toLowerCase());
}

/** The chips over the grid: one per active value, in the pane's order. */
export function chips(f: Filters): { section: FilterSection | "favorites"; value: string }[] {
  const out: { section: FilterSection | "favorites"; value: string }[] = [];
  for (const section of ["types", "dates", "apps", "sites", "tags"] as const)
    for (const value of f[section]) out.push({ section, value });
  if (f.favorites) out.push({ section: "favorites", value: "" });
  return out;
}

/** A section's values to show: those with items, and those chosen even at 0. */
export function visibleValues(counts: Count[], chosen: string[]): Count[] {
  const shown = counts.filter((c) => c.count > 0);
  for (const v of chosen)
    if (!shown.some((c) => c.value.toLowerCase() === v.toLowerCase()))
      shown.push({ value: v, count: 0 });
  return shown;
}

/** "2026-11" as "November 2026" (or "noviembre de 2026"), "2026-11-12" as a date. */
export function dateLabel(value: string, lang: string): string {
  const [y, m, d] = value.split("-").map(Number);
  if (!y) return value;
  if (!m) return String(y);
  const date = new Date(y, m - 1, d ?? 1);
  return d
    ? date.toLocaleDateString(lang, { day: "numeric", month: "short", year: "numeric" })
    : date.toLocaleDateString(lang, { month: "long", year: "numeric" });
}
