// Tags and favorites of library items (LIB-009 to LIB-011): the commands, and the
// dialogs that ask for them. Dates and files never change.
import i18next from "i18next";
import { create } from "zustand";
import { run } from "../lib/commands";
import { useEditor } from "../editor/state";
import { NO_FILTERS } from "./query";
import { useLibrary } from "./state";

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

/** Marks or unmarks as favorites: all, unless all already are (LIB-011). */
export async function toggleFavorite(ids: string[]): Promise<void> {
  if (ids.length === 0) return;
  try {
    await run("library.favorite", { ids });
  } catch (e) {
    fail(e);
  }
}

export async function changeTags(ids: string[], add: string[], remove: string[]): Promise<boolean> {
  try {
    await run("library.tag", { ids, add, remove });
    return true;
  } catch (e) {
    fail(e);
    return false;
  }
}

/** The tag editor (LIB-009) for these items. */
export const useTagEditor = create<{ ids: string[] | null }>(() => ({ ids: null }));

export function editTags(ids: string[]): void {
  if (ids.length > 0) useTagEditor.setState({ ids });
}

/** A tag of the filter pane being renamed, merged or deleted (LIB-010). */
export interface TagAction {
  action: "rename" | "merge" | "delete" | "select";
  tag: string;
  count: number;
}

export const useTagAction = create<{ pending: TagAction | null }>(() => ({ pending: null }));

export function askTagAction(a: TagAction): void {
  if (a.action === "select") {
    void selectTagged(a.tag);
    return;
  }
  useTagAction.setState({ pending: a });
}

/** Seleccionar sus elementos (LIB-010): the tag alone as filter, every item selected. */
async function selectTagged(tag: string): Promise<void> {
  const lib = useLibrary.getState();
  lib.setFilters({ ...NO_FILTERS, tags: [tag] });
  await useLibrary.getState().load();
  const ids = useLibrary.getState().items.map((i) => i.id);
  useLibrary.getState().select(ids, ids[0] ?? null, ids[0] ?? null);
}

/** Renames a tag; onto an existing one, `merge` must be given. Returns "exists" when
 * the core asks to merge. */
export async function renameTag(
  tag: string,
  to: string,
  merge: boolean,
): Promise<"ok" | "exists" | "failed"> {
  try {
    await run("library.tags", { action: "rename", tag, to, merge });
    return "ok";
  } catch (e) {
    const details = (e as { details?: { reason?: string } }).details;
    if (details?.reason === "exists") return "exists";
    fail(e);
    return "failed";
  }
}

export async function mergeTag(tag: string, into: string): Promise<void> {
  try {
    await run("library.tags", { action: "merge", tag, into });
  } catch (e) {
    fail(e);
  }
}

export async function deleteTag(tag: string): Promise<void> {
  try {
    const r = await run<{ changed: number }>("library.tags", { action: "delete", tag });
    useEditor.getState().showToast(i18next.t("ui.library.tags.deleted", { tag, count: r.changed }));
  } catch (e) {
    fail(e);
  }
}

/** Every tag of the library with its count, for the editor's suggestions. */
export async function allTags(): Promise<{ tag: string; count: number }[]> {
  try {
    const r = await run<{ tags: { tag: string; count: number }[] }>("library.tags", {
      sort: "count",
    });
    return r.tags;
  } catch {
    return [];
  }
}
