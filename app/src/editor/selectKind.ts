// Select all the objects of one kind (EDT-023): every arrow, every step… For editing
// them together. An object in a group brings its whole group; locked ones stay out.
import type { DocObject } from "./state";

/** Kinds that have the button: the annotation tools whose marks are objects. */
export const SELECTABLE_KINDS = [
  "arrow",
  "line",
  "text",
  "callout",
  "shape",
  "stamp",
  "step",
  "highlight",
  "pen",
  "magnify",
  "spotlight",
  "blur",
  "simplify",
] as const;

export function hasSelectAll(kind: string): boolean {
  return (SELECTABLE_KINDS as readonly string[]).includes(kind);
}

/** The ids to select: the objects of `kind`, each with its outermost group. */
export function idsOfKind(objects: DocObject[], kind: string): string[] {
  const free = objects.filter((o) => !o.locked);
  const groups = new Set(
    free.filter((o) => o.type === kind && o.groups?.[0]).map((o) => o.groups?.[0] as string),
  );
  return free
    .filter((o) => o.type === kind || (o.groups?.[0] !== undefined && groups.has(o.groups[0])))
    .map((o) => o.id);
}
