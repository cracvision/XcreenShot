// What the settings transfer dialogs read of an import preview (APP-011).

export const GROUPS = ["preferences", "shortcuts", "themes", "toolbar"] as const;
export type Group = (typeof GROUPS)[number];

/** How many things a group of the preview changes. */
export function changesOf(group: Group, g: Record<string, unknown> | undefined): number {
  if (!g) return 0;
  if (group === "themes") {
    const n = (k: string) => {
      const c = g[k] as { added?: number; replaced?: number } | undefined;
      return (c?.added ?? 0) + (c?.replaced ?? 0);
    };
    return n("themes") + n("effects") + Number(g.favorites ?? 0);
  }
  if (group === "toolbar") return g.changes ? 1 : 0;
  return Array.isArray(g.changes) ? g.changes.length : 0;
}
