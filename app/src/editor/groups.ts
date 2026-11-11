// Groups (EDT-082): which objects a click takes. Each object carries its groups,
// outermost first. Outside any group a click takes the whole outermost group; after
// a double click enters a group, a click takes the next group inside it, or the object.
import type { DocObject } from "./state";

/** What a click on `o` takes at the level of `entered`: a group id, the object itself
 *  (null), or nothing because `o` is not inside the entered group (undefined). */
export function unitOf(o: DocObject, entered: string[]): string | null | undefined {
  const groups = o.groups ?? [];
  if (entered.some((g, i) => groups[i] !== g)) return undefined;
  return groups[entered.length] ?? null;
}

/** The ids a click or a box takes: each object with the rest of its group. */
export function expand(objects: DocObject[], ids: string[], entered: string[]): string[] {
  const units = new Set<string>();
  const loose = new Set<string>();
  for (const o of objects) {
    if (!ids.includes(o.id)) continue;
    const u = unitOf(o, entered);
    if (typeof u === "string") units.add(u);
    else loose.add(o.id);
  }
  return objects
    .filter((o) => loose.has(o.id) || (o.groups ?? []).some((g) => units.has(g)))
    .map((o) => o.id);
}
