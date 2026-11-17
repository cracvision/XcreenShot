// What the recent tray does with its items besides opening and closing them
// (TRY-010, TRY-012, TRY-016): the selection, duplicates, and the Library's actions on
// the items' library entries (details, tags, favorite, show in library).
import i18next from "i18next";
import { run } from "../lib/commands";
import { editTags, toggleFavorite } from "../library/marks";
import { revealInLibrary, useLibrary, useTray } from "../library/state";
import { focusedTrayItem } from "./tray";
import { useEditor } from "./state";

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

/** What a tray action is about: the selection when the item is in it and there are
 * several, else the item (the one with the focus, or the one shown). */
export function trayTargets(
  id: string | null = focusedTrayItem() ?? useEditor.getState().active,
): string[] {
  if (!id) return [];
  const { selected } = useTray.getState();
  return selected.length > 1 && selected.includes(id) ? selected : [id];
}

/** The library IDs of tray items (those in the library). */
export async function libraryIdsOf(docs: string[]): Promise<string[]> {
  const r = await run<{ items: { documentId: string; libraryId: string | null }[] }>("tray.list");
  return r.items
    .filter((i) => docs.includes(i.documentId) && i.libraryId)
    .map((i) => i.libraryId as string);
}

/** Duplicate (TRY-010): copies beside their originals, in the tray. */
export async function duplicateDocs(docs: string[]): Promise<void> {
  if (docs.length === 0) return;
  try {
    const r = await run<{ copies: { path: string }[] }>("library.duplicate", {
      documentIds: docs,
      open: true,
    });
    const [one] = r.copies;
    const name = one?.path.split(/[\\/]/).pop() ?? "";
    useEditor
      .getState()
      .showToast(i18next.t("ui.editor.tray.duplicated", { count: r.copies.length, name }));
  } catch (e) {
    fail(e);
  }
}

/** Duplicate in the Library view (TRY-010): the copies appear in the library. */
export async function duplicateItems(ids: string[]): Promise<void> {
  if (ids.length === 0) return;
  try {
    const r = await run<{ copies: { path: string }[] }>("library.duplicate", { ids });
    const [one] = r.copies;
    const name = one?.path.split(/[\\/]/).pop() ?? "";
    useEditor
      .getState()
      .showToast(i18next.t("ui.editor.tray.duplicated", { count: r.copies.length, name }));
  } catch (e) {
    fail(e);
  }
}

/** Tags… of tray items (TRY-016, LIB-009). */
export async function tagDocs(docs: string[]): Promise<void> {
  try {
    editTags(await libraryIdsOf(docs));
  } catch (e) {
    fail(e);
  }
}

/** Mark as favorite (TRY-016, LIB-011). */
export async function favoriteDocs(docs: string[]): Promise<void> {
  try {
    await toggleFavorite(await libraryIdsOf(docs));
  } catch (e) {
    fail(e);
  }
}

/** Show in library (TRY-016, LIB-014): the item selected there. */
export async function revealDoc(doc: string): Promise<void> {
  try {
    const [id] = await libraryIdsOf([doc]);
    if (id) await revealInLibrary(id);
  } catch (e) {
    fail(e);
  }
}

/** Details (TRY-016, LIB-012): the item in the Library with its Details pane open. */
export async function detailsOfDoc(doc: string): Promise<void> {
  await revealDoc(doc);
  useLibrary.getState().setShape({ details: true });
}

/** Copy all of a tray item (TRY-016, SHR-015). */
export async function copyAllOf(doc: string): Promise<void> {
  try {
    await run("document.copy_all", { documentId: doc });
    useEditor.getState().showToast(i18next.t("ui.editor.copied"));
  } catch (e) {
    fail(e);
  }
}
